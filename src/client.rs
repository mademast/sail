#[derive(Default, Clone)]
pub struct Client {
	state: State,
	reply: String,
	message: Message,
}

use std::collections::HashSet;
use std::net::IpAddr;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use trust_dns_resolver::Resolver;

use crate::command::Command;
use crate::message::Message;
use crate::response::ResponseCode;

impl Client {
	pub fn initiate(forward_paths: Vec<String>, reverse_path: String, data: Vec<String>) -> Self {
		Self {
			message: Message {
				reverse_path,
				forward_paths,
				data,
			},
			..Default::default()
		}
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
			.map(|path| path.split_once('@').unwrap().1) //map paths to the second half of the string
			.collect();

		let mut paths_by_domain: Vec<(&str, Vec<String>)> = vec![];

		for domain in domains {
			paths_by_domain.push((
				domain,
				self.message
					.forward_paths
					.clone()
					.into_iter()
					.filter(|path| path.split_once('@').unwrap().1 == domain) //filter for paths to the current domain
					.collect(),
			))
		}

		for (domain, paths) in paths_by_domain {
			//parse ipv4 and ipv6 literals
			let address = if let Some(address) = domain.strip_prefix("[Ipv6:") {
				let address: IpAddr = address.strip_suffix("]").unwrap().parse().unwrap();
				address
			} else if let Some(address) = domain.strip_prefix("[") {
				let address: IpAddr = address.strip_suffix("]").unwrap().parse().unwrap();
				address
			} else if let Some(address) = Self::get_mx_record(domain) {
				address
			} else {
				unreachable!()
			};

			tokio::spawn(Self::send_to_ip(
				address,
				paths,
				self.message.reverse_path.clone(),
				self.message.data.clone(),
			));

			todo!() //todo: genny help we need to make tcp connections or something this is probably not the place to do it tho
		}
	}
	async fn send_to_ip(addr: IpAddr, paths: Vec<String>, reverse_path: String, data: Vec<String>) {
		let mut stream = TcpStream::connect(format!("{}{}", addr, "25"))
			.await
			.unwrap();
		let mut client = Self::initiate(paths, reverse_path, data);

		let mut buf = vec![0; 1024];

		while !client.should_exit() {
			let read = stream.read(&mut buf).await.unwrap();

			// A zero sized read, this connection has died or been terminated by the server
			if read == 0 {
				println!("Connection unexpectedly closed by server");
				return;

				let command = client.push(String::from_utf8_lossy(&buf[..read]).as_ref());

				if let Some(command) = command {
					stream.write_all(command.as_string().as_bytes()).await.unwrap();
				}
			}
		}
	}
	fn should_exit(&self) -> bool {
		self.state == State::ShouldExit
	}
}

#[derive(Clone, Copy, PartialEq)]
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
