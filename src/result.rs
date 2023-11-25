use std::fmt;
use std::fmt::Debug;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

pub type Result<T> = std::result::Result<T, Error>;

impl From<&str> for Error {
    fn from(message: &str) -> Error {
        Error::new(message)
    }
}

impl From<prost::EncodeError> for Error {
    fn from(err: prost::EncodeError) -> Error {
        Error::new(&format!("Encoding error: {}", err))
    }
}

impl From<prost::DecodeError> for Error {
    fn from(err: prost::DecodeError) -> Error {
        Error::new(&format!("Decode error: {}", err))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::new(&format!("IO error: {}", err))
    }
}

impl From<std::sync::mpsc::RecvError> for Error {
    fn from(err: std::sync::mpsc::RecvError) -> Error {
        Error::new(&format!("Recv error: {}", err))
    }
}

impl<T> From<tokio::sync::mpsc::error::SendError<T>> for Error
where
    T: Debug,
{
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Error {
        Error::new(&format!("Send error: {:?}", err))
    }
}
