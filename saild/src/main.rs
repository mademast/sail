mod binconfig;
pub mod fs;
mod net;
mod sailconfig;

use binconfig::BinConfig;
use sailconfig::ServerConfig;

use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
	let binconf = match BinConfig::get() {
		Some(conf) => conf,
		None => return,
	};

	let listener = TcpListener::bind(binconf.socket_address()).await.unwrap();

	// Quick, bad config based on port for testing
	let config = match binconf.port {
		25 => ServerConfig {
			hostnames: vec!["localhost".parse().unwrap()],
			relays: vec!["nove.dev".parse().unwrap(), "genbyte.dev".parse().unwrap()],
			users: vec!["genny".parse().unwrap(), "devon".parse().unwrap()],
			maildir: binconf.maildir,
		},
		_ => ServerConfig {
			hostnames: vec!["localhost.localdomain".parse().unwrap()],
			users: vec!["alice".parse().unwrap(), "bob".parse().unwrap()],
			relays: vec!["localhost".parse().unwrap()],
			maildir: binconf.maildir,
		},
	};

	let (tx, rx) = tokio::sync::watch::channel(false);

	// make the arc before we move sail into receive_messages. Ideally we'd do
	// something else so we can update the config later, but we are currently not
	// architected for that
	let dynconf = Arc::new(config.clone());
	let listen_task = tokio::spawn(crate::net::listen(listener, dynconf, rx));
	let signal_listener = tokio::spawn(async {
		use tokio::signal::unix::{signal, SignalKind};
		let mut a = (
			tokio::signal::ctrl_c(),
			signal(SignalKind::alarm()).unwrap(),
			signal(SignalKind::hangup()).unwrap(),
			signal(SignalKind::interrupt()).unwrap(),
			signal(SignalKind::pipe()).unwrap(),
			signal(SignalKind::quit()).unwrap(),
			signal(SignalKind::terminate()).unwrap(),
			signal(SignalKind::user_defined1()).unwrap(),
			signal(SignalKind::user_defined2()).unwrap(),
		);
		tokio::select! {
			_ = a.0 => (),
			_ = a.1.recv() => (),
			_ = a.2.recv() => (),
			_ = a.3.recv() => (),
			_ = a.4.recv() => (),
			_ = a.5.recv() => (),
			_ = a.6.recv() => (),
			_ = a.7.recv() => ()
		};
	});

	//TODO: handle graceful shut Serverdowns
	#[allow(unused_must_use)]
	{
		signal_listener.await;
		println!("\nReceived shutdown signal, beginning graceful shutdown...");
		tx.send(true);
		listen_task.await;
	}
}
