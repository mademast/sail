use super::Validator;
use std::{
	fmt::Display,
	net::{AddrParseError, IpAddr},
};
use thiserror::Error;

/// A Domain as defined by RFC. This can either be a fully-qualified domain name or an IP literal.
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
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

	/// Parses a correctly formed Domain into this struct
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		if Validator::validate_domain(s) {
			Ok(Self::FQDN(s.into()))
		} else if let Some(stripped) = s.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
			let ip = if let Some(ipv6_literal) = stripped.strip_prefix("IPv6:") {
				// Only parse ipv6 if it claims to be one
				IpAddr::V6(ipv6_literal.parse()?)
			} else {
				IpAddr::V4(stripped.parse()?)
			};
			Ok(Self::Literal(ip))
		} else {
			Err(ParseDomainError::InvalidDomain)
		}
	}
}

#[derive(Error, Debug)]
pub enum ParseDomainError {
	#[error("failed to parse address")]
	AddrParseError(#[from] AddrParseError),
	#[error("invalid domain or address")]
	InvalidDomain,
}
