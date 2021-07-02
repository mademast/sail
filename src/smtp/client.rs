use std::fmt::Display;

use super::{
	args::{ForwardPath, Path, ReversePath},
	Command::*,
	ResponseCode,
};

/// A small wrapper around Path as a type-checked, compile-time feature to try
// and stop us from doing stupid things and trying to relay local messages.
#[derive(Debug, Clone)]
pub struct ForeignPath(pub Path);

impl From<ForeignPath> for ForwardPath {
	fn from(other: ForeignPath) -> Self {
		Self::Regular(other.0)
	}
}

#[derive(Debug, Clone)]
pub struct ForeignMessage {
	pub reverse_path: ReversePath,
	pub forward_paths: Vec<ForeignPath>,
	pub data: Vec<String>,
}

impl ForeignMessage {
	pub fn from_parts(
		reverse_path: ReversePath,
		forward_paths: Vec<ForeignPath>,
		data: Vec<String>,
	) -> Self {
		Self {
			reverse_path,
			forward_paths,
			data,
		}
	}
}

impl Default for ForeignMessage {
	fn default() -> Self {
		Self {
			reverse_path: ReversePath::Null,
			forward_paths: vec![],
			data: vec![],
		}
	}
}

#[derive(Default, Clone)]
pub struct Client {
	state: State,
	reply: String,
	message: ForeignMessage,
}

impl Client {
	pub fn initiate(message: ForeignMessage) -> Self {
		Self {
			message,
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

	fn process_reply(&mut self) -> Option<Output> {
		if self.reply.len() < 3 || !self.reply.is_ascii() {
			return None;
		}
		let code = self.reply.split_at(3).0;

		//todo: parse multiline replies e.g. ehlo
		//todo: handle the unknown response codes
		let code = ResponseCode::from_code(code.parse().ok()?)?;

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
					Output::Command(Mail(self.message.reverse_path.clone()))
				}
				_ => todo!(),
			},
			State::SentReversePath => match code {
				ResponseCode::Okay => {
					self.state = State::SendingForwardPaths;
					Output::Command(Rcpt(self.message.forward_paths.pop()?.into()))
				}
				_ => todo!(),
			},
			State::SendingForwardPaths => {
				if let Some(path) = self.message.forward_paths.pop() {
					match code {
						ResponseCode::Okay | ResponseCode::UserNotLocalWillForward => {
							Output::Command(Rcpt(path.into()))
						}
						_ => todo!(),
					}
				} else {
					match code {
						ResponseCode::Okay | ResponseCode::UserNotLocalWillForward => {
							self.state = State::SentForwardPaths;
							Output::Command(Data)
						}
						_ => todo!(),
					}
				}
			}
			State::SentForwardPaths => match code {
				ResponseCode::StartMailInput => {
					self.state = State::SentData;
					Output::Data(self.message.data.clone())
				}
				_ => todo!(),
			},
			State::SentData => match code {
				ResponseCode::Okay => {
					self.state = State::ShouldExit;
					Output::Command(Quit)
				}
				_ => todo!(),
			},
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
	ShouldExit,
}

impl Default for State {
	fn default() -> Self {
		State::Initiated
	}
}

pub enum Output {
	Command(super::Command),
	Data(Vec<String>),
}

impl Display for Output {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Command(command) => write!(f, "{}", command),
			Self::Data(data) => write!(f, "{}\r\n.\r\n", data.join("\r\n")),
		}
	}
}
