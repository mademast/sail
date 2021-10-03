use crate::smtp::args::{Domain, ForwardPath, LocalPart, Path};

pub trait Config: Send + Sync {
	/// Check if a forward path should be relayed or delivered locally
	fn forward_path_is_local(&self, forward: &ForwardPath) -> bool;

	/// Returns the hostname that the server will present itself as
	fn primary_host(&self) -> Domain;

	/// Determines if a path is valid or not.
	/// This is used during the RCPT command on the server to determine if it
	/// should accept a forward path or not, whether it's for relay or local delivery.
	fn path_is_valid(&self, path: &Path) -> bool;
}
