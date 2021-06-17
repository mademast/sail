#[allow(dead_code)]
use crate::ArgParser;
use crate::{Response, ResponseCode};

pub struct Transaction {
	state: State,
	command: String,
	reverse_path: Option<String>,
	forward_path: Option<String>,
	data: Option<String>,
}

#[allow(dead_code)]
impl Transaction {
	pub fn initiate() -> (Self, Response) {
		(
			Self {
				state: State::Initiated,
				command: String::new(),
				reverse_path: None,
				forward_path: None,
				data: None,
			},
			Response::with_message(ResponseCode::ServiceReady, "Sail ready"),
		)
	}

	pub fn push(&mut self, line: &str) -> Option<Response> {
		self.command.push_str(line);

		if self.command.ends_with("\r\n") {
			let resp = Some(self.run_command());
			self.command.clear();

			resp
		} else {
			None
		}
	}

	pub fn should_exit(&self) -> bool {
		self.state == State::Exit
	}

	fn run_command(&mut self) -> Response {
		let command = Self::parse_command(&self.command.strip_suffix("\r\n").unwrap());
		match command {
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
			Command::Invalid => Self::syntax_error(),
		}
	}

	fn helo(&mut self, client_domain: &str) -> Response {
		match self.state {
			State::Initiated => {
				if ArgParser::validate_domain(client_domain) {
					self.state = State::Greeted;

					Response::with_message(
						ResponseCode::Okay,
						format!("sail Hello {}", client_domain),
					)
				} else {
					Response::with_message(ResponseCode::InvalidParameters, "Bad domain")
				}
			}
			_ => Self::bad_command(),
		}
	}

	fn ehlo(&mut self, client_domain: &str) -> Response {
		match self.state {
			State::Initiated => {
				//TODO: Check for address literals, too
				if ArgParser::validate_domain(client_domain) {
					self.state = State::Greeted;

					Response::with_message(ResponseCode::Okay, "Okay").push("Help")
				} else {
					Response::with_message(ResponseCode::InvalidParameters, "Bad domain")
				}
			}
			_ => Self::bad_command(),
		}
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

	fn mail(&mut self, reverse_path: &str) -> Response {
		let reverse_path = &reverse_path[5..];
		println!("'{}'", reverse_path);
		if self.state == State::Greeted && ArgParser::validate_reverse_path(reverse_path) {
			self.state = State::GotReversePath;
			self.reverse_path = Some(reverse_path.to_string());

			Response::with_message(ResponseCode::Okay, "Okay")
		} else if self.state == State::Greeted {
			Response::with_message(ResponseCode::InvalidParameters, "Bad reverse path")
		} else {
			Self::bad_command()
		}
	}

	fn rcpt(&mut self, forward_path: &str) -> Response {
		let forward_path = &forward_path[3..];
		if (self.state == State::GotReversePath || self.state == State::GotForwardPath)
			&& ArgParser::validate_forward_path(forward_path)
		{
			self.state = State::GotForwardPath;
			self.forward_path = Some(forward_path.to_string());

			Response::with_message(ResponseCode::Okay, "Okay")
		} else if self.state == State::GotReversePath || self.state == State::GotForwardPath {
			Response::with_message(ResponseCode::InvalidParameters, "Bad forward path")
		} else {
			Self::bad_command()
		}
	}

	fn rset(&mut self) -> Response {
		self.state = State::Initiated;
		self.data = None;
		self.reverse_path = None;
		self.forward_path = None;

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

	fn parse_command(command: &str) -> Command {
		if command.len() < 4 {
			return Command::Invalid;
		}
		match command.split_at(4) {
			("HELO", client_domain) => Command::Helo(client_domain.trim().to_owned()),
			("EHLO", client_domain) => Command::Ehlo(client_domain.trim().to_owned()),
			("MAIL", reverse_path) => Command::Mail(reverse_path.trim().to_owned()),
			("RCPT", forward_path) => Command::Rcpt(forward_path.trim().to_owned()),
			("DATA", _) => Command::Data,
			("RSET", _) => Command::Rset,
			("VRFY", target) => Command::Vrfy(target.trim().to_owned()),
			("EXPN", list) => Command::Expn(list.trim().to_owned()),
			("HELP", command) => Command::Help(command.trim().to_owned()),
			("NOOP", _) => Command::Noop,
			("QUIT", _) => Command::Quit,
			_ => Command::Invalid,
		}
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
	GotData,
	Exit,
}
enum Command {
	Helo(String),
	Ehlo(String),
	Mail(String),
	Rcpt(String),
	Data,
	Rset,
	Vrfy(String),
	Expn(String),
	Help(String),
	Noop,
	Quit,
	Invalid,
}
