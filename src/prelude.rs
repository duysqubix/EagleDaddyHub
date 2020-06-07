use crate::api;

#[derive(Debug)]
pub enum Error {
    SerialError(serialport::Error),
    IOError(std::io::Error),
    DecodeError(std::str::Utf8Error),
    ApiError(api::Error),
    InvalidMode(String),
}

impl From<serialport::Error> for Error {
    fn from(err: serialport::Error) -> Self {
        Error::SerialError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(err: std::str::Utf8Error) -> Self {
        Error::DecodeError(err)
    }
}

impl From<api::Error> for Error {
    fn from(err: api::Error) -> Self {
        Error::ApiError(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Error::SerialError(ref err) => write!(f, "{}", err),
            Error::IOError(ref err) => write!(f, "{}", err),
            Error::DecodeError(ref err) => write!(f, "{}", err),
            Error::InvalidMode(ref err) => write!(f, "{}", err),
            Error::ApiError(ref err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;
