mod config;
mod dns;

use std::{
	str::FromStr,
	sync::mpsc::{channel, Receiver, Sender},
};

use config::Config;
use sail::smtp::{
	args::{Domain, ForwardPath},
	Message, Server,
};
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	net::TcpListener,
	net::TcpStream,
};

struct Sail {
	config: Config,
	receiver: Receiver<Message>,
	messages: Vec<Message>,
}

impl Sail {
	async fn receive_messages(self) {
		loop {
			let message = self
				.receiver
				.recv()
				.expect("No more senders, what happened?"); //TODO: Not this! Handle the error

			self.handle_message(message);

			//Here we'd check if we relay or save and act approriatly. but FIRST we should write
			//it to the FS as the RFC says that we should not lose messages if we crash. Maybe we
			//try once, as that shouldn't take long, and then if we fail we write? For now, we print.
			//println!("{}", message.data.join("\r\n"));

			// put the runner in Client, or another struct that sits above client.
			//it should try once, then write to disk and sleep for a while.
			//rfc guidelines help.
			//client should be like server; not handling any networking or async anything, just interacting with strings.
		}
	}

	fn handle_message(&self, mut msg: Message) {
		let message_data = msg.data.join("\r\n");
		msg.forward_paths = msg
			.forward_paths
			.into_iter()
			.filter(|forward| {
				if self.config.forward_path_is_local(forward) {
					self.deliver_local(forward, message_data.clone());
					false
				} else {
					true
				}
			})
			.collect();

		println!("{:?}", &msg.forward_paths);
	}

	fn deliver_local(&self, forward: &ForwardPath, data: String) {
		println!("LOCAL TO: {}\n{}", forward, data);
	}
}

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
		receiver,
		messages: vec![],
	};

	let receive_task = tokio::spawn(sail.receive_messages());
	let listen_task = tokio::spawn(listen(listener, sender));

	// Maybe we join or something? At some point we have to handle graceful shutdowns
	// so we'd need to handle that somehow. Some way to tell both things to shutdown.
	listen_task.await.unwrap();
	receive_task.await.unwrap();
}
