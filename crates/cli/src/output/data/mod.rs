use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Result data for navigate command.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavigateData {
	pub url: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub actual_url: Option<String>,
	pub title: String,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub errors: Vec<String>,
	#[serde(default, skip_serializing_if = "Vec::is_empty")]
	pub warnings: Vec<String>,
}

/// Result data for click command.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClickData {
	pub before_url: String,
	pub after_url: String,
	pub navigated: bool,
	pub selector: String,
	#[serde(skip_serializing_if = "Vec::is_empty", default)]
	pub downloads: Vec<DownloadedFile>,
}

/// Information about a downloaded file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadedFile {
	pub url: String,
	pub suggested_filename: String,
	pub path: PathBuf,
}

/// Result data for screenshot command.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenshotData {
	pub path: PathBuf,
	pub full_page: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub width: Option<u32>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub height: Option<u32>,
}

/// Result data for text command.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextData {
	pub text: String,
	pub selector: String,
	pub match_count: usize,
}

/// Result data for fill command.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillData {
	pub selector: String,
	pub text: String,
}

/// Result data for eval command.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EvalData {
	pub result: serde_json::Value,
	pub expression: String,
}

/// Result data for session start command.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStartData {
	pub ws_endpoint: Option<String>,
	pub cdp_endpoint: Option<String>,
	pub browser: String,
	pub headless: bool,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub workspace_id: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub namespace: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub session_key: Option<String>,
}

/// Result data for elements command.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementsData {
	pub elements: Vec<InteractiveElement>,
	pub count: usize,
}

/// An interactive element found on the page.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveElement {
	pub tag: String,
	pub selector: String,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub text: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub href: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub name: Option<String>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub id: Option<String>,
	pub x: i32,
	pub y: i32,
	pub width: i32,
	pub height: i32,
}

/// Result data for snapshot command.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotData {
	pub url: String,
	pub title: String,
	pub viewport_width: i32,
	pub viewport_height: i32,
	pub text: String,
	pub elements: Vec<InteractiveElement>,
	pub element_count: usize,
}
