#[allow(dead_code)]

pub struct Transaction {
	state: State,
	reverse_path: Option<String>,
	forward_path: Option<String>,
	data: Option<String>,
}

#[allow(dead_code)]
impl Transaction {
	pub fn initiate() -> (Self, String) {
		(
			Self {
				state: State::Initiated,
				reverse_path: None,
				forward_path: None,
				data: None,
			},
			String::from("220 Sail Ready"),
		)
	}
	pub fn run_command(&mut self, command: &str) -> String {
		let command = Self::parse_command(command);
		match command {
			Command::Helo(client_domain) => self.helo(&client_domain),
			Command::Ehlo(client_domain) => self.ehlo(&client_domain),
			Command::Mail(reverse_path) => self.mail(&reverse_path),
			Command::Rcpt(forward_path) => self.rcpt(&forward_path),
			Command::Data => self.data(),
			Command::Rset => self.rset(),
			Command::Vrfy(_) => todo!(),
			Command::Expn(_) => Self::not_implemented(),
			Command::Help(_) => String::from("214 Please review RFC 5321"),
			Command::Noop => String::from("250 OK"),
			Command::Quit => self.quit(),
			Command::Invalid => Self::syntax_error(),
		}
	}

	fn helo(&mut self, client_domain: &str) -> String {
		match self.state {
			State::Initiated => {
				if Self::validate_domain(client_domain) {
					self.state = State::Greeted;
					"250 Sail".to_string()
				} else {
					String::from("501 Bad Domain")
				}
			}
			_ => Self::bad_command(),
		}
	}
	fn ehlo(&mut self, client_domain: &str) -> String {
		match self.state {
			State::Initiated => {
				if Self::validate_domain(client_domain) {
					self.state = State::Greeted;
					"250-Sail\r\n250 Help".to_string()
				} else {
					String::from("501 Bad Domain")
				}
			}
			_ => Self::bad_command(),
		}
	}
	fn validate_domain(domain: &str) -> bool {
		todo!()
	}
	//todo: parse these, don't validate them. separate the parameters, break them into reverse_path structs and whatnot
	fn validate_reverse_path(reverse_path: &str) -> bool {
		todo!() //this can also have mail parameters, apparently
	}
	fn validate_forward_path(forward_path: &str) -> bool {
		todo!()
	}

	fn data(&mut self) -> String {
		if self.state == State::GotForwardPath {
			self.state = State::LoadingData;
			"354 Start mail input; end with <CRLF>.<CRLF>".to_string()
		} else {
			Self::bad_command()
		}
	}
	fn mail(&mut self, reverse_path: &str) -> String {
		if self.state == State::Greeted && Self::validate_reverse_path(reverse_path) {
			self.state = State::GotReversePath;
			self.reverse_path = Some(reverse_path[6..].to_string());
			String::from("250 OK")
		} else if self.state == State::Greeted {
			"501 Bad Reverse Path".to_string()
		} else {
			Self::bad_command()
		}
	}
	fn rcpt(&mut self, forward_path: &str) -> String {
		if (self.state == State::GotReversePath || self.state == State::GotForwardPath)
			&& Self::validate_forward_path(forward_path)
		{
			self.state = State::GotForwardPath;
			self.forward_path = Some(forward_path[4..].to_string());
			String::from("250 OK")
		} else if self.state == State::GotReversePath || self.state == State::GotForwardPath {
			"501 Bad Forward Path".to_string()
		} else {
			Self::bad_command()
		}
	}
	fn rset(&mut self) -> String {
		self.state = State::Initiated;
		self.data = None;
		self.reverse_path = None;
		self.forward_path = None;
		String::from("250 OK")
	}
	fn quit(&mut self) -> String {
		self.state = State::Exit;
		String::from("221 Sail Goodbye")
	}

	fn not_implemented() -> String {
		String::from("502 Command Not Implemented")
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
	fn bad_command() -> String {
		String::from("503 bad sequence of commands")
	}
	fn syntax_error() -> String {
		String::from("500 Syntax Error")
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
