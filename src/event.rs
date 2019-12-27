use core::convert::TryFrom;
use core::str::FromStr;
use std::net::{IpAddr, SocketAddr};

use chrono::{DateTime, TimeZone, Utc};

use regex::{Captures, Regex};
use crate::error::{Error, Result};

#[derive(Debug, PartialEq)]
pub struct Login {
	host: SocketAddr,
	user: String,
}

#[derive(Debug, PartialEq)]
pub enum SshdEventKind {
	FailedLogin(Login),
	SuccessfulLogin(Login),
}

const SSHD_LOGIN: &'static str = "(?P<action>Failed|Accepted) \
                                  (?P<thing>password|publickey|none) \
                                  for \
                                  (invalid user )?\
                                  (?P<user>\\w+) \
                                  from \
                                  (?P<host>[a-f\\d:\\.]+) \
                                  port \
                                  (?P<port>\\d+)";

lazy_static::lazy_static! {
	static ref SSHD_LOGIN_RE: Regex = Regex::new(SSHD_LOGIN).unwrap();
}

fn realtime_timestamp_to_datetime<Tz>(ts: &str) -> DateTime<Tz>
where
	Tz: TimeZone,
	DateTime<Tz>: From<DateTime<Utc>>,
{
	let ts: u64 = ts.parse().unwrap();

	use core::convert::TryInto;

	let secs: i64 = (ts / 1_000_000).try_into().unwrap();
	let nanos: u32 = (ts % 1_000_000).try_into().unwrap();

	Utc.timestamp(secs, nanos).into()
}

impl FromStr for SshdEventKind {
	type Err = Error;

	fn from_str(s: &str) -> Result<SshdEventKind> {
		use SshdEventKind::*;

		let captures: Captures = SSHD_LOGIN_RE
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

#[derive(Debug, PartialEq)]
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

impl TryFrom<serde_json::Value> for Event {
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

		let time: DateTime<Utc> = realtime_timestamp_to_datetime(time);

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
