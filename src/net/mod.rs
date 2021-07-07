use std::{net::IpAddr, sync::Arc, time::Duration};

use thiserror::Error;
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	net::{TcpListener, TcpStream},
	sync::mpsc::UnboundedSender as Sender,
	time::{error::Elapsed, timeout},
};

use crate::{
	config::Config,
	smtp::{args::Domain, Client, ForeignMessage, Message, Server},
};

use self::dns::DnsLookup;

pub mod dns;

//runs as long as the user remains connected
// handles low-level tcp read and write nonsense, passes strings back and forth with the business logic in transaction.
async fn serve(
	mut stream: TcpStream,
	message_sender: Sender<Message>,
	config: Arc<dyn Config>,
) -> io::Result<()> {
	let (mut transaction, inital_response) = Server::initiate(message_sender, config);
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
pub async fn listen(
	listener: TcpListener,
	message_sender: Sender<Message>,
	config: Arc<dyn Config>,
) {
	loop {
		let (stream, clientaddr) = listener.accept().await.unwrap();

		println!("connection from {}", clientaddr);

		tokio::spawn(serve(stream, message_sender.clone(), config.clone()));
	}
}

pub async fn relay(domain: Domain, message: ForeignMessage) -> Option<Message> {
	match run(domain, message.clone()).await {
		Ok(maybe_message) => maybe_message,
		Err(err) => Into::<Message>::into(message).into_undeliverable(err.to_string()),
	}
}

async fn run(domain: Domain, message: ForeignMessage) -> Result<Option<Message>, RelayError> {
	let ip = match &domain {
		Domain::FQDN(domain) => DnsLookup::new(&format!("{}.", &domain.to_string()))
			.await
			.unwrap()
			.next_address()
			.await
			.unwrap(),
		Domain::Literal(ip) => ip.to_owned(),
	};
	for path in &message.forward_paths {
		if path.0.domain != domain {
			return Err(RelayError::MismatchedDomains);
		}
	}

	send_to_ip(ip, message).await
}

async fn send_to_ip(addr: IpAddr, message: ForeignMessage) -> Result<Option<Message>, RelayError> {
	eprintln!("{}:{}", addr, 25);
	//todo: send failed connection message if port 25 is blocked, or something
	let mut stream = timeout(
		Duration::from_millis(2500),
		TcpStream::connect(format!("{}:{}", addr, 25)),
	)
	.await??;

	let mut client = Client::initiate(message.clone());

	let mut buf = vec![0; 1024];

	while !client.should_exit() {
		let read = stream.read(&mut buf).await.unwrap();

		// A zero sized read, this connection has died or been terminated by the server
		if read == 0 {
			println!("Connection unexpectedly closed by server");
			//todo: This should be an error. We'ce come so far, we can't lose the message here
			return Ok(None);
		}

		println!("{}", String::from_utf8_lossy(&buf[..read]));
		let command = client.push(String::from_utf8_lossy(&buf[..read]).as_ref());

		if let Some(command) = command {
			println!("{}", command.to_string());
			stream.write_all(command.to_string().as_bytes()).await?;
		}
	}

	Ok(client.undeliverable())
}

#[derive(Debug, Error)]
pub enum RelayError {
	#[error("there are no forward paths in the provided message")]
	NoForwardPaths,
	#[error("there were forward paths with more than one domain")]
	MismatchedDomains,
	#[error("timed out before reaching the server")]
	ConnectionTimeout(#[from] Elapsed),
	#[error("there was an error connecting to the host")]
	ConnectionError(#[from] std::io::Error),
}
