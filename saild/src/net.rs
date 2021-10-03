use std::sync::Arc;

use sail::{
	config::Config,
	smtp::{Message, Server},
};
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	net::{TcpListener, TcpStream},
	sync::{mpsc, watch},
};

//runs as long as the user remains connected
// handles low-level tcp read and write nonsense, passes strings back and forth with the business logic in transaction.
async fn serve(
	mut stream: TcpStream,
	message_sender: mpsc::UnboundedSender<Message>,
	config: Arc<dyn Config>,
	mut rx: watch::Receiver<bool>,
) -> io::Result<()> {
	let (mut transaction, inital_response) = Server::initiate(message_sender, config);
	stream
		.write_all(inital_response.as_string().as_bytes())
		.await?;

	let mut buf = vec![0; 1024];

	while !transaction.should_exit() {
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
	message_sender: mpsc::UnboundedSender<Message>,
	config: Arc<dyn Config>,
	mut rx: watch::Receiver<bool>,
) {
	loop {
		let (stream, clientaddr) = tokio::select! {
			_ = rx.changed() => break,
			Ok((stream, clientaddr)) = listener.accept() => (stream, clientaddr)
		};

		println!("connection from {}", clientaddr);

		tokio::spawn(serve(
			stream,
			message_sender.clone(),
			config.clone(),
			rx.clone(),
		));
	}
}
