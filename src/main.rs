use std::net::{TcpListener, TcpStream};

use sail::Protocol;
use smol::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	Async,
};

pub struct Connection {
	stream: Async<TcpStream>,
	protocol: Protocol,
}

impl Connection {
	pub fn new(stream: Async<TcpStream>) -> Self {
		Self {
			stream,
			protocol: Protocol::new(),
		}
	}

	pub async fn serve(&mut self) -> io::Result<()> {
		//TODO: send 220 initiator
		println!("serving!");
		let mut buf = vec![0; 1024];

		loop {
			let read = self.stream.read(&mut buf).await?;

			for i in 0..read {
				print!("{:02X} ", buf[i]);
			}
			println!("\n{}", String::from_utf8_lossy(&buf[..read]));

			let response = self
				.protocol
				.push(String::from_utf8_lossy(&buf[..read]).as_ref());

			if let Some(respond) = response {
				self.stream.write_all(respond.as_bytes()).await?;
			}
		}

		Ok(())
	}
}

fn main() {
	smol::block_on(async {
		let listener = Async::<TcpListener>::bind(([127, 0, 0, 1], 8000)).unwrap();

		loop {
			let (stream, clientaddr) = listener.accept().await.unwrap();

			println!("connection from {}", clientaddr);

			smol::spawn(async move {
				let mut con = Connection::new(stream);
				con.serve().await
			})
			.detach();
		}
	});
}
