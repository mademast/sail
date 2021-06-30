use std::net::IpAddr;

use async_recursion::async_recursion;
use sail::smtp::args::Domain;
use thiserror::Error;
use trust_dns_resolver::{
	config::{ResolverConfig, ResolverOpts},
	error::ResolveError,
	TokioAsyncResolver,
};

pub struct DnsLookup {
	/// A Vec containing possible mail server names. It is sorted in reverse
	/// order of preference. The least prefered servers are at the front of the
	// Vec. This lets you use Vec::pop to get the next preferred server.
	mx_records: Vec<String>,
	/// A Vec containing possible IP addresses of the last popped domain.
	ip_addresses: Vec<IpAddr>,
}

impl DnsLookup {
	pub async fn mx_records(fqdn: &str) -> Result<Self, DnsLookupError> {
		let resolver =
			TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default())?;

		let mut mx_rec: Vec<(u16, String)> = resolver
			.mx_lookup(fqdn)
			.await?
			.iter()
			.map(|mx| (mx.preference(), mx.exchange().to_string()))
			.collect();

		mx_rec.sort_by(|(pref1, _), (pref2, _)| pref1.cmp(pref2).reverse());

		Ok(Self {
			mx_records: mx_rec
				.into_iter()
				.map(|(preference, domain)| domain)
				.collect(),
			ip_addresses: vec![],
		})
	}

	#[async_recursion]
	pub async fn next_address(&mut self) -> Result<IpAddr, DnsLookupError> {
		match self.ip_addresses.pop() {
			Some(addr) => Ok(addr),
			None => {
				let domain = self.mx_records.pop().ok_or(DnsLookupError::NoMoreRecords)?;
				self.ip_addresses = Self::get_addresses(&domain).await?;
				self.next_address().await
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
