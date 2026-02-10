use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;

static HAR_LOCK: Mutex<()> = Mutex::new(());

fn lock_har() -> std::sync::MutexGuard<'static, ()> {
	HAR_LOCK.lock().unwrap_or_else(|e| e.into_inner())
}

fn pw_binary() -> PathBuf {
	let mut path = std::env::current_exe().unwrap();
	path.pop();
	path.pop();
	path.push("pw");
	path
}

fn workspace_root() -> PathBuf {
	std::env::temp_dir().join("pw-cli-har-config")
}

fn clear_context_store() {
	let _ = std::fs::remove_dir_all(workspace_root());
}

fn run_pw_json(args: &[&str]) -> (bool, String, String) {
	let workspace = workspace_root();
	let workspace_str = workspace.to_string_lossy().to_string();
	let mut all_args = vec![
		"--no-project",
		"--workspace",
		&workspace_str,
		"--namespace",
		"default",
		"-f",
		"json",
	];
	all_args.extend_from_slice(args);

	let output = Command::new(pw_binary())
		.args(&all_args)
		.output()
		.expect("Failed to execute pw");
	let stdout = String::from_utf8_lossy(&output.stdout).to_string();
	let stderr = String::from_utf8_lossy(&output.stderr).to_string();
	(output.status.success(), stdout, stderr)
}

fn run_pw_raw(args: &[&str]) -> (bool, String, String) {
	let output = Command::new(pw_binary())
		.args(args)
		.output()
		.expect("Failed to execute pw");
	let stdout = String::from_utf8_lossy(&output.stdout).to_string();
	let stderr = String::from_utf8_lossy(&output.stderr).to_string();
	(output.status.success(), stdout, stderr)
}

fn parse_json(stdout: &str) -> serde_json::Value {
	serde_json::from_str(stdout).expect("Expected valid JSON output")
}

#[test]
fn har_show_is_disabled_by_default() {
	let _lock = lock_har();
	clear_context_store();

	let (success, stdout, stderr) = run_pw_json(&["har", "show"]);
	assert!(success, "Command failed: {}", stderr);

	let value = parse_json(&stdout);
	assert_eq!(value["ok"], true);
	assert_eq!(value["data"]["enabled"], false);
	assert!(value["data"]["har"].is_null());
}

#[test]
fn har_set_persists_and_show_reflects_config() {
	let _lock = lock_har();
	clear_context_store();

	let (success, _stdout, stderr) = run_pw_json(&[
		"har",
		"set",
		"network.har",
		"--content",
		"embed",
		"--mode",
		"minimal",
		"--omit-content",
		"--url-filter",
		"*.api.example.com",
	]);
	assert!(success, "har set failed: {}", stderr);

	let (success, stdout, stderr) = run_pw_json(&["har", "show"]);
	assert!(success, "har show failed: {}", stderr);

	let value = parse_json(&stdout);
	assert_eq!(value["ok"], true);
	assert_eq!(value["data"]["enabled"], true);
	assert_eq!(value["data"]["har"]["path"], "network.har");
	assert_eq!(value["data"]["har"]["contentPolicy"], "embed");
	assert_eq!(value["data"]["har"]["mode"], "minimal");
	assert_eq!(value["data"]["har"]["omitContent"], true);
	assert_eq!(value["data"]["har"]["urlFilter"], "*.api.example.com");
}

#[test]
fn har_set_persists_namespace_config_file() {
	let _lock = lock_har();
	clear_context_store();

	let (success, _stdout, stderr) = run_pw_json(&[
		"har",
		"set",
		"captures/network.har",
		"--content",
		"embed",
		"--mode",
		"minimal",
		"--omit-content",
		"--url-filter",
		"*.api.example.com",
	]);
	assert!(success, "har set failed: {}", stderr);

	let config_path = workspace_root()
		.join("playwright")
		.join(".pw-cli-v3")
		.join("namespaces")
		.join("default")
		.join("config.json");
	let config = std::fs::read_to_string(&config_path)
		.unwrap_or_else(|e| panic!("Failed to read {}: {}", config_path.display(), e));
	let value: serde_json::Value =
		serde_json::from_str(&config).expect("Expected valid config JSON");
	assert_eq!(value["har"]["path"], "captures/network.har");
	assert_eq!(value["har"]["contentPolicy"], "embed");
	assert_eq!(value["har"]["mode"], "minimal");
	assert_eq!(value["har"]["omitContent"], true);
	assert_eq!(value["har"]["urlFilter"], "*.api.example.com");
}

#[test]
fn har_clear_disables_subsequent_commands() {
	let _lock = lock_har();
	clear_context_store();

	let (success, _stdout, stderr) = run_pw_json(&["har", "set", "network.har"]);
	assert!(success, "har set failed: {}", stderr);

	let (success, stdout, stderr) = run_pw_json(&["har", "clear"]);
	assert!(success, "har clear failed: {}", stderr);
	let value = parse_json(&stdout);
	assert_eq!(value["data"]["cleared"], true);

	let (success, stdout, stderr) = run_pw_json(&["har", "show"]);
	assert!(success, "har show failed: {}", stderr);
	let value = parse_json(&stdout);
	assert_eq!(value["data"]["enabled"], false);
}

#[test]
fn root_help_lists_har_subcommand_and_not_legacy_flags() {
	let _lock = lock_har();
	let (success, stdout, _stderr) = run_pw_raw(&["--help"]);
	assert!(success, "Help should succeed");
	assert!(
		stdout.contains("har"),
		"Expected har command in help output"
	);
	assert!(
		!stdout.contains("--har"),
		"Legacy --har flag should not appear"
	);
	assert!(
		!stdout.contains("--har-content"),
		"Legacy --har-content flag should not appear"
	);
}
