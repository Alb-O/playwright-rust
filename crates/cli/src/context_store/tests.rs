use std::path::PathBuf;

use super::*;

fn empty_global_store() -> ContextStore {
	ContextStore {
		scope: ContextScope::Global,
		path: PathBuf::from("/tmp/global.json"),
		file: ContextStoreFile::default(),
	}
}

#[test]
fn cdp_endpoint_reads_from_global_not_project() {
	let mut store = empty_global_store();
	store.file.contexts.insert(
		"default".to_string(),
		StoredContext {
			cdp_endpoint: Some("ws://global-endpoint".to_string()),
			..Default::default()
		},
	);

	let selected = SelectedContext {
		name: "default".to_string(),
		scope: ContextScope::Project,
		data: StoredContext {
			cdp_endpoint: Some("ws://project-endpoint".to_string()),
			..Default::default()
		},
	};

	let state = ContextState::test_new(
		ContextBook {
			global: store,
			project: None,
		},
		Some(selected),
	);

	assert_eq!(state.cdp_endpoint(), Some("ws://global-endpoint"));
}

#[test]
fn cdp_endpoint_writes_to_global_not_project() {
	let selected = SelectedContext {
		name: "default".to_string(),
		scope: ContextScope::Project,
		data: StoredContext::default(),
	};

	let mut state = ContextState::test_new(
		ContextBook {
			global: empty_global_store(),
			project: None,
		},
		Some(selected),
	);

	state.set_cdp_endpoint(Some("ws://new-endpoint".to_string()));

	assert_eq!(state.cdp_endpoint(), Some("ws://new-endpoint"));
}

#[test]
fn cdp_endpoint_updates_selected_when_global_default() {
	// When selected context IS the global default, set_cdp_endpoint
	// must also update selected.data so persist() doesn't overwrite it
	let selected = SelectedContext {
		name: "default".to_string(),
		scope: ContextScope::Global,
		data: StoredContext::default(),
	};

	let mut state = ContextState::test_new(
		ContextBook {
			global: empty_global_store(),
			project: None,
		},
		Some(selected),
	);

	state.set_cdp_endpoint(Some("ws://new-endpoint".to_string()));

	// Both the store and selected.data should have the new endpoint
	assert_eq!(state.cdp_endpoint(), Some("ws://new-endpoint"));
	assert_eq!(
		state.selected().unwrap().data.cdp_endpoint,
		Some("ws://new-endpoint".to_string())
	);
}

#[test]
fn cdp_endpoint_does_not_update_selected_when_project_context() {
	// When selected context is project-scoped, set_cdp_endpoint
	// should NOT update selected.data (CDP is only in global)
	let selected = SelectedContext {
		name: "myproject".to_string(),
		scope: ContextScope::Project,
		data: StoredContext::default(),
	};

	let mut state = ContextState::test_new(
		ContextBook {
			global: empty_global_store(),
			project: None,
		},
		Some(selected),
	);

	state.set_cdp_endpoint(Some("ws://new-endpoint".to_string()));

	// Store has the endpoint
	assert_eq!(state.cdp_endpoint(), Some("ws://new-endpoint"));
	// But selected.data does not (it's a project context)
	assert_eq!(state.selected().unwrap().data.cdp_endpoint, None);
}
