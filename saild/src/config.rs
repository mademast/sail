use std::{
	net::{IpAddr, SocketAddr},
	path::PathBuf,
	str::FromStr,
};

use confindent::Confindent;
use getopts::Options;
use sail::smtp::args::{Domain, ForwardPath};
use thiserror::Error;

pub struct Config {
	pub address: IpAddr,
	pub port: u16,
	pub maildir: MaildirTemplate,
	pub hostnames: Vec<Domain>,
	pub relays: Vec<Domain>,
}

#[allow(clippy::or_fun_call)]
impl Config {
	fn print_usage<S: AsRef<str>>(prgm: S, opts: &Options) {
		let brief = format!("Usage: {} [options]", prgm.as_ref());
		println!("{}", opts.usage(&brief));
	}

	pub fn socket_address(&self) -> SocketAddr {
		SocketAddr::new(self.address, self.port)
	}

	pub fn get() -> Option<Self> {
		let args: Vec<String> = std::env::args().collect();

		let mut opts = Options::new();
		opts.optflag("h", "help", "Print this help message");
		opts.optopt(
			"l",
			"listen-address",
			"The IP address Sail will listen for incoming connections on\nDefault: localhost",
			"IP_ADDR",
		);
		opts.optopt(
			"p",
			"port",
			"The port Sail will listen on\nDefault: 25",
			"PORT",
		);
		opts.optopt(
			"c",
			"config",
			"An alternate location to read the config from\nDefault: /etc/sail/sail.conf",
			"PATH",
		);

		let matches = match opts.parse(&args[1..]) {
			Ok(m) => m,
			Err(_e) => return None,
		};

		if matches.opt_present("help") {
			Self::print_usage(&args[0], &opts);
			return None;
		}

		let conf_path = matches
			.opt_str("config")
			.unwrap_or("/etc/sail/sail.conf".into());

		let config = match Confindent::from_file(conf_path) {
			Ok(c) => c,
			Err(_) => match Confindent::from_file("sail.conf") {
				Ok(c) => c,
				Err(err) => {
					eprintln!("failed to parse conf file: {}", err);
					return None;
				}
			},
		};

		// Options specified on the command line take priority. We only take the
		// cli_key and convert to the config key internally so that we can remain
		// consistent.
		let find_value = |cli_key: &str| -> Option<String> {
			let conf_key: String = cli_key
				.split('-')
				.map(|word| {
					// https://stackoverflow.com/a/38406885
					// wow, is this really the best way to Title Case something? omg
					let mut c = word.chars();
					match c.next() {
						None => String::new(),
						Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
					}
				})
				.collect();

			matches
				.opt_str(cli_key)
				.or(config.child_value(conf_key).map(|s| s.into()))
		};

		let address_string = find_value("listen-address").unwrap_or("localhost".into());
		let address = match address_string.parse() {
			Ok(addr) => addr,
			Err(_e) => {
				eprintln!("Failed to parse '{}' as an IP Address", address_string);
				return None;
			}
		};

		let port_string = find_value("port").unwrap_or("25".into());
		let port = match port_string.parse() {
			Ok(p) => p,
			Err(_e) => {
				eprintln!("Failed to parse '{}' as a port", port_string);
				return None;
			}
		};

		let maildir = match config.child_value("Maildir").unwrap().parse() {
			Ok(mdt) => mdt,
			Err(e) => {
				eprintln!("Could not parse Maildir path: {}", e);
				return None;
			}
		};

		let hostnames = match config.child_owned("Hostnames") {
			None => {
				eprintln!("'Hostnames' not found in config. Who are we accepting mail for?");
				return None;
			}
			Some(joined) => Self::parse_domains(&joined)?,
		};

		let relays = match config.child_owned("Relays") {
			None => vec![],
			Some(joined) => {
				//possible future work: allow granular relays - which local-parts are acceptable to relay to?
				//this complicates parsing a bit since roughly anything goes for a local-part
				//i believe confindent has support for nested structures, though - could have this a multi-line structure?
				Self::parse_domains(&joined)?
			}
		};

		Some(Self {
			address,
			port,
			maildir,
			hostnames,
			relays,
		})
	}

	fn parse_domains(joined: &str) -> Option<Vec<Domain>> {
		let splits = joined.split(',');
		let mut names = vec![];
		for split in splits {
			let domain: Domain = match split.parse() {
				Err(_e) => {
					eprintln!("Failed to parse {split} as a domain");
					return None;
				}
				Ok(d) => d,
			};

			names.push(domain);
		}

		Some(names)
	}
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaildirTemplate {
	tokens: Vec<TemplateToken>,
}

impl MaildirTemplate {
	pub fn as_path(&self, forward: &ForwardPath) -> PathBuf {
		PathBuf::from(
			self.tokens
				.clone()
				.into_iter()
				.map(|tok| match tok {
					TemplateToken::Text(text) => text,
					TemplateToken::Variable { name, modifiers } => {
						let mut string = match name {
							MaildirToken::DestinationUser => match forward {
								ForwardPath::Regular(path) => path.local_part.to_string(),
								ForwardPath::Postmaster => String::from("postmaster"),
							},
							MaildirToken::DestinationDomain => match forward {
								ForwardPath::Regular(path) => path.domain.to_string(),
								ForwardPath::Postmaster => todo!(),
							},
						};

						for modi in modifiers {
							match modi {
								TemplateModifier::Lowercase => string = string.to_lowercase(),
								TemplateModifier::Uppercase => string = string.to_uppercase(),
								TemplateModifier::Strip => {
									string = string.split('+').next().unwrap().into()
								}
							}
						}

						string
					}
				})
				.collect::<String>(),
		)
	}
}

impl FromStr for MaildirTemplate {
	type Err = ParseTemplateError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut tokens = vec![];

		let mut curr = s;
		loop {
			match curr.split_once('{') {
				None => {
					if !curr.is_empty() {
						tokens.push(TemplateToken::Text(curr.into()));
					}
					break;
				}
				Some((text, string)) => {
					if !text.is_empty() {
						tokens.push(TemplateToken::Text(text.into()));
					}

					match string.split_once('}') {
						None => return Err(ParseTemplateError::UnclosedVariable),
						Some((variable, string)) => {
							curr = string;
							tokens.push(TemplateToken::parse_variable(variable)?);
						}
					}
				}
			}
		}

		Ok(Self { tokens })
	}
}

#[derive(Clone, Debug, PartialEq)]
enum MaildirToken {
	DestinationUser,
	DestinationDomain,
}

impl FromStr for MaildirToken {
	type Err = ParseMaildirTokenError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"destination user" => Ok(MaildirToken::DestinationUser),
			"destination domain" => Ok(MaildirToken::DestinationDomain),
			_ => Err(ParseMaildirTokenError::UnrecognizedVariable(s.into())),
		}
	}
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ParseMaildirTokenError {
	#[error("'{0}' is not a recognized variable")]
	UnrecognizedVariable(String),
}

#[derive(Clone, Debug, PartialEq)]
enum TemplateToken {
	Text(String),
	Variable {
		name: MaildirToken,
		modifiers: Vec<TemplateModifier>,
	},
}

impl TemplateToken {
	pub fn parse_variable<S: AsRef<str>>(string: S) -> Result<TemplateToken, ParseTemplateError> {
		let string = string.as_ref();

		match string.split_once(':') {
			None => {
				// No modifiers were present
				Ok(TemplateToken::Variable {
					name: string.trim().parse()?,
					modifiers: vec![],
				})
			}
			Some((name, raw_modifiers)) => {
				if raw_modifiers.starts_with("and ") || raw_modifiers.ends_with(" and") {
					return Err(ParseModifiersError::UnbalanceAnd.into());
				}

				let modifiers = raw_modifiers
					.split(" and ")
					.map(|s| s.parse())
					.collect::<Result<Vec<TemplateModifier>, ParseModifiersError>>()?;

				Ok(TemplateToken::Variable {
					name: name.parse()?,
					modifiers,
				})
			}
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum TemplateModifier {
	Lowercase,
	Uppercase,
	Strip,
}

impl FromStr for TemplateModifier {
	type Err = ParseModifiersError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"lowercase" => Ok(TemplateModifier::Lowercase),
			"uppercase" => Ok(TemplateModifier::Uppercase),
			"strip" => Ok(TemplateModifier::Strip),
			_ => Err(ParseModifiersError::UnrecognizedModifier(s.into())),
		}
	}
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ParseTemplateError {
	#[error("Hit the end of the line and the variable was still open!")]
	UnclosedVariable,
	#[error("{0}")]
	MalformedName(#[from] ParseMaildirTokenError),
	#[error("{0}")]
	MalformedModifiers(#[from] ParseModifiersError),
}

#[derive(Clone, Debug, Error, PartialEq)]
pub enum ParseModifiersError {
	#[error("The modifier list started or ended with 'and'")]
	UnbalanceAnd,
	#[error("The modifier '{0}' was not understood")]
	UnrecognizedModifier(String),
}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn template_parse() {
		let tp = MaildirTemplate::from_str(
			"/srv/mail/{destination user:strip and lowercase}/{destination domain:uppercase}",
		)
		.unwrap();

		assert_eq!(
			tp,
			MaildirTemplate {
				tokens: vec![
					TemplateToken::Text(String::from("/srv/mail/")),
					TemplateToken::Variable {
						name: MaildirToken::DestinationUser,
						modifiers: vec![TemplateModifier::Strip, TemplateModifier::Lowercase]
					},
					TemplateToken::Text(String::from("/")),
					TemplateToken::Variable {
						name: MaildirToken::DestinationDomain,
						modifiers: vec![TemplateModifier::Uppercase]
					},
				]
			}
		)
	}

	#[test]
	fn maildir_as_path() {
		let mdtpl: MaildirTemplate =
			"/srv/mail/{destination user:strip and lowercase}/{destination domain:uppercase}"
				.parse()
				.unwrap();

		assert_eq!(
			mdtpl.as_path(&ForwardPath::Regular(
				"<GEN+tag@nyble.dev>".parse().unwrap()
			)),
			PathBuf::from("/srv/mail/gen/NYBLE.DEV")
		)
	}
}
