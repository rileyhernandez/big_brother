use crate::data::{DataAction, DataEntry, Database};
use crate::error::Error;
use log::info;
use menu::device::Device;
use menu::libra::Libra;
use menu::read::Read;
use phidget::{Phidget, devices::VoltageRatioInput};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

const BUFFER_LENGTH: usize = 20;
const MAX_NOISE: f64 = 3.0;
pub struct Scale {
    vin: VoltageRatioInput,
    device: Device,
    gain: f64,
    offset: f64,
    weight_buffer: Vec<f64>,
    last_stable_weight: Option<f64>,
}
impl Scale {
    pub fn new(
        phidget_sn: i32,
        load_cell_id: i32,
        device: Device,
        sample_period: Duration,
        gain: f64,
        offset: f64,
    ) -> Result<Self, Error> {
        let mut vin = VoltageRatioInput::new();
        vin.set_channel(load_cell_id).map_err(Error::Phidget)?;
        vin.set_serial_number(phidget_sn).map_err(Error::Phidget)?;
        vin.open_wait(Duration::from_secs(5))
            .map_err(Error::Phidget)?;
        vin.set_data_interval(sample_period)
            .map_err(Error::Phidget)?;
        info!(
            "Phidget {}, Load Cell {} Connected!",
            vin.serial_number().map_err(Error::Phidget)?,
            vin.channel().map_err(Error::Phidget)?
        );
        sleep(Duration::from_secs(1));
        Ok(Self {
            vin,
            device,
            gain,
            offset,
            weight_buffer: Vec::with_capacity(BUFFER_LENGTH),
            last_stable_weight: None,
        })
    }
    pub fn restart(&mut self) -> Result<(), Error> {
        self.vin.close().map_err(Error::Phidget)?;
        self.vin
            .open_wait(Duration::from_secs(5))
            .map_err(Error::Phidget)?;
        self.weight_buffer.clear();
        self.last_stable_weight = None;
        sleep(Duration::from_secs(2));
        Ok(())
    }
    pub fn get_device(&self) -> Device {
        self.device.clone()
    }
    pub fn get_raw_reading(&self) -> Result<f64, Error> {
        self.vin.voltage_ratio().map_err(Error::Phidget)
    }
    fn get_reading(&self) -> Result<f64, Error> {
        self.get_raw_reading().map(|r| r * self.gain - self.offset)
    }
    fn update_buffer(&mut self, weight: f64) {
        if self.weight_buffer.len() < BUFFER_LENGTH {
            self.weight_buffer.push(weight);
        } else {
            self.weight_buffer.remove(0);
            self.weight_buffer.push(weight);
        }
    }
    fn is_stable(&self) -> bool {
        if self.weight_buffer.len() != BUFFER_LENGTH {
            return false;
        }
        let max = self
            .weight_buffer
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let min = self
            .weight_buffer
            .iter()
            .fold(f64::INFINITY, |a, &b| a.min(b));
        max - min < MAX_NOISE
    }
    pub fn get_weight(&mut self) -> Result<Weight, Error> {
        let reading = self.get_reading()?;
        self.update_buffer(reading);
        if self.is_stable() {
            Ok(Weight::Stable(reading))
        } else {
            Ok(Weight::Unstable(reading))
        }
    }
    pub fn check_for_action(&mut self) -> Option<DataEntry> {
        if self.is_stable() {
            let last = self.weight_buffer.last().unwrap();
            if let Some(last_stable) = self.last_stable_weight {
                let delta = last - last_stable;
                if delta.abs() > MAX_NOISE {
                    println!("New stable value: {:?}g", last);
                    println!("Delta since last stable: {}", delta);
                    self.last_stable_weight = Some(*last);
                    let action = {
                        if delta > 0. {
                            DataAction::Refilled
                        } else {
                            DataAction::Served
                        }
                    };
                    return Some(DataEntry::new(
                        action,
                        delta,
                        self.device.clone(),
                        Database::get_timestamp().unwrap(),
                        "Caldo HQ".into(),
                        "Fake Chicken Wings".into(),
                    ));
                }
            }
            self.last_stable_weight = Some(*last);
        }
        None
    }
    fn from_libra_menu(libra: Libra) -> Result<Self, Error> {
        Self::new(
            libra.config.phidget_id,
            libra.config.load_cell_id,
            libra.device,
            Duration::from_millis(250),
            libra.config.gain,
            libra.config.offset,
        )
    }
    pub fn from_config(path: &Path) -> Result<Vec<Self>, Error> {
        Libra::read_as_vec(path)?
            .into_iter()
            .map(Self::from_libra_menu)
            .collect()
    }
}
#[derive(Debug)]
pub enum Weight {
    Stable(f64),
    Unstable(f64),
}
impl Weight {
    pub fn get_amount(&self) -> f64 {
        match self {
            Weight::Stable(value) => *value,
            Weight::Unstable(value) => *value,
        }
    }
}
impl std::fmt::Display for Weight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Weight::Stable(w) => write!(f, "Stable: {} g", w.trunc() as usize),
            Weight::Unstable(w) => write!(f, "Unstable: {} g", w.trunc() as usize),
        }
    }
}
