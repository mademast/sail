pub enum Command {
	Helo(String),
	Ehlo(String),
	Mail(String),
	Rcpt(String),
	Data,
	Rset,
	Vrfy(String),
	Expn(String),
	Help(String),
	Noop,
	Quit,
	Invalid,
}

impl Command {
    pub fn as_string(&self) -> String {
        match self {
            Command::Helo(parameters) => format!("HELO {}", parameters),
            Command::Ehlo(parameters) => format!("EHLO {}", parameters),
            Command::Mail(parameters) => format!("MAIL {}", parameters),
            Command::Rcpt(parameters) => format!("RCPT {}", parameters),
            Command::Data => String::from("DATA"),
            Command::Rset => String::from("RSET"),
            Command::Vrfy(parameters) => format!("VRFY {}", parameters),
            Command::Expn(parameters) => format!("EXPN {}", parameters),
            Command::Help(parameters) => format!("HELP {}", parameters),
            Command::Noop => String::from("NOOP"),
            Command::Quit => String::from("QUIT"),
            Command::Invalid => String::from("NOOP"), //shouldn't ever be constructed
        }
    }
}