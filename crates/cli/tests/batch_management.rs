use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn pw_binary() -> PathBuf {
	let mut path = std::env::current_exe().unwrap();
	path.pop();
	path.pop();
	path.push("pw");
	path
}

fn workspace_root() -> PathBuf {
	std::env::temp_dir().join("pw-cli-batch-management")
}

fn clear_context_store() {
	let _ = std::fs::remove_dir_all(workspace_root());
}

fn run_pw_batch(lines: &[&str]) -> (bool, String, String) {
	let workspace = workspace_root();
	let workspace_str = workspace.to_string_lossy().to_string();

	let mut child = Command::new(pw_binary())
		.args(["--no-project", "--workspace", &workspace_str, "--namespace", "default", "run"])
		.stdin(Stdio::piped())
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.spawn()
		.expect("Failed to start pw run");

	{
		let stdin = child.stdin.as_mut().expect("stdin unavailable");
		for line in lines {
			writeln!(stdin, "{line}").expect("failed to write batch request");
		}
	}

	let output = child.wait_with_output().expect("failed waiting for pw run");
	let stdout = String::from_utf8_lossy(&output.stdout).to_string();
	let stderr = String::from_utf8_lossy(&output.stderr).to_string();
	(output.status.success(), stdout, stderr)
}

fn parse_ndjson(stdout: &str) -> Vec<serde_json::Value> {
	stdout
		.lines()
		.filter(|line| !line.trim().is_empty())
		.map(|line| serde_json::from_str::<serde_json::Value>(line).expect("line should be valid JSON"))
		.collect()
}

#[test]
fn batch_supports_har_show() {
	clear_context_store();

	let (success, stdout, stderr) = run_pw_batch(&[r#"{"id":"1","command":"har.show","args":{}}"#, r#"{"id":"2","command":"quit","args":{}}"#]);

	assert!(success, "batch run failed: {stderr}");
	let lines = parse_ndjson(&stdout);
	assert!(lines.len() >= 2, "expected at least two response lines, got: {stdout}");

	let first = &lines[0];
	assert_eq!(first["id"], "1");
	assert_eq!(first["ok"], true);
	assert_eq!(first["command"], "har.show");
	assert_eq!(first["data"]["enabled"], false);
}

#[test]
fn batch_rejects_auth_login_as_interactive() {
	clear_context_store();

	let (success, stdout, stderr) = run_pw_batch(&[
		r#"{"id":"1","command":"auth.login","args":{"url":"https://example.com"}}"#,
		r#"{"id":"2","command":"quit","args":{}}"#,
	]);

	assert!(success, "batch run should stay healthy: {stderr}");
	let lines = parse_ndjson(&stdout);
	assert!(lines.len() >= 2, "expected at least two response lines, got: {stdout}");

	let first = &lines[0];
	assert_eq!(first["id"], "1");
	assert_eq!(first["ok"], false);
	assert_eq!(first["command"], "auth.login");
	assert_eq!(first["error"]["code"], "UNSUPPORTED_MODE");
}
