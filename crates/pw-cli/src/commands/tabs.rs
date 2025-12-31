use crate::context::CommandContext;
use crate::error::{PwError, Result};
use crate::output::{OutputFormat, ResultBuilder, print_result};
use crate::session_broker::{SessionBroker, SessionRequest};
use pw::WaitUntil;
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TabInfo {
    index: usize,
    title: String,
    url: String,
}

pub async fn list(
    ctx: &CommandContext,
    broker: &mut SessionBroker<'_>,
    format: OutputFormat,
) -> Result<()> {
    let request = SessionRequest::from_context(WaitUntil::Load, ctx);
    let session = broker.session(request).await?;
    let context = session.context();
    let pages = context.pages();

    let mut tabs = Vec::new();
    for (i, page) in pages.iter().enumerate() {
        // Get URL via JS evaluation since page.url() may not be updated for existing tabs
        let url = page.evaluate_value("window.location.href").await.unwrap_or_else(|_| page.url());
        // Strip quotes from JSON string result
        let url = url.trim_matches('"').to_string();
        let title = page.title().await.unwrap_or_default();
        tabs.push(TabInfo {
            index: i,
            title,
            url,
        });
    }

    let result = ResultBuilder::new("tabs list")
        .data(json!({
            "tabs": tabs,
            "count": tabs.len(),
        }))
        .build();

    print_result(&result, format);
    session.close().await
}

pub async fn switch(
    target: &str,
    ctx: &CommandContext,
    broker: &mut SessionBroker<'_>,
    format: OutputFormat,
) -> Result<()> {
    let request = SessionRequest::from_context(WaitUntil::Load, ctx);
    let session = broker.session(request).await?;
    let context = session.context();
    let pages = context.pages();

    let (index, page) = find_page(&pages, target).await?;
    
    // Bring the page to front
    page.bring_to_front().await?;

    let url = page.evaluate_value("window.location.href").await.unwrap_or_else(|_| page.url());
    let url = url.trim_matches('"').to_string();
    let title = page.title().await.unwrap_or_default();

    let result = ResultBuilder::new("tabs switch")
        .data(json!({
            "switched": true,
            "index": index,
            "title": title,
            "url": url,
        }))
        .build();

    print_result(&result, format);
    session.close().await
}

pub async fn close_tab(
    target: &str,
    ctx: &CommandContext,
    broker: &mut SessionBroker<'_>,
    format: OutputFormat,
) -> Result<()> {
    let request = SessionRequest::from_context(WaitUntil::Load, ctx);
    let session = broker.session(request).await?;
    let context = session.context();
    let pages = context.pages();

    let (index, page) = find_page(&pages, target).await?;
    
    let url = page.evaluate_value("window.location.href").await.unwrap_or_else(|_| page.url());
    let url = url.trim_matches('"').to_string();
    let title = page.title().await.unwrap_or_default();

    // Close the page
    page.close().await?;

    let result = ResultBuilder::new("tabs close")
        .data(json!({
            "closed": true,
            "index": index,
            "title": title,
            "url": url,
        }))
        .build();

    print_result(&result, format);
    session.close().await
}

pub async fn new_tab(
    url: Option<&str>,
    ctx: &CommandContext,
    broker: &mut SessionBroker<'_>,
    format: OutputFormat,
) -> Result<()> {
    let request = SessionRequest::from_context(WaitUntil::Load, ctx);
    let session = broker.session(request).await?;
    let context = session.context();
    
    // Create new page
    let page = context.new_page().await?;

    // Navigate if URL provided
    if let Some(url) = url {
        page.goto(url, None).await?;
    }

    let final_url = page.evaluate_value("window.location.href").await.unwrap_or_else(|_| page.url());
    let final_url = final_url.trim_matches('"').to_string();
    let title = page.title().await.unwrap_or_default();

    // Get the new index
    let new_index = context.pages().len().saturating_sub(1);

    let result = ResultBuilder::new("tabs new")
        .data(json!({
            "created": true,
            "index": new_index,
            "title": title,
            "url": final_url,
        }))
        .build();

    print_result(&result, format);
    session.close().await
}

async fn find_page<'a>(pages: &'a [pw::protocol::Page], target: &str) -> Result<(usize, &'a pw::protocol::Page)> {
    // Try parsing as index first
    if let Ok(index) = target.parse::<usize>() {
        return pages.get(index).map(|p| (index, p)).ok_or_else(|| {
            PwError::Context(format!(
                "Tab index {} out of range (0-{})",
                index,
                pages.len().saturating_sub(1)
            ))
        });
    }

    // Otherwise search by URL or title pattern
    let target_lower = target.to_lowercase();
    
    for (i, page) in pages.iter().enumerate() {
        let url = page.evaluate_value("window.location.href")
            .await
            .unwrap_or_else(|_| page.url())
            .trim_matches('"')
            .to_lowercase();
        let title = page.title().await.unwrap_or_default().to_lowercase();
        
        if url.contains(&target_lower) || title.contains(&target_lower) {
            return Ok((i, page));
        }
    }

    Err(PwError::Context(format!(
        "No tab found matching '{}'",
        target
    )))
}
