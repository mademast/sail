use super::args::{ForeignPath, ForwardPath, Path, ReversePath};

#[derive(Default, Clone, Debug)]
pub struct Message {
	pub reverse_path: ReversePath,
	pub forward_paths: Vec<ForwardPath>,
	pub data: String,
}

impl Message {
	pub fn into_parts(self) -> (ReversePath, Vec<ForwardPath>, String) {
		let Message {
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
			data: reason.into(),
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
pub struct ForeignMessage {
	pub reverse_path: ReversePath,
	pub forward_paths: Vec<ForeignPath>,
	pub data: String,
}

impl ForeignMessage {
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

impl Default for ForeignMessage {
	fn default() -> Self {
		Self {
			reverse_path: ReversePath::Null,
			forward_paths: vec![],
			data: String::new(),
		}
	}
}

impl From<ForeignMessage> for Message {
	fn from(other: ForeignMessage) -> Self {
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

pub struct UndeliverableNotice {
	pub forward_path: ForwardPath,
	pub reason: String,
}
