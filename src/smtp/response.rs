use std::{cmp::Ordering, num::ParseIntError};

use thiserror::Error;

pub struct Response {
	pub code: ResponseCode,
	messages: Vec<String>,
}

impl Response {
	pub fn new(code: ResponseCode) -> Self {
		Self {
			code,
			messages: vec![],
		}
	}

	pub fn with_message<S: Into<String>>(code: ResponseCode, message: S) -> Self {
		Self {
			code,
			messages: vec![message.into()],
		}
	}

	pub fn push(&mut self, message: &str) {
		self.messages.push(message.to_owned());
	}

	pub fn insert(&mut self, index: usize, message: &str) {
		self.messages.insert(index, message.to_owned());
	}

	pub fn code(&self) -> ResponseCode {
		self.code
	}

	pub fn as_string(&self) -> String {
		let mut working = self.messages.clone();
		let mut ret = format!("{} ", self.code.as_code());

		if let Some(message) = working.pop() {
			ret.push_str(&message);
		}

		for message in working {
			ret.insert_str(0, &format!("{}-{}\r\n", self.code.as_code(), message));
		}

		ret.push_str("\r\n");
		ret
	}
}

//todo: genny
// this is bad and makes me sad.
impl std::str::FromStr for Response {
	type Err = ParseResponseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut lines = s.trim_end().rsplit("\r\n");

		let mut response = match lines.next() {
			Some(line) => match line.len().cmp(&3) {
				Ordering::Less => return Err(ParseResponseError::MalformedResponse),
				Ordering::Equal => Response::with_message(line.parse()?, ""),
				Ordering::Greater => {
					let split = line
						.split_once(' ')
						.ok_or(ParseResponseError::MalformedResponse)?;
					let code: ResponseCode = split.0.parse()?;

					Response::with_message(code, split.1.trim())
				}
			},
			None => return Err(ParseResponseError::EmptyString),
		};

		loop {
			match lines.next() {
				Some(line) => {
					if line.len() > 4 {
						let split = line
							.split_once('-')
							.ok_or(ParseResponseError::MalformedResponse)?;
						let code = split.0.parse()?;

						if response.code() != code {
							return Err(ParseResponseError::MixedResponseCode);
						} else {
							response.insert(0, split.1.trim());
						}
					} else {
						return Err(ParseResponseError::MalformedResponse);
					}
				}
				None => return Ok(response),
			}
		}
	}
}

#[derive(Error, Debug)]
pub enum ParseResponseError {
	#[error("multiline responses may not mix reply codes")]
	MixedResponseCode,
	#[error("the response was malformed")]
	MalformedResponse,
	#[error("the response code did not make sense")]
	MalformedResponseCode,
	#[error("the response code was invalid")]
	InvalidResponseCode(#[from] ParseIntError),
	#[error("the reply was empty")]
	EmptyString,
}

#[derive(Clone, Copy, Debug)]
pub enum ResponseCode {
	UnrecognizedCommand,   // 500
	InvalidParameters,     // 501
	CommandNotImplemented, // 502
	BadCommandSequence,    // 503

	SystemStatus,   // 211
	HelpMessage,    // 214
	ServiceReady,   // 220
	ServiceClosing, // 221

	ServiceNotAvailable, // 421 (Service not available, closing transmission channel)

	Okay,                    // 250
	UserNotLocalWillForward, // 251 (will forward to <forward-path>)
	CannotVrfyUser,          // 252 (but will attempt delivery)

	UnableToAcceptParameters,  // 455
	MailRcptParametersError,   // 555
	TemporaryMailFail,         // 450 (action not taken: mailbox unavailable)
	PermanentMailFail,         // 550
	ProcessingError,           // 451
	UserNotLocal,              // 551 (please try <forward-path>)
	InsufficientStorage,       // 452
	ExceededStorageAllocation, // 552
	MailboxNameNotAllowed,     // 553

	StartMailInput,  // 354
	TransactionFail, // 554

	UnknownPositiveCompletion(u16), // 2xx
	UnknownPositiveWaiting(u16),    // 3xx
	UnknownNegativeTemporary(u16),  // 4xx
	UnknownNegativePermanent(u16),  // 5xx
}

impl PartialEq for ResponseCode {
	fn eq(&self, other: &Self) -> bool {
		self.as_code() == other.as_code()
	}
}

impl ResponseCode {
	pub fn from_code(code: u16) -> Option<ResponseCode> {
		let response_code = match code {
			500 => Some(ResponseCode::UnrecognizedCommand),
			501 => Some(ResponseCode::InvalidParameters),
			502 => Some(ResponseCode::CommandNotImplemented),
			503 => Some(ResponseCode::BadCommandSequence),

			211 => Some(ResponseCode::SystemStatus),
			214 => Some(ResponseCode::HelpMessage),
			220 => Some(ResponseCode::ServiceReady),
			221 => Some(ResponseCode::ServiceClosing),

			421 => Some(ResponseCode::ServiceNotAvailable),

			250 => Some(ResponseCode::Okay),
			251 => Some(ResponseCode::UserNotLocalWillForward),
			252 => Some(ResponseCode::CannotVrfyUser),

			455 => Some(ResponseCode::UnableToAcceptParameters),
			555 => Some(ResponseCode::MailRcptParametersError),
			450 => Some(ResponseCode::TemporaryMailFail),
			550 => Some(ResponseCode::PermanentMailFail),
			451 => Some(ResponseCode::ProcessingError),
			551 => Some(ResponseCode::UserNotLocal),
			452 => Some(ResponseCode::InsufficientStorage),
			552 => Some(ResponseCode::ExceededStorageAllocation),
			553 => Some(ResponseCode::MailboxNameNotAllowed),

			354 => Some(ResponseCode::StartMailInput),
			554 => Some(ResponseCode::TransactionFail),
			_ => None,
		};

		if response_code.is_none() {
			match code / 100 {
				2 => Some(ResponseCode::UnknownPositiveCompletion(code)),
				3 => Some(ResponseCode::UnknownPositiveWaiting(code)),
				4 => Some(ResponseCode::UnknownNegativeTemporary(code)),
				5 => Some(ResponseCode::UnknownNegativePermanent(code)),
				_ => None,
			}
		} else {
			response_code
		}
	}

	pub fn as_code(self) -> u16 {
		match self {
			ResponseCode::UnrecognizedCommand => 550,
			ResponseCode::InvalidParameters => 501,
			ResponseCode::CommandNotImplemented => 502,
			ResponseCode::BadCommandSequence => 503,

			ResponseCode::SystemStatus => 211,
			ResponseCode::HelpMessage => 214,
			ResponseCode::ServiceReady => 220,
			ResponseCode::ServiceClosing => 221,

			ResponseCode::ServiceNotAvailable => 421,

			ResponseCode::Okay => 250,
			ResponseCode::UserNotLocalWillForward => 251,
			ResponseCode::CannotVrfyUser => 252,

			ResponseCode::UnableToAcceptParameters => 455,
			ResponseCode::MailRcptParametersError => 555,
			ResponseCode::TemporaryMailFail => 450,
			ResponseCode::PermanentMailFail => 550,
			ResponseCode::ProcessingError => 451,
			ResponseCode::UserNotLocal => 551,
			ResponseCode::InsufficientStorage => 452,
			ResponseCode::ExceededStorageAllocation => 552,
			ResponseCode::MailboxNameNotAllowed => 553,

			ResponseCode::StartMailInput => 354,
			ResponseCode::TransactionFail => 554,

			// Should these enums carry the value they were created from with
			// them so we can convert back to a number losslessly?
			ResponseCode::UnknownPositiveCompletion(code) => code,
			ResponseCode::UnknownPositiveWaiting(code) => code,
			ResponseCode::UnknownNegativeTemporary(code) => code,
			ResponseCode::UnknownNegativePermanent(code) => code,
		}
	}

	pub fn is_negative(&self) -> bool {
		let first = self.as_code() / 100;

		first == 4 || first == 5
	}

	pub fn is_positive(&self) -> bool {
		let first = self.as_code() / 100;

		first == 2 || first == 3
	}
}

impl std::str::FromStr for ResponseCode {
	type Err = ParseResponseError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.len() == 3 {
			Ok(ResponseCode::from_code(s.parse()?)
				.ok_or(ParseResponseError::MalformedResponseCode)?)
		} else {
			Err(ParseResponseError::MalformedResponseCode)
		}
	}
}

#[cfg(test)]
mod test {

	use super::*;

	#[test]
	fn response_code_unknowns() {
		assert_eq!(
			ResponseCode::from_code(299),
			Some(ResponseCode::UnknownPositiveCompletion(299))
		);

		assert_eq!(
			ResponseCode::from_code(399),
			Some(ResponseCode::UnknownPositiveWaiting(399))
		);

		assert_eq!(
			ResponseCode::from_code(499),
			Some(ResponseCode::UnknownNegativeTemporary(499))
		);

		assert_eq!(
			ResponseCode::from_code(599),
			Some(ResponseCode::UnknownNegativePermanent(599))
		);
	}

	#[test]
	fn response_as_string_multiline() {
		let mut resp = Response::with_message(ResponseCode::Okay, "line1");
		resp.push("line2");

		assert_eq!(resp.as_string(), String::from("250-line1\r\n250 line2\r\n"));
	}

	#[test]
	fn response_as_string_singleline() {
		let resp = Response::with_message(ResponseCode::Okay, "line1");

		assert_eq!(resp.as_string(), String::from("250 line1\r\n"));
	}

	#[test]
	fn response_as_string_nolines() {
		let resp = Response::new(ResponseCode::Okay);

		assert_eq!(resp.as_string(), String::from("250 \r\n"));
	}

	#[test]
	fn response_parse_singleline() {
		let string = "250 Okay";
		let response: Response = string.parse().unwrap();

		assert_eq!(response.code, ResponseCode::Okay);
		assert_eq!(response.messages, vec!["Okay"])
	}

	#[test]
	fn response_parse_multiline() {
		let string = "250-Okay\r\n250 Okay Final";
		let response: Response = string.parse().unwrap();

		assert_eq!(response.code, ResponseCode::Okay);
		assert_eq!(response.messages, vec!["Okay", "Okay Final"])
	}
}
