//! Command invocation adapter for CLI enum variants.
//!
//! Converts clap-parsed CLI enums into registry command IDs plus typed JSON
//! payloads using each command module's `Raw` request type.

use serde::Serialize;

use crate::cli::{AuthAction, Commands, DaemonAction, HarAction, PageAction, ProtectAction, SessionAction, TabsAction};
use crate::commands::auth::{CookiesRaw, ListenRaw, LoginRaw, ShowRaw};
use crate::commands::connect::ConnectRaw;
use crate::commands::daemon::{DaemonStartRaw, DaemonStatusRaw, DaemonStopRaw};
use crate::commands::har::{HarClearRaw, HarSetRaw, HarShowRaw};
use crate::commands::init::InitRaw;
use crate::commands::registry::CommandId;
use crate::commands::session::{SessionClearRaw, SessionStartRaw, SessionStatusRaw, SessionStopRaw};
use crate::commands::tabs::{TabsCloseRaw, TabsListRaw, TabsNewRaw, TabsSwitchRaw};
use crate::error::Result;

/// Registry target with serialized command args.
#[derive(Debug, Clone)]
pub(crate) struct CommandInvocation {
	pub(crate) id: CommandId,
	pub(crate) args: serde_json::Value,
}

fn invocation<T: Serialize>(id: CommandId, raw: T) -> Result<CommandInvocation> {
	Ok(CommandInvocation {
		id,
		args: serde_json::to_value(raw)?,
	})
}

/// Converts a parsed CLI command into a registry invocation.
///
/// Returns `Ok(None)` for non-registry commands (`run`, `relay`, `test`).
pub(crate) fn from_cli_command(command: Commands) -> Result<Option<CommandInvocation>> {
	use CommandId as Id;

	let invocation = match command {
		Commands::Navigate(raw) => invocation(Id::Navigate, raw)?,
		Commands::Screenshot(raw) => invocation(Id::Screenshot, raw)?,
		Commands::Click(raw) => invocation(Id::Click, raw)?,
		Commands::Fill(raw) => invocation(Id::Fill, raw)?,
		Commands::Wait(raw) => invocation(Id::Wait, raw)?,
		Commands::Page(action) => from_page_action(action)?,
		Commands::Auth { action } => from_auth_action(action)?,
		Commands::Session { action } => from_session_action(action)?,
		Commands::Daemon { action } => from_daemon_action(action)?,
		Commands::Connect {
			endpoint,
			clear,
			launch,
			discover,
			kill,
			port,
			user_data_dir,
		} => invocation(
			Id::Connect,
			ConnectRaw {
				endpoint,
				clear,
				launch,
				discover,
				kill,
				port,
				user_data_dir,
			},
		)?,
		Commands::Tabs(action) => from_tabs_action(action)?,
		Commands::Protect(action) => from_protect_action(action)?,
		Commands::Har { action } => from_har_action(action)?,
		Commands::Init {
			path,
			template,
			no_config,
			no_example,
			typescript,
			force,
			nix,
		} => invocation(
			Id::Init,
			InitRaw {
				path,
				template,
				no_config,
				no_example,
				typescript,
				force,
				nix,
			},
		)?,
		Commands::Run | Commands::Relay { .. } | Commands::Test { .. } => return Ok(None),
	};

	Ok(Some(invocation))
}

fn from_page_action(action: PageAction) -> Result<CommandInvocation> {
	use CommandId as Id;

	match action {
		PageAction::Console(raw) => invocation(Id::PageConsole, raw),
		PageAction::Eval(raw) => invocation(Id::PageEval, raw),
		PageAction::Html(raw) => invocation(Id::PageHtml, raw),
		PageAction::Coords(raw) => invocation(Id::PageCoords, raw),
		PageAction::CoordsAll(raw) => invocation(Id::PageCoordsAll, raw),
		PageAction::Text(raw) => invocation(Id::PageText, raw),
		PageAction::Read(raw) => invocation(Id::PageRead, raw),
		PageAction::Elements(raw) => invocation(Id::PageElements, raw),
		PageAction::Snapshot(raw) => invocation(Id::PageSnapshot, raw),
	}
}

fn from_auth_action(action: AuthAction) -> Result<CommandInvocation> {
	use CommandId as Id;

	match action {
		AuthAction::Login { url, output, timeout } => invocation(
			Id::AuthLogin,
			LoginRaw {
				url,
				output: Some(output),
				timeout_secs: Some(timeout),
			},
		),
		AuthAction::Cookies { url, format } => invocation(Id::AuthCookies, CookiesRaw { url, format: Some(format) }),
		AuthAction::Show { file } => invocation(Id::AuthShow, ShowRaw { file }),
		AuthAction::Listen { host, port } => invocation(Id::AuthListen, ListenRaw { host, port }),
	}
}

fn from_session_action(action: SessionAction) -> Result<CommandInvocation> {
	use CommandId as Id;

	match action {
		SessionAction::Status => invocation(Id::SessionStatus, SessionStatusRaw::default()),
		SessionAction::Clear => invocation(Id::SessionClear, SessionClearRaw::default()),
		SessionAction::Start { headful } => invocation(Id::SessionStart, SessionStartRaw { headful }),
		SessionAction::Stop => invocation(Id::SessionStop, SessionStopRaw::default()),
	}
}

fn from_daemon_action(action: DaemonAction) -> Result<CommandInvocation> {
	use CommandId as Id;

	match action {
		DaemonAction::Start { foreground } => invocation(Id::DaemonStart, DaemonStartRaw { foreground }),
		DaemonAction::Stop => invocation(Id::DaemonStop, DaemonStopRaw::default()),
		DaemonAction::Status => invocation(Id::DaemonStatus, DaemonStatusRaw::default()),
	}
}

fn from_tabs_action(action: TabsAction) -> Result<CommandInvocation> {
	use CommandId as Id;

	match action {
		TabsAction::List => invocation(Id::TabsList, TabsListRaw::default()),
		TabsAction::Switch { target } => invocation(Id::TabsSwitch, TabsSwitchRaw { target }),
		TabsAction::Close { target } => invocation(Id::TabsClose, TabsCloseRaw { target }),
		TabsAction::New { url } => invocation(Id::TabsNew, TabsNewRaw { url }),
	}
}

fn from_protect_action(action: ProtectAction) -> Result<CommandInvocation> {
	use CommandId as Id;

	match action {
		ProtectAction::Add { pattern } => invocation(Id::ProtectAdd, crate::commands::protect::ProtectAddRaw { pattern }),
		ProtectAction::Remove { pattern } => invocation(Id::ProtectRemove, crate::commands::protect::ProtectRemoveRaw { pattern }),
		ProtectAction::List => invocation(Id::ProtectList, crate::commands::protect::ProtectListRaw::default()),
	}
}

fn from_har_action(action: HarAction) -> Result<CommandInvocation> {
	use CommandId as Id;

	match action {
		HarAction::Set {
			file,
			content,
			mode,
			omit_content,
			url_filter,
		} => invocation(
			Id::HarSet,
			HarSetRaw {
				file,
				content,
				mode,
				omit_content,
				url_filter,
			},
		),
		HarAction::Show => invocation(Id::HarShow, HarShowRaw::default()),
		HarAction::Clear => invocation(Id::HarClear, HarClearRaw::default()),
	}
}
