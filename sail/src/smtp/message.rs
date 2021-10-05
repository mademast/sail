use super::args::{ForeignPath, ForwardPath, Path, ReversePath};

pub struct Message {
	pub headers: Vec<(String, String)>,
	pub body: String,
}

#[derive(Default, Clone, Debug)]
pub struct Envelope {
	pub reverse_path: ReversePath,
	pub forward_paths: Vec<ForwardPath>,
	pub data: String,
}

impl Envelope {
	pub fn new() -> Self {
		Self {
			reverse_path: ReversePath::Null,
			forward_paths: vec![],
			data: String::new(),
		}
	}

	pub fn into_parts(self) -> (ReversePath, Vec<ForwardPath>, String) {
		let Envelope {
			reverse_path,
			forward_paths,
			data,
		} = self;
		(reverse_path, forward_paths, data)
	}

	pub fn into_undeliverable<S: Into<String>>(self, reason: S) -> Option<Self> {
		match self.reverse_path {
			ReversePath::Null => None,
			ReversePath::Regular(reverse) => Some(Self::undeliverable(reason.into(), reverse)),
		}
	}

	pub fn undeliverable<S: Into<String>>(reason: S, reverse_path: Path) -> Self {
		Self {
			reverse_path: ReversePath::Null,
			forward_paths: vec![ForwardPath::Regular(reverse_path)],
			data: reason.into(), //TODO: Genny: pls make this properly formatted with headers and such, i beg of you
		}
	}

	pub fn push<S: AsRef<str>>(&mut self, line: S) {
		self.data.push_str(line.as_ref());
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
	pub data: String,
}

impl ForeignEnvelope {
	pub fn from_parts(
		reverse_path: ReversePath,
		forward_paths: Vec<ForeignPath>,
		data: String,
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
			data: String::new(),
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
