// Browser protocol object
//
// Represents a browser instance created by BrowserType.launch()

use crate::channel::Channel;
use crate::channel_owner::{ChannelOwner, ChannelOwnerImpl, ParentOrConnection};
use crate::error::Result;
use serde_json::Value;
use std::any::Any;
use std::sync::Arc;

/// Browser represents a browser instance.
///
/// A Browser is created when you call [`BrowserType::launch()`]. It provides methods
/// to create browser contexts and pages.
///
/// # Example
///
/// ```ignore
/// // TODO: Change to `no_run` in Slice 3 when launch() is implemented
/// use playwright_core::protocol::Playwright;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let playwright = Playwright::launch().await?;
///     let chromium = playwright.chromium();
///
///     // Launch browser
///     let browser = chromium.launch().await?;
///
///     println!("Browser: {} version {}", browser.name(), browser.version());
///
///     // Close browser
///     browser.close().await?;
///
///     Ok(())
/// }
/// ```
///
/// See: <https://playwright.dev/docs/api/class-browser>
#[derive(Clone)]
pub struct Browser {
    base: ChannelOwnerImpl,
    version: String,
    name: String,
}

impl Browser {
    /// Creates a new Browser from protocol initialization
    ///
    /// This is called by the object factory when the server sends a `__create__` message
    /// for a Browser object.
    ///
    /// # Arguments
    ///
    /// * `parent` - The parent BrowserType object
    /// * `type_name` - The protocol type name ("Browser")
    /// * `guid` - The unique identifier for this browser instance
    /// * `initializer` - The initialization data from the server
    ///
    /// # Errors
    ///
    /// Returns error if initializer is missing required fields (version, name)
    pub fn new(
        parent: Arc<dyn ChannelOwner>,
        type_name: String,
        guid: String,
        initializer: Value,
    ) -> Result<Self> {
        let base = ChannelOwnerImpl::new(
            ParentOrConnection::Parent(parent),
            type_name,
            guid,
            initializer.clone(),
        );

        let version = initializer["version"]
            .as_str()
            .ok_or_else(|| {
                crate::error::Error::ProtocolError(
                    "Browser initializer missing 'version' field".to_string(),
                )
            })?
            .to_string();

        let name = initializer["name"]
            .as_str()
            .ok_or_else(|| {
                crate::error::Error::ProtocolError(
                    "Browser initializer missing 'name' field".to_string(),
                )
            })?
            .to_string();

        Ok(Self {
            base,
            version,
            name,
        })
    }

    /// Returns the browser version string
    ///
    /// # Example
    ///
    /// ```ignore
    /// // TODO: Change to `no_run` in Slice 3 when launch() is implemented
    /// # use playwright_core::protocol::Playwright;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// println!("Browser version: {}", browser.version());
    /// # Ok(())
    /// # }
    /// ```
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the browser name (e.g., "chromium", "firefox", "webkit")
    ///
    /// # Example
    ///
    /// ```ignore
    /// // TODO: Change to `no_run` in Slice 3 when launch() is implemented
    /// # use playwright_core::protocol::Playwright;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// assert_eq!(browser.name(), "chromium");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the channel for sending protocol messages
    ///
    /// TODO: Remove #[allow(dead_code)] in Slice 4 when Browser::close() is implemented
    #[allow(dead_code)]
    fn channel(&self) -> &Channel {
        self.base.channel()
    }
}

impl ChannelOwner for Browser {
    fn guid(&self) -> &str {
        self.base.guid()
    }

    fn type_name(&self) -> &str {
        self.base.type_name()
    }

    fn parent(&self) -> Option<Arc<dyn ChannelOwner>> {
        self.base.parent()
    }

    fn connection(&self) -> Arc<dyn crate::connection::ConnectionLike> {
        self.base.connection()
    }

    fn initializer(&self) -> &Value {
        self.base.initializer()
    }

    fn channel(&self) -> &Channel {
        self.base.channel()
    }

    fn dispose(&self, reason: crate::channel_owner::DisposeReason) {
        self.base.dispose(reason)
    }

    fn adopt(&self, child: Arc<dyn ChannelOwner>) {
        self.base.adopt(child)
    }

    fn add_child(&self, guid: String, child: Arc<dyn ChannelOwner>) {
        self.base.add_child(guid, child)
    }

    fn remove_child(&self, guid: &str) {
        self.base.remove_child(guid)
    }

    fn on_event(&self, _method: &str, _params: Value) {
        // TODO: Handle browser events in future phases
    }

    fn was_collected(&self) -> bool {
        self.base.was_collected()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl std::fmt::Debug for Browser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Browser")
            .field("guid", &self.guid())
            .field("name", &self.name)
            .field("version", &self.version)
            .finish()
    }
}

// Note: Browser testing is done via integration tests since it requires:
// - A real Connection with object registry
// - Protocol messages from the server
// - BrowserType.launch() to create Browser objects
// See: crates/playwright-core/tests/browser_launch_integration.rs (Phase 2 Slice 3)
