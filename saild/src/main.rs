mod net;
mod sailconfig;

use sailconfig::SailConfig;

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use sail::config::Config;
use sail::smtp::{
	args::{Domain, ForeignPath, ForwardPath},
	ForeignMessage, Message,
};

use confindent::Confindent;
use getopts::Options;

use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::watch;

struct Sail {
	config: Arc<SailConfig>,
	sender: mpsc::UnboundedSender<Message>,
	local_messages: Vec<Message>,
	foreign_messages: Vec<(Domain, ForeignMessage)>,
	rx: watch::Receiver<bool>,
}

impl Sail {
	async fn receive_messages(
		mut self,
		mut receiver: mpsc::UnboundedReceiver<Message>,
		sender: mpsc::UnboundedSender<Message>,
	) {
		loop {
			let message = tokio::select! {
				_ = self.rx.changed() => break,
				Some(message) = receiver.recv() => message
			};

			self.handle_message(message);

			// Here we'd check if we relay or save and act appropriately. but FIRST we should write
			// it to the FS as the RFC says that we should not lose messages if we crash. Maybe we
			// try once, as that shouldn't take long, and then if we fail we write?

			// put the runner in Client, or another struct that sits above client.
			// it should try once, then write to disk and sleep for a while.
			// rfc guidelines help.
			// client should be like server; not handling any networking or async anything, just interacting with strings.
		}
	}

	// filters local messages from foreign (to be relayed) messages
	fn handle_message(&mut self, message: Message) {
		let (reverse, forwards, data) = message.into_parts();

		let mut foreign_map: HashMap<Domain, Vec<ForeignPath>> = HashMap::new();
		let locals: Vec<ForwardPath> = forwards
			.into_iter()
			.filter(|forward| {
				if self.config.forward_path_is_local(forward) {
					true // locals stay in the vec
				} else if let ForwardPath::Regular(path) = forward {
					// get the vector for a specific domain, but if there isn't one, make it.
					match foreign_map.get_mut(&path.domain) {
						Some(vec) => vec.push(ForeignPath { 0: path.clone() }),
						None => {
							foreign_map
								.insert(path.domain.clone(), vec![ForeignPath { 0: path.clone() }]);
						}
					}

					false
				} else {
					// should've been caught by forward_path_is_local, maybe
					// print a warning if we reach here?
					true
				}
			})
			.collect();

		let domains_messages: Vec<(Domain, ForeignMessage)> = foreign_map
			.into_iter()
			.map(|(domain, foreign_paths)| {
				(
					domain,
					ForeignMessage::from_parts(reverse.clone(), foreign_paths, data.clone()),
				)
			})
			.collect();

		if !locals.is_empty() {
			tokio::spawn(Self::deliver_local(
				Message {
					reverse_path: reverse,
					forward_paths: locals,
					data,
				},
				self.rx.clone(),
			));
		}

		for (domain, message) in domains_messages {
			tokio::spawn(sail::net::relay(domain, message /*, self.rx.clone()*/));
		}
	}

	//todo: save to fs. Return an undeliverable message if we can't. remember to watch for shutdown signals
	async fn deliver_local(message: Message, rx: watch::Receiver<bool>) -> Option<Message> {
		let (reverse, forwards, data) = message.into_parts();

		print!("REVERSE: {}\nLOCAL TO:", reverse);
		for path in forwards {
			print!(" {}", path);
		}
		print!("\n{}", data);

		None
	}
}

#[tokio::main]
async fn main() {
	let binconf = match BinConfig::get() {
		Some(conf) => conf,
		None => return,
	};

	let listener = TcpListener::bind(binconf.socket_address()).await.unwrap();

	// Quick, bad config based on port for testing
	let config = match binconf.port {
		25 => SailConfig {
			hostnames: vec!["localhost".parse().unwrap()],
			relays: vec!["nove.dev".parse().unwrap(), "genbyte.dev".parse().unwrap()],
			users: vec!["genny".parse().unwrap(), "devon".parse().unwrap()],
		},
		_ => SailConfig {
			hostnames: vec!["localhost.localdomain".parse().unwrap()],
			users: vec!["alice".parse().unwrap(), "bob".parse().unwrap()],
			relays: vec!["localhost".parse().unwrap()],
		},
	};

	let (tx, rx) = tokio::sync::watch::channel(false);

	let (sender, receiver) = mpsc::unbounded_channel();
	let sail = Sail {
		config: Arc::new(config),
		sender: sender.clone(),
		local_messages: vec![],
		foreign_messages: vec![],
		rx: rx.clone(),
	};

	// make the arc before we move sail into receive_messages. Ideally we'd do
	// something else so we can update the config later, but we are currently not
	// architected for that
	let dynconf = sail.config.clone();
	let receive_task = tokio::spawn(sail.receive_messages(receiver, sender.clone()));
	let listen_task = tokio::spawn(crate::net::listen(listener, sender, dynconf, rx));
	let signal_listener = tokio::spawn(async {
		use tokio::signal::unix::{signal, SignalKind};
		let mut a = (
			tokio::signal::ctrl_c(),
			signal(SignalKind::alarm()).unwrap(),
			signal(SignalKind::hangup()).unwrap(),
			signal(SignalKind::interrupt()).unwrap(),
			signal(SignalKind::pipe()).unwrap(),
			signal(SignalKind::quit()).unwrap(),
			signal(SignalKind::terminate()).unwrap(),
			signal(SignalKind::user_defined1()).unwrap(),
			signal(SignalKind::user_defined2()).unwrap(),
		);
		tokio::select! {
			_ = a.0 => (),
			_ = a.1.recv() => (),
			_ = a.2.recv() => (),
			_ = a.3.recv() => (),
			_ = a.4.recv() => (),
			_ = a.5.recv() => (),
			_ = a.6.recv() => (),
			_ = a.7.recv() => ()
		};
	});

	//TODO: handle graceful shut Serverdowns
	#[allow(unused_must_use)]
	{
		signal_listener.await;
		println!("\nReceived shutdown signal, beginning graceful shutdown...");
		tx.send(true);
		tokio::join!(listen_task, receive_task);
	}
}

struct BinConfig {
	address: IpAddr,
	port: u16,
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
