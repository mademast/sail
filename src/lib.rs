mod command;

use std::fmt::format;

#[derive(Default)]
pub struct Protocol {
	state: State,
	// Filled when this struct is written to
	command_buffer: String,
	// Filled by MAIL FROM
	reverse_path_buffer: String,
	// Filled by RCPT TO
	forward_path_buffer: String,
	// Filled by DATA
	mail_data_buffer: String,
}

impl Protocol {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn push(&mut self, incoming: &str) -> Option<String> {
		self.command_buffer.push_str(incoming);

		if self.command_is_complete() {
			self.run_command()
		} else {
			None
		}
	}

	fn command_is_complete(&self) -> bool {
		self.command_buffer.ends_with("\r\n")
	}

	//TODO: Maybe have a self.parse_command because there is a lot of grammar for it all.
	fn run_command(&mut self) -> Option<String> {
		let cmd_str = self.command_buffer.trim_end().to_owned();
		let mut split = cmd_str.splitn(2, ' ');
		let cmd = split
			.next()
			.expect("How did this happen? First split was None.");
		let args = split.next();

		match self.state {
			State::WaitingHelo => self.waitinghelo(cmd, args),
		}
	}

	fn waitinghelo(&mut self, cmd: &str, args: Option<&str>) -> Option<String> {
		println!("'{}' '{:?}'", cmd, args);
		if cmd != "HELO" {
			//TODO: Check command is valid and return 503 if it was bad sequence
			return Some(String::from("500 Syntax error\n"));
		}

		if args.is_none() {
			return Some(String::from("501 Expected a hostname!\n"));
		}

		Some(format!("250 Nice to meet you, {}\n", args.unwrap()))
	}
}

pub enum State {
	WaitingHelo,
}

impl Default for State {
	fn default() -> Self {
		State::WaitingHelo
	}
}
