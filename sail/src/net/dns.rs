use std::net::IpAddr;

use thiserror::Error;
use trust_dns_resolver::{
	config::{ResolverConfig, ResolverOpts},
	error::{ResolveError, ResolveErrorKind},
	TokioAsyncResolver,
};

pub struct DnsLookup {
	/// A Vec containing possible mail server names. It is sorted in reverse
	/// order of preference. The least preferred servers are at the front of the
	/// Vec. This lets you use Vec::pop to get the next preferred server.
	mx_records: Vec<String>,
	/// A Vec containing possible IP addresses of the last popped domain.
	ip_addresses: Vec<IpAddr>,
}

impl DnsLookup {
	pub async fn new(fqdn: &str) -> Result<Self, DnsLookupError> {
		let resolver =
			TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default())?;

		match resolver.mx_lookup(fqdn).await {
			Ok(mxlookup) => {
				let mut mx_rec: Vec<(u16, String)> = mxlookup
					.iter()
					.map(|mx| (mx.preference(), mx.exchange().to_string()))
					.collect();

				mx_rec.sort_by(|(pref1, _), (pref2, _)| pref1.cmp(pref2).reverse());

				Ok(Self {
					mx_records: mx_rec.into_iter().map(|(_, domain)| domain).collect(),
					ip_addresses: vec![],
				})
			}

			Err(err) => match err.kind() {
				ResolveErrorKind::NoRecordsFound { .. } => Ok(Self {
					mx_records: vec![],
					ip_addresses: Self::get_addresses(fqdn).await?,
				}),
				ResolveErrorKind::Message(_) => todo!(),
				ResolveErrorKind::Msg(_) => todo!(),
				ResolveErrorKind::Io(_) => todo!(),
				ResolveErrorKind::Proto(_) => todo!(),
				ResolveErrorKind::Timeout => todo!(),
			},
		}
	}

	pub async fn next_address(&mut self) -> Result<IpAddr, DnsLookupError> {
		loop {
			match self.ip_addresses.pop() {
				Some(addr) => return Ok(addr),
				None => {
					let domain = self.mx_records.pop().ok_or(DnsLookupError::NoMoreRecords)?;
					self.ip_addresses = Self::get_addresses(&domain).await?;
					continue;
				}
			}
		}
	}

	async fn get_addresses(fqdn: &str) -> Result<Vec<IpAddr>, DnsLookupError> {
		let resolver =
			TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default())?;

		let ip = resolver.lookup_ip(fqdn).await?;
		Ok(ip.iter().collect())
	}
}

#[derive(Debug, Error)]
pub enum DnsLookupError {
	#[error("failed to resolve domain name")]
	ResolveError(#[from] ResolveError),
	#[error("no more MX records to check")]
	NoMoreRecords,
}
