use config::ConfigError;
use core::fmt::{self, Debug, Display, Formatter};
use core::result;
use std::error;

#[derive(Debug)]
pub struct Message {
	timestamp: chrono::DateTime<chrono::Utc>,
	message: String,
	ident: String,
	raw: serde_json::Value,
}

#[derive(Debug)]
pub struct SshdMessage(Message);

#[derive(Debug)]
pub struct FailedLoginSshdMessage(SshdMessage);

#[derive(Debug)]
pub enum EventKind {
	FailedLogin,
	SuccessfulLogin,
}

#[derive(Debug)]
pub struct Event {
	kind: EventKind,
	message: Message,
}

impl core::convert::TryFrom<Message> for SshdMessage {
	type Error = Error;

	fn try_from(msg: Message) -> Result<SshdMessage> {
		match msg.ident.as_str() {
			"sshd" => Ok(SshdMessage(msg)),
			_ => Err(Error::WrongUnit(msg.ident)),
		}
	}
}

impl core::convert::TryFrom<SshdMessage> for FailedLoginSshdMessage {
	type Error = Error;

	fn try_from(msg: SshdMessage) -> Result<FailedLoginSshdMessage> {
		let message: &String = &msg.0.message;

		if message.starts_with("Failed") {
			Ok(FailedLoginSshdMessage(msg))
		} else {
			Err(Error::NotFailedLogin(msg))
		}
	}
}

impl core::convert::TryFrom<serde_json::Value> for Message {
	type Error = Error;

	fn try_from(raw: serde_json::Value) -> Result<Message> {
		let k: &serde_json::Map<String, serde_json::Value> = match &raw {
			serde_json::Value::Object(obj) => obj,
			_ => unimplemented!(),
		};

		let timestamp: String = k
			.get("__REALTIME_TIMESTAMP")
			.ok_or(Error::MalformedMessage("missing timestamp"))?
			.as_str()
			.unwrap()
			.to_string();
		let timestamp: u64 = timestamp
			.parse()
			.or(Err(Error::MalformedMessage("malformed timestamp")))?;

		use core::convert::TryInto;
		let secs: i64 = (timestamp / 1_000_000).try_into().unwrap();
		let nanos: u32 = (timestamp % 1_000_000).try_into().unwrap();

		use chrono::offset::TimeZone;
		let timestamp: chrono::DateTime<chrono::Utc> = chrono::Utc.timestamp(secs, nanos);

		let message: String = k
			.get("MESSAGE")
			.ok_or(Error::MalformedMessage("missing message"))?
			.as_str()
			.unwrap()
			.to_string();

		let ident: String = k
			.get("SYSLOG_IDENTIFIER")
			.ok_or(Error::MalformedMessage("missing syslog identifier"))?
			.as_str()
			.unwrap()
			.to_string();

		Ok(Message {
			ident,
			message,
			timestamp,
			raw
		})
	}
}

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
	InvalidConfig(ConfigError),
	InvalidInputType(String),
	InvalidNetAddr(String),
	Io(std::io::Error),
	Json(serde_json::error::Error),
	MalformedMessage(&'static str),
	NotFailedLogin(SshdMessage),
	WrongUnit(String),
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
				Self::MalformedMessage(e) => format!("malformed message: {}", e),
				Self::NotFailedLogin(e) => format!("not a failed login: {:?}", e),
				Self::WrongUnit(e) => format!("wrong unit: {}", e),
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

#[derive(Debug)]
#[non_exhaustive]
pub enum InputType {
	JournaldJson,
}

impl core::str::FromStr for InputType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		match s.trim() {
			"journald-json" => Ok(Self::JournaldJson),
			b => Err(Error::InvalidInputType(b.to_string())),
		}
	}
}

pub type Result<T> = result::Result<T, Error>;
