use std::str::FromStr;

use crate::args::{Domain, ForwardPath, ReversePath, Validator};
use crate::client::Client;
use crate::command::Command;
use crate::message::Message;
use crate::{Response, ResponseCode};

#[derive(Default)]
pub struct Transaction {
	state: State,
	command: String,
	message: Message,
}

impl Transaction {
	pub fn initiate() -> (Self, Response) {
		(
			Default::default(),
			Response::with_message(ResponseCode::ServiceReady, "Sail ready"),
		)
	}

	pub async fn push(&mut self, line: &str) -> Option<Response> {
		self.command.push_str(line);

		// Return early if it's not a line
		if !self.command.ends_with("\r\n") {
			return None;
		}

		if self.state == State::LoadingData {
			let resp = self.loading_data().await;
			self.command.clear();

			resp
		} else {
			let resp = Some(self.run_command());
			self.command.clear();

			resp
		}
	}

	pub fn should_exit(&self) -> bool {
		self.state == State::Exit
	}

	async fn loading_data(&mut self) -> Option<Response> {
		if self.command == ".\r\n" {
			// Data is complete
			Some(self.got_data().await)
		//transparency to allow clients to send \r\n.\r\n without breaking SMTP
		} else if self.command.starts_with('.') {
			self.message.data.push(self.command[1..].to_string());
			None
		} else {
			self.message.data.push(self.command.clone());
			None
		}
	}

	//TODO: Check that the data is valid! (rfc 5322)
	async fn got_data(&mut self) -> Response {
		println!("{}", self.message.data.join("\r\n"));
		Client::run(self.message.clone()).await;
		//todo: serialize into a file (serde, perhaps?) and pass off to the client process
		//alternatively, pass into a thread that we spawn here to handle that. That might be the better option?

		self.rset();
		self.state = State::Greeted;

		Response::with_message(ResponseCode::Okay, "message accepted for delivery")
	}

	fn run_command(&mut self) -> Response {
		let command = Command::from_str(&self.command.trim_end()); //todo: TERRIBLE IDEA; DO NOT LEAVE THIS HERE PLEASE

		match command {
			Ok(command) => match command {
				Command::Helo(client_domain) => self.helo(&client_domain),
				Command::Ehlo(client_domain) => self.ehlo(&client_domain),
				Command::Mail(reverse_path) => self.mail(&reverse_path),
				Command::Rcpt(forward_path) => self.rcpt(&forward_path),
				Command::Data => self.data(),
				Command::Rset => self.rset(),
				Command::Vrfy(_) => todo!(),
				Command::Expn(_) => Self::not_implemented(),
				Command::Help(_) => {
					Response::with_message(ResponseCode::HelpMessage, "Please review RFC 5321")
				}
				Command::Noop => Response::with_message(ResponseCode::Okay, "Okay"),
				Command::Quit => self.quit(),
			},
			Err(err) => match err {
				crate::command::ParseCommandError::InvalidCommand => Self::syntax_error(),
				crate::command::ParseCommandError::InvalidPath(_) => {
					Response::with_message(ResponseCode::InvalidParameters, "Bad path")
				}
				crate::command::ParseCommandError::InvalidDomain(err) => {
					Response::with_message(ResponseCode::InvalidParameters, "Bad domain")
				}
			},
		}
	}

	fn helo(&mut self, client_domain: &Domain) -> Response {
		// 4.1.4 does not say the same thing about HELO, so we check the state
		match self.state {
			State::Initiated => {
				self.state = State::Greeted;

				Response::with_message(ResponseCode::Okay, format!("sail Hello {}", client_domain))
			}
			_ => Self::bad_command(),
		}
	}

	fn ehlo(&mut self, client_domain: &Domain) -> Response {
		// Section 4.1.4 says that EHLO may appear later in the session, and
		// that the state should be reset and the buffers cleared (like RSET)
		// So here we just call rset and set the state later.
		// We must, however, check to be sure it's valid first. To reset on
		// an invalid EHLO is to break the spec.
		self.rset();
		self.state = State::Greeted;

		Response::with_message(ResponseCode::Okay, "Okay").push("Help")
	}

	//todo: parse these, don't validate them. separate the parameters, break them into reverse_path structs and whatnot
	fn validate_reverse_path(reverse_path: &str) -> bool {
		todo!() //this can also have mail parameters, apparently
	}

	fn validate_forward_path(forward_path: &str) -> bool {
		todo!()
	}

	fn data(&mut self) -> Response {
		if self.state == State::GotForwardPath {
			self.state = State::LoadingData;
			Response::with_message(ResponseCode::StartMailInput, "Start mail input")
		} else {
			Self::bad_command()
		}
	}

	fn mail(&mut self, reverse_path: &ReversePath) -> Response {
		if self.state == State::Greeted {
			self.state = State::GotReversePath;
			self.message.reverse_path = reverse_path.to_owned();

			Response::with_message(ResponseCode::Okay, "Okay")
		} else {
			Self::bad_command()
		}
	}

	fn rcpt(&mut self, forward_path: &ForwardPath) -> Response {
		if self.state == State::GotReversePath || self.state == State::GotForwardPath {
			self.state = State::GotForwardPath;
			self.message.forward_paths.push(forward_path.to_owned());

			Response::with_message(ResponseCode::Okay, "Okay")
		} else {
			Self::bad_command()
		}
	}

	fn rset(&mut self) -> Response {
		self.message.data.clear();
		self.message.reverse_path = Default::default();
		self.message.forward_paths.clear();

		self.state = match self.state {
			State::Initiated => State::Initiated,
			_ => State::Greeted,
		};

		Response::with_message(ResponseCode::Okay, "Okay")
	}

	fn quit(&mut self) -> Response {
		self.state = State::Exit;

		Response::with_message(ResponseCode::ServiceClosing, "sail Goodbye")
	}

	fn not_implemented() -> Response {
		Response::with_message(
			ResponseCode::CommandNotImplemented,
			"Command not implemented",
		)
	}

	fn bad_command() -> Response {
		Response::with_message(ResponseCode::BadCommandSequence, "bad sequence of commands")
	}

	fn syntax_error() -> Response {
		Response::with_message(ResponseCode::UnrecognizedCommand, "Syntax Error")
	}
}

#[derive(PartialEq)]
enum State {
	Initiated,
	Greeted,
	GotReversePath,
	GotForwardPath,
	LoadingData,
	Exit,
}

impl Default for State {
	fn default() -> Self {
		Self::Initiated
	}
}
