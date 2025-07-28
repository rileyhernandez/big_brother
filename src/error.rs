#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Scale Error: {0}")]
    Scale(#[from] scale::error::Error),
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
    #[error("Failed to start scale")]
    Initialization,
    #[error("System Logging Error: {0}")]
    Syslog(#[from] syslog::Error),
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Env Var Error: {0}")]
    Env(#[from] std::env::VarError),
    #[error("Other: {0}")]
    Other(String)
}
