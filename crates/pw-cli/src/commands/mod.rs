mod auth;
mod click;
mod console;
mod coords;
mod elements;
mod eval;
mod html;
pub mod init;
mod navigate;
mod screenshot;
mod text;
mod wait;

use crate::cli::{AuthAction, Commands};
use crate::context::CommandContext;
use crate::error::{PwError, Result};
use crate::relay;

pub async fn dispatch(command: Commands, ctx: Option<CommandContext>) -> Result<()> {
    match command {
        Commands::Relay { host, port } => relay::run_relay_server(&host, port)
            .await
            .map_err(PwError::Anyhow),
        Commands::Navigate { url } => navigate::execute(&url, ctx.as_ref().unwrap()).await,
        Commands::Console { url, timeout_ms } => {
            console::execute(&url, timeout_ms, ctx.as_ref().unwrap()).await
        }
        Commands::Eval { url, expression } => eval::execute(&url, &expression, ctx.as_ref().unwrap()).await,
        Commands::Html { url, selector } => html::execute(&url, &selector, ctx.as_ref().unwrap()).await,
        Commands::Coords { url, selector } => coords::execute_single(&url, &selector, ctx.as_ref().unwrap()).await,
        Commands::CoordsAll { url, selector } => coords::execute_all(&url, &selector, ctx.as_ref().unwrap()).await,
        Commands::Screenshot { url, output, full_page } => {
            screenshot::execute(&url, &output, full_page, ctx.as_ref().unwrap()).await
        }
        Commands::Click { url, selector } => click::execute(&url, &selector, ctx.as_ref().unwrap()).await,
        Commands::Text { url, selector } => text::execute(&url, &selector, ctx.as_ref().unwrap()).await,
        Commands::Elements { url } => elements::execute(&url, ctx.as_ref().unwrap()).await,
        Commands::Wait { url, condition } => wait::execute(&url, &condition, ctx.as_ref().unwrap()).await,
        Commands::Auth { action } => match action {
            AuthAction::Login { url, output, timeout } => {
                auth::login(&url, &output, timeout, ctx.as_ref().unwrap()).await
            }
            AuthAction::Cookies { url, format } => {
                auth::cookies(&url, &format, ctx.as_ref().unwrap()).await
            }
            AuthAction::Show { file } => auth::show(&file).await,
        },
        Commands::Init { path, template, no_config, no_example, typescript, force, nix } => {
            init::execute(init::InitOptions {
                path,
                template,
                no_config,
                no_example,
                typescript,
                force,
                nix,
            })
        }
    }
}
