#![deny(unsafe_code)]

//! Verification test suite for Phase 355: Script Runtime Permission Sandboxing.
//!
//! Tests default prompt state on unapproved file I/O, temporary session grants,
//! permanent settings persistence, and explicit deny blocking.

use phonon_gui::scripting::lua_engine::LuaEngine;
use phonon_gui::scripting::permissions::{PermissionKind, PermissionManager, PermissionState};

#[test]
fn test_default_prompt_state_blocks_unauthorized_io() {
    let mut manager = PermissionManager::new();
    let read_req = PermissionKind::FileRead("design_netlist.sp".to_string());
    let write_req = PermissionKind::FileWrite("simulation_out.csv".to_string());

    // Initially query returns PendingPrompt
    assert_eq!(manager.query_permission(&read_req), PermissionState::PendingPrompt);
    assert_eq!(manager.query_permission(&write_req), PermissionState::PendingPrompt);

    // Requesting permission raises pending error and sets pending_request
    let res = manager.request_permission(read_req.clone());
    assert!(res.is_err());
    let err_msg = res.unwrap_err();
    assert!(err_msg.contains("Permission pending user approval"));
    assert_eq!(manager.pending_request, Some(read_req));
}

#[test]
fn test_session_allow_grants_access_and_clears_pending() {
    let mut manager = PermissionManager::new();
    let read_req = PermissionKind::FileRead("model.cir".to_string());

    let _ = manager.request_permission(read_req.clone());
    assert!(manager.pending_request.is_some());

    // Grant for session
    manager.allow_session(read_req.clone());
    assert!(manager.pending_request.is_none());
    assert_eq!(manager.query_permission(&read_req), PermissionState::AllowedSession);

    // Subsequent request succeeds
    assert!(manager.request_permission(read_req.clone()).is_ok());

    // Clearing session rules resets back to pending
    manager.clear_session_rules();
    assert_eq!(manager.query_permission(&read_req), PermissionState::PendingPrompt);
}

#[test]
fn test_permanent_allow_and_settings_persistence() {
    let mut manager = PermissionManager::new();
    let write_req = PermissionKind::FileWrite("/tmp/phonon_report.txt".to_string());

    manager.allow_permanent(write_req.clone());
    assert_eq!(manager.query_permission(&write_req), PermissionState::AllowedAlways);
    assert!(manager.request_permission(write_req.clone()).is_ok());

    // Export settings
    let exported = manager.export_settings();
    assert!(exported.contains("write:/tmp/phonon_report.txt=AllowedAlways"));

    // Import into fresh manager
    let mut restored_mgr = PermissionManager::new();
    restored_mgr.import_settings(&exported);
    assert_eq!(restored_mgr.query_permission(&write_req), PermissionState::AllowedAlways);
}

#[test]
fn test_deny_explicitly_blocks_file_operations() {
    let mut manager = PermissionManager::new();
    let read_req = PermissionKind::FileRead("/etc/passwd".to_string());

    manager.deny(read_req.clone(), false);
    assert_eq!(manager.query_permission(&read_req), PermissionState::Denied);

    let res = manager.request_permission(read_req);
    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Permission denied"));
}

#[test]
fn test_lua_engine_sandboxing_integration() {
    let mut engine = LuaEngine::new();

    // Attempting to read a file without permission triggers pending prompt error
    let script = r#"
        local content = phonon.read_file("unauthorized.txt")
    "#;
    let res = engine.run_script(script);
    assert!(res.is_err());
    assert!(engine.permissions.pending_request.is_some());
    let req = engine.permissions.pending_request.as_ref().unwrap();
    assert_eq!(req.path(), "unauthorized.txt");

    // Once explicitly allowed, file operations proceed
    engine.permissions.allow_session(PermissionKind::FileWrite("sandbox_test.txt".to_string()));
    engine.permissions.allow_session(PermissionKind::FileRead("sandbox_test.txt".to_string()));

    let script_allowed = r#"
        phonon.write_file("sandbox_test.txt", "Phonon CAD Test")
        local txt = phonon.read_file("sandbox_test.txt")
        print("File content: " .. txt)
    "#;
    let res2 = engine.run_script(script_allowed);
    assert!(res2.is_ok());
    assert!(engine.output_log.iter().any(|l| l.contains("Phonon CAD Test")));

    // Clean up temporary test file
    #[cfg(not(target_arch = "wasm32"))]
    let _ = std::fs::remove_file("sandbox_test.txt");
}
