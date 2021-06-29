use crate::args::{Domain, ForwardPath, ParseDomainError, ParsePathError, ReversePath};
use thiserror::Error;

pub enum Command {
	Helo(Domain),
	Ehlo(Domain),
	Mail(ReversePath),
	Rcpt(ForwardPath),
	Data,
	Rset,
	Vrfy(String),
	Expn(String),
	Help(String),
	Noop,
	Quit,
}

impl std::fmt::Display for Command {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Command::Helo(parameters) => format!("HELO {}", parameters),
				Command::Ehlo(parameters) => format!("EHLO {}", parameters),
				Command::Mail(parameters) => format!("MAIL FROM:{}", parameters),
				Command::Rcpt(parameters) => format!("RCPT TO:{}", parameters),
				Command::Data => String::from("DATA"),
				Command::Rset => String::from("RSET"),
				Command::Vrfy(parameters) => format!("VRFY {}", parameters),
				Command::Expn(parameters) => format!("EXPN {}", parameters),
				Command::Help(parameters) => format!("HELP {}", parameters),
				Command::Noop => String::from("NOOP"),
				Command::Quit => String::from("QUIT"),
			}
		)
	}
}

//TODO: TO: and FROM: sections of RCPT and MAIL commands. this breaks everything.
impl std::str::FromStr for Command {
	type Err = ParseCommandError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let command = s.split_once(' ').unwrap_or((s, ""));
		match (command.0.to_ascii_uppercase().as_str(), command.1) {
			("HELO", client_domain) => Ok(Command::Helo(Domain::from_str(client_domain.trim())?)),
			("EHLO", client_domain) => Ok(Command::Ehlo(Domain::from_str(client_domain.trim())?)),
			("MAIL", reverse_path) => {
				Ok(Command::Mail(ReversePath::from_str(reverse_path.trim())?))
			}
			("RCPT", forward_path) => {
				Ok(Command::Rcpt(ForwardPath::from_str(forward_path.trim())?))
			}
			("DATA", "") => Ok(Command::Data),
			("RSET", "") => Ok(Command::Rset),
			("VRFY", target) => Ok(Command::Vrfy(target.trim().to_owned())),
			("EXPN", list) => Ok(Command::Expn(list.trim().to_owned())),
			("HELP", command) => Ok(Command::Help(command.trim().to_owned())),
			("NOOP", _) => Ok(Command::Noop),
			("QUIT", "") => Ok(Command::Quit),
			_ => Err(ParseCommandError::InvalidCommand),
		}
	}
}

#[derive(Error, Debug)]
pub enum ParseCommandError {
	#[error("unknown command")]
	InvalidCommand,
	#[error("invalid path")]
	InvalidPath(#[from] ParsePathError),
	#[error("invalid domain")]
	InvalidDomain(#[from] ParseDomainError),
}
