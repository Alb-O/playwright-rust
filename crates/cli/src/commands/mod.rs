mod auth;
pub(crate) mod click;
mod connect;
pub(crate) mod contract;
mod daemon;
pub(crate) mod def;
mod engine;
pub(crate) mod exec_flow;
pub(crate) mod fill;
pub(crate) mod flow;
pub(crate) mod graph;
mod har;
pub mod init;
pub(crate) mod navigate;
pub(crate) mod page;
mod profile;
mod protect;
pub(crate) mod registry;
pub(crate) mod screenshot;
mod session;
mod tabs;
pub mod test;
pub(crate) mod wait;

use crate::cli::{Cli, Commands};
use crate::error::Result;

pub async fn dispatch(cli: Cli) -> Result<()> {
	match cli.command {
		Commands::Exec(args) => engine::run_exec(args, cli.format).await?,
		Commands::Batch(args) => engine::run_batch(args, cli.format).await?,
		Commands::Profile(args) => engine::run_profile(args.action, cli.format).await?,
		Commands::Daemon(args) => engine::run_daemon(args.action, cli.format).await?,
	}

	Ok(())
}
