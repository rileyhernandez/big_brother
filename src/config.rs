use std::env;
use log::info;
use menu::device::{Device, Model};
use menu::libra::Libra;
use crate::error::Error;
#[cfg(feature = "config")]
use menu::pull;
const CONFIG_PATH: &str = "http://127.0.0.1:8080";

#[cfg(feature = "config")]
pub fn get_config_from_cloud() -> Result<(), Error> {
    let current_directory = env::current_dir()?;
    let config_path = current_directory.join("config.toml");
    // if config_path.exists() {
    if false {
        info!("config.toml already exists");
        Err(Error::Initialization)
    } else {
        info!("Compiling config.toml...");
        let test_device = Device::new(Model::LibraV0, 0);
        let a = Libra::pull_from_backend(test_device, CONFIG_PATH)?;
        println!("{:?}", a);
        Ok(())
    }
}