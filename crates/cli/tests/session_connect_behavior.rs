use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::json;
use tempfile::TempDir;

fn pw_binary() -> PathBuf {
	let mut path = std::env::current_exe().expect("current_exe should resolve");
	path.pop();
	path.pop();
	path.push("pw");
	path
}

fn run_pw(workdir: &Path, args: &[&str]) -> (bool, String, String) {
	let output = Command::new(pw_binary())
		.current_dir(workdir)
		.args(args)
		.output()
		.expect("failed to execute pw");

	let stdout = String::from_utf8_lossy(&output.stdout).to_string();
	let stderr = String::from_utf8_lossy(&output.stderr).to_string();
	(output.status.success(), stdout, stderr)
}

fn run_exec_json(workdir: &Path, op: &str, input: serde_json::Value) -> (bool, serde_json::Value, String) {
	let (success, stdout, stderr) = run_pw(workdir, &["-f", "json", "exec", op, "--input", &input.to_string()]);
	let parsed = serde_json::from_str::<serde_json::Value>(&stdout).unwrap_or_else(|_| json!({ "raw": stdout }));
	(success, parsed, stderr)
}

fn descriptor_path(workdir: &Path) -> PathBuf {
	workdir
		.join("playwright")
		.join(".pw-cli-v4")
		.join("profiles")
		.join("default")
		.join("sessions")
		.join("session.json")
}

fn config_path(workdir: &Path) -> PathBuf {
	workdir
		.join("playwright")
		.join(".pw-cli-v4")
		.join("profiles")
		.join("default")
		.join("config.json")
}

fn write_descriptor_missing_endpoint(workdir: &Path) {
	let path = descriptor_path(workdir);
	std::fs::create_dir_all(path.parent().expect("session dir should exist")).expect("session dir should be created");
	std::fs::write(
		path,
		r#"{
  "schema_version": 1,
  "pid": 1234,
  "browser": "chromium",
  "headless": true,
  "cdp_endpoint": null,
  "ws_endpoint": null,
  "workspace_id": "workspace-id",
  "namespace": "default",
  "session_key": "workspace-id:default:chromium:headless",
  "driver_hash": "0.0.0-test",
  "created_at": 1
}"#,
	)
	.expect("descriptor should be written");
}

#[test]
fn session_status_reports_inactive_without_descriptor() {
	let tmp = TempDir::new().expect("temp dir should be created");
	let (success, json, stderr) = run_exec_json(tmp.path(), "session.status", json!({}));
	assert!(success, "session.status failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["data"]["active"], false);
	assert_eq!(
		json["data"]["message"],
		"No session descriptor for namespace; run a browser command to create one"
	);
}

#[test]
fn session_clear_removes_descriptor_file() {
	let tmp = TempDir::new().expect("temp dir should be created");
	write_descriptor_missing_endpoint(tmp.path());
	assert!(descriptor_path(tmp.path()).exists(), "descriptor should exist before clear");

	let (success, json, stderr) = run_exec_json(tmp.path(), "session.clear", json!({}));
	assert!(success, "session.clear failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["data"]["cleared"], true);
	assert!(!descriptor_path(tmp.path()).exists(), "descriptor should be removed");
}

#[test]
fn session_stop_cleans_up_descriptor_without_endpoint() {
	let tmp = TempDir::new().expect("temp dir should be created");
	write_descriptor_missing_endpoint(tmp.path());

	let (success, json, stderr) = run_exec_json(tmp.path(), "session.stop", json!({}));
	assert!(success, "session.stop failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["data"]["stopped"], false);
	assert_eq!(json["data"]["message"], "Descriptor missing endpoint; removed descriptor");
	assert!(!descriptor_path(tmp.path()).exists(), "descriptor should be removed after stop fallback");
}

#[test]
fn connect_set_show_clear_round_trip() {
	let tmp = TempDir::new().expect("temp dir should be created");

	let endpoint = "http://127.0.0.1:9222";
	let (success, json, stderr) = run_exec_json(tmp.path(), "connect", json!({ "endpoint": endpoint }));
	assert!(success, "connect set failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["data"]["endpoint"], endpoint);

	let (success, json, stderr) = run_exec_json(tmp.path(), "connect", json!({}));
	assert!(success, "connect show failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["data"]["endpoint"], endpoint);

	let (success, json, stderr) = run_exec_json(tmp.path(), "connect", json!({ "clear": true }));
	assert!(success, "connect clear failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["data"]["action"], "cleared");

	let (success, json, stderr) = run_exec_json(tmp.path(), "connect", json!({}));
	assert!(success, "connect show after clear failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert!(json["data"]["endpoint"].is_null());
}

#[test]
fn connect_set_persists_to_profile_config_defaults() {
	let tmp = TempDir::new().expect("temp dir should be created");
	let endpoint = "http://127.0.0.1:9333";
	let (success, _json, stderr) = run_exec_json(tmp.path(), "connect", json!({ "endpoint": endpoint }));
	assert!(success, "connect set failed: {stderr}");

	let config_path = config_path(tmp.path());
	assert!(config_path.exists(), "config should exist after connect set");
	let config = std::fs::read_to_string(config_path).expect("config should be readable");
	let value: serde_json::Value = serde_json::from_str(&config).expect("config should parse");
	assert_eq!(value["defaults"]["cdpEndpoint"], endpoint);
}
