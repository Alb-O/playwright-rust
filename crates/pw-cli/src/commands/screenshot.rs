use std::path::Path;

use crate::browser::BrowserSession;
use crate::context::CommandContext;
use crate::error::Result;
use pw::{ScreenshotOptions, WaitUntil};
use tracing::info;

pub async fn execute(url: &str, output: &Path, full_page: bool, ctx: &CommandContext) -> Result<()> {
    // Resolve output path using project context
    let output = ctx.screenshot_path(output);
    
    info!(target = "pw", %url, path = %output.display(), full_page, browser = %ctx.browser, "screenshot");

    let session = BrowserSession::with_auth_and_browser(
        WaitUntil::NetworkIdle,
        ctx.auth_file(),
        ctx.browser,
    ).await?;
    session.goto(url).await?;

    if let Some(parent) = output.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let screenshot_opts = ScreenshotOptions {
        full_page: Some(full_page),
        ..Default::default()
    };

    session
        .page()
        .screenshot_to_file(&output, Some(screenshot_opts))
        .await?;

    info!(target = "pw", path = %output.display(), "screenshot saved");
    session.close().await
}
