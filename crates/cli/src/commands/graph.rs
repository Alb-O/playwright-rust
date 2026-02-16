//! Generated command graph and dispatch helpers.
//!
//! This module defines command metadata once and generates:
//! * command lookup by name/alias
//! * registry-backed execution dispatch
//! * CLI enum to registry invocation mapping

use pw_cli_command_macros::command_graph;

command_graph! {
	commands: [
		Navigate => crate::commands::navigate::NavigateCommand {
			names: ["navigate", "nav"],
			cli: crate::cli::Commands::Navigate(raw) => raw,
		},
		Click => crate::commands::click::ClickCommand {
			names: ["click"],
			cli: crate::cli::Commands::Click(raw) => raw,
		},
		Fill => crate::commands::fill::FillCommand {
			names: ["fill"],
			cli: crate::cli::Commands::Fill(raw) => raw,
		},
		Wait => crate::commands::wait::WaitCommand {
			names: ["wait"],
			cli: crate::cli::Commands::Wait(raw) => raw,
		},
		Screenshot => crate::commands::screenshot::ScreenshotCommand {
			names: ["screenshot", "ss"],
			cli: crate::cli::Commands::Screenshot(raw) => raw,
		},
		PageText => crate::commands::page::text::TextCommand {
			names: ["page.text"],
			cli: crate::cli::Commands::Page(crate::cli::PageAction::Text(raw)) => raw,
		},
		PageHtml => crate::commands::page::html::HtmlCommand {
			names: ["page.html"],
			cli: crate::cli::Commands::Page(crate::cli::PageAction::Html(raw)) => raw,
		},
		PageEval => crate::commands::page::eval::EvalCommand {
			names: ["page.eval"],
			cli: crate::cli::Commands::Page(crate::cli::PageAction::Eval(raw)) => raw,
		},
		PageConsole => crate::commands::page::console::ConsoleCommand {
			names: ["page.console"],
			cli: crate::cli::Commands::Page(crate::cli::PageAction::Console(raw)) => raw,
		},
		PageRead => crate::commands::page::read::ReadCommand {
			names: ["page.read"],
			cli: crate::cli::Commands::Page(crate::cli::PageAction::Read(raw)) => raw,
		},
		PageElements => crate::commands::page::elements::ElementsCommand {
			names: ["page.elements"],
			cli: crate::cli::Commands::Page(crate::cli::PageAction::Elements(raw)) => raw,
		},
		PageSnapshot => crate::commands::page::snapshot::SnapshotCommand {
			names: ["page.snapshot"],
			cli: crate::cli::Commands::Page(crate::cli::PageAction::Snapshot(raw)) => raw,
		},
		PageCoords => crate::commands::page::coords::CoordsCommand {
			names: ["page.coords"],
			cli: crate::cli::Commands::Page(crate::cli::PageAction::Coords(raw)) => raw,
		},
		PageCoordsAll => crate::commands::page::coords::CoordsAllCommand {
			names: ["page.coords-all", "page.coords_all"],
			cli: crate::cli::Commands::Page(crate::cli::PageAction::CoordsAll(raw)) => raw,
		},
		AuthLogin => crate::commands::auth::LoginCommand {
			names: ["auth.login", "auth-login"],
			cli: crate::cli::Commands::Auth { action: crate::cli::AuthAction::Login { url, output, timeout } } => crate::commands::auth::LoginRaw {
				url,
				output: Some(output),
				timeout_secs: Some(timeout),
			},
		},
		AuthCookies => crate::commands::auth::CookiesCommand {
			names: ["auth.cookies", "auth-cookies"],
			cli: crate::cli::Commands::Auth { action: crate::cli::AuthAction::Cookies { url, format } } => crate::commands::auth::CookiesRaw {
				url,
				format: Some(format),
			},
		},
		AuthShow => crate::commands::auth::ShowCommand {
			names: ["auth.show", "auth-show"],
			cli: crate::cli::Commands::Auth { action: crate::cli::AuthAction::Show { file } } => crate::commands::auth::ShowRaw { file },
		},
		AuthListen => crate::commands::auth::ListenCommand {
			names: ["auth.listen", "auth-listen"],
			cli: crate::cli::Commands::Auth { action: crate::cli::AuthAction::Listen { host, port } } => crate::commands::auth::ListenRaw { host, port },
		},
		SessionStatus => crate::commands::session::SessionStatusCommand {
			names: ["session.status", "session-status"],
			cli: crate::cli::Commands::Session { action: crate::cli::SessionAction::Status } => crate::commands::session::SessionStatusRaw::default(),
		},
		SessionClear => crate::commands::session::SessionClearCommand {
			names: ["session.clear", "session-clear"],
			cli: crate::cli::Commands::Session { action: crate::cli::SessionAction::Clear } => crate::commands::session::SessionClearRaw::default(),
		},
		SessionStart => crate::commands::session::SessionStartCommand {
			names: ["session.start", "session-start"],
			cli: crate::cli::Commands::Session { action: crate::cli::SessionAction::Start { headful } } => crate::commands::session::SessionStartRaw { headful },
		},
		SessionStop => crate::commands::session::SessionStopCommand {
			names: ["session.stop", "session-stop"],
			cli: crate::cli::Commands::Session { action: crate::cli::SessionAction::Stop } => crate::commands::session::SessionStopRaw::default(),
		},
		DaemonStart => crate::commands::daemon::DaemonStartCommand {
			names: ["daemon.start", "daemon-start"],
			cli: crate::cli::Commands::Daemon { action: crate::cli::DaemonAction::Start { foreground } } => crate::commands::daemon::DaemonStartRaw { foreground },
		},
		DaemonStop => crate::commands::daemon::DaemonStopCommand {
			names: ["daemon.stop", "daemon-stop"],
			cli: crate::cli::Commands::Daemon { action: crate::cli::DaemonAction::Stop } => crate::commands::daemon::DaemonStopRaw::default(),
		},
		DaemonStatus => crate::commands::daemon::DaemonStatusCommand {
			names: ["daemon.status", "daemon-status"],
			cli: crate::cli::Commands::Daemon { action: crate::cli::DaemonAction::Status } => crate::commands::daemon::DaemonStatusRaw::default(),
		},
		Connect => crate::commands::connect::ConnectCommand {
			names: ["connect"],
			cli: crate::cli::Commands::Connect {
				endpoint,
				clear,
				launch,
				discover,
				kill,
				port,
				user_data_dir,
			} => crate::commands::connect::ConnectRaw {
				endpoint,
				clear,
				launch,
				discover,
				kill,
				port,
				user_data_dir,
			},
		},
		TabsList => crate::commands::tabs::TabsListCommand {
			names: ["tabs.list", "tabs-list"],
			cli: crate::cli::Commands::Tabs(crate::cli::TabsAction::List) => crate::commands::tabs::TabsListRaw::default(),
		},
		TabsSwitch => crate::commands::tabs::TabsSwitchCommand {
			names: ["tabs.switch", "tabs-switch"],
			cli: crate::cli::Commands::Tabs(crate::cli::TabsAction::Switch { target }) => crate::commands::tabs::TabsSwitchRaw { target },
		},
		TabsClose => crate::commands::tabs::TabsCloseCommand {
			names: ["tabs.close", "tabs-close"],
			cli: crate::cli::Commands::Tabs(crate::cli::TabsAction::Close { target }) => crate::commands::tabs::TabsCloseRaw { target },
		},
		TabsNew => crate::commands::tabs::TabsNewCommand {
			names: ["tabs.new", "tabs-new"],
			cli: crate::cli::Commands::Tabs(crate::cli::TabsAction::New { url }) => crate::commands::tabs::TabsNewRaw { url },
		},
		ProtectAdd => crate::commands::protect::ProtectAddCommand {
			names: ["protect.add", "protect-add"],
			cli: crate::cli::Commands::Protect(crate::cli::ProtectAction::Add { pattern }) => crate::commands::protect::ProtectAddRaw { pattern },
		},
		ProtectRemove => crate::commands::protect::ProtectRemoveCommand {
			names: ["protect.remove", "protect-remove"],
			cli: crate::cli::Commands::Protect(crate::cli::ProtectAction::Remove { pattern }) => crate::commands::protect::ProtectRemoveRaw { pattern },
		},
		ProtectList => crate::commands::protect::ProtectListCommand {
			names: ["protect.list", "protect-list"],
			cli: crate::cli::Commands::Protect(crate::cli::ProtectAction::List) => crate::commands::protect::ProtectListRaw::default(),
		},
		HarSet => crate::commands::har::HarSetCommand {
			names: ["har.set", "har-set"],
			cli: crate::cli::Commands::Har {
				action: crate::cli::HarAction::Set {
					file,
					content,
					mode,
					omit_content,
					url_filter,
				},
			} => crate::commands::har::HarSetRaw {
				file,
				content,
				mode,
				omit_content,
				url_filter,
			},
		},
		HarShow => crate::commands::har::HarShowCommand {
			names: ["har.show", "har-show"],
			cli: crate::cli::Commands::Har { action: crate::cli::HarAction::Show } => crate::commands::har::HarShowRaw::default(),
		},
		HarClear => crate::commands::har::HarClearCommand {
			names: ["har.clear", "har-clear"],
			cli: crate::cli::Commands::Har { action: crate::cli::HarAction::Clear } => crate::commands::har::HarClearRaw::default(),
		},
		Init => crate::commands::init::InitCommand {
			names: ["init"],
			cli: crate::cli::Commands::Init {
				path,
				template,
				no_config,
				no_example,
				typescript,
				force,
				nix,
			} => crate::commands::init::InitRaw {
				path,
				template,
				no_config,
				no_example,
				typescript,
				force,
				nix,
			},
		},
	],
	passthrough: [
		crate::cli::Commands::Run,
		crate::cli::Commands::Relay { .. },
		crate::cli::Commands::Test { .. },
	],
}
