use std::net::{IpAddr, SocketAddr};

use confindent::Confindent;
use getopts::Options;

pub struct BinConfig {
	pub address: IpAddr,
	pub port: u16,
}

#[allow(clippy::or_fun_call)]
impl BinConfig {
	fn print_usage<S: AsRef<str>>(prgm: S, opts: &Options) {
		let brief = format!("Usage: {} [options]", prgm.as_ref());
		println!("{}", opts.usage(&brief));
	}

	pub fn socket_address(&self) -> SocketAddr {
		SocketAddr::new(self.address, self.port)
	}

	pub fn get() -> Option<Self> {
		let args: Vec<String> = std::env::args().collect();

		let mut opts = Options::new();
		opts.optflag("h", "help", "Print this help message");
		opts.optopt(
			"l",
			"listen-address",
			"The IP address Sail will listen for incoming connections on\nDefault: localhost",
			"IP_ADDR",
		);
		opts.optopt(
			"p",
			"port",
			"The port Sail will listen on\nDefault: 25",
			"PORT",
		);
		opts.optopt(
			"c",
			"config",
			"An alternate location to read the config from\nDefault: /etc/sail/sail.conf",
			"PATH",
		);

		let matches = match opts.parse(&args[1..]) {
			Ok(m) => m,
			Err(_e) => return None,
		};

		if matches.opt_present("help") {
			Self::print_usage(&args[0], &opts);
			return None;
		}

		let conf_path = matches
			.opt_str("config")
			.unwrap_or("/etc/sail/sail.conf".into());

		let config = match Confindent::from_file(conf_path) {
			Ok(c) => c,
			Err(_) => match Confindent::from_file("sail.conf") {
				Ok(c) => c,
				Err(err) => {
					eprintln!("failed to parse conf file: {}", err);
					return None;
				}
			},
		};

		// Options specified on the command line take priority. We only take the
		// cli_key and convert to the config key internally so that we can remain
		// consistent.
		let find_value = |cli_key: &str| -> Option<String> {
			let conf_key: String = cli_key
				.split('-')
				.map(|word| {
					// https://stackoverflow.com/a/38406885
					let mut c = word.chars();
					match c.next() {
						None => String::new(),
						Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
					}
				})
				.collect();

			matches
				.opt_str(cli_key)
				.or(config.child_value(conf_key).map(|s| s.into()))
		};

		let address_string = find_value("listen-address").unwrap_or("localhost".into());
		let address = match address_string.parse() {
			Ok(addr) => addr,
			Err(_e) => {
				eprintln!("Failed to parse '{}' as an IP Address", address_string);
				return None;
			}
		};

		let port_string = find_value("port").unwrap_or("25".into());
		let port = match port_string.parse() {
			Ok(p) => p,
			Err(_e) => {
				eprintln!("Failed to parse '{}' as a port", port_string);
				return None;
			}
		};

		Some(Self { address, port })
	}
}
