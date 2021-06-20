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

		self.process_reply()
	}
	fn process_reply(&mut self) -> Option<Command> {
		if self.reply.len() < 3 || !self.reply.is_ascii() {
			return None;
		}
		let (code, text) = self.reply.split_at(3);

		//todo: parse unknown response codes according to their first digit
		let code = ResponseCode::from_code(code.parse().ok()?)?;

		match self.state {
			_ => todo!(),
		}
	}
}

enum State {
	Initiated,
	Greeted,
	SentForwardPath,
	SendingReversePaths,
	SendingData,
}

impl Default for State {
	fn default() -> Self {
		State::Initiated
	}
}
