use std::path::PathBuf;

struct MailCache {
	cache_base: PathBuf,
}

impl MailCache {
	pub fn new<B: Into<PathBuf>>(cache: B) -> Self {
		Self {
			cache_base: cache.into(),
		}
	}
}
