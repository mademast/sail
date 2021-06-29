use crate::args::{Domain, ParseDomainError, Validator};
use std::{
	fmt::{Display, Formatter},
	str::FromStr,
};
use thiserror::Error;

pub struct Path {
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

impl Path {
	fn parse_naked_path(naked: &str) -> Result<Self, ParsePathError> {
		let splits = naked.rsplit_once("@");

		if let Some((local_part, domain)) = splits {
			// Check if it's an address literal first, and if it isn't, check if it's a domain
			if Validator::validate_local_part(local_part) {
				Ok(Self {
					local_part: local_part.to_string(),
					domain: Domain::from_str(domain)?,
				})
			} else {
				Err(ParsePathError::InvalidLocalPart)
			}
		} else {
			Err(ParsePathError::NoAtSign)
		}
	}
}

impl Display for Path {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "<{}@{}>", self.local_part, self.domain)
	}
}

impl Display for ForwardPath {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Postmaster => write!(f, "<postmaster>"),
			Self::Regular(path) => write!(f, "{}", path),
		}
	}
}
impl Display for ReversePath {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Null => write!(f, "<>"),
			Self::Regular(path) => write!(f, "{}", path),
		}
	}
}

impl FromStr for Path {
	type Err = ParsePathError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if let Some(path) = s.strip_prefix("<") {
			if let Some(stripped) = path.strip_suffix(">") {
				if !stripped.starts_with('@') {
					// ADLs have to start with @
					Self::parse_naked_path(stripped)
				} else {
					let splits: Vec<&str> = stripped.splitn(2, ":").collect();

					if splits.len() != 2 {
						Err(ParsePathError::AdlWithoutColon)
					} else {
						Self::parse_naked_path(splits[1])
					}
				}
			} else {
				Err(ParsePathError::Brackets)
			}
		} else {
			Err(ParsePathError::Brackets)
		}
	}
}

#[derive(Error, Debug)]
pub enum ParsePathError {
	#[error("no enclosing angle brackets")]
	Brackets,
	#[error("no @")]
	NoAtSign,
	#[error("adl is malformed")]
	AdlWithoutColon,
	#[error("invalid local part")]
	InvalidLocalPart,
	#[error("invalid domain")]
	InvalidDomain(#[from] ParseDomainError),
}
