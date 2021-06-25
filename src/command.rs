pub enum Command {
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

impl Command {
	pub fn parse(command: &str) -> Command {
		let command = command.split_once(' ').unwrap_or((command, ""));
		match (command.0.to_ascii_uppercase().as_str(), command.1) {
			("HELO", client_domain) => Command::Helo(client_domain.trim().to_owned()),
			("EHLO", client_domain) => Command::Ehlo(client_domain.trim().to_owned()),
			("MAIL", reverse_path) => Command::Mail(reverse_path.trim().to_owned()),
			("RCPT", forward_path) => Command::Rcpt(forward_path.trim().to_owned()),
			("DATA", "") => Command::Data,
			("RSET", "") => Command::Rset,
			("VRFY", target) => Command::Vrfy(target.trim().to_owned()),
			("EXPN", list) => Command::Expn(list.trim().to_owned()),
			("HELP", command) => Command::Help(command.trim().to_owned()),
			("NOOP", _) => Command::Noop,
			("QUIT", "") => Command::Quit,
			_ => Command::Invalid,
		}
	}

	pub fn as_string(&self) -> String {
		match self {
			Command::Helo(parameters) => format!("HELO {}", parameters),
			Command::Ehlo(parameters) => format!("EHLO {}", parameters),
			Command::Mail(parameters) => format!("MAIL {}", parameters),
			Command::Rcpt(parameters) => format!("RCPT {}", parameters),
			Command::Data => String::from("DATA"),
			Command::Rset => String::from("RSET"),
			Command::Vrfy(parameters) => format!("VRFY {}", parameters),
			Command::Expn(parameters) => format!("EXPN {}", parameters),
			Command::Help(parameters) => format!("HELP {}", parameters),
			Command::Noop => String::from("NOOP"),
			Command::Quit => String::from("QUIT"),
			Command::Invalid => String::from("NOOP"), //shouldn't ever be constructed
		}
	}
}
