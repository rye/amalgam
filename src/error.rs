use core::{fmt, result};

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	InvalidConfig(config::ConfigError),
	InvalidInputType(String),
	InvalidNetAddr(String),
	Io(std::io::Error),
	Json(serde_json::error::Error),
	MalformedEvent(String),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
	fn fmt(&self, f: &mut fmt::Formatter) -> result::Result<(), fmt::Error> {
		write!(
			f,
			"{}",
			match self {
				Self::InvalidConfig(e) => format!("invalid config: {}", e),
				Self::InvalidInputType(e) => format!("invalid input type: {}", e),
				Self::InvalidNetAddr(e) => format!("invalid netaddr: {}", e),
				Self::Io(e) => format!("input/output error: {}", e),
				Self::Json(e) => format!("json parse error: {}", e),
				Self::MalformedEvent(e) => format!("malformed message: {}", e),
			}
		)
	}
}


impl From<config::ConfigError> for Error {
	fn from(e: config::ConfigError) -> Error {
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
