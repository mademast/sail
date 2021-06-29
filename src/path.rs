use crate::{
	argparser::ArgParser,
	domain::{self, Domain},
};
use std::{
	fmt::{Display, Formatter},
	str::FromStr,
};
use thiserror::Error;

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
					let splits = stripped.rsplit_once("@");

					if let Some((local_part, domain)) = splits {
						// Check if it's an address literal first, and if it isn't, check if it's a domain
						if ArgParser::validate_local_part(local_part) {
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
				} else {
					todo!() //todo: ADLs
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
	#[error("invalid local part")]
	InvalidLocalPart,
	#[error("invalid domain")]
	InvalidDomain(#[from] domain::ParseDomainError),
}
