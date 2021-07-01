mod config;
mod dns;

use std::{
	collections::HashMap,
	str::FromStr,
	sync::mpsc::{channel, Receiver, Sender},
};

use config::Config;
use sail::smtp::{
	args::{Domain, ForwardPath},
	ForeignMessage, ForeignPath, Message, Server,
};
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	net::TcpListener,
	net::TcpStream,
};

struct Sail {
	config: Config,
	local_messages: Vec<Message>,
	foreign_messages: Vec<(Domain, ForeignMessage)>,
}

impl Sail {
	async fn receive_messages(self, receiver: Receiver<Message>) {
		loop {
			let message = receiver.recv().expect("No more senders, what happened?"); //TODO: Not this! Handle the error

			self.handle_message(message).await;

			//Here we'd check if we relay or save and act appropriately. but FIRST we should write
			//it to the FS as the RFC says that we should not lose messages if we crash. Maybe we
			//try once, as that shouldn't take long, and then if we fail we write? For now, we print.
			//println!("{}", message.data.join("\r\n"));

			// put the runner in Client, or another struct that sits above client.
			//it should try once, then write to disk and sleep for a while.
			//rfc guidelines help.
			//client should be like server; not handling any networking or async anything, just interacting with strings.
		}
	}

	// filters local messages from foreign (to be relayed) messages
	async fn handle_message(&self, message: Message) {
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

		self.deliver_local(Message {
			reverse_path: reverse.clone(),
			forward_paths: locals,
			data: data.clone(),
		})
		.await;
	}

	//todo: something other than this? we'd need a database of users and whatnot, though
	async fn deliver_local(&self, message: Message) {
		let (reverse, forwards, data) = message.into_parts();

		print!("REVERSE: {}\nLOCAL TO:", reverse);
		for path in forwards {
			print!(" {}", path);
		}
		println!("\n{}", data.join("\r\n"))
	}

	async fn relay(domain: Domain, message: ForeignMessage) {}
}

//runs as long as the user remains connected
// handles low-level tcp read and write nonsense, passes strings back and forth with the business logic in transaction.
async fn serve(mut stream: TcpStream, message_sender: Sender<Message>) -> io::Result<()> {
	let (mut transaction, inital_response) = Server::initiate(message_sender);
	stream
		.write_all(inital_response.as_string().as_bytes())
		.await?;

	let mut buf = vec![0; 1024];

	while !transaction.should_exit() {
		let read = stream.read(&mut buf).await?;

		// A zero sized read, this connection has died or been terminated by the client
		if read == 0 {
			println!("Connection unexpectedly closed by client");

			return Ok(());
		}

		let response = transaction.push(String::from_utf8_lossy(&buf[..read]).as_ref());

		if let Some(response) = response {
			stream.write_all(response.as_string().as_bytes()).await?;
		}
	}

	Ok(())
}

//waits for new connections, dispatches new task to handle each new inbound connection
async fn listen(listener: TcpListener, message_sender: Sender<Message>) {
	loop {
		let (stream, clientaddr) = listener.accept().await.unwrap();

		println!("connection from {}", clientaddr);

		tokio::spawn(serve(stream, message_sender.clone()));
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
		25 => Config {
			hostnames: vec![Domain::from_str("localhost").unwrap()],
		},
		_ => Config { hostnames: vec![] },
	};

	let (sender, receiver) = channel();
	let sail = Sail {
		config,
		local_messages: vec![],
		foreign_messages: vec![],
	};

	let receive_task = tokio::spawn(sail.receive_messages(receiver));
	let listen_task = tokio::spawn(listen(listener, sender));

	// Maybe we join or something? At some point we have to handle graceful shutdowns
	// so we'd need to handle that somehow. Some way to tell both things to shutdown.
	//we could also just await on the listener, as long as the receiver is running first.
	listen_task.await.unwrap();
	receive_task.await.unwrap();
}
