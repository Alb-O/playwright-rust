use clap::Parser;
use pw_cli::{cli::{Cli, Commands}, commands, context::CommandContext, logging};
use tracing::error;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    logging::init_logging(cli.verbose);

    let ctx = match cli.command {
        Commands::Relay { .. } => None,
        _ => Some(CommandContext::new(
            cli.browser,
            cli.no_project,
            cli.auth,
            cli.cdp_endpoint,
        )),
    };

    if let Err(err) = commands::dispatch(cli.command, ctx).await {
        error!(target = "pw", error = %err, "command failed");
        std::process::exit(1);
    }
}
