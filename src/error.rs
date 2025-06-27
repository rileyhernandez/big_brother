use thiserror;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Phidget Error: {0}")]
    Phidget(#[from] phidget::Error),
    #[error("Scale Weighing Timed Out!")]
    ScaleTimeout,
    #[error("This feature is not yet implemented!")]
    NotImplemented,
    #[error("Rusqlite Error: {0}")]
    Rusqlite(#[from] rusqlite::Error),
    #[error("Time Offset Error: {0}")]
    TimeOffset(#[from] time::error::IndeterminateOffset),
    #[error("Reqwest Error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Serde Json Error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("Date Formatting Error: {0}")]
    DateFormat(#[from] time::error::Format),
    #[error("Menu Error: {0}")]
    Menu(#[from] menu::error::Error),
}
