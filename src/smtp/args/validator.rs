use std::str::FromStr;

use pest::Parser;
use pest_derive::*;

use super::Domain;

#[derive(Parser)]
#[grammar = "smtp/args/smtp_grammar.pest"]
pub struct GrammarParser;

pub struct Validator;
impl Validator {
	pub fn validate_local_part(local: &str) -> bool {
		GrammarParser::parse(Rule::validate_local_part, local).is_ok()
	}

	pub fn validate_domain(domain: &str) -> bool {
		GrammarParser::parse(Rule::validate_domain, domain).is_ok()
	}

	pub fn validate_mailbox(mailbox: &str) -> bool {
		let splits = mailbox.rsplit_once("@");

		if let Some((localpart, domain)) = splits {
			// Check if it's an address literal first, and if it isn't, check if it's a domain
			Self::validate_local_part(localpart) && (Domain::from_str(domain)).is_ok()
		} else {
			false
		}
	}

	pub fn validate_path(path: &str) -> bool {
		if let Some(path) = path.strip_prefix("<") {
			if let Some(stripped) = path.strip_suffix(">") {
				if !stripped.starts_with('@') {
					// ADLs have to start with @
					return Self::validate_mailbox(stripped);
				}

				let splits: Vec<&str> = stripped.splitn(2, ':').collect();

				splits.len() > 2
					&& GrammarParser::parse(Rule::validate_adl, splits[0]).is_ok()
					&& Self::validate_mailbox(splits[1])
			} else {
				false
			}
		} else {
			false
		}
	}

	pub fn validate_reverse_path(reverse: &str) -> bool {
		reverse == "<>" || Self::validate_path(reverse)
	}

	pub fn validate_forward_path(forward: &str) -> bool {
		forward.to_lowercase() == "<postmaster>" || Self::validate_path(forward)
	}
}
