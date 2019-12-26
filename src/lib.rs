use config::ConfigError;
use core::fmt::{self, Debug, Display, Formatter};
use core::result;
use std::error;
use std::net::{IpAddr, SocketAddr};

#[derive(Debug)]
pub struct Login {
	host: SocketAddr,
	user: String,
}

lazy_static::lazy_static! {
	static ref SSHD_LOGIN_RE: regex::Regex = regex::Regex::new(r"(?P<action>Failed|Accepted) (?P<thing>password|publickey|none) for (invalid user )?(?P<user>\w+) from (?P<host>[a-f\d:\.]+) port (?P<port>\d+)").unwrap();
}

impl std::str::FromStr for SshdEventKind {
	type Err = Error;

	fn from_str(s: &str) -> Result<SshdEventKind> {
		use SshdEventKind::*;

		let captures: regex::Captures = SSHD_LOGIN_RE
			.captures(s)
			.ok_or(Error::MalformedEvent(format!("{} did not match regex", s)))?;

		let action: Option<&str> = captures.name("action").map(Into::into);
		let thing: Option<&str> = captures.name("thing").map(Into::into);
		let user: Option<&str> = captures.name("user").map(Into::into);
		let host: Option<&str> = captures.name("host").map(Into::into);
		let port: Option<&str> = captures.name("port").map(Into::into);

		match (action, thing, user, host, port) {
			(Some("Failed"), _, Some(user), Some(host), Some(port))
			| (Some("Accepted"), _, Some(user), Some(host), Some(port)) => {
				let host: IpAddr = host.parse::<IpAddr>().unwrap();
				let port: u16 = port.parse::<u16>().unwrap();

				let host: SocketAddr = SocketAddr::new(host, port);
				let user: String = user.to_string();

				let login: Login = Login { host, user };
				Ok(match action {
					Some("Failed") => FailedLogin(login),
					Some("Accepted") => SuccessfulLogin(login),
					_ => unreachable!(),
				})
			}
			_ => todo!(),
		}
	}
}

#[derive(Debug)]
pub enum SshdEventKind {
	FailedLogin(Login),
	SuccessfulLogin(Login),
}

#[derive(Debug)]
pub enum EventKind {
	Sshd(Option<SshdEventKind>),
}

#[derive(Debug)]
pub struct Event {
	ident: String,
	kind: EventKind,
	raw: serde_json::Value,
	time: chrono::DateTime<chrono::Utc>,
}

impl Event {
	pub fn kind(&self) -> &EventKind {
		&self.kind
	}
}

fn realtime_timestamp_to_datetime<Tz>(ts: &str) -> chrono::DateTime<Tz>
where
	Tz: chrono::offset::TimeZone,
	chrono::DateTime<Tz>: core::convert::From<chrono::DateTime<chrono::Utc>>,
{
	let ts: u64 = ts.parse().unwrap();

	use core::convert::TryInto;
	let secs: i64 = (ts / 1_000_000).try_into().unwrap();
	let nanos: u32 = (ts % 1_000_000).try_into().unwrap();

	use chrono::offset::TimeZone;
	chrono::Utc.timestamp(secs, nanos).into()
}

impl core::convert::TryFrom<serde_json::Value> for Event {
	type Error = Error;

	fn try_from(raw: serde_json::Value) -> Result<Event> {
		let obj: &serde_json::Map<String, serde_json::Value> = match &raw {
			serde_json::Value::Object(obj) => obj,
			_ => unimplemented!(),
		};

		let time: &str = obj
			.get("__REALTIME_TIMESTAMP")
			.ok_or(Error::MalformedEvent("missing timestamp".to_string()))?
			.as_str()
			.unwrap();

		let time: chrono::DateTime<chrono::Utc> = realtime_timestamp_to_datetime(time);

		let ident: String = obj
			.get("SYSLOG_IDENTIFIER")
			.ok_or(Error::MalformedEvent(
				"missing syslog identifier".to_string(),
			))?
			.as_str()
			.unwrap()
			.to_string();

		let kind: EventKind = match ident.as_str() {
			"sshd" => {
				use EventKind::Sshd;

				let message: &str = obj
					.get("MESSAGE")
					.ok_or(Error::MalformedEvent("missing message field".to_string()))?
					.as_str()
					.unwrap();

				if let Ok(sek) = message.parse::<SshdEventKind>() {
					Sshd(Some(sek))
				} else {
					Sshd(None)
				}
			}
			_ => unimplemented!(),
		};

		Ok(Event {
			ident,
			kind,
			raw,
			time,
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
	MalformedEvent(String),
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
				Self::MalformedEvent(e) => format!("malformed message: {}", e),
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
