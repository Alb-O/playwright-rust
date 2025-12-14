use crate::browser::{js, BrowserSession};
use crate::context::CommandContext;
use crate::error::Result;
use crate::types::{ElementCoords, IndexedElementCoords};
use pw::WaitUntil;
use tracing::info;

pub async fn execute_single(url: &str, selector: &str, ctx: &CommandContext) -> Result<()> {
    info!(target = "pw", %url, %selector, browser = %ctx.browser, "coords single");
    let session = BrowserSession::with_auth_and_browser(
        WaitUntil::NetworkIdle,
        ctx.auth_file(),
        ctx.browser,
        ctx.cdp_endpoint(),
    ).await?;
    session.goto(url).await?;

    let result_json = session
        .page()
        .evaluate_value(&js::get_element_coords_js(selector))
        .await?;

    if result_json == "null" {
        println!("Element not found or not visible");
    } else {
        let coords: ElementCoords = serde_json::from_str(&result_json)?;
        println!("{}", serde_json::to_string_pretty(&coords)?);
    }

    session.close().await
}

pub async fn execute_all(url: &str, selector: &str, ctx: &CommandContext) -> Result<()> {
    info!(target = "pw", %url, %selector, browser = %ctx.browser, "coords all");
    let session = BrowserSession::with_auth_and_browser(
        WaitUntil::NetworkIdle,
        ctx.auth_file(),
        ctx.browser,
        ctx.cdp_endpoint(),
    ).await?;
    session.goto(url).await?;

    let results_json = session
        .page()
        .evaluate_value(&js::get_all_element_coords_js(selector))
        .await?;

    let results: Vec<IndexedElementCoords> = serde_json::from_str(&results_json)?;
    println!("{}", serde_json::to_string_pretty(&results)?);

    session.close().await
}
