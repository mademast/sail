use pest::Parser;
use pest_derive::*;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct GrammarParser;

pub struct ArgParser;
impl ArgParser {
	pub fn validate_domain(domain: &str) -> bool {
		GrammarParser::parse(Rule::domain, domain).is_ok()
	}
}

#[cfg(test)]
mod test {
	use super::*;

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
}
