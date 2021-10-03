use std::{net::IpAddr, sync::Arc, time::Duration};

use thiserror::Error;
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	net::{TcpListener, TcpStream},
	sync::{mpsc, watch},
	time::{error::Elapsed, timeout},
};

use crate::{
	config::Config,
	smtp::{args::Domain, Client, ForeignMessage, Message, Server},
};

use self::dns::DnsLookup;

pub mod dns;

pub async fn relay(
	domain: Domain,
	message: ForeignMessage,
	// rx: watch::Receiver<bool>,
) -> Option<Message> {
	match run(domain, message.clone() /*, rx*/).await {
		Ok(_) => None,
		Err(err) => match err {
			RelayError::UndeliverableMail(message) => message,
			_ => Into::<Message>::into(message).into_undeliverable(err.to_string()), //FIXME: this is incomprehensible. how do left swimming turbofish work
		},
	}
}

async fn run(
	domain: Domain,
	message: ForeignMessage,
	// rx: watch::Receiver<bool>,
) -> Result<(), RelayError> {
	for path in &message.forward_paths {
		if path.0.domain != domain {
			return Err(RelayError::MismatchedDomains);
		}
	}

	let ip = match domain {
		Domain::FQDN(domain) => DnsLookup::new(&format!("{}.", domain))
			.await
			.unwrap()
			.next_address()
			.await
			.unwrap(),
		Domain::Literal(ip) => ip,
	};

	send_to_ip(ip, message /*, rx*/).await
}

async fn send_to_ip(
	addr: IpAddr,
	message: ForeignMessage,
	// mut rx: watch::Receiver<bool>,
) -> Result<(), RelayError> {
	println!("{}:{}", addr, 25);
	//todo: send failed connection message if port 25 is blocked, or something
	let mut stream = timeout(
		Duration::from_millis(2500),
		TcpStream::connect(format!("{}:{}", addr, 25)),
	)
	.await??;

	let mut client = Client::initiate(message.clone());

	let mut buf = vec![0; 1024];

	while !client.should_exit() {
		let read = stream.read(&mut buf).await?;
		/*tokio::select! {
			_ = rx.changed() => {
				timeout(
					Duration::from_millis(500),
					stream.write_all(b"421 Server has exited. No messages have been sent. Your progress have not been saved.\r\n") //this is a client... we should be sending QUIT, or, even better, just continuing to send it or something lmao
				)
				.await??;
				return Ok(());
			},
			Ok(read) = stream.read(&mut buf) => read,
		};*/

		// A zero sized read, this connection has died or been terminated by the server
		if read == 0 {
			eprintln!("Connection unexpectedly closed by server");
			return Err(RelayError::ConnectionClosed);
		}

		println!("{}", String::from_utf8_lossy(&buf[..read]));
		let command = client.push(String::from_utf8_lossy(&buf[..read]).as_ref());

		if let Some(command) = command {
			println!("{}", command.to_string());
			timeout(
				Duration::from_millis(500),
				stream.write_all(command.to_string().as_bytes()),
			)
			.await??;
		}
	}

	Err(RelayError::UndeliverableMail(client.undeliverable()))
}

#[derive(Debug, Error)]
pub enum RelayError {
	#[error("there are no forward paths in the provided message")]
	NoForwardPaths,
	#[error("there were forward paths with more than one domain")]
	MismatchedDomains,
	#[error("timed out before reaching the server")]
	ConnectionTimeout(#[from] Elapsed),
	#[error("Connection unexpectedly closed by server")]
	ConnectionClosed,
	#[error("there was an error connecting to the host")]
	ConnectionError(#[from] std::io::Error),
	#[error("Undeliverable mail")]
	UndeliverableMail(Option<Message>),
}
