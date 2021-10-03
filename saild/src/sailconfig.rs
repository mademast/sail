use sail::{
	config::Config,
	smtp::args::{Domain, ForwardPath, LocalPart, Path},
};

#[derive(Clone)]
pub struct SailConfig {
	//TODO: Properly load a config and don't have this be public!
	pub hostnames: Vec<Domain>,
	pub relays: Vec<Domain>,
	pub users: Vec<LocalPart>,
}

impl SailConfig {
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

impl Config for SailConfig {
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
}
