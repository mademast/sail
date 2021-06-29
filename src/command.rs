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

	fn domains() -> Vec<String> {
		let mut domains = get_test_data("valid_domains.txt");
		let mut ips = get_test_data("valid_ip.txt");

		domains.append(&mut ips);

		domains
	}

	#[test]
	fn helo_ehlo() {
		let domains = domains();
		let helos = case_modifier("helo");
		let ehlos = case_modifier("ehlo");
		for domain in domains {
			for helo in &helos {
				Command::from_str(&format!("{} {}", helo, domain)).unwrap();
			}
			for ehlo in &ehlos {
				Command::from_str(&format!("{} {}", ehlo, domain)).unwrap();
			}
		}
	}

	#[test]
	fn mail_and_rcpt() {
		let domains = domains();
		let local_parts = get_test_data("valid_localparts.txt");
		let mails = case_modifier("MAIL FROM:");
		let rcpts = case_modifier("RCPT TO:");

		for domain in domains {
			for local_part in &local_parts {
				for mail in &mails {
					Command::from_str(&format!("{}<{}@{}>", mail, local_part, domain)).unwrap();
				}
				for rcpt in &rcpts {
					Command::from_str(&format!("{}<{}@{}>", rcpt, local_part, domain)).unwrap();
				}
			}
		}
	}

	#[test]
	fn singletons_test() {
		let datas = case_modifier("data");
		let rsets = case_modifier("rset");
		let noops = case_modifier("noop");
		let quits = case_modifier("quit");

		for data in datas {
			Command::from_str(&data).unwrap();
		}
		for rset in rsets {
			Command::from_str(&rset).unwrap();
		}
		for noop in noops {
			Command::from_str(&noop).unwrap();
		}
		for quit in quits {
			Command::from_str(&quit).unwrap();
		}
	}
}
