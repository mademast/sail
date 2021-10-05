use std::fmt::Display;

use crate::smtp::Response;

use super::{args::ForeignPath, Command::*, Envelope, ForeignEnvelope, ResponseCode};

#[derive(Default, Clone)]
pub struct Client {
	state: State,
	reply: String,
	Envelope: ForeignEnvelope,

	last_sent_path: Option<ForeignPath>,
	rejected_forward_paths: Vec<ForeignPath>,
}

impl Client {
	pub fn initiate(Envelope: ForeignEnvelope) -> Self {
		Self {
			Envelope,
			..Default::default()
		}
	}

	pub fn push(&mut self, reply: &str) -> Option<Output> {
		self.reply.push_str(reply);

		if !self.reply.ends_with("\r\n") {
			return None;
		}

		self.process_reply()
	}

	pub fn undeliverable(self) -> Option<Envelope> {
		if !self.rejected_forward_paths.is_empty() {
			if let Some(mut msg) = Into::<Envelope>::into(self.Envelope).into_undeliverable("") {
				for path in self.rejected_forward_paths {
					//todo: better Envelopes. can we take the text part of the
					//resposne and put it here?
					msg.push(format!("The host rejected {}", path.0));
				}

				Some(msg)
			} else {
				None
			}
		} else {
			None
		}
	}

	fn invalid_forward(&mut self) {
		self.rejected_forward_paths
			.push(self.last_sent_path.take().unwrap())
	}

	fn process_reply(&mut self) -> Option<Output> {
		//todo: oh no.
		if self.reply.len() < 3
			|| !self.reply.is_ascii()
			|| (self.reply.len() > 4
				&& self.reply.trim_end().split("\r\n").last()?.chars().nth(3)? == '-')
		{
			return None;
		}

		let response: Response = self.reply.parse().unwrap();
		self.reply.clear();

		//todo: parse multiline replies e.g. ehlo
		//todo: handle the unknown response codes
		let code: ResponseCode = response.code;

		// we MUST only exit when we receive a reply from the server
		if self.state == State::SentQuit {
			if code != ResponseCode::ServiceClosing {
				// RFC says server MUST send the 221 service closing
				// we're still allowed to exit if it's not 221
				eprintln!("server sent something other than a 221 to our quit.");
			}

			self.state = State::ShouldExit;
			return None;
		}

		Some(match self.state {
			State::Initiated => match code {
				ResponseCode::ServiceReady => {
					self.state = State::Greeted;
					Output::Command(Ehlo("Sail".parse().unwrap())) //todo: use actual hostname, not Sail
				}
				_ => todo!(),
			},
			State::Greeted => match code {
				ResponseCode::Okay => {
					self.state = State::SentReversePath;
					Output::Command(Mail(self.Envelope.reverse_path.clone()))
				}
				_ => todo!(),
			},
			State::SentReversePath => match code {
				ResponseCode::Okay => {
					self.state = State::SendingForwardPaths;
					Output::Command(Rcpt(self.Envelope.forward_paths.pop()?.into()))
				}
				_ => todo!(),
			},
			State::SendingForwardPaths => {
				if code.is_negative() {
					self.invalid_forward();
				}

				if let Some(path) = self.Envelope.forward_paths.pop() {
					self.last_sent_path = Some(path.clone());
					Output::Command(Rcpt(path.into()))
				} else {
					self.state = State::SentForwardPaths;
					Output::Command(Data)
				}
			}
			State::SentForwardPaths => {
				if code.is_negative() {
					self.invalid_forward();
				}

				match code {
					ResponseCode::StartMailInput => {
						self.state = State::SentData;
						Output::Data(self.Envelope.data.clone())
					}
					_ => todo!(),
				}
			}
			State::SentData => match code {
				ResponseCode::Okay => {
					self.state = State::SentQuit;
					Output::Command(Quit)
				}
				_ => todo!(),
			},
			State::SentQuit => unreachable!(), // handled above
			State::ShouldExit => unreachable!(),
		})
	}

	pub fn should_exit(&self) -> bool {
		self.state == State::ShouldExit
	}
}

#[derive(Clone, Copy, PartialEq)]
enum State {
	Initiated,
	Greeted,
	SentReversePath,
	SendingForwardPaths,
	SentForwardPaths,
	SentData,
	SentQuit,
	ShouldExit,
}

impl Default for State {
	fn default() -> Self {
		State::Initiated
	}
}

pub enum Output {
	Command(super::Command),
	Data(String),
}

impl Display for Output {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Command(command) => write!(f, "{}\r\n", command),
			Self::Data(data) => write!(f, "{}\r\n.\r\n", data),
		}
	}
}
