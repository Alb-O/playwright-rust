use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use crate::styles::cli_styles;
use crate::types::BrowserKind;

#[derive(Parser, Debug)]
#[command(name = "pw")]
#[command(about = "Playwright CLI - Browser automation from the command line")]
#[command(version)]
#[command(styles = cli_styles())]
pub struct Cli {
    /// Increase verbosity (-v info, -vv debug)
    #[arg(short, long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Load authentication state from file (cookies, localStorage)
    #[arg(long, global = true, value_name = "FILE")]
    pub auth: Option<PathBuf>,

    /// Browser to use for automation
    #[arg(short, long, global = true, value_enum, default_value = "chromium")]
    pub browser: BrowserKind,

    /// Disable project detection (use current directory paths)
    #[arg(long, global = true)]
    pub no_project: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Navigate to URL and check for console errors
    #[command(alias = "nav")]
    Navigate { url: String },

    /// Capture console messages and errors
    #[command(alias = "con")]
    Console {
        url: String,
        /// Time to wait for console messages (ms)
        #[arg(default_value = "3000")]
        timeout_ms: u64,
    },

    /// Evaluate JavaScript and return result
    Eval { url: String, expression: String },

    /// Get HTML content (full page or specific selector)
    Html {
        url: String,
        /// CSS selector (defaults to html)
        #[arg(default_value = "html")]
        selector: String,
    },

    /// Get coordinates for first matching element
    Coords { url: String, selector: String },

    /// Get coordinates and info for all matching elements
    CoordsAll { url: String, selector: String },

    /// Take screenshot
    #[command(alias = "ss")]
    Screenshot {
        url: String,
        /// Output file path
        #[arg(short, long, default_value = "screenshot.png")]
        output: PathBuf,
        /// Capture the full scrollable page instead of just the viewport
        #[arg(long)]
        full_page: bool,
    },

    /// Click element and show resulting URL
    Click { url: String, selector: String },

    /// Get text content of element
    Text { url: String, selector: String },

    /// List interactive elements (buttons, links, inputs, selects)
    #[command(alias = "els")]
    Elements { url: String },

    /// Wait for condition (selector, timeout, or load state)
    Wait { url: String, condition: String },

    /// Authentication and session management
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },

    /// Initialize a new playwright project structure
    Init {
        /// Project directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,

        /// Template type: standard (full structure) or minimal (tests only)
        #[arg(long, short, default_value = "standard", value_enum)]
        template: InitTemplate,

        /// Skip creating playwright.config.js
        #[arg(long)]
        no_config: bool,

        /// Skip creating example test file
        #[arg(long)]
        no_example: bool,

        /// Use TypeScript for config and tests
        #[arg(long)]
        typescript: bool,

        /// Force overwrite existing files
        #[arg(long, short)]
        force: bool,

        /// Generate Nix browser setup script (for NixOS/Nix users)
        #[arg(long)]
        nix: bool,
    },
}

/// Project template type for init command
#[derive(Clone, Debug, ValueEnum, Default)]
pub enum InitTemplate {
    /// Full structure: tests/, scripts/, results/, reports/, screenshots/
    #[default]
    Standard,
    /// Minimal structure: tests/ only
    Minimal,
}

#[derive(Subcommand, Debug)]
pub enum AuthAction {
    /// Interactive login - opens browser for manual login, then saves session
    Login {
        /// URL to navigate to for login
        url: String,
        /// File to save authentication state to
        #[arg(short, long, default_value = "auth.json")]
        output: PathBuf,
        /// Wait time in seconds for manual login (default: 60)
        #[arg(short, long, default_value = "60")]
        timeout: u64,
    },

    /// Show cookies for a URL (uses saved auth if --auth provided)
    Cookies {
        /// URL to get cookies for
        url: String,
        /// Output format: json or table
        #[arg(short, long, default_value = "table")]
        format: String,
    },

    /// Show current storage state (cookies + localStorage)
    Show {
        /// Auth file to display
        #[arg(default_value = "auth.json")]
        file: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_screenshot_command() {
        let args = vec!["pw", "screenshot", "https://example.com", "-o", "/tmp/test.png"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Screenshot { url, output, full_page } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(output, PathBuf::from("/tmp/test.png"));
                assert!(!full_page);
            }
            _ => panic!("Expected Screenshot command"),
        }
    }

    #[test]
    fn parse_screenshot_default_output() {
        let args = vec!["pw", "screenshot", "https://example.com"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Screenshot { url, output, full_page } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(output, PathBuf::from("screenshot.png"));
                assert!(!full_page);
            }
            _ => panic!("Expected Screenshot command"),
        }
    }

    #[test]
    fn parse_html_command() {
        let args = vec!["pw", "html", "https://example.com", "div.content"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Html { url, selector } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(selector, "div.content");
            }
            _ => panic!("Expected Html command"),
        }
    }

    #[test]
    fn parse_wait_command() {
        let args = vec!["pw", "wait", "https://example.com", "networkidle"];
        let cli = Cli::try_parse_from(args).unwrap();

        match cli.command {
            Commands::Wait { url, condition } => {
                assert_eq!(url, "https://example.com");
                assert_eq!(condition, "networkidle");
            }
            _ => panic!("Expected Wait command"),
        }
    }

    #[test]
    fn verbose_flag_short_and_long() {
        let short_args = vec!["pw", "-v", "screenshot", "https://example.com"];
        let short_cli = Cli::try_parse_from(short_args).unwrap();
        assert_eq!(short_cli.verbose, 1);

        let long_args = vec!["pw", "--verbose", "screenshot", "https://example.com"];
        let long_cli = Cli::try_parse_from(long_args).unwrap();
        assert_eq!(long_cli.verbose, 1);
        
        let double_v = vec!["pw", "-vv", "screenshot", "https://example.com"];
        let double_cli = Cli::try_parse_from(double_v).unwrap();
        assert_eq!(double_cli.verbose, 2);
    }

    #[test]
    fn invalid_command_fails() {
        let args = vec!["pw", "unknown-command", "https://example.com"];
        assert!(Cli::try_parse_from(args).is_err());
    }
}
