use std::io;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("input/output error")]
    Io(#[from] io::Error),
    #[error("unexpected error")]
    Unexpected,
}

impl From<std::convert::Infallible> for Error {
    fn from(_err: std::convert::Infallible) -> Self {
        Error::Unexpected
    }
}
