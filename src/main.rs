use amalgam::{Event, InputType, Result};
use clap::{App, Arg};
use core::convert::TryFrom;
use serde_json::{from_str, Value};
use std::io::{stdin, BufRead};

// At a high level, read:
// - Log information (stdin)
// - Existing configuration (ipset/iptables)
// - Configuration (file)
//
// Ideally, perform actions in streaming mode;
// configuration defines "safe" hosts and networks.
fn main() -> Result<()> {
	let argument_settings = App::new(env!("CARGO_PKG_NAME"))
		.version(env!("CARGO_PKG_VERSION"))
		.author(env!("CARGO_PKG_AUTHORS"))
		.about(env!("CARGO_PKG_DESCRIPTION"))
		.arg(
			Arg::with_name("config")
				.short("c")
				.long("config")
				.required(false)
				.takes_value(true)
				.value_name("FILE")
				.help("Sets a configuration file to load"),
		)
		.get_matches();

	let mut settings = config::Config::default();

	settings.set_default("input.type", "journald-json")?;
	settings.set_default("networks.allowed", vec!["127.0.0.0/8"])?;

	settings.merge(config::Environment::with_prefix("AMALGAM"))?;

	if let Some(config) = argument_settings.value_of("config") {
		settings.merge(config::File::with_name(config))?;
	}

	let events: Vec<Event> = stdin()
		.lock()
		.lines()
		.map(|line| -> Result<Event> {
			let v: Value = from_str(&line?)?;
			Event::try_from(v)
		})
		.filter_map(Result::ok)
		.filter(|event: &amalgam::Event| match event.kind() {
			amalgam::EventKind::Sshd(Some(_)) => true,
			_ => false,
		})
		.inspect(|e| println!("Event: {:?}", e))
		.collect();

	println!("{}", events.len());

	Ok(())
}
