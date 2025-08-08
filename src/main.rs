use crate::data::{DataEntry, Database};
use crate::error::Error;
use log::{LevelFilter, debug, error, info, warn};
use std::time::{Duration, Instant};
use std::{env, thread};
use std::path::Path;
use syslog::{Facility};
use scale::scale::DisconnectedScale;
use scale::error::Error as ScaleError;
use scale::scale::Action as ScaleAction;

mod data;
mod error;

fn main() {
    let args: Vec<String> = env::args().collect();
    let log_level = match args.get(1).map(|s| s.as_str()) {
        Some("debug") => LevelFilter::Debug,
        Some("prod") => LevelFilter::Info,
        _ => {
            // Default to production mode if the argument is missing or invalid.
            println!(
                "Usage: {} [debug|prod]",
                args.first().unwrap_or(&"libra".to_string())
            );
            println!("Defaulting to 'prod' mode.");
            LevelFilter::Info
        }
    };
    syslog::init(Facility::LOG_USER, log_level, Some("libra")).expect("Couldn't initialize syslog");

    if let Err(e) = libra() {
        error!("Unrecoverable Error: {e}")
    }
}

fn libra() -> Result<(), Error> {
    info!("Libra application starting");
    let config_path = Path::new("/etc/libra/config.toml");
    let disconnected_scales = DisconnectedScale::from_config(&config_path)?;
    let mut scales = Vec::with_capacity(disconnected_scales.len());
    for disconnected_scale in disconnected_scales {
        let device = disconnected_scale.get_device();
        match disconnected_scale.connect() {
            Ok(scale) => {
                info!("Scale connected: {}", device);
                scales.push(scale);
            }
            Err(e) => match e {
                ScaleError::Phidget(phidget_error) => {
                    warn!("Scale failed to connect: {}", device);
                    warn!("{phidget_error}")
                }
                _ => {
                    error!("Unrecoverable Error");
                    return Err(Error::from(e));
                }
            },
        }
    }

    let database_path = Path::new("/var/lib/libra/data.db");
    let database = Database::new(database_path.into())?;

    let initial_data_entries: Vec<DataEntry> = scales
        .iter_mut()
        .map(|scale| match scale.get_weight() {
            Ok(weight) => Ok(DataEntry::new(
                ScaleAction::Starting,
                weight.get_amount(),
                scale.get_device(),
                Database::get_timestamp().expect("Couldn't get timestamp"),
                scale.get_config().location,
                scale.get_config().ingredient,
            )),
            Err(_e) => {
                error!("Device: {}", scale.get_device());
                Err(Error::Initialization)
            }
        })
        .filter_map(|result| {
            if let Err(e) = result {
                match e {
                    Error::Initialization => None,
                    _ => panic!("Unrecoverable error: {e}"),
                }
            } else {
                Some(result)
            }
        })
        .collect::<Result<_, _>>()?;
    database.log_all(initial_data_entries)?;

    let heartbeat_periods: Vec<Duration> = scales.iter().map(|scale| scale.get_config().heartbeat_period).collect();
    let heartbeat_period = heartbeat_periods
        .into_iter()
        .min().ok_or(Error::Initialization)?;
    let phidget_sample_periods: Vec<Duration> = scales.iter().map(|scale| scale.get_config().phidget_sample_period).collect();
    let phidget_sample_period = phidget_sample_periods
        .into_iter()
        .min().ok_or(Error::Initialization)?;
    let mut current_time = Instant::now();
    let mut last_heartbeat = current_time;
    loop {
        let is_time_for_heartbeat = if current_time - last_heartbeat > heartbeat_period {
            last_heartbeat = current_time;
            true
        } else {
            false
        };

        let mut weights = Vec::with_capacity(scales.len());
        let mut data_entries = Vec::with_capacity(scales.len());
        scales.iter_mut().try_for_each(|scale| {
            match scale.get_weight() {
                Ok(weight) => weights.push(weight),
                Err(e) => match e {
                    ScaleError::Phidget(err) => {
                        warn!("Phidget error: {err}");
                        info!("Restarting scale...");
                        if let Err(e) = scale.restart() {
                            warn!("Couldn't restart scale: {e}");
                            data_entries.push(DataEntry::new(
                                ScaleAction::Offline,
                                0.,
                                scale.get_device(),
                                Database::get_timestamp()?,
                                scale.get_config().location,
                                scale.get_config().ingredient,
                            ))
                        } else if let Ok(weight) = scale.get_weight() {
                            info!("Scale restarted");
                            data_entries.push(DataEntry::new(
                                ScaleAction::Starting,
                                weight.get_amount(),
                                scale.get_device(),
                                Database::get_timestamp()?,
                                scale.get_config().location,
                                scale.get_config().ingredient,
                            ))
                        }
                    }
                    _ => {
                        error!("{}", scale.get_device());
                        return Err(Error::from(e))
                    },
                },
            }
            if let Some((scale_action, delta)) = scale.check_for_action() {
                let scale_config = scale.get_config();
                let data_entry = DataEntry::new(
                    scale_action,
                    delta,
                    scale.get_device(),
                    Database::get_timestamp()?,
                    scale_config.location,
                    scale_config.ingredient,
                );
                data_entries.push(data_entry);
            } else if is_time_for_heartbeat {
                let scale_config = scale.get_config();
                let data_entry = DataEntry::new(
                    ScaleAction::Heartbeat,
                    weights.last().ok_or(Error::Other("Couldn't get weight".into()))?.get_amount(),
                    scale.get_device(),
                    Database::get_timestamp()?,
                    scale_config.location,
                    scale_config.ingredient,
                );
                data_entries.push(data_entry);
            }
            Ok::<(), Error>(())
        })?;
        debug!("{weights:?}");
        database.log_all(data_entries)?;

        while current_time.elapsed() < phidget_sample_period {
            thread::sleep(Duration::from_millis(50));
        }
        current_time = Instant::now();
    }
}
