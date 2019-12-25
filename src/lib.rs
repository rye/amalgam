use config::ConfigError;
use core::fmt::{self, Debug, Display, Formatter};
use core::result;
use std::error;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	InvalidConfig(ConfigError),
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> result::Result<(), fmt::Error> {
		write!(f, "{}", match self {
			Self::InvalidConfig(e) => format!("invalid config: {}", e),
		})
	}
}

impl error::Error for Error {}

impl From<ConfigError> for Error {
	fn from(e: ConfigError) -> Error {
		Error::InvalidConfig(e)
	}
}

pub type Result<T> = result::Result<T, Error>;
