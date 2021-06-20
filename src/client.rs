#[derive(Default)]
pub struct Client {
	state: State,
	reply: String,
	reverse_path: String,
	forward_path: Vec<String>,
	data: Vec<String>,
}

use crate::command::Command;
use crate::response::ResponseCode;

impl Client {
	pub fn initiate(
		forward_path: Vec<String>,
		reverse_path: String,
		data: Vec<String>,
	) -> (Self, String) {
		(
			Self {
				reverse_path,
				forward_path,
				data,
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
					Some(Command::Mail(self.reverse_path.clone()))
				}
				_ => todo!(),
			},
			State::SentReversePath => match code {
				ResponseCode::Okay => {
					self.state = State::SendingForwardPaths;
					Some(Command::Mail(self.forward_path.pop()?))
				}
				_ => todo!(),
			},
			State::SendingForwardPaths => {
				if let Some(path) = self.forward_path.pop() {
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
