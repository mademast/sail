use super::args::{ForwardPath, Path, ReversePath};

#[derive(Default, Clone, Debug)]
pub struct Message {
	pub reverse_path: ReversePath,
	pub forward_paths: Vec<ForwardPath>,
	pub data: Vec<String>,
}

impl Message {
	pub fn into_parts(self) -> (ReversePath, Vec<ForwardPath>, Vec<String>) {
		let Message {
			reverse_path,
			forward_paths,
			data,
		} = self;
		(reverse_path, forward_paths, data)
	}

	pub fn into_undeliverable(self) -> Option<Self> {
		match self.reverse_path {
			ReversePath::Null => None,
			ReversePath::Regular(reverse) => Some(Self::undeliverable(vec![], reverse)),
		}
	}

	pub fn undeliverable(reasons: Vec<String>, reverse_path: Path) -> Self {
		Self {
			reverse_path: ReversePath::Null,
			forward_paths: vec![ForwardPath::Regular(reverse_path)],
			data: reasons,
		}
	}

	pub fn push_line<S: Into<String>>(&mut self, line: S) {
		self.data.push(line.into());
	}

	/// Take in a String and remove leading periods from lines. This function
	/// does not expect to receive the final ".\r\n" that ends the DATA command,
	/// but will strip it if it's found.
	pub fn raw_data(&mut self, raw_data: &str) {
		// Remove the final \r\n so we don't get an empty string ending our vcetor
		let mut lines: Vec<&str> = raw_data.trim_end_matches("\r\n").split("\r\n").collect();

		if lines.ends_with(&["."]) {
			lines.pop();
		}

		for line in lines {
			if line.starts_with('.') {
				//transparency to allow clients to send \r\n.\r\n without breaking SMTP
				self.data.push(line[1..].to_string())
			} else {
				self.data.push(line.to_string())
			}
		}
	}
}
