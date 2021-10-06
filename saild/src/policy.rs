use crate::fs::Maildir;

use std::{collections::HashMap, path::PathBuf};

use sail::{
	policy::Policy,
	smtp::{
		args::{Domain, ForeignPath, ForwardPath, LocalPart, Path},
		Envelope, ForeignEnvelope, Message, Response, ResponseCode,
	},
};

#[derive(Clone)]
pub struct ServerPolicy {
	//TODO: Properly load a config and don't have this be public!
	pub hostnames: Vec<Domain>,
	pub relays: Vec<Domain>,
	pub users: Vec<LocalPart>,
	pub maildir: PathBuf,
}

impl ServerPolicy {
	fn path_is_local(&self, path: &Path) -> bool {
		self.hostnames.contains(&path.domain)
	}

	fn path_is_foreign(&self, path: &Path) -> bool {
		self.relays.contains(&path.domain)
	}

	// Determine if a user is valid for local delivery
	fn user_is_valid(&self, local: &LocalPart) -> bool {
		self.users.contains(local)
	}
}

impl Policy for ServerPolicy {
	fn forward_path_is_local(&self, forward: &ForwardPath) -> bool {
		match forward {
			ForwardPath::Postmaster => true,
			ForwardPath::Regular(path) => self.path_is_local(path),
		}
	}

	fn primary_host(&self) -> Domain {
		//TODO: Remove unwrap
		self.hostnames.first().unwrap().clone()
	}

	fn path_is_valid(&self, path: &Path) -> bool {
		self.path_is_foreign(path)
			|| (self.path_is_local(path) && self.user_is_valid(&path.local_part))
	}

	fn message_received(&mut self, message: Envelope) -> Response {
		let (reverse, forwards, content) = message.into_parts();
		// Seperate the message by domains and whether or not the message is local.
		//TODO: divide message into local and relay

		let mut foreign_map: HashMap<Domain, Vec<ForeignPath>> = HashMap::new();
		let locals: Vec<ForwardPath> = forwards
			.into_iter()
			.filter(|forward| {
				if self.forward_path_is_local(forward) {
					true // locals stay in the vec
				} else if let ForwardPath::Regular(path) = forward {
					// get the vector for a specific domain, but if there isn't one, make it.
					match foreign_map.get_mut(&path.domain) {
						Some(vec) => vec.push(ForeignPath { 0: path.clone() }),
						None => {
							foreign_map
								.insert(path.domain.clone(), vec![ForeignPath { 0: path.clone() }]);
						}
					}

					false
				} else {
					// should've been caught by forward_path_is_local, maybe
					// print a warning if we reach here?
					true
				}
			})
			.collect();

		// # Saving locally
		// Try and save it to the file system. If we fail, tell the server that it's rejected
		// as we have nowhere to save it! If it succeeds, tell the server as such (return 250).
		//TODO: Save the local bits to disk
		for local in locals {
			println!(
				"From: {}\nFor:{}\n\n{}",
				reverse.to_string(),
				local.to_string(),
				content
			);
		}
		let md = Maildir::new(&self.maildir);
		md.create_directories().unwrap();
		md.save(content.clone()).unwrap();

		// # Relaying Onwards
		// First, check if the server this would relay to is in our list that we're allowed to
		// relay to (we do NOT want to be an open relay, that is a bad thing).
		// There's an async task setup to deal with mail relay. If we accept the mail, send it
		// there and return 250. If we don't accept it, tell the server as such.
		for (domain, forwards) in foreign_map.into_iter() {
			let envelope = ForeignEnvelope::from_parts(reverse.clone(), forwards, content.clone());

			tokio::spawn(sail::net::relay(
				domain, envelope, /*, self.rx.clone()*/
			));
		}

		Response::new(ResponseCode::Okay)
	}
}

struct DeliveryEnvelope {
	foreign: Vec<(Domain, Vec<ForeignPath>)>,
	message: Message,
}
