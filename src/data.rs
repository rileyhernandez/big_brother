use crate::error::Error;
use crate::error::Error::{Rusqlite, TimeOffset};
use menu::device::Device;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use time::{OffsetDateTime, format_description::well_known::Iso8601};
use scale::scale::Action as ScaleAction;

pub struct Database {
    connection: Connection,
}
impl Database {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let connection = Connection::open(path).map_err(Rusqlite)?;
        connection
            .execute(
                "CREATE TABLE IF NOT EXISTS libra_logs (
                model TEXT NOT NULL,
                number TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                action TEXT NOT NULL,
                amount NUMBER NOT NULL,
                location TEXT NOT NULL,
                ingredient TEXT NOT NULL,
                synced INTEGER NOT NULL DEFAULT 0 CHECK (synced IN (0, 1))
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
                "INSERT INTO libra_logs (model, number, timestamp, action, amount, location, ingredient, synced) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    format!("{:?}", data_entry.device.model),
                    data_entry.device.number.to_string(),
                    data_entry.timestamp.format(&Iso8601::DEFAULT)?,
                    data_entry.scale_action.to_string(),
                    data_entry.amount,
                    data_entry.location,
                    data_entry.ingredient,
                    0, // sync bool
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
pub struct DataEntry {
    scale_action: ScaleAction,
    amount: f64,
    device: Device,
    timestamp: OffsetDateTime,
    location: String,
    ingredient: String,
}
impl DataEntry {
    pub fn new(
        scale_action: ScaleAction,
        amount: f64,
        device: Device,
        timestamp: OffsetDateTime,
        location: String,
        ingredient: String,
    ) -> Self {
        Self {
            scale_action,
            amount,
            device,
            timestamp,
            location,
            ingredient,
        }
    }
}
