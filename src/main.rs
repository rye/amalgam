use std::collections::HashMap;

// At a high level, read:
// - Log information (stdin)
// - Existing configuration (ipset/iptables)
// - Configuration (file)
//
// Ideally, perform actions in streaming mode;
// configuration defines "safe" hosts and networks.
fn main() {
	let mut settings = config::Config::default();
	settings
		.merge(config::File::with_name("config"))
		.unwrap()
		.merge(config::Environment::with_prefix("AMALGAM"))
		.unwrap();

	println!(
		"{:?}",
		settings.try_into::<HashMap<String, String>>().unwrap()
	);
}
