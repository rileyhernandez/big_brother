use crate::data::DataAction;
use crate::error::Error;
use log::info;
use menu::device::Device;
use menu::libra::Libra;
use menu::read::Read;
use phidget::{Phidget, devices::VoltageRatioInput};
use serde_json::{Value as JsonValue, json};
use std::path::Path;
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
        std::thread::sleep(Duration::from_secs(1));
        Ok(Self {
            vin,
            device,
            gain,
            offset,
            weight_buffer: Vec::with_capacity(BUFFER_LENGTH),
            last_stable_weight: None,
        })
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
    pub fn check_last_stable(&mut self) -> Option<DataAction> {
        if self.is_stable() {
            let last = self.weight_buffer.last().unwrap();
            if let Some(last_stable) = self.last_stable_weight {
                if (last_stable - last).abs() > MAX_NOISE {
                    self.last_stable_weight = Some(*last);
                    println!("New stable value: {:?}g", last);
                    println!("Delta since last stable: {}", last - last_stable);
                }
                None
            } else {
                self.last_stable_weight = Some(*last);
                None
            }
        } else {
            None
        }
    }
    // pub fn weight_once_stable(&mut self, timeout: Duration) -> Result<f64, Error> {
    //     // self.weight_buffer = Vec::with_capacity(BUFFER_LENGTH);
    //     let start_time = std::time::Instant::now();
    //     loop {
    //         let weight = self.get_weight()?;
    //         if let Weight::Stable(w) = weight {
    //             return Ok(w);
    //         }
    //         if start_time.elapsed() > timeout {
    //             return Err(Error::ScaleTimeout);
    //         }
    //         std::thread::sleep(Duration::from_millis(250));
    //     }
    // }
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
    pub fn to_json_value(&self) -> JsonValue {
        match self {
            Weight::Stable(value) => json!({ "stable": value }),
            Weight::Unstable(value) => json!({ "unstable": value }),
        }
    }
    pub fn to_json_string(&self) -> Result<String, Error> {
        serde_json::to_string(&self.to_json_value()).map_err(Error::SerdeJson)
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
