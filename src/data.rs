use crate::error::Error;
use crate::error::Error::{Rusqlite, TimeOffset};
use menu::device::Device;
use rusqlite::{Connection, params};
use std::path::PathBuf;
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
                amount NUMBER NOT NULL,
                location TEXT NOT NULL,
                ingredient TEXT NOT NULL
            )",
                [],
            )
            .map_err(Rusqlite)?;
        Ok(Self { connection })
    }
    pub fn get_timestamp() -> Result<OffsetDateTime, Error> {
        OffsetDateTime::now_local().map_err(TimeOffset)
    }
    pub fn log(&self, data_entry: &DataEntry) -> Result<(), Error> {
        self.connection
            .execute(
                "INSERT INTO libra_logs (scale, timestamp, action, amount, location, ingredient) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    data_entry.device.to_string(),
                    data_entry.timestamp.format(&Iso8601::DEFAULT)?,
                    data_entry.data_action.to_string(),
                    data_entry.amount,
                    data_entry.location,
                    data_entry.ingredient
                ],
            )
            .map_err(Rusqlite)?;
        Ok(())
    }
    pub fn log_all(&self, data_entries: Vec<DataEntry>) -> Result<(), Error> {
        let _ = data_entries
            .iter()
            .map(|log| self.log(log))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(())
    }
}

pub enum DataAction {
    Served,
    RanOut,
    Refilled,
    Starting,
}
impl std::fmt::Display for DataAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DataAction::Served => write!(f, "Served"),
            DataAction::RanOut => write!(f, "Ran Out"),
            DataAction::Refilled => write!(f, "Refilled"),
            DataAction::Starting => write!(f, "Starting"),
        }
    }
}
pub struct DataEntry {
    data_action: DataAction,
    amount: f64,
    device: Device,
    timestamp: OffsetDateTime,
    location: String,
    ingredient: String,
}
impl DataEntry {
    pub fn new(
        data_action: DataAction,
        amount: f64,
        device: Device,
        timestamp: OffsetDateTime,
        location: String,
        ingredient: String,
    ) -> Self {
        Self {
            data_action,
            amount,
            device,
            timestamp,
            location,
            ingredient,
        }
    }
}
