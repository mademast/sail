use std::net::{TcpListener, TcpStream};

use sail::Transaction;
use smol::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	Async,
};

pub async fn serve(mut stream: Async<TcpStream>) -> io::Result<()> {
	let (mut transaction, inital_response) = Transaction::initiate();
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

		for byte in buf.iter().take(read) {
			print!("{:02X} ", byte);
		}
		println!("\n{}", String::from_utf8_lossy(&buf[..read]));

		let response = transaction.push(String::from_utf8_lossy(&buf[..read]).as_ref());

		if let Some(response) = response {
			stream.write_all(response.as_string().as_bytes()).await?;
		}
	}

	Ok(())
}

fn main() {
	smol::block_on(async {
		let listener = Async::<TcpListener>::bind(([127, 0, 0, 1], 8000)).unwrap();

		loop {
			let (stream, clientaddr) = listener.accept().await.unwrap();

			println!("connection from {}", clientaddr);

			smol::spawn(async move { serve(stream).await.unwrap() }).detach();
		}
	});
}
