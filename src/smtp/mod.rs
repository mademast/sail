pub mod args;
mod client;
mod command;
mod message;
mod response;
mod server;

pub use client::{Client, ForeignMessage, ForeignPath};
pub use command::Command;
pub use message::Message;
pub use response::{Response, ResponseCode};
pub use server::Server;

mod test {

	#[test]
	#[ignore] //only run in CI contexts
	fn send_trigger() {
		//sends a message to an email set by us
		use std::{env::var, str::FromStr};

		use super::{
			super::{net, smtp::Message},
			args::{Domain, Path, ReversePath},
			ForeignMessage, ForeignPath,
		};
		let path = Path::from_str(&format!("<{}>", var("TRIGGER_EMAIL").unwrap())).unwrap();
		let forward_paths = vec![ForeignPath(path.clone())];
		let reverse_path = ReversePath::Regular(path);
		let data = "".to_string();

		let message = ForeignMessage {
			forward_paths,
			reverse_path,
			data,
		};
		let (_, rx) = tokio::sync::watch::channel(false);
		let future = net::relay(Domain::from_str("oracle.nove.dev").unwrap(), message, rx);

		let undeliverable: Option<Message> = tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.unwrap()
			.block_on(future);

		dbg!(&undeliverable);
		assert!(undeliverable.is_none())
	}
}
