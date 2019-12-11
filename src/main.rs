//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;

pub use substrate_cli::{VersionInfo, IntoExit, error};

fn main() {
	let version = VersionInfo {
		name: "Moonbeam Node",
		commit: env!("VERGEN_SHA_SHORT"),
		version: env!("CARGO_PKG_VERSION"),
		executable_name: "moonbeam",
		author: "PureStake",
		description: "Moonbeam Node",
		support_url: "https://moonbeam.network",
	};

	if let Err(e) = cli::run(::std::env::args(), cli::Exit, version) {
		eprintln!("Fatal error: {}\n\n{:?}", e, e);
		std::process::exit(1)
	}
}
