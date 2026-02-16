use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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

#[test]
fn exec_profile_show_returns_default_config_when_missing() {
	let tmp = TempDir::new().expect("temp dir should be created");

	let (success, json, stderr) = run_exec_json(tmp.path(), "profile.show", json!({ "name": "default" }));
	assert!(success, "profile.show failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["op"], "profile.show");
	assert_eq!(json["data"]["schema"], 4);
}

#[test]
fn exec_profile_set_normalizes_schema_and_show_roundtrips() {
	let tmp = TempDir::new().expect("temp dir should be created");
	let file_path = tmp.path().join("profile.json");
	std::fs::write(
		&file_path,
		r#"{"schema":0,"defaults":{"baseUrl":"https://base.example"},"protectedUrls":["admin"]}"#,
	)
	.expect("profile file should be written");

	let (success, json, stderr) = run_exec_json(
		tmp.path(),
		"profile.set",
		json!({
			"name": "default",
			"file": file_path,
		}),
	);
	assert!(success, "profile.set failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["data"]["written"], true);

	let (success, json, stderr) = run_exec_json(tmp.path(), "profile.show", json!({ "name": "default" }));
	assert!(success, "profile.show failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["data"]["schema"], 4);
	assert_eq!(json["data"]["defaults"]["baseUrl"], "https://base.example");
	assert_eq!(json["data"]["protectedUrls"], json!(["admin"]));
}

#[test]
fn batch_supports_profile_show_operation() {
	let tmp = TempDir::new().expect("temp dir should be created");
	let mut child = Command::new(pw_binary())
		.current_dir(tmp.path())
		.args(["-f", "ndjson", "batch"])
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect("failed to start pw batch");

	{
		let stdin = child.stdin.as_mut().expect("stdin unavailable");
		writeln!(
			stdin,
			r#"{{"schemaVersion":5,"requestId":"1","op":"profile.show","input":{{"name":"default"}}}}"#
		)
		.expect("failed to write batch request");
		writeln!(stdin, r#"{{"schemaVersion":5,"requestId":"2","op":"quit","input":{{}}}}"#).expect("failed to write batch quit");
	}

	let output = child.wait_with_output().expect("failed waiting for pw batch");
	assert!(output.status.success(), "batch process failed: {}", String::from_utf8_lossy(&output.stderr));

	let stdout = String::from_utf8_lossy(&output.stdout);
	let first = stdout.lines().find(|line| !line.trim().is_empty()).expect("missing batch response");
	let json: serde_json::Value = serde_json::from_str(first).expect("line should be valid JSON");
	assert_eq!(json["requestId"], "1");
	assert_eq!(json["ok"], true);
	assert_eq!(json["op"], "profile.show");
}

#[test]
fn profile_wrapper_returns_same_data_as_exec() {
	let tmp = TempDir::new().expect("temp dir should be created");
	let (success, exec_json, stderr) = run_exec_json(tmp.path(), "profile.show", json!({ "name": "default" }));
	assert!(success, "exec profile.show failed: {stderr}");

	let (success, stdout, stderr) = run_pw(tmp.path(), &["-f", "json", "profile", "show", "default"]);
	assert!(success, "pw profile show failed: {stderr}");
	let wrapper_json: serde_json::Value = serde_json::from_str(&stdout).expect("wrapper output should be JSON");

	assert_eq!(wrapper_json["ok"], true);
	assert_eq!(wrapper_json["op"], "profile.show");
	assert_eq!(wrapper_json["data"], exec_json["data"]);
}

#[test]
fn profile_list_does_not_create_profile_state_files() {
	let tmp = TempDir::new().expect("temp dir should be created");
	let config_path = tmp
		.path()
		.join("playwright")
		.join(".pw-cli-v4")
		.join("profiles")
		.join("default")
		.join("config.json");
	let cache_path = tmp
		.path()
		.join("playwright")
		.join(".pw-cli-v4")
		.join("profiles")
		.join("default")
		.join("cache.json");

	let (success, json, stderr) = run_exec_json(tmp.path(), "profile.list", json!({}));
	assert!(success, "profile.list failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert!(json["data"]["profiles"].as_array().is_some());
	assert!(!config_path.exists(), "profile.list should not materialize config");
	assert!(!cache_path.exists(), "profile.list should not materialize cache");
}

#[test]
fn profile_delete_removes_profile_directory() {
	let tmp = TempDir::new().expect("temp dir should be created");
	let file_path = tmp.path().join("profile.json");
	std::fs::write(&file_path, r#"{"schema":4,"defaults":{"baseUrl":"https://delete-me.example"}}"#).expect("profile file should be written");

	let (success, _json, stderr) = run_exec_json(
		tmp.path(),
		"profile.set",
		json!({
			"name": "throwaway",
			"file": file_path,
		}),
	);
	assert!(success, "profile.set failed: {stderr}");

	let profile_dir = tmp.path().join("playwright").join(".pw-cli-v4").join("profiles").join("throwaway");
	assert!(profile_dir.exists(), "profile directory should exist before delete");

	let (success, json, stderr) = run_exec_json(tmp.path(), "profile.delete", json!({ "name": "throwaway" }));
	assert!(success, "profile.delete failed: {stderr}");
	assert_eq!(json["ok"], true);
	assert_eq!(json["data"]["removed"], true);
	assert!(!profile_dir.exists(), "profile directory should be removed after delete");
}
