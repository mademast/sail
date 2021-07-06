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
}
