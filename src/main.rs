use std::net::{TcpListener, TcpStream};

use sail::Transaction;
use smol::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	Async,
};

pub async fn serve(mut stream: Async<TcpStream>) -> io::Result<()> {
	let (mut transaction, inital_message) = Transaction::initiate();
	stream.write_all(inital_message.as_bytes()).await?;

	let mut buf = vec![0; 1024];

	while !transaction.should_exit() {
		let read = stream.read(&mut buf).await?;

		for i in 0..read {
			print!("{:02X} ", buf[i]);
		}
		println!("\n{}", String::from_utf8_lossy(&buf[..read]));

		let response = transaction.push(String::from_utf8_lossy(&buf[..read]).as_ref());

		if let Some(respond) = response {
			stream.write_all(respond.as_bytes()).await?;
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
