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
			// Check if it's an address literal first, and if it isn't, check if it's a domain
			GrammarParser::parse(Rule::validate_local_part, localpart).is_ok()
				&& (std::net::IpAddr::from_str(domain).is_ok()
					|| GrammarParser::parse(Rule::validate_domain, domain).is_ok())
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
		let should_pass = [
			"domaasdas(()89090(7689%&$*&(6-9670*&^)*&^in",
			"0domain",
			"domain0",
			"0-domain",
			"domain-0",
		];

		// These should all pass on their own
		for name in should_pass.iter() {
			assert!(
				ArgParser::validate_domain(name),
				"ArgParser::validate_domain() failed on {}",
				name
			)
		}

		// ... as well as when joined with a dot
		for name in should_pass.iter() {
			for name2 in should_pass.iter() {
				let catname = format!("{}.{}", name, name2);
				assert!(
					ArgParser::validate_domain(&catname),
					"ArgParser::validate_domain() failed on {}",
					catname
				)
			}
		}

		// should not allow: leading/trailing period/hyphen
		for name in should_pass.iter() {
			let fmtname = format!(".{}", name);
			assert!(
				!ArgParser::validate_domain(&fmtname),
				"ArgParser::validate_domain() succeeded on {}",
				fmtname
			);

			let fmtname = format!("{}.", name);
			assert!(
				!ArgParser::validate_domain(&fmtname),
				"ArgParser::validate_domain() succeeded on {}",
				fmtname
			);

			let fmtname = format!("-{}", name);
			assert!(
				!ArgParser::validate_domain(&fmtname),
				"ArgParser::validate_domain() succeeded on {}",
				fmtname
			);

			let fmtname = format!("{}-", name);
			assert!(
				!ArgParser::validate_domain(&fmtname),
				"ArgParser::validate_domain() succeeded on {}",
				fmtname
			);
		}
	}

	#[test]
	fn mailbox() {
		let domains = valid_domains();
		let locals = valid_localparts();

		for domain in domains.iter() {
			for local in locals.iter() {
				let fmtname = format!("{}@{}", local, domain);
				assert!(
					ArgParser::validate_mailbox(&fmtname),
					"ArgParser::validate_mailbox() failed on {}",
					fmtname
				)
			}
		}

		//TODO: Write failing tests
	}

	//TODO: Write tests for ArgParser::validate_path
}
