use thiserror::Error;

use super::Validator;

#[derive(Clone, Debug)]
pub struct LocalPart(String);

impl std::fmt::Display for LocalPart {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}
impl std::str::FromStr for LocalPart {
	type Err = InvalidLocalPart;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if Validator::validate_local_part(s) {
			Ok(Self(s.to_owned()))
		} else {
			Err(InvalidLocalPart::InvalidSyntax)
		}
	}
}

#[derive(Error, Debug)]
pub enum InvalidLocalPart {
	#[error("invalid local part syntax")]
	InvalidSyntax,
}
