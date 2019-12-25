use config::ConfigError;
use core::fmt::{self, Debug, Display, Formatter};
use core::result;
use std::error;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	InvalidConfig(ConfigError),
	InvalidInputType(String),
	InvalidNetAddr(String),
	Io(std::io::Error),
	Json(serde_json::error::Error),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> result::Result<(), fmt::Error> {
		write!(
			f,
			"{}",
			match self {
				Self::InvalidConfig(e) => format!("invalid config: {}", e),
				Self::InvalidInputType(e) => format!("invalid input type: {}", e),
				Self::InvalidNetAddr(e) => format!("invalid netaddr: {}", e),
				Self::Io(e) => format!("input/output error: {}", e),
				Self::Json(e) => format!("json parse error: {}", e),
			}
		)
	}
}

impl error::Error for Error {}

impl From<ConfigError> for Error {
	fn from(e: ConfigError) -> Error {
		Error::InvalidConfig(e)
	}
}

impl From<netaddr2::Error> for Error {
	fn from(e: netaddr2::Error) -> Error {
		match e {
			netaddr2::Error::ParseError(e) => Error::InvalidNetAddr(e),
		}
	}
}

impl From<serde_json::Error> for Error {
	fn from(e: serde_json::Error) -> Error {
		Error::Json(e)
	}
}

impl From<std::io::Error> for Error {
	fn from(e: std::io::Error) -> Error {
		Error::Io(e)
	}
}

pub type Result<T> = result::Result<T, Error>;
