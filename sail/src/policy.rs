use crate::smtp::{
	args::{Domain, ForwardPath, Path},
	Envelope, Response,
};

pub trait Policy: Send + Sync {
	/// Check if a forward path should be relayed or delivered locally
	fn forward_path_is_local(&self, forward: &ForwardPath) -> bool;

	/// Returns the hostname that the server will present itself as
	fn primary_host(&self) -> Domain;

	/// Determines if a path is valid or not.
	/// This is used during the RCPT command on the server to determine if it
	/// should accept a forward path or not, whether it's for relay or local delivery.
	fn path_is_valid(&self, path: &Path) -> bool;

	//TODO: rewrite this
	/// Called when the server receives a message.
	/// If the message is accepted, this function should return a Response with the code 250.
	/// If the message is rejected, the Response should have a code indicating a negative state.
	fn message_received(&mut self, message: Envelope) -> Response;
}

pub trait IntoMessageResponse {
	fn into_message_response(self) -> Response;
}
