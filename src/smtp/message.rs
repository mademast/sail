use super::args::{ForwardPath, ReversePath};

#[derive(Default, Clone, Debug)]
pub struct Message {
	pub reverse_path: ReversePath,
	pub forward_paths: Vec<ForwardPath>,
	pub data: Vec<String>,
}

impl Message {
	pub fn into_parts(self) -> (ReversePath, Vec<ForwardPath>, Vec<String>) {
		match self {
			Message {
				reverse_path,
				forward_paths,
				data,
			} => (reverse_path, forward_paths, data),
		}
	}

	pub fn undeliverable(reasons: Vec<String>, reverse_path: ReversePath) -> Option<Self> {
		match reverse_path {
			ReversePath::Null => None,
			ReversePath::Regular(path) => Some(Self {
				reverse_path: ReversePath::Null,
				forward_paths: vec![ForwardPath::Regular(path)],
				data: reasons,
			}),
		}
	}
}
