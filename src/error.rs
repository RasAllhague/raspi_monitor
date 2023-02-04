use std::{fmt, error::Error};

#[derive(Debug)]
pub enum SysInfoError {
    Io(std::io::Error),
    Serde(serde_json::error::Error),
}

impl fmt::Display for SysInfoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SysInfoError::Serde(err) => write!(f, "serde_json error: {err}"),
            SysInfoError::Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl Error for SysInfoError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            SysInfoError::Serde(err) => Some(err),
            SysInfoError::Io(err) => Some(err),
        }
    }
}

impl From<std::io::Error> for SysInfoError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::error::Error> for SysInfoError {
    fn from(e: serde_json::error::Error) -> Self {
        Self::Serde(e)
    }
}