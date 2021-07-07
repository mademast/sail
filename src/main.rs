use sail::config::{Config, SailConfig};
use sail::smtp::{
	args::{Domain, ForwardPath},
	ForeignMessage, ForeignPath, Message,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

struct Sail {
	config: Arc<SailConfig>,
	sender: UnboundedSender<Message>,

	delivery_tasks: Vec<JoinHandle<Option<Message>>>,
	local_messages: Vec<Message>,
	foreign_messages: Vec<(Domain, ForeignMessage)>,
}

impl Sail {
	async fn receive_messages(mut self, mut receiver: UnboundedReceiver<Message>) {
		loop {
			let message = receiver
				.recv()
				.await
				.expect("No more senders, what happened?"); //TODO: Not this! Handle the error

			self.handle_message(message);

			//Here we'd check if we relay or save and act appropriately. but FIRST we should write
			//it to the FS as the RFC says that we should not lose messages if we crash. Maybe we
			//try once, as that shouldn't take long, and then if we fail we write?

			// put the runner in Client, or another struct that sits above client.
			//it should try once, then write to disk and sleep for a while.
			//rfc guidelines help.
			//client should be like server; not handling any networking or async anything, just interacting with strings.
		}
	}

	// filters local messages from foreign (to be relayed) messages
	fn handle_message(&mut self, message: Message) {
		let (reverse, forwards, data) = message.into_parts();

		let mut foreign_map: HashMap<Domain, Vec<ForeignPath>> = HashMap::new();
		let locals: Vec<ForwardPath> = forwards
			.into_iter()
			.filter(|forward| {
				if self.config.forward_path_is_local(&forward) {
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
			self.delivery_tasks
				.push(tokio::spawn(Self::deliver_local(Message {
					reverse_path: reverse,
					forward_paths: locals,
					data,
				})));
		}

		for (domain, message) in domains_messages {
			self.delivery_tasks
				.push(tokio::spawn(sail::net::relay(domain, message)));
		}
	}

	//todo: save to fs. Return an undeliverable message if we can't
	async fn deliver_local(message: Message) -> Option<Message> {
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
	let port: u16 = std::env::args()
		.nth(1)
		.unwrap_or("8000".into())
		.parse()
		.unwrap_or(8000);
	let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();

	// Quick, bad config based on port for testing
	let config = match port {
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

	let (sender, receiver) = unbounded_channel();
	let sail = Sail {
		config: Arc::new(config),
		sender: sender.clone(),

		delivery_tasks: vec![],
		local_messages: vec![],
		foreign_messages: vec![],
	};

	// make the arc before we move sail into receive_messages. Ideally we'd do
	// something else so we can update the config later, but we are currently not
	// architected for that
	let dynconf = sail.config.clone();
	let receive_task = tokio::spawn(sail.receive_messages(receiver));
	let listen_task = tokio::spawn(sail::net::listen(listener, sender, dynconf));

	// Maybe we join or something? At some point we have to handle graceful shutdowns
	// so we'd need to handle that somehow. Some way to tell both things to shutdown.
	//we could also just await on the listener, as long as the receiver is running first.
	listen_task.await.unwrap();
	receive_task.await.unwrap();
}
