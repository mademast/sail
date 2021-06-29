mod domain;
mod path;
mod validator;

pub use domain::*;
pub use path::*;
pub use validator::*;

#[cfg(test)]
pub mod test {
	use std::str::FromStr;

	use super::*;

	pub fn valid_hostnames() -> Vec<String> {
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

	pub fn invalid_hostnames() -> Vec<String> {
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

	pub fn valid_address_literals() -> Vec<String> {
		let mut valid = vec![];

		valid.push("[10.0.0.0]".into());
		valid.push("[192.168.1.1]".into());
		valid.push("[IPv6:a0:40:29:bf:de:28:8c:ea]".into()); //full
		valid.push("[IPv6:a0:40:00:00:de:28:8c:ea]".into()); //full with 2 nulls
		valid.push("[IPv6:a0:40:00:00:00:00:8c:ea]".into()); //full with 4 nulls
		valid.push("[IPv6:a0:40::de:28:8c:ea]".into()); //compressed
		valid.push("[IPv6:a0:40::8c:ea]".into()); //compressed replace 4 nulls

		//TODO: push IPv6v4 full and compressed literals, too. Don't forget to add to invalid

		valid
	}

	pub fn invalid_address_literals() -> Vec<String> {
		let mut valid = vec![];

		valid.push("[10.0.0.0".into()); // unclosed brackets
		valid.push("10.0.0.1]".into()); // unoponed
		valid.push("[192.168.1.256]".into()); // invalid IPv4

		valid.push("[a0:40:29:bf:de:28:8c:ea]".into()); //no IPv6 tag
		valid.push("[IPv6:192.168.1.1]".into()); //IPv6 but it's v4

		valid.push("[IPv6:a0:40:29:bf:de:28:8c:ea:ef]".into()); //full, but too much
		valid.push("[IPv6:a0:40:29:bf:de:28:8c:gg]".into()); //invalid hex

		valid.push("[IPv6:a0:40:::de:28:8c:ea]".into()); //compressed, but too many colons
		valid.push("[IPv6:a0:40:29:bf:de:28::]".into()); //ends compressed
		valid.push("[IPv6:::40:29:bf:de:28:8c:ea]".into()); // starts compressed

		//TODO: Weird IPv6 and v4 portmanteau invalids

		valid
	}

	pub fn valid_domains() -> Vec<String> {
		let mut valid = valid_hostnames();
		valid.extend(valid_address_literals());

		valid
	}

	pub fn invalid_domains() -> Vec<String> {
		let mut invalid = invalid_hostnames();
		invalid.extend(invalid_address_literals());

		invalid
	}

	pub fn valid_localparts() -> Vec<String> {
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

	#[test]
	pub fn domain_pass() {
		let strings = valid_domains();

		for domain in strings {
			assert!(Domain::from_str(&domain).is_ok())
		}
	}

	#[test]
	pub fn domain_fail() {
		let strings = invalid_hostnames();

		for domain in strings {
			assert!(Domain::from_str(&domain).is_err())
		}
	}
}
