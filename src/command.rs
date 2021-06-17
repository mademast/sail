use pest::Parser;
use pest_derive::*;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct GrammarParser;

pub enum Command {
	Helo(Result<String>),
}

impl Command {
	fn verify_domain(domain: &str) -> bool {
		GrammarParser::parse(Rule::domain, domain).is_ok()
	}
}

pub enum ParseError {}

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn domain() {
		let should_pass = ["domain", "0domain", "domain0", "0-domain", "domain-0"];

		// These should all pass on their own
		for name in should_pass.iter() {
			if Command::verify_domain(name) == false {
				panic!("Command::verify_domain() failed on {}", name)
			}
		}

		// ... as well as when joined with a dot
		for name in should_pass.iter() {
			for name2 in should_pass.iter() {
				let catname = format!("{}.{}", name, name2);
				if Command::verify_domain(&catname) == false {
					panic!("Command::verify_domain() failed on {}", catname)
				}
			}
		}

		// should not allow: leading/trailing period/hyphen
		for name in should_pass.iter() {
			let fmtname = format!(".{}", name);
			if Command::verify_domain(&fmtname) == true {
				panic!("Command::verify_domain() succeeded on {}", fmtname)
			}

			let fmtname = format!("{}.", name);
			if Command::verify_domain(&fmtname) == true {
				panic!("Command::verify_domain() succeeded on {}", fmtname)
			}

			let fmtname = format!("-{}", name);
			if Command::verify_domain(&fmtname) == true {
				panic!("Command::verify_domain() succeeded on {}", fmtname)
			}

			let fmtname = format!("{}-", name);
			if Command::verify_domain(&fmtname) == true {
				panic!("Command::verify_domain() succeeded on {}", fmtname)
			}
		}
	}
}
