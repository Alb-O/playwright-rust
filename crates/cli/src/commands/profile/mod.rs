use std::path::PathBuf;

use clap::Args;
use pw_rs::dirs;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::commands::def::{BoxFut, CommandDef, CommandOutcome, ContextDelta, ExecCtx};
use crate::context_store::storage::StatePaths;
use crate::context_store::types::{CliConfig, SCHEMA_VERSION};
use crate::error::Result;
use crate::output::CommandInputs;
use crate::target::ResolveEnv;
use crate::workspace::{STATE_VERSION_DIR, normalize_profile};

#[derive(Debug, Clone, Default, Args, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileListRaw {}

#[derive(Debug, Clone)]
pub struct ProfileListResolved;

pub struct ProfileListCommand;

impl CommandDef for ProfileListCommand {
	const NAME: &'static str = "profile.list";

	type Raw = ProfileListRaw;
	type Resolved = ProfileListResolved;
	type Data = serde_json::Value;

	fn resolve(_raw: Self::Raw, _env: &ResolveEnv<'_>) -> Result<Self::Resolved> {
		Ok(ProfileListResolved)
	}

	fn execute<'exec, 'ctx>(_args: &'exec Self::Resolved, exec: ExecCtx<'exec, 'ctx>) -> BoxFut<'exec, Result<CommandOutcome<Self::Data>>>
	where
		'ctx: 'exec,
	{
		Box::pin(async move {
			let root = exec.ctx_state.workspace_root().join(dirs::PLAYWRIGHT).join(STATE_VERSION_DIR).join("profiles");
			let mut profiles = Vec::new();

			if root.exists() {
				for entry in std::fs::read_dir(root)? {
					let entry = entry?;
					if entry.file_type()?.is_dir() {
						profiles.push(entry.file_name().to_string_lossy().to_string());
					}
				}
			}

			profiles.sort();
			let data = json!({ "profiles": profiles });

			Ok(CommandOutcome {
				inputs: CommandInputs::default(),
				data,
				delta: ContextDelta::default(),
			})
		})
	}
}

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileShowRaw {
	#[arg(value_name = "NAME")]
	pub name: String,
}

#[derive(Debug, Clone)]
pub struct ProfileShowResolved {
	pub name: String,
}

pub struct ProfileShowCommand;

impl CommandDef for ProfileShowCommand {
	const NAME: &'static str = "profile.show";

	type Raw = ProfileShowRaw;
	type Resolved = ProfileShowResolved;
	type Data = serde_json::Value;

	fn resolve(raw: Self::Raw, _env: &ResolveEnv<'_>) -> Result<Self::Resolved> {
		Ok(ProfileShowResolved {
			name: normalize_profile(&raw.name),
		})
	}

	fn execute<'exec, 'ctx>(args: &'exec Self::Resolved, exec: ExecCtx<'exec, 'ctx>) -> BoxFut<'exec, Result<CommandOutcome<Self::Data>>>
	where
		'ctx: 'exec,
	{
		Box::pin(async move {
			let paths = StatePaths::new(exec.ctx_state.workspace_root(), &args.name);
			let config = if paths.config.exists() {
				let content = std::fs::read_to_string(paths.config)?;
				serde_json::from_str::<CliConfig>(&content)?
			} else {
				CliConfig::new()
			};
			let data = serde_json::to_value(config)?;

			Ok(CommandOutcome {
				inputs: CommandInputs {
					extra: Some(json!({ "name": args.name })),
					..Default::default()
				},
				data,
				delta: ContextDelta::default(),
			})
		})
	}
}

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSetRaw {
	#[arg(value_name = "NAME")]
	pub name: String,
	#[arg(value_name = "FILE")]
	pub file: PathBuf,
}

#[derive(Debug, Clone)]
pub struct ProfileSetResolved {
	pub name: String,
	pub file: PathBuf,
}

pub struct ProfileSetCommand;

impl CommandDef for ProfileSetCommand {
	const NAME: &'static str = "profile.set";

	type Raw = ProfileSetRaw;
	type Resolved = ProfileSetResolved;
	type Data = serde_json::Value;

	fn resolve(raw: Self::Raw, _env: &ResolveEnv<'_>) -> Result<Self::Resolved> {
		Ok(ProfileSetResolved {
			name: normalize_profile(&raw.name),
			file: raw.file,
		})
	}

	fn execute<'exec, 'ctx>(args: &'exec Self::Resolved, exec: ExecCtx<'exec, 'ctx>) -> BoxFut<'exec, Result<CommandOutcome<Self::Data>>>
	where
		'ctx: 'exec,
	{
		Box::pin(async move {
			let paths = StatePaths::new(exec.ctx_state.workspace_root(), &args.name);
			let content = std::fs::read_to_string(&args.file)?;
			let mut config = serde_json::from_str::<CliConfig>(&content)?;

			if config.schema == 0 {
				config.schema = SCHEMA_VERSION;
			}

			if let Some(parent) = paths.config.parent() {
				std::fs::create_dir_all(parent)?;
			}
			std::fs::write(paths.config, serde_json::to_string_pretty(&config)?)?;

			Ok(CommandOutcome {
				inputs: CommandInputs {
					extra: Some(json!({
						"name": args.name,
						"file": args.file,
					})),
					..Default::default()
				},
				data: json!({
					"profile": args.name,
					"written": true,
				}),
				delta: ContextDelta::default(),
			})
		})
	}
}

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileDeleteRaw {
	#[arg(value_name = "NAME")]
	pub name: String,
}

#[derive(Debug, Clone)]
pub struct ProfileDeleteResolved {
	pub name: String,
}

pub struct ProfileDeleteCommand;

impl CommandDef for ProfileDeleteCommand {
	const NAME: &'static str = "profile.delete";

	type Raw = ProfileDeleteRaw;
	type Resolved = ProfileDeleteResolved;
	type Data = serde_json::Value;

	fn resolve(raw: Self::Raw, _env: &ResolveEnv<'_>) -> Result<Self::Resolved> {
		Ok(ProfileDeleteResolved {
			name: normalize_profile(&raw.name),
		})
	}

	fn execute<'exec, 'ctx>(args: &'exec Self::Resolved, exec: ExecCtx<'exec, 'ctx>) -> BoxFut<'exec, Result<CommandOutcome<Self::Data>>>
	where
		'ctx: 'exec,
	{
		Box::pin(async move {
			let paths = StatePaths::new(exec.ctx_state.workspace_root(), &args.name);
			let removed = if paths.profile_dir.exists() {
				std::fs::remove_dir_all(paths.profile_dir)?;
				true
			} else {
				false
			};

			Ok(CommandOutcome {
				inputs: CommandInputs {
					extra: Some(json!({ "name": args.name })),
					..Default::default()
				},
				data: json!({
					"profile": args.name,
					"removed": removed,
				}),
				delta: ContextDelta::default(),
			})
		})
	}
}
