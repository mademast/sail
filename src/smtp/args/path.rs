use super::{
	localpart::{InvalidLocalPart, LocalPart},
	Domain, ParseDomainError,
};
use std::{
	fmt::{Display, Formatter},
	str::FromStr,
};
use thiserror::Error;

#[derive(Clone, Debug)]
pub struct Path {
	pub local_part: LocalPart,
	pub domain: Domain,
}

#[derive(Clone, Debug)]
pub enum ForwardPath {
	Postmaster,
	Regular(Path),
}

#[derive(Clone, Debug)]
pub enum ReversePath {
	Null,
	Regular(Path),
}

impl Path {
	fn parse_naked_path(naked: &str) -> Result<Self, ParsePathError> {
		if let Some((local_part, domain)) = naked.rsplit_once("@") {
			// Check if it's an address literal first, and if it isn't, check if it's a domain
			Ok(Self {
				local_part: local_part.parse()?,
				domain: domain.parse()?,
			})
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
				if let Some(stripped) = stripped.strip_prefix('@') {
					let splits: Vec<&str> = stripped.splitn(2, ':').collect();

					if splits.len() != 2 {
						Err(ParsePathError::AdlWithoutColon)
					} else {
						for domain in splits[0].split(",@") {
							Domain::from_str(domain)?;
						}
						Self::parse_naked_path(splits[1])
					}
				} else {
					// ADLs have to start with @
					Self::parse_naked_path(stripped)
				}
			} else {
				Err(ParsePathError::Brackets)
			}
		} else {
			Err(ParsePathError::Brackets)
		}
	}
}

impl FromStr for ForwardPath {
	type Err = ParsePathError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.to_ascii_lowercase() == "<postmaster>" {
			Ok(Self::Postmaster)
		} else if let Some(stripped) = s.strip_suffix(":postmaster>") {
			if let Some(stripped) = stripped.strip_prefix("<@") {
				let domains = stripped.split(",@");
				for domain in domains {
					Domain::from_str(domain)?;
				}
				Ok(Self::Postmaster)
			} else {
				Err(ParsePathError::InvalidAdlSyntax)
			}
		} else {
			Ok(Self::Regular(s.parse()?))
		}
	}
}

impl FromStr for ReversePath {
	type Err = ParsePathError;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if s.to_ascii_lowercase() == "<>" {
			Ok(Self::Null)
		} else {
			Ok(Self::Regular(s.parse()?))
		}
	}
}

impl Default for ReversePath {
	fn default() -> Self {
		Self::Null
	}
}
impl Default for ForwardPath {
	fn default() -> Self {
		Self::Postmaster
	}
}

#[derive(Error, Debug)]
pub enum ParsePathError {
	#[error("no enclosing angle brackets")]
	Brackets,
	#[error("no @")]
	NoAtSign,
	#[error("ADL syntax without colon")]
	AdlWithoutColon,
	#[error("Invalid ADL syntax")]
	InvalidAdlSyntax,
	#[error("invalid local part")]
	InvalidLocalPart(#[from] InvalidLocalPart),
	#[error("invalid domain")]
	InvalidDomain(#[from] ParseDomainError),
}
