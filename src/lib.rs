use core::str::FromStr;

mod event;
pub use event::{Event, EventKind, Login};

mod error;
pub use error::{Error, Result};

#[derive(Debug)]
#[non_exhaustive]
pub enum InputType {
	JournaldJson,
}

impl FromStr for InputType {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self> {
		match s.trim() {
			"journald-json" => Ok(Self::JournaldJson),
			b => Err(Error::InvalidInputType(b.to_string())),
		}
	}
}

use std::net::IpAddr;
use std::collections::HashMap;

pub type Host = IpAddr;
pub type History = HashMap<Host, Vec<Event>>;
