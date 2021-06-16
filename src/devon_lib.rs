pub struct Transaction {
	state: State,
    reverse_path: Option<String>,
    forward_path: Option<String>,
    data: Option<String>,
}

impl Transaction {
	pub fn initiate() -> (Self, String) {
		(
			Self {
				state: State::Initiated,
                reverse_path: None,
                forward_path: None,
                data: None
			},
			String::from("220 Sail Ready"),
		)
	}
    fn parse_command(command: &str) -> Command {
        if command.len() < 4 {
            return Command::Invalid;
        }
        match command.split_at(4) {
            ("HELO", client_domain) => Command::Helo(client_domain.trim().to_owned()),
            ("EHLO", client_domain) => Command::Ehlo(client_domain.trim().to_owned()),
            ("MAIL", reverse_path) => Command::Mail(reverse_path.trim().to_owned()),
            ("RCPT", forward_path) => Command::Rcpt(forward_path.trim().to_owned()),
            ("DATA", _) => Command::Data,
            ("RSET", _) => Command::Rset,
            ("VRFY", target) => Command::Vrfy(target.trim().to_owned()),
            ("EXPN", list) => Command::Expn(list.trim().to_owned()),
            ("HELP", command) => Command::Help(command.trim().to_owned()),
            ("NOOP", _) => Command::Noop,
            ("QUIT", _) => Command::Quit,
            _ => Command::Invalid,
        }
    }
    fn bad_command() -> String {
        String::from("503 bad sequence of commands")
    }
}

enum State {
	Initiated,
    Greeted,
    GotReversePath,
    GotForwardPath,
    LoadingData,
    GotData,
}
enum Command {
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
    Invalid
}
