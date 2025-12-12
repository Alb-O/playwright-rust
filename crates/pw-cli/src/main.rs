use clap::Parser;
use pw_cli::{cli::Cli, commands, context::CommandContext, logging};
use tracing::error;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    logging::init_logging(cli.verbose);

    // Create command context with global options
    let ctx = CommandContext::new(cli.browser, cli.no_project, cli.auth);

    if let Err(err) = commands::dispatch(cli.command, ctx).await {
        error!(target = "pw", error = %err, "command failed");
        std::process::exit(1);
    }
}
