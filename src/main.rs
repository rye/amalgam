use amalgam::Result;
use clap::{App, Arg};

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

	// Order of preference (ascending):
	// - Defaults
	// - Environment-specified things
	// - Config file specified things
	// - Argument-specified things
	let mut settings = config::Config::default();

	// Set defaults
	settings.set_default("input.type", "journald-json")?;

	// Load from the environment
	settings.merge(config::Environment::with_prefix("AMALGAM"))?;

	// Load from config file, if specified.
	if let Some(config) = argument_settings.value_of("config") {
		settings.merge(config::File::with_name(config))?;
	}

	// (If anything in argument_settings gets specified, those go here.)

	// Finally, `settings` is now ready for consumption.

	Ok(())
}
