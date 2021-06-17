use std::str::FromStr;

use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct GrammarParser;

pub struct ArgParser;
impl ArgParser {
	pub fn validate_domain(domain: &str) -> bool {
		GrammarParser::parse(Rule::validate_domain, domain).is_ok()
	}

	//TODO: Accept address literals as they appear for RFC5321. We need to
	//handle general literals as well as IPV6 having "IPv6:" before it
	pub fn validate_mailbox(mailbox: &str) -> bool {
		let splits = mailbox.rsplit_once("@");

		if let Some((localpart, domain)) = splits {
			if GrammarParser::parse(Rule::validate_local_part, localpart).is_err() {
				false
			} else if std::net::IpAddr::from_str(domain).is_err()
				&& GrammarParser::parse(Rule::validate_domain, domain).is_err()
			{
				// Check if it's an address literal first, and if it isn't, check if it's a domain
				false
			} else {
				true
			}
		} else {
			false
		}
	}

	pub fn validate_path(path: &str) -> bool {
		if let Some(path) = path.strip_prefix("<") {
			if let Some(stripped) = path.strip_suffix(">") {
				if !stripped.starts_with("@") {
					// ADLs have to start with @
					return Self::validate_mailbox(stripped);
				}

				let splits: Vec<&str> = stripped.splitn(2, ":").collect();

				if splits.len() < 2 {
					false
				} else if GrammarParser::parse(Rule::validate_adl, splits[0]).is_err() {
					false
				} else if !Self::validate_mailbox(splits[1]) {
					false
				} else {
					true
				}
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
		Self::validate_path(forward)
	}
}

#[cfg(test)]
mod test {
	use super::*;

	fn valid_domains() -> Vec<String> {
		let mut valid = vec![];
		let should_pass = ["domain", "0domain", "domain0", "0-domain", "domain-0"];

		// These should all pass on their own
		for name in should_pass.iter() {
			valid.push(name.to_string());
		}

		// ... as well as when joined with a dot
		for name in should_pass.iter() {
			for name2 in should_pass.iter() {
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
		for name in should_pass.iter() {
			if ArgParser::validate_domain(name) == false {
				panic!("ArgParser::validate_domain() failed on {}", name)
			}
		}

		// ... as well as when joined with a dot
		for name in should_pass.iter() {
			for name2 in should_pass.iter() {
				let catname = format!("{}.{}", name, name2);
				if ArgParser::validate_domain(&catname) == false {
					panic!("ArgParser::validate_domain() failed on {}", catname)
				}
			}
		}

		// should not allow: leading/trailing period/hyphen
		for name in should_pass.iter() {
			let fmtname = format!(".{}", name);
			if ArgParser::validate_domain(&fmtname) == true {
				panic!("ArgParser::validate_domain() succeeded on {}", fmtname)
			}

			let fmtname = format!("{}.", name);
			if ArgParser::validate_domain(&fmtname) == true {
				panic!("ArgParser::validate_domain() succeeded on {}", fmtname)
			}

			let fmtname = format!("-{}", name);
			if ArgParser::validate_domain(&fmtname) == true {
				panic!("ArgParser::validate_domain() succeeded on {}", fmtname)
			}

			let fmtname = format!("{}-", name);
			if ArgParser::validate_domain(&fmtname) == true {
				panic!("ArgParser::validate_domain() succeeded on {}", fmtname)
			}
		}
	}

	#[test]
	fn mailbox() {
		let domains = valid_domains();
		let locals = valid_localparts();

		for domain in domains.iter() {
			for local in locals.iter() {
				let fmtname = format!("{}@{}", local, domain);
				if ArgParser::validate_mailbox(&fmtname) == false {
					panic!("ArgParser::validate_mailbox() failed on {}", fmtname)
				}
			}
		}

		//TODO: Write failing tests
	}

	//TODO: Write tests for ArgParser::validate_path
}
