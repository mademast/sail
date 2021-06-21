#[derive(Default)]
pub struct Client {
	state: State,
	reply: String,
	message: Message,
}

use std::collections::HashSet;
use std::net::IpAddr;

use trust_dns_resolver::Resolver;

use crate::command::Command;
use crate::message::Message;
use crate::response::ResponseCode;

impl Client {
	pub fn initiate(
		forward_paths: Vec<String>,
		reverse_path: String,
		data: Vec<String>,
	) -> (Self, String) {
		(
			Self {
				message: Message {
					reverse_path,
					forward_paths,
					data,
				},
				..Default::default()
			},
			Command::Ehlo("Sail".to_string()).as_string(),
		)
	}
	pub fn push(&mut self, reply: &str) -> Option<Command> {
		self.reply.push_str(reply);

		if !self.reply.ends_with("\r\n") {
			return None;
		}

		//todo: process shouldExit and sendingData state variants

		self.process_reply()
	}
	fn process_reply(&mut self) -> Option<Command> {
		if self.reply.len() < 3 || !self.reply.is_ascii() {
			return None;
		}
		let (code, text) = self.reply.split_at(3);

		//todo: parse multiline replies e.g. ehlo
		//todo: parse unknown response codes according to their first digit
		let code = ResponseCode::from_code(code.parse().ok()?)?;

		match self.state {
			State::Initiated => match code {
				ResponseCode::ServiceReady => {
					self.state = State::Greeted;
					Some(Command::Ehlo("Sail".to_string()))
				}
				_ => todo!(),
			},
			State::Greeted => match code {
				ResponseCode::Okay => {
					self.state = State::SentReversePath;
					Some(Command::Mail(self.message.reverse_path.clone()))
				}
				_ => todo!(),
			},
			State::SentReversePath => match code {
				ResponseCode::Okay => {
					self.state = State::SendingForwardPaths;
					Some(Command::Mail(self.message.forward_paths.pop()?))
				}
				_ => todo!(),
			},
			State::SendingForwardPaths => {
				if let Some(path) = self.message.forward_paths.pop() {
					match code {
						ResponseCode::Okay | ResponseCode::UserNotLocalWillForward => {
							Some(Command::Mail(path))
						}
						_ => todo!(),
					}
				} else {
					match code {
						ResponseCode::Okay | ResponseCode::UserNotLocalWillForward => {
							self.state = State::SendingData;
							Some(Command::Data)
						}
						_ => todo!(),
					}
				}
			}
			State::SendingData => unreachable!(),
			State::ShouldExit => unreachable!(),
		}
	}

	fn get_mx_record(fqdn: &str) -> Option<IpAddr> {
		let mut resolver = Resolver::default().ok()?;
		let mut mx_rec: Vec<(u16, String)> = resolver
			.mx_lookup(fqdn)
			.ok()?
			.iter()
			.map(|mx| (mx.preference(), mx.exchange().to_string()))
			.collect();
		mx_rec.sort_by(|(pref1, _), (pref2, _)| pref1.cmp(pref2));
		let mx_domain = mx_rec.first()?.1.clone();
		let mx_ip = resolver.lookup_ip(mx_domain).ok()?;
		mx_ip.iter().next()
	}

	fn run(self) {
		let domains: HashSet<&str> = self
			.message
			.forward_paths
			.iter()
			.map(|path| path.split_once('@').unwrap().1)
			.collect();

		let mut paths_by_domain: Vec<(&str, Vec<String>)> = vec![];

		for domain in domains {
			paths_by_domain.push(
				(domain,
				self.message
					.forward_paths
					.clone()
					.into_iter()
					.filter(|path| path.split_once('@').unwrap().1 == domain)
					.collect())
			)
		}

		for (domain, paths) in paths_by_domain {
			if let Some(address) = Self::get_mx_record(domain) {
				todo!() //todo: genny help we need to make tcp connections or something this is probably not the place to do it tho
			}
		}
	}
}

enum State {
	Initiated,
	Greeted,
	SentReversePath,
	SendingForwardPaths,
	SendingData,
	ShouldExit,
}

impl Default for State {
	fn default() -> Self {
		State::Initiated
	}
}
