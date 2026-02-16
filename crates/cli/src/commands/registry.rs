//! Command registry and generated dispatch glue.

use crate::output::{CommandInputs, OutputFormat, ResultBuilder, print_result};

/// Print success result in the given format.
pub fn emit_success(command: &'static str, inputs: CommandInputs, data: serde_json::Value, format: OutputFormat) {
	let result = ResultBuilder::new(command).inputs(inputs).data(data).build();
	print_result(&result, format);
}

/// The registry macro: generates a `CommandId` enum, `lookup_command`, and `run_command`.
///
/// Usage example:
/// ```ignore
/// command_registry! {
///   Navigate => crate::commands::navigate::NavigateCommand { names: ["navigate", "nav"] },
///   Click => crate::commands::click::ClickCommand { names: ["click"] },
///   Login => crate::commands::auth::LoginCommand { names: ["login", "auth-login"], interactive: true },
/// }
/// ```
#[macro_export]
macro_rules! command_registry {
	(
		$(
			$id:ident => $ty:path {
				names: [ $($name:literal),+ $(,)? ]
				$(, interactive: $interactive:literal )?
				$(, batch: $batch:literal )?
			}
		),+ $(,)?
	) => {
		#[derive(Debug, Clone, Copy, PartialEq, Eq)]
		pub enum CommandId { $($id),+ }

		pub fn lookup_command(name: &str) -> Option<CommandId> {
			match name {
				$(
					$($name)|+ => Some(CommandId::$id),
				)+
				_ => None,
			}
		}

		#[cfg_attr(not(test), allow(dead_code))]
		pub fn command_name(id: CommandId) -> &'static str {
			match id {
				$(
					CommandId::$id => <$ty as $crate::commands::def::CommandDef>::NAME,
				)+
			}
		}

		/// Run a command by `CommandId`, returning a type-erased outcome.
		///
		/// This function is the *only* place that:
		/// * deserializes `Raw`
		/// * calls `resolve(...)`
		/// * awaits `execute(...)`
		pub async fn run_command(
			id: CommandId,
			args: serde_json::Value,
			has_cdp: bool,
			exec: $crate::commands::def::ExecCtx<'_, '_>,
		) -> $crate::error::Result<$crate::commands::def::ErasedOutcome> {
			match id {
				$(
					CommandId::$id => {
						type Cmd = $ty;
						use $crate::commands::def::ExecMode;

						let interactive_only = {
							let explicit = false $(|| $interactive)?;
							explicit || <Cmd as $crate::commands::def::CommandDef>::INTERACTIVE_ONLY
						};
						let batch_enabled = true $(&& $batch)?;

						if exec.mode == ExecMode::Batch {
							if !batch_enabled {
								return Err($crate::error::PwError::UnsupportedMode(format!(
									"command '{}' is not available in batch/ndjson mode",
									<Cmd as $crate::commands::def::CommandDef>::NAME
								)));
							}
							if interactive_only {
								return Err($crate::error::PwError::UnsupportedMode(format!(
									"command '{}' is interactive-only and cannot run in batch/ndjson mode",
									<Cmd as $crate::commands::def::CommandDef>::NAME
								)));
							}
						}

						let raw: <Cmd as $crate::commands::def::CommandDef>::Raw =
							serde_json::from_value(args)
							.map_err(|e| $crate::error::PwError::Context(format!("INVALID_INPUT: {}", e)))?;

						<Cmd as $crate::commands::def::CommandDef>::validate_mode(&raw, exec.mode)?;
						let resolved = {
							let env = $crate::target::ResolveEnv::new(
								&*exec.ctx_state,
								has_cdp,
								<Cmd as $crate::commands::def::CommandDef>::NAME,
							);
							<Cmd as $crate::commands::def::CommandDef>::resolve(raw, &env)?
						};
						let outcome =
							<Cmd as $crate::commands::def::CommandDef>::execute(&resolved, exec).await?;
						outcome.erase(<Cmd as $crate::commands::def::CommandDef>::NAME)
					}
				)+
			}
		}
	};
}

command_registry! {
	Navigate => crate::commands::navigate::NavigateCommand { names: ["navigate", "nav"] },
	Click => crate::commands::click::ClickCommand { names: ["click"] },
	Fill => crate::commands::fill::FillCommand { names: ["fill"] },
	Wait => crate::commands::wait::WaitCommand { names: ["wait"] },
	Screenshot => crate::commands::screenshot::ScreenshotCommand { names: ["screenshot", "ss"] },
	PageText => crate::commands::page::text::TextCommand { names: ["page.text"] },
	PageHtml => crate::commands::page::html::HtmlCommand { names: ["page.html"] },
	PageEval => crate::commands::page::eval::EvalCommand { names: ["page.eval"] },
	PageConsole => crate::commands::page::console::ConsoleCommand { names: ["page.console"] },
	PageRead => crate::commands::page::read::ReadCommand { names: ["page.read"] },
	PageElements => crate::commands::page::elements::ElementsCommand { names: ["page.elements"] },
	PageSnapshot => crate::commands::page::snapshot::SnapshotCommand { names: ["page.snapshot"] },
	PageCoords => crate::commands::page::coords::CoordsCommand { names: ["page.coords"] },
	PageCoordsAll => crate::commands::page::coords::CoordsAllCommand { names: ["page.coords-all", "page.coords_all"] },
	AuthLogin => crate::commands::auth::LoginCommand { names: ["auth.login", "auth-login"] },
	AuthCookies => crate::commands::auth::CookiesCommand { names: ["auth.cookies", "auth-cookies"] },
	AuthShow => crate::commands::auth::ShowCommand { names: ["auth.show", "auth-show"] },
	AuthListen => crate::commands::auth::ListenCommand { names: ["auth.listen", "auth-listen"] },
	SessionStatus => crate::commands::session::SessionStatusCommand { names: ["session.status", "session-status"] },
	SessionClear => crate::commands::session::SessionClearCommand { names: ["session.clear", "session-clear"] },
	SessionStart => crate::commands::session::SessionStartCommand { names: ["session.start", "session-start"] },
	SessionStop => crate::commands::session::SessionStopCommand { names: ["session.stop", "session-stop"] },
	DaemonStart => crate::commands::daemon::DaemonStartCommand { names: ["daemon.start", "daemon-start"] },
	DaemonStop => crate::commands::daemon::DaemonStopCommand { names: ["daemon.stop", "daemon-stop"] },
	DaemonStatus => crate::commands::daemon::DaemonStatusCommand { names: ["daemon.status", "daemon-status"] },
	Connect => crate::commands::connect::ConnectCommand { names: ["connect"] },
	TabsList => crate::commands::tabs::TabsListCommand { names: ["tabs.list", "tabs-list"] },
	TabsSwitch => crate::commands::tabs::TabsSwitchCommand { names: ["tabs.switch", "tabs-switch"] },
	TabsClose => crate::commands::tabs::TabsCloseCommand { names: ["tabs.close", "tabs-close"] },
	TabsNew => crate::commands::tabs::TabsNewCommand { names: ["tabs.new", "tabs-new"] },
	ProtectAdd => crate::commands::protect::ProtectAddCommand { names: ["protect.add", "protect-add"] },
	ProtectRemove => crate::commands::protect::ProtectRemoveCommand { names: ["protect.remove", "protect-remove"] },
	ProtectList => crate::commands::protect::ProtectListCommand { names: ["protect.list", "protect-list"] },
	HarSet => crate::commands::har::HarSetCommand { names: ["har.set", "har-set"] },
	HarShow => crate::commands::har::HarShowCommand { names: ["har.show", "har-show"] },
	HarClear => crate::commands::har::HarClearCommand { names: ["har.clear", "har-clear"] },
	Init => crate::commands::init::InitCommand { names: ["init"] },
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn lookup_command_by_primary_name() {
		assert_eq!(lookup_command("navigate"), Some(CommandId::Navigate));
		assert_eq!(lookup_command("click"), Some(CommandId::Click));
		assert_eq!(lookup_command("page.text"), Some(CommandId::PageText));
		assert_eq!(lookup_command("connect"), Some(CommandId::Connect));
		assert_eq!(lookup_command("session.status"), Some(CommandId::SessionStatus));
		assert_eq!(lookup_command("har.show"), Some(CommandId::HarShow));
	}

	#[test]
	fn lookup_command_by_alias() {
		assert_eq!(lookup_command("nav"), Some(CommandId::Navigate));
		assert_eq!(lookup_command("ss"), Some(CommandId::Screenshot));
	}

	#[test]
	fn lookup_command_unknown_returns_none() {
		assert_eq!(lookup_command("unknown"), None);
		assert_eq!(lookup_command(""), None);
		assert_eq!(lookup_command("navigat"), None);
	}

	#[test]
	fn command_name_returns_primary() {
		assert_eq!(command_name(CommandId::Navigate), "navigate");
		assert_eq!(command_name(CommandId::Screenshot), "screenshot");
		assert_eq!(command_name(CommandId::PageText), "page.text");
		assert_eq!(command_name(CommandId::Connect), "connect");
		assert_eq!(command_name(CommandId::SessionStatus), "session.status");
		assert_eq!(command_name(CommandId::HarShow), "har.show");
	}
}
