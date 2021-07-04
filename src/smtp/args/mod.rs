mod domain;
mod localpart;
mod path;
mod validator;

pub use domain::*;
pub use localpart::*;
pub use path::*;
pub use validator::*;

#[cfg(test)]
mod test {
	use std::str::FromStr;

	use super::*;

	fn valid_hostnames() -> Vec<String> {
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

	fn invalid_hostnames() -> Vec<String> {
		let mut invalid = vec![];
		let valid = valid_hostnames();

		for domain in valid {
			// No leading/trailing dots or hyphens
			invalid.push(format!(".{}", domain));
			invalid.push(format!("{}.", domain));
			invalid.push(format!("-{}", domain));
			invalid.push(format!("{}-", domain));
		}

		invalid
	}

	fn valid_address_literals() -> Vec<String> {
		vec![
			String::from("[10.0.0.0]"),
			String::from("[192.168.1.1]"),
			String::from("[IPv6:a0:40:29:bf:de:28:8c:ea]"), //full
			String::from("[IPv6:a0:40:00:00:de:28:8c:ea]"), //full with 2 nulls
			String::from("[IPv6:a0:40:00:00:00:00:8c:ea]"), //full with 4 nulls
			String::from("[IPv6:a0:40::de:28:8c:ea]"),      //compressed
			String::from("[IPv6:a0:40::8c:ea]"),            //compressed replace 4 nulls
		]

		//TODO: push IPv6v4 full and compressed literals, too. Don't forget to add to invalid
	}

	fn invalid_address_literals() -> Vec<String> {
		vec![
			String::from("[10.0.0.0"),                         // unclosed brackets
			String::from("10.0.0.1]"),                         // unopened
			String::from("[192.168.1.256]"),                   // invalid IPv4
			String::from("[a0:40:29:bf:de:28:8c:ea]"),         //no IPv6 tag
			String::from("[IPv6:192.168.1.1]"),                //IPv6 but it's v4
			String::from("[IPv6:a0:40:29:bf:de:28:8c:ea:ef]"), //full, but too much
			String::from("[IPv6:a0:40:29:bf:de:28:8c:gg]"),    //invalid hex
			String::from("[IPv6:a0:40:::de:28:8c:ea]"),        //compressed, but too many colons
		]

		//TODO: Weird IPv6 and v4 portmanteau invalids
	}

	fn valid_domains() -> Vec<String> {
		let mut valid = valid_hostnames();
		valid.extend(valid_address_literals());

		valid
	}

	fn invalid_domains() -> Vec<String> {
		let mut invalid = invalid_hostnames();
		invalid.extend(invalid_address_literals());

		invalid
	}

	fn valid_localparts() -> Vec<String> {
		vec![
			String::from("\"\""),
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

	pub fn invalid_localparts() -> Vec<String> {
		vec![
			String::from(""),
			String::from("\""),         //one quote
			String::from(".user"),      //leading dot
			String::from("user user"),  //space without quote string
			String::from("@user"),      //@ in dot string
			String::from("user."),      //trailing dot
			String::from("\"\"\""),     //triple quote
			String::from("\"user\\\""), //single backslash
		]
	}

	#[test]
	fn domain_pass() {
		let strings = valid_domains();

		for domain in strings {
			assert!(Domain::from_str(&domain).is_ok(), "failed on {}", domain)
		}
	}

	#[test]
	fn domain_fail() {
		let strings = invalid_domains();

		for domain in strings {
			println!("{}", domain);
			assert!(Domain::from_str(&domain).is_err(), "passed on {}", domain)
		}
	}

	#[test]
	fn path_pass() {
		let domains = valid_domains();
		let locals = valid_localparts();

		for domain in domains {
			for local in &locals {
				assert!(
					Path::from_str(&format!("<{}@{}>", local, domain)).is_ok(),
					"failed on {}",
					format!("<{}@{}>", local, domain)
				);
			}
		}
	}

	#[test]
	pub fn path_fail() {
		let invalid_domains = invalid_domains();
		let invalid_locals = invalid_localparts();

		let valid_domains = valid_domains();
		let valid_locals = valid_localparts();

		let test = |local: &String, domain: &String| {
			assert!(
				Path::from_str(&format!("<{}@{}>", local, domain)).is_err(),
				"passed on {}",
				format!("<{}@{}>", local, domain)
			);
		};

		// Should fail if the domain is bad but local good
		for domain in &invalid_domains {
			for local in &valid_locals {
				test(local, domain)
			}
		}

		// Should fail if the local is bad but domian good
		for domain in &valid_domains {
			for local in &invalid_locals {
				test(local, domain)
			}
		}

		// and if they're both bad
		for domain in &invalid_domains {
			for local in &invalid_locals {
				test(local, domain)
			}
		}
	}

	#[test]
	fn reverse_path_null() {
		assert!(ReversePath::from_str("<>").is_ok());
	}

	#[test]
	fn forward_path_postmaster() {
		let postmasters = vec!["postmaster", "POSTMASTER", "Postmaster", "PoStMaStEr"];

		for postmaster in postmasters {
			let path = format!("<{}>", postmaster);
			assert!(ForwardPath::from_str(&path).is_ok(), "failed on {}", path)
		}
	}
}
