use std::{fs::OpenOptions, io::Write, path::PathBuf, time::SystemTime};

use gethostname::gethostname;
use rand::Rng;
use sail::smtp::Message;

pub struct Maildir {
	maildir: PathBuf,
}

impl Maildir {
	pub fn new<B: Into<PathBuf>>(maildir: B) -> Self {
		Self {
			maildir: maildir.into(),
		}
	}

	pub fn create_directories(&self) -> std::io::Result<()> {
		let mut tmp = self.maildir.clone();
		tmp.push("tmp");
		let mut new = self.maildir.clone();
		new.push("new");
		let mut cur = self.maildir.clone();
		cur.push("cur");

		if !self.maildir.exists() {
			std::fs::create_dir_all(&self.maildir)?;
			std::fs::create_dir(tmp)?;
			std::fs::create_dir(new)?;
			std::fs::create_dir(cur)
		} else {
			std::fs::create_dir_all(tmp)?;
			std::fs::create_dir_all(new)?;
			std::fs::create_dir_all(cur)
		}
	}

	//TODO: Don't unwrap in here. Keep trying until we get a unique name, but these should be truly unique.
	pub fn save(&self, message: Message) -> std::io::Result<()> {
		let unique_name = Self::get_unique_name();
		let mut tmp_path = self.maildir.clone();
		tmp_path.push("tmp");
		tmp_path.push(&unique_name);

		let mut new_path = self.maildir.clone();
		new_path.push("new");
		new_path.push(unique_name);

		{
			let mut tmp = OpenOptions::new()
				.write(true)
				.create_new(true)
				.open(&tmp_path)
				.expect("Failed to open unique file for writing!");
			tmp.write_all(message.to_string().as_bytes())?
		}
		std::fs::rename(tmp_path, new_path)
	}

	//TODO: This is not to Maildir "spec"
	fn get_unique_name() -> String {
		let time = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.expect("SystemTime unwrap failed! Is your system clock before the unix epoch?");
		let middle: u32 = rand::thread_rng().gen();
		let hostname = gethostname().to_string_lossy().replace('/', "-");

		format!("{}.{:08x}.{}", time.as_secs(), middle, hostname)
	}
}
