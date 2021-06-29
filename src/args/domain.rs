use crate::args::Validator;
use std::{
	fmt::Display,
	net::{AddrParseError, IpAddr},
};
use thiserror::Error;

#[derive(Clone, Hash, PartialEq, Eq)]
pub enum Domain {
	FQDN(String),
	Literal(IpAddr),
}

impl Display for Domain {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::FQDN(domain) => domain.clone(),
				Self::Literal(ip) => match ip {
					IpAddr::V4(ip) => format!("[{}]", ip),
					IpAddr::V6(ip) => format!("[IPv6:{}]", ip),
				},
			}
		)
	}
}

impl std::str::FromStr for Domain {
	type Err = ParseDomainError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if let Some(literal) = s.strip_prefix("[") {
			if let Some(stripped) = literal.strip_suffix("]") {
				if let Some(ipv6_literal) = stripped.strip_prefix("IPv6:") {
					// Only parse ipv6 if it claims to be one
					Ok(Self::Literal(IpAddr::V6(ipv6_literal.parse()?)))
				} else {
					Ok(Self::Literal(IpAddr::V4(stripped.parse()?)))
				}
			} else {
				Err(ParseDomainError::Brackets)
			}
		} else if Validator::validate_domain(s) {
			Ok(Self::FQDN(s.to_string()))
		} else {
			Err(ParseDomainError::InvalidDomain)
		}
	}
}

#[derive(Error, Debug)]
pub enum ParseDomainError {
	#[error("unmatched brackets")]
	Brackets,
	#[error("failed to parse address")]
	AddrParseError(#[from] AddrParseError),
	#[error("invalid domain or address")]
	InvalidDomain,
}
