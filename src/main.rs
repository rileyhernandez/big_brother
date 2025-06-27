use crate::data::{Backend, DataAction, Database, Log};
use crate::error::Error;
use crate::scale::{Scale, Weight};
use menu::device::{Device, Model};
use std::path::Path;
use std::time::Duration;
use std::{env, thread};
use tokio;
use tokio::time::MissedTickBehavior;

mod config;
mod data;
mod error;
mod scale;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // TODO: log boot in db

    let current_directory = env::current_dir().unwrap();
    let config_path = current_directory.join("config.toml");
    let mut scales = Scale::from_config(&*config_path)?;

    let database_path = current_directory.join("data.db");
    let database = Database::new(database_path)?;

    let mut interval = tokio::time::interval(Duration::from_millis(500));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        let weights = scales
            .iter_mut()
            .map(|scale| {
                let weight = scale.get_weight();
                scale.check_last_stable();
                weight
            })
            .collect::<Result<Vec<Weight>, Error>>()?;
        println!("{:?}", weights);
        let log = Log::new(DataAction::RanOut, 69., scales[0].get_device().clone());
        let logs = [log];
        database.log_all(&logs)?;
        interval.tick().await;
    }

    // TODO: erroring out logging in db
}

async fn demo(mut scales: Vec<Scale>) -> Result<(), Error> {
    let backend = Backend::new("http://127.0.0.1:8080/");

    let mut interval = tokio::time::interval(Duration::from_millis(500));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        let readings = scales
            .iter_mut()
            .map(Scale::get_weight)
            .collect::<Result<Vec<Weight>, Error>>()?;
        println!("{:?}", readings);
        if let Err(e) = backend.post(readings).await {
            eprintln!("{}", e);
        };
        interval.tick().await;
    }
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
