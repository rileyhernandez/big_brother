use crate::data::{DataAction, DataEntry, Database};
use crate::error::Error;
use crate::scale::{DisconnectedScale, Scale};
use std::time::{Duration, Instant};
use std::{env, thread};
mod data;
mod error;
mod scale;

fn main() -> Result<(), Error> {
    // TODO: log boot in db

    let current_directory = env::current_dir().unwrap();
    let config_path = current_directory.join("config.toml");

    //todo: need to do this
    let mut scales = Scale::from_config(&config_path)?;
    let disconnected_scales = DisconnectedScale::from_config(&config_path)?;
    let mut scales = disconnected_scales
        .into_iter()
        .map( |disconnected_scale| {
            match disconnected_scale.connect() {
                Ok(scale) => Ok(scale),
                Err(e) => {
                    match e {
                        Error::Phidget(phidget_error) => {
                            todo!()
                        }
                        _ => Err(e)
                    }
                }
            }
        })
        .collect::<Result<Vec<_>, Error>>()?;

    let database_path = current_directory.join("data.db");
    let database = Database::new(database_path)?;

    let initial_data_entries: Vec<DataEntry> = scales
        .iter_mut()
        .map(|scale| match scale.get_weight() {
            Ok(weight) => Ok(DataEntry::new(
                DataAction::Starting,
                weight.get_amount(),
                scale.get_device(),
                Database::get_timestamp().expect("Couldn't get timestamp"),
                "Caldo HQ".into(),
                "Fake Chicken Wings".into(),
            )),
            Err(e) => {
                scale.log_error(e);
                Err(Error::Initialization)
            }
        })
        .filter_map(|result| {
            if let Err(e) = result {
                match e {
                    Error::Initialization => None,
                    _ => panic!("Unrecoverable error: {:?}", e),
                }
            } else {
                Some(result)
            }
        })
        .collect::<Result<_, _>>()?;
    database.log_all(initial_data_entries)?;

    let mut current_time = Instant::now();
    loop {
        // todo: take out weights before prod
        let mut weights = Vec::with_capacity(scales.len());
        let mut data_entries = Vec::with_capacity(scales.len());
        scales.iter_mut().try_for_each(|scale| {
            match scale.get_weight() {
                Ok(weight) => weights.push(weight),
                Err(e) => match e {
                    Error::Phidget(err) => {
                        eprintln!("Phidget error: {:?}", err);
                        println!("Restarting scale...");
                        if let Err(e) = scale.restart() {
                            eprintln!("Couldn't restart scale: {:?}", e);
                        } else if let Ok(weight) = scale.get_weight() {
                            println!("Scale restarted");
                            data_entries.push(DataEntry::new(
                                DataAction::Starting,
                                weight.get_amount(),
                                scale.get_device(),
                                Database::get_timestamp()?,
                                "Caldo HQ".into(),
                                "Fake Chicken Wings".into(),
                            ))
                        }
                    }
                    _ => {
                        eprintln!("Unrecoverable error: {:?}", e);
                        return Err(e);
                    }
                },
            }
            if let Some(data_entry) = scale.check_for_action() {
                data_entries.push(data_entry);
            }
            Ok::<(), Error>(())
        })?;
        println!("{:?}", weights);
        database.log_all(data_entries)?;

        while current_time.elapsed() < Duration::from_millis(1000) {
            thread::sleep(Duration::from_millis(250));
        }
        current_time = Instant::now();
    }

    // TODO: erroring out logging in db
}

#[cfg(test)]
mod tests {
    use crate::scale::Scale;
    use std::path::Path;

    #[test]
    fn menu() {
        let path = Path::new("/home/riley/Downloads/test/config.toml");
        let _ = Scale::from_config(path);
    }
}
