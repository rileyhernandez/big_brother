use crate::error::Error;
use crate::error::Error::{DateFormat, Reqwest, Rusqlite, TimeOffset};
use crate::scale::Weight;
use menu::device::Device;
use reqwest::Url;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use time::{OffsetDateTime, format_description::well_known::Iso8601};

pub struct Database {
    connection: Connection,
}
impl Database {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let connection = Connection::open(path).map_err(Rusqlite)?;
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS libra_logs (
                scale TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                amount NUMBER NOT NULL
            )",
                [],
            )
            .map_err(Rusqlite)?;
        Ok(Self { connection })
    }
    fn get_timestamp() -> Result<String, Error> {
        OffsetDateTime::now_local()
            .map_err(TimeOffset)?
            .format(&Iso8601::DEFAULT)
            .map_err(DateFormat)
    }
    pub fn log(&self, log: &Log, timestamp: &str) -> Result<(), Error> {
        let now = Database::get_timestamp()?;
        self.connection
            .execute(
                "INSERT INTO libra_logs (scale, timestamp, action, amount) VALUES (?1, ?2, ?3, ?4)",
                params![log.device.to_string(), now, log.data_action.to_string(), log.amount],
            )
            .map_err(Rusqlite)?;
        Ok(())
    }
    pub fn log_all(&self, logs: &[Log]) -> Result<(), Error> {
        let now = Database::get_timestamp()?;
        let _ = logs
            .iter()
            .map(|log| {
                self.log(log, &now)
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}

pub enum DataAction {
    Served,
    RanOut,
    Refilled,
}
impl std::fmt::Display for DataAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataAction::Served => write!(f, "Served"),
            DataAction::RanOut => write!(f, "Ran Out"),
            DataAction::Refilled => write!(f, "Refilled"),
        }
    }
}
pub struct Log {
    data_action: DataAction,
    amount: f64,
    device: Device,
}
impl Log {
    pub fn new(data_action: DataAction, amount: f64, device: Device) -> Self {
        Self {
            data_action,
            amount,
            device,
        }
    }
}
pub struct Backend {
    client: reqwest::Client,
    url: Url,
}
impl Backend {
    pub fn new(url: &str) -> Self {
        let url = Url::from_str(url).unwrap();
        Self {
            client: reqwest::Client::new(),
            url,
        }
    }
    pub async fn post(&self, weights: Vec<Weight>) -> Result<(), Error> {
        for (id, weight) in weights.iter().enumerate() {
            let path = format!("{id}");
            self.client
                .post(self.url.clone().join(&path).unwrap())
                .body(weight.to_json_string()?)
                .timeout(Duration::from_millis(250))
                .header("Content-Type", "application/json")
                .send()
                .await
                .map_err(Reqwest)?;
        }
        Ok(())
    }
}
