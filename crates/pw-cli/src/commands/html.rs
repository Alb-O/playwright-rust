use crate::browser::BrowserSession;
use crate::context::CommandContext;
use crate::error::Result;
use pw::WaitUntil;
use tracing::info;

pub async fn execute(url: &str, selector: &str, ctx: &CommandContext) -> Result<()> {
    if selector == "html" {
        info!(target = "pw", %url, browser = %ctx.browser, "get full page HTML");
    } else {
        info!(target = "pw", %url, %selector, browser = %ctx.browser, "get HTML for selector");
    }

    let session = BrowserSession::with_auth_and_browser(
        WaitUntil::NetworkIdle,
        ctx.auth_file(),
        ctx.browser,
    ).await?;
    session.goto(url).await?;

    let locator = session.page().locator(selector).await;
    let html = locator.inner_html().await?;

    println!("{html}");
    session.close().await
}
