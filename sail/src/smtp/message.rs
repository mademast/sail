use core::fmt;
use std::str::FromStr;

use thiserror::Error;
use time::{format_description::well_known::Rfc2822, OffsetDateTime};

use super::args::{ForeignPath, ForwardPath, ReversePath};

#[derive(Clone, Debug, Default)]
pub struct Message {
	pub headers: Vec<(String, String)>,
	pub body: String,
}

impl Message {
	pub fn new(date: OffsetDateTime, sender: ReversePath, body: String) -> Self {
		let headers = vec![
			(String::from("From"), sender.to_string()),
			(String::from("Date"), date.format(&Rfc2822).unwrap()),
		];

		//TODO: break the body at 80
		Self { headers, body }
	}

	pub fn new_now(sender: ReversePath, body: String) -> Self {
		Self::new(
			OffsetDateTime::now_local().unwrap_or(OffsetDateTime::now_utc()),
			sender,
			body,
		)
	}

	pub fn empty() -> Self {
		Message {
			headers: vec![],
			body: String::new(),
		}
	}
}

impl FromStr for Message {
	type Err = ParseMessageError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut lines = s.lines();

		let mut ret = Message::empty();

		// Just findin' the headers
		for line in &mut lines {
			if line.is_empty() {
				// Empty line indicates the beginning of the body, we're done looking for headers
				break;
			}

			//TODO: Not unwrap, that's for sure
			if line.starts_with(|c| c == ' ' || c == '\t') {
				// This is a folded line, unfold
				if let Some((_, body)) = ret.headers.last_mut() {
					body.push(' ');
					body.push_str(line.trim_start());
				} else {
					return Err(ParseMessageError::MalformedHeaders);
				}
			}

			match line.split_once(':') {
				None => return Err(ParseMessageError::MalformedHeaders),
				Some((field, body)) => ret.headers.push((field.to_owned(), body.to_owned())),
			}
		}

		ret.body = lines.collect::<Vec<&str>>().join("\r\n");

		Ok(ret)
	}
}

impl fmt::Display for Message {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		//TOOD: Conform to the RFC and max line length 80 col

		for (field, body) in &self.headers {
			write!(f, "{}:{}", field, body)?;
		}

		write!(f, "{}", self.body)
	}
}

#[derive(Clone, Copy, Debug, Error)]
pub enum ParseMessageError {
	#[error("The messages headers were malformed")]
	MalformedHeaders,
}

#[derive(Default, Clone, Debug)]
pub struct Envelope {
	pub reverse_path: ReversePath,
	pub forward_paths: Vec<ForwardPath>,
	pub data: Message,
}

impl Envelope {
	pub fn new(reverse: ReversePath) -> Self {
		Self {
			reverse_path: reverse,
			forward_paths: vec![],
			data: Message::empty(),
		}
	}

	pub fn add_recipient(&mut self, forward_path: ForwardPath) {
		self.forward_paths.push(forward_path)
	}

	pub fn into_parts(self) -> (ReversePath, Vec<ForwardPath>, Message) {
		let Envelope {
			reverse_path,
			forward_paths,
			data,
		} = self;
		(reverse_path, forward_paths, data)
	}

	/*pub fn into_undeliverable<S: Into<String>>(
		self,
		primary_host: Domain,
		reason: S,
	) -> Option<Self> {
		match self.reverse_path {
			ReversePath::Null => None,
			ReversePath::Regular(reverse) => {
				Some(Self::undeliverable(primary_host, reverse, reason.into()))
			}
		}
	}

	pub fn undeliverable<S: Into<String>>(
		primary_host: Domain,
		reverse_path: Path,
		reason: S,
	) -> Self {
		Self {
			reverse_path: ReversePath::Null,
			forward_paths: vec![ForwardPath::Regular(reverse_path)],
			data: Message::new(
				SystemTime::now(),
				ReversePath::Regular(format!("postmaster@{}", primary_host).parse().unwrap()),
				reason.into(),
			),
		}
	}*/

	pub fn push<S: AsRef<str>>(&mut self, line: S) {
		self.data.body.push_str(line.as_ref());
	}

	/// Take in a String and remove leading periods from lines. This function
	/// does not expect to receive the final ".\r\n" that ends the DATA command,
	/// but will strip it if it's found.
	pub fn raw_data(&mut self, raw_data: &str) {
		// Remove the final \r\n so we don't get an empty string ending our vector
		let mut lines: Vec<&str> = raw_data.trim_end_matches("\r\n").split("\r\n").collect();

		if lines.ends_with(&["."]) {
			lines.pop();
		}

		for line in lines {
			if line.starts_with('.') {
				//transparency to allow clients to send \r\n.\r\n without breaking SMTP
				self.push(line.strip_prefix('.').unwrap())
			} else {
				self.push(line.to_string())
			}

			self.push("\r\n");
		}
	}
}

#[derive(Debug, Clone)]
pub struct ForeignEnvelope {
	pub reverse_path: ReversePath,
	pub forward_paths: Vec<ForeignPath>,
	pub data: Message,
}

impl ForeignEnvelope {
	pub fn from_parts(
		reverse_path: ReversePath,
		forward_paths: Vec<ForeignPath>,
		data: Message,
	) -> Self {
		Self {
			reverse_path,
			forward_paths,
			data,
		}
	}
}

impl Default for ForeignEnvelope {
	fn default() -> Self {
		Self {
			reverse_path: ReversePath::Null,
			forward_paths: vec![],
			data: Message::default(),
		}
	}
}

impl From<ForeignEnvelope> for Envelope {
	fn from(other: ForeignEnvelope) -> Self {
		Self {
			reverse_path: other.reverse_path,
			forward_paths: other
				.forward_paths
				.into_iter()
				.map(|fpath| fpath.into())
				.collect(),
			data: other.data,
		}
	}
}
