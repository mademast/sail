use std::sync::Arc;

use sail::smtp::Server;
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	net::{TcpListener, TcpStream},
	sync::watch,
};

use crate::sailconfig::ServerConfig;

//runs as long as the user remains connected
// handles low-level tcp read and write nonsense, passes strings back and forth with the business logic in transaction.
async fn serve(
	mut stream: TcpStream,
	config: Arc<ServerConfig>,
	mut rx: watch::Receiver<bool>,
) -> io::Result<()> {
	let (mut transaction, inital_response) = Server::initiate(Box::new(config.as_ref().clone()));
	stream
		.write_all(inital_response.as_string().as_bytes())
		.await?;

	let mut buf = vec![0; 1024];

	while !transaction.should_exit() {
		#[allow(unused_must_use)]
		let read = tokio::select! {
			Ok(read) = stream.read(&mut buf) => read,
			_ = rx.changed() => {
				stream.write_all(b"421 Server has exited. No messages have been sent. Your progress have not been saved.\r\n").await;
				return Ok(());
			},
		};

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
	config: Arc<ServerConfig>,
	mut rx: watch::Receiver<bool>,
) {
	loop {
		let (stream, clientaddr) = tokio::select! {
			_ = rx.changed() => break,
			Ok((stream, clientaddr)) = listener.accept() => (stream, clientaddr)
		};

		println!("connection from {}", clientaddr);

		tokio::spawn(serve(stream, config.clone(), rx.clone()));
	}
}
