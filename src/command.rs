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

impl std::str::FromStr for Command {
	type Err = ParseCommandError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let command = s.split_once(' ').unwrap_or((s, ""));

		match (
			command.0.to_ascii_uppercase().as_str(),
			command.1.trim_end(),
		) {
			("HELO", client_domain) => Ok(Command::Helo(client_domain.parse()?)),
			("EHLO", client_domain) => Ok(Command::Ehlo(client_domain.parse()?)),

			("MAIL", reverse_path) => {
				let reverse_path = reverse_path.split_once(':').unwrap_or(("", ""));
				match (reverse_path.0.to_ascii_uppercase().as_str(), reverse_path.1) {
					("FROM", reverse_path) => Ok(Command::Mail(reverse_path.trim_end().parse()?)),
					_ => Err(ParseCommandError::InvalidCommand),
				}
			}

			("RCPT", forward_path) => {
				let forward_path = forward_path.split_once(':').unwrap_or(("", ""));
				match (forward_path.0.to_ascii_uppercase().as_str(), forward_path.1) {
					("TO", forward_path) => Ok(Command::Rcpt(forward_path.trim_end().parse()?)),
					_ => Err(ParseCommandError::InvalidCommand),
				}
			}

			("DATA", "") => Ok(Command::Data),
			("RSET", "") => Ok(Command::Rset),
			("VRFY", target) => Ok(Command::Vrfy(target.to_owned())),
			("EXPN", list) => Ok(Command::Expn(list.to_owned())),
			("HELP", command) => Ok(Command::Help(command.to_owned())),
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

#[cfg(test)]
mod test {
	use super::*;
	use std::{fs::File, io::Read, str::FromStr};

	fn get_test_data(filename: &str) -> Vec<String> {
		let mut file = File::open(format!("testfiles/data/{}", filename)).unwrap();
		let mut buf = String::new();
		file.read_to_string(&mut buf).unwrap();
		buf.lines().map(|line| line.to_owned()).collect()
	}

	fn case_modifier(original: &str) -> [String; 3] {
		[
			original.to_string(),
			original.to_ascii_lowercase(),
			original.to_ascii_uppercase(),
		]
	}

	#[test]
	fn helo() {
		let domains = get_test_data("valid_domains.txt");
		for helo in case_modifier("helo") {
			for domain in &domains {
				Command::from_str(&format!("{} {}", helo, domain)).unwrap();
			}
		}
	}
	
	#[test]
	fn ehlo() {
		let domains = get_test_data("valid_domains.txt");
		for ehlo in case_modifier("ehlo") {
			for domain in &domains {
				Command::from_str(&format!("{} {}", ehlo, domain)).unwrap();
			}
		}
	}
}
