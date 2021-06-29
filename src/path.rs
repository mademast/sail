use std::fmt::{Display, Formatter, Result};

use crate::domain::Domain;

struct Path {
	local_part: String,
	domain: Domain,
}

pub enum ForwardPath {
	Postmaster,
	Regular(Path),
}

pub enum ReversePath {
	Null,
	Regular(Path),
}

impl Display for Path {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		write!(f, "<{}@{}>", self.local_part, self.domain)
	}
}

impl Display for ForwardPath {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			Self::Postmaster => write!(f, "<postmaster>"),
			Self::Regular(path) => write!(f, "{}", path),
		}
	}
}
impl Display for ReversePath {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		match self {
			Self::Null => write!(f, "<>"),
			Self::Regular(path) => write!(f, "{}", path),
		}
	}
}
