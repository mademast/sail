use sail::Transaction;
use tokio::io;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

pub async fn serve(mut stream: TcpStream) -> io::Result<()> {
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

		let response = transaction
			.push(String::from_utf8_lossy(&buf[..read]).as_ref())
			.await;

		if let Some(response) = response {
			stream.write_all(response.as_string().as_bytes()).await?;
		}
	}

	Ok(())
}

#[tokio::main]
async fn main() {
	let port: u16 = std::env::args()
		.skip(1)
		.next()
		.unwrap_or("8000".into())
		.parse()
		.unwrap();
	let listener = TcpListener::bind(("127.0.0.1", port)).await.unwrap();

	loop {
		let (stream, clientaddr) = listener.accept().await.unwrap();

		println!("connection from {}", clientaddr);

		tokio::spawn(serve(stream));
	}
}
