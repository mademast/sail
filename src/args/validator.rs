use std::str::FromStr;

use pest::Parser;
use pest_derive::*;

use crate::args::Domain;

#[derive(Parser)]
#[grammar = "smtp_grammar.pest"]
pub struct GrammarParser;

pub struct Validator;
impl Validator {
	pub fn validate_local_part(local: &str) -> bool {
		GrammarParser::parse(Rule::validate_local_part, local).is_ok()
	}

	pub fn validate_domain(domain: &str) -> bool {
		GrammarParser::parse(Rule::validate_domain, domain).is_ok()
	}

	//TODO: Accept address literals as they appear for RFC5321. We need to
	//handle general literals as well as IPV6 having "IPv6:" before it
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

#[cfg(test)]
mod test {
	use super::*;

	fn valid_domains() -> Vec<String> {
		let mut valid = vec![];
		let should_pass = ["domain", "0domain", "domain0", "0-domain", "domain-0"];

		// These should all pass on their own
		for name in should_pass {
			valid.push(name.to_string());
		}

		// ... as well as when joined with a dot
		for name in should_pass {
			for name2 in should_pass {
				valid.push(format!("{}.{}", name, name2));
			}
		}

		valid
	}

	fn valid_localparts() -> Vec<String> {
		vec![
			String::from("user"),
			String::from("user24234"),
			String::from("user.user"),
			String::from("user23423.user"),
			String::from("user.user.user"),
			String::from("\"user\""),
			String::from("\"user user\""),
			String::from("\"user \\\" user\""),
			String::from("\"user.user\""),
			String::from("\"user73 456\""),
			String::from("\"user %#W$@\""),
		]
	}

	#[test]
	fn domain() {
		let should_pass = ["domain", "0domain", "domain0", "0-domain", "domain-0"];

		// These should all pass on their own
		for name in should_pass {
			assert!(
				Validator::validate_domain(name),
				"Validator::validate_domain() failed on {}",
				name
			)
		}

		// ... as well as when joined with a dot
		for name in should_pass {
			for name2 in should_pass {
				let catname = format!("{}.{}", name, name2);
				assert!(
					Validator::validate_domain(&catname),
					"Validator::validate_domain() failed on {}",
					catname
				)
			}
		}

		// should not allow: leading/trailing period/hyphen
		for name in should_pass {
			let fmtname = format!(".{}", name);
			assert!(
				!Validator::validate_domain(&fmtname),
				"Validator::validate_domain() succeeded on {}",
				fmtname
			);

			let fmtname = format!("{}.", name);
			assert!(
				!Validator::validate_domain(&fmtname),
				"Validator::validate_domain() succeeded on {}",
				fmtname
			);

			let fmtname = format!("-{}", name);
			assert!(
				!Validator::validate_domain(&fmtname),
				"Validator::validate_domain() succeeded on {}",
				fmtname
			);

			let fmtname = format!("{}-", name);
			assert!(
				!Validator::validate_domain(&fmtname),
				"Validator::validate_domain() succeeded on {}",
				fmtname
			);
		}
	}

	#[test]
	fn mailbox() {
		let domains = valid_domains();
		let locals = valid_localparts();

		for domain in domains {
			for local in &locals {
				let fmtname = format!("{}@{}", local, domain);
				assert!(
					Validator::validate_mailbox(&fmtname),
					"Validator::validate_mailbox() failed on {}",
					fmtname
				)
			}
		}

		//TODO: Write failing tests
	}

	//TODO: Write tests for Validator::validate_address_literal

	//TODO: Write tests for Validator::validate_path
}
