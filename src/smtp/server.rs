use std::sync::mpsc::Sender;

use crate::config::Config;

use super::{
	args::{Domain, ForwardPath, ReversePath},
	Command, Message, Response, ResponseCode,
};

pub struct Server {
	config: Config,
	primary_host: Domain,
	message_sender: Sender<Message>,
	state: State,
	command: String,
	message: Message,
}

impl Server {
	pub fn initiate(message_sender: Sender<Message>, config: Config) -> (Self, Response) {
		let primary_host = config
			.hostnames
			.first()
			.unwrap_or(&"sail".parse::<Domain>().unwrap())
			.to_owned();
		(
			Self {
				config,
				primary_host: primary_host.clone(),
				message_sender,
				state: Default::default(),
				command: Default::default(),
				message: Default::default(),
			},
			Response::with_message(
				ResponseCode::ServiceReady,
				format!("{} (Sail) ready", primary_host),
			),
		)
	}

	pub fn push(&mut self, line: &str) -> Option<Response> {
		self.command.push_str(line);

		// Return early if it's not a line
		if !self.command.ends_with("\r\n") {
			return None;
		}

		if self.state == State::LoadingData {
			let resp = self.loading_data();
			self.command.clear();

			resp
		} else {
			let resp = self.run_command();
			self.command.clear();

			Some(resp)
		}
	}

	pub fn should_exit(&self) -> bool {
		self.state == State::Exit
	}

	fn loading_data(&mut self) -> Option<Response> {
		if self.command == ".\r\n" {
			// Data is complete
			Some(self.got_data())
		//transparency to allow clients to send \r\n.\r\n without breaking SMTP
		} else if self.command.starts_with('.') {
			self.message.data.push(self.command[1..].to_string());
			None
		} else {
			self.message.data.push(self.command.clone());
			None
		}
	}

	fn got_data(&mut self) -> Response {
		//TODO: Fail here if the mail data fails verification as per RFC 5322

		//TODO: Instead of this bad expect, send an error to the client. This is our fault,
		//it'll be one of the 500s.
		self.message_sender
			.send(self.message.clone())
			.expect("Failed to send message");

		self.rset();
		self.state = State::Greeted;

		Response::with_message(ResponseCode::Okay, "message accepted for delivery")
	}

	fn run_command(&mut self) -> Response {
		let command = self.command.trim_end().parse();

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
				super::command::ParseCommandError::InvalidCommand => Self::syntax_error(),
				super::command::ParseCommandError::InvalidPath(_) => {
					Response::with_message(ResponseCode::InvalidParameters, "Bad path")
				}
				super::command::ParseCommandError::InvalidDomain(err) => {
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

				Response::with_message(
					ResponseCode::Okay,
					format!("{} (sail) greets {}", self.primary_host, client_domain),
				)
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

		Response::with_message(
			ResponseCode::Okay,
			format!("{} (sail) greets {}", self.primary_host, client_domain),
		)
		.push("Help")
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
			match forward_path {
				ForwardPath::Postmaster => self.add_rcpt(forward_path),
				ForwardPath::Regular(path) => {
					if self.config.relays.contains(&path.domain)
						|| self.config.hostnames.contains(&path.domain)
							&& self.config.users.contains(&path.local_part)
					{
						self.add_rcpt(forward_path)
					} else {
						Self::bad_command() //todo: correct responses
					}
				}
			}
		} else {
			Self::bad_command()
		}
	}

	fn add_rcpt(&mut self, forward_path: &ForwardPath) -> Response {
		self.state = State::GotForwardPath;
		self.message.forward_paths.push(forward_path.to_owned());

		Response::with_message(ResponseCode::Okay, "Okay")
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

		Response::with_message(
			ResponseCode::ServiceClosing,
			&format!("{} Goodbye", self.primary_host),
		)
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
