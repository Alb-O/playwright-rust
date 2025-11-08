// Assertions - Auto-retry assertions for testing
//
// Provides expect() API with auto-retry logic matching Playwright's assertions.
//
// See: https://playwright.dev/docs/test-assertions

use crate::error::Result;
use crate::protocol::Locator;
use std::time::Duration;

/// Default timeout for assertions (5 seconds, matching Playwright)
const DEFAULT_ASSERTION_TIMEOUT: Duration = Duration::from_secs(5);

/// Default polling interval for assertions (100ms)
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Creates an expectation for a locator with auto-retry behavior.
///
/// Assertions will retry until they pass or timeout (default: 5 seconds).
///
/// # Example
///
/// ```no_run
/// use playwright_core::{expect, protocol::Playwright};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let playwright = Playwright::launch().await?;
/// let browser = playwright.chromium().launch().await?;
/// let page = browser.new_page().await?;
///
/// page.goto("https://example.com", None).await?;
///
/// // Assert element is visible (with auto-retry)
/// expect(page.locator("h1").await).to_be_visible().await?;
///
/// // Assert element is hidden
/// expect(page.locator("dialog").await).to_be_hidden().await?;
/// # Ok(())
/// # }
/// ```
///
/// See: <https://playwright.dev/docs/test-assertions>
pub fn expect(locator: Locator) -> Expectation {
    Expectation::new(locator)
}

/// Expectation wraps a locator and provides assertion methods with auto-retry.
pub struct Expectation {
    locator: Locator,
    timeout: Duration,
    poll_interval: Duration,
    negate: bool,
}

impl Expectation {
    /// Creates a new expectation for the given locator.
    pub(crate) fn new(locator: Locator) -> Self {
        Self {
            locator,
            timeout: DEFAULT_ASSERTION_TIMEOUT,
            poll_interval: DEFAULT_POLL_INTERVAL,
            negate: false,
        }
    }

    /// Sets a custom timeout for this assertion.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// expect(page.locator("slow-element").await)
    ///     .with_timeout(Duration::from_secs(10))
    ///     .to_be_visible()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets a custom poll interval for this assertion.
    ///
    /// Default is 100ms.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Negates the assertion.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// // Assert element is NOT visible
    /// expect(page.locator("dialog").await)
    ///     .not()
    ///     .to_be_visible()
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// Note: We intentionally use `.not()` method instead of implementing `std::ops::Not`
    /// to match Playwright's API across all language bindings (JS/Python/Java/.NET).
    #[allow(clippy::should_implement_trait)]
    pub fn not(mut self) -> Self {
        self.negate = true;
        self
    }

    /// Asserts that the element is visible.
    ///
    /// This assertion will retry until the element becomes visible or timeout.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// expect(page.locator("button").await).to_be_visible().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See: <https://playwright.dev/docs/test-assertions#locator-assertions-to-be-visible>
    pub async fn to_be_visible(self) -> Result<()> {
        let start = std::time::Instant::now();
        let selector = self.locator.selector().to_string();

        loop {
            let is_visible = self.locator.is_visible().await?;

            // Check if condition matches (with negation support)
            let matches = if self.negate { !is_visible } else { is_visible };

            if matches {
                return Ok(());
            }

            // Check timeout
            if start.elapsed() >= self.timeout {
                let message = if self.negate {
                    format!(
                        "Expected element '{}' NOT to be visible, but it was visible after {:?}",
                        selector, self.timeout
                    )
                } else {
                    format!(
                        "Expected element '{}' to be visible, but it was not visible after {:?}",
                        selector, self.timeout
                    )
                };
                return Err(crate::error::Error::AssertionTimeout(message));
            }

            // Wait before next poll
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Asserts that the element is hidden (not visible).
    ///
    /// This assertion will retry until the element becomes hidden or timeout.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// expect(page.locator("dialog").await).to_be_hidden().await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See: <https://playwright.dev/docs/test-assertions#locator-assertions-to-be-hidden>
    pub async fn to_be_hidden(self) -> Result<()> {
        // to_be_hidden is the opposite of to_be_visible
        // Use negation to reuse the visibility logic
        let negated = Expectation {
            negate: !self.negate, // Flip negation
            ..self
        };
        negated.to_be_visible().await
    }

    /// Asserts that the element has the specified text content (exact match).
    ///
    /// This assertion will retry until the element has the exact text or timeout.
    /// Text is trimmed before comparison.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// expect(page.locator("h1").await).to_have_text("Welcome").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See: <https://playwright.dev/docs/test-assertions#locator-assertions-to-have-text>
    pub async fn to_have_text(self, expected: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let selector = self.locator.selector().to_string();
        let expected = expected.trim();

        loop {
            // Get text content (using inner_text for consistency with Playwright)
            let actual_text = self.locator.inner_text().await?;
            let actual = actual_text.trim();

            // Check if condition matches (with negation support)
            let matches = if self.negate {
                actual != expected
            } else {
                actual == expected
            };

            if matches {
                return Ok(());
            }

            // Check timeout
            if start.elapsed() >= self.timeout {
                let message = if self.negate {
                    format!(
                        "Expected element '{}' NOT to have text '{}', but it did after {:?}",
                        selector, expected, self.timeout
                    )
                } else {
                    format!(
                        "Expected element '{}' to have text '{}', but had '{}' after {:?}",
                        selector, expected, actual, self.timeout
                    )
                };
                return Err(crate::error::Error::AssertionTimeout(message));
            }

            // Wait before next poll
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Asserts that the element's text matches the specified regex pattern.
    ///
    /// This assertion will retry until the element's text matches the pattern or timeout.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// expect(page.locator("h1").await).to_have_text_regex(r"Welcome.*").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn to_have_text_regex(self, pattern: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let selector = self.locator.selector().to_string();
        let re = regex::Regex::new(pattern)
            .map_err(|e| crate::error::Error::InvalidArgument(format!("Invalid regex: {}", e)))?;

        loop {
            let actual_text = self.locator.inner_text().await?;
            let actual = actual_text.trim();

            // Check if condition matches (with negation support)
            let matches = if self.negate {
                !re.is_match(actual)
            } else {
                re.is_match(actual)
            };

            if matches {
                return Ok(());
            }

            // Check timeout
            if start.elapsed() >= self.timeout {
                let message = if self.negate {
                    format!(
                        "Expected element '{}' NOT to match pattern '{}', but it did after {:?}",
                        selector, pattern, self.timeout
                    )
                } else {
                    format!(
                        "Expected element '{}' to match pattern '{}', but had '{}' after {:?}",
                        selector, pattern, actual, self.timeout
                    )
                };
                return Err(crate::error::Error::AssertionTimeout(message));
            }

            // Wait before next poll
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Asserts that the element contains the specified text (substring match).
    ///
    /// This assertion will retry until the element contains the text or timeout.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// expect(page.locator("p").await).to_contain_text("substring").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See: <https://playwright.dev/docs/test-assertions#locator-assertions-to-contain-text>
    pub async fn to_contain_text(self, expected: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let selector = self.locator.selector().to_string();

        loop {
            let actual_text = self.locator.inner_text().await?;
            let actual = actual_text.trim();

            // Check if condition matches (with negation support)
            let matches = if self.negate {
                !actual.contains(expected)
            } else {
                actual.contains(expected)
            };

            if matches {
                return Ok(());
            }

            // Check timeout
            if start.elapsed() >= self.timeout {
                let message = if self.negate {
                    format!(
                        "Expected element '{}' NOT to contain text '{}', but it did after {:?}",
                        selector, expected, self.timeout
                    )
                } else {
                    format!(
                        "Expected element '{}' to contain text '{}', but had '{}' after {:?}",
                        selector, expected, actual, self.timeout
                    )
                };
                return Err(crate::error::Error::AssertionTimeout(message));
            }

            // Wait before next poll
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Asserts that the element's text contains a substring matching the regex pattern.
    ///
    /// This assertion will retry until the element contains the pattern or timeout.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// expect(page.locator("p").await).to_contain_text_regex(r"sub.*ing").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn to_contain_text_regex(self, pattern: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let selector = self.locator.selector().to_string();
        let re = regex::Regex::new(pattern)
            .map_err(|e| crate::error::Error::InvalidArgument(format!("Invalid regex: {}", e)))?;

        loop {
            let actual_text = self.locator.inner_text().await?;
            let actual = actual_text.trim();

            // Check if condition matches (with negation support)
            let matches = if self.negate {
                !re.is_match(actual)
            } else {
                re.is_match(actual)
            };

            if matches {
                return Ok(());
            }

            // Check timeout
            if start.elapsed() >= self.timeout {
                let message = if self.negate {
                    format!(
                        "Expected element '{}' NOT to contain pattern '{}', but it did after {:?}",
                        selector, pattern, self.timeout
                    )
                } else {
                    format!(
                        "Expected element '{}' to contain pattern '{}', but had '{}' after {:?}",
                        selector, pattern, actual, self.timeout
                    )
                };
                return Err(crate::error::Error::AssertionTimeout(message));
            }

            // Wait before next poll
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Asserts that the input element has the specified value.
    ///
    /// This assertion will retry until the input has the exact value or timeout.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// expect(page.locator("input").await).to_have_value("expected value").await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// See: <https://playwright.dev/docs/test-assertions#locator-assertions-to-have-value>
    pub async fn to_have_value(self, expected: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let selector = self.locator.selector().to_string();

        loop {
            let actual = self.locator.input_value(None).await?;

            // Check if condition matches (with negation support)
            let matches = if self.negate {
                actual != expected
            } else {
                actual == expected
            };

            if matches {
                return Ok(());
            }

            // Check timeout
            if start.elapsed() >= self.timeout {
                let message = if self.negate {
                    format!(
                        "Expected input '{}' NOT to have value '{}', but it did after {:?}",
                        selector, expected, self.timeout
                    )
                } else {
                    format!(
                        "Expected input '{}' to have value '{}', but had '{}' after {:?}",
                        selector, expected, actual, self.timeout
                    )
                };
                return Err(crate::error::Error::AssertionTimeout(message));
            }

            // Wait before next poll
            tokio::time::sleep(self.poll_interval).await;
        }
    }

    /// Asserts that the input element's value matches the specified regex pattern.
    ///
    /// This assertion will retry until the input value matches the pattern or timeout.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use playwright_core::{expect, protocol::Playwright};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let playwright = Playwright::launch().await?;
    /// # let browser = playwright.chromium().launch().await?;
    /// # let page = browser.new_page().await?;
    /// expect(page.locator("input").await).to_have_value_regex(r"value.*").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn to_have_value_regex(self, pattern: &str) -> Result<()> {
        let start = std::time::Instant::now();
        let selector = self.locator.selector().to_string();
        let re = regex::Regex::new(pattern)
            .map_err(|e| crate::error::Error::InvalidArgument(format!("Invalid regex: {}", e)))?;

        loop {
            let actual = self.locator.input_value(None).await?;

            // Check if condition matches (with negation support)
            let matches = if self.negate {
                !re.is_match(&actual)
            } else {
                re.is_match(&actual)
            };

            if matches {
                return Ok(());
            }

            // Check timeout
            if start.elapsed() >= self.timeout {
                let message = if self.negate {
                    format!(
                        "Expected input '{}' NOT to match pattern '{}', but it did after {:?}",
                        selector, pattern, self.timeout
                    )
                } else {
                    format!(
                        "Expected input '{}' to match pattern '{}', but had '{}' after {:?}",
                        selector, pattern, actual, self.timeout
                    )
                };
                return Err(crate::error::Error::AssertionTimeout(message));
            }

            // Wait before next poll
            tokio::time::sleep(self.poll_interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expectation_defaults() {
        // Verify default timeout and poll interval constants
        assert_eq!(DEFAULT_ASSERTION_TIMEOUT, Duration::from_secs(5));
        assert_eq!(DEFAULT_POLL_INTERVAL, Duration::from_millis(100));
    }
}
