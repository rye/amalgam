use config::ConfigError;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	ConfigInvalidError(ConfigError),
}

impl core::fmt::Display for Error {
	fn fmt(&self, f: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
		write!(f, "{}", match self {
			Self::ConfigInvalidError(e) => format!("invalid config: {}", e),
		})
	}
}

impl std::error::Error for Error {}

impl From<ConfigError> for Error {
	fn from(e: ConfigError) -> Error {
		Error::ConfigInvalidError(e)
	}
}

pub type Result<T> = core::result::Result<T, Error>;
