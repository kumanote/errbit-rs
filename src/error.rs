pub use anyhow::Result;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
    #[error("IO error: {reason:?}")]
    IO { reason: String },
    #[error("API response error: [{status_code:?}]{reason:?}")]
    Gateway { status_code: u16, reason: String },
}
