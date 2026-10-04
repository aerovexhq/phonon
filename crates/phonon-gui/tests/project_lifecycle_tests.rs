#![deny(unsafe_code)]

use phonon_gui::schematic::{
    CanvasCommand, ComponentKind, SchematicComponent, SchematicWire,
};
use phonon_gui::storage::{
    bytes_to_hex, hex_to_bytes, MemoryStorageAdapter, ProjectStorageManager, StorageAdapter,
};
use phonon_gui::widgets::{
    DemoCircuitKind, PendingAction, ProjectDialog, ProjectDialogMode,
};
use phonon_gui::PhononApp;

#[test]
fn test_hex_conversion_roundtrip() {
    let data = b"Phonon Studio VFS Storage Test Payload 1234567890";
    let hex = bytes_to_hex(data);
    let decoded = hex_to_bytes(&hex).expect("Failed to decode hex");
    assert_eq!(decoded, data);

    assert!(hex_to_bytes("odd").is_err());
    assert!(hex_to_bytes("zz").is_err());
}

#[test]
fn test_memory_storage_adapter_crud() {
    let mut adapter = MemoryStorageAdapter::new();
    assert_eq!(adapter.list_projects().unwrap().len(), 0);

    let p1_data = b"Project 1 Data";
    adapter.save_project("Project1", p1_data).unwrap();
    let p2_data = b"Project 2 Data";
    adapter.save_project("Project2", p2_data).unwrap();

    let list = adapter.list_projects().unwrap();
    assert_eq!(list, vec!["Project1".to_string(), "Project2".to_string()]);

    let loaded = adapter.load_project("Project1").unwrap();
    assert_eq!(loaded, p1_data);

    adapter.delete_project("Project1").unwrap();
    assert_eq!(adapter.list_projects().unwrap(), vec!["Project2".to_string()]);
    assert!(adapter.load_project("Project1").is_err());

    // Autosave round-trip
    assert!(adapter.load_autosave().is_none());
    adapter.save_autosave(b"Autosave snapshot").unwrap();
    assert_eq!(
        adapter.load_autosave().unwrap(),
        b"Autosave snapshot".to_vec()
    );
    adapter.clear_autosave().unwrap();
    assert!(adapter.load_autosave().is_none());
}

#[test]
fn test_project_storage_manager_roundtrip() {
    let mem = Box::new(MemoryStorageAdapter::new());
    let mut manager = ProjectStorageManager::with_adapter(mem);

    let comp = SchematicComponent::new(1, ComponentKind::Resistor, egui::Pos2::new(100.0, 100.0), 1);
    let wire = SchematicWire::manhattan_route(1, egui::Pos2::new(100.0, 100.0), egui::Pos2::new(150.0, 100.0));

    manager
        .save_project("TestCircuit", &[comp.clone()], &[wire.clone()])
        .unwrap();

    let list = manager.list_projects().unwrap();
    assert_eq!(list, vec!["TestCircuit".to_string()]);

    let loaded = manager.load_project("TestCircuit").unwrap();
    assert_eq!(loaded.title, "TestCircuit");
    assert_eq!(loaded.components.len(), 1);
    assert_eq!(loaded.wires.len(), 1);
    assert_eq!(loaded.components[0].id, 1);
    assert_eq!(loaded.wires[0].id, 1);

    manager.delete_project("TestCircuit").unwrap();
    assert!(manager.load_project("TestCircuit").is_err());
}

#[test]
fn test_app_clean_baseline_on_boot() {
    let app = PhononApp::default();
    assert!(!app.is_dirty());
    assert_eq!(app.modification_epoch, app.clean_epoch);
    assert!(app.pending_confirmation_action.is_none());
    assert!(!app.should_close);
}

#[test]
fn test_app_dirty_tracking_on_mutation_and_undo() {
    let mut app = PhononApp::default();
    assert!(!app.is_dirty());

    // Adding a component marks app as dirty
    let comp = SchematicComponent::new(10, ComponentKind::Capacitor, egui::Pos2::new(50.0, 50.0), 1);
    app.history.record(CanvasCommand::AddComponent(comp.clone()));
    app.components.push(comp);
    app.mark_dirty();

    assert!(app.is_dirty());
    let epoch_after_edit = app.modification_epoch;
    assert!(epoch_after_edit > app.clean_epoch);

    // Ctrl+Z Undo still preserves dirty state because modification history diverged from saved baseline
    app.undo();
    assert!(app.is_dirty());
    assert!(app.modification_epoch > epoch_after_edit);

    // Explicit save marks app clean
    app.mark_clean();
    assert!(!app.is_dirty());
    assert_eq!(app.modification_epoch, app.clean_epoch);
}

#[test]
fn test_confirmation_interception_when_dirty() {
    let mut app = PhononApp::default();
    assert!(!app.is_dirty());

    // When clean, action executes immediately without interception
    app.request_action(PendingAction::CloseApp);
    assert!(app.should_close);
    assert!(app.pending_confirmation_action.is_none());

    // Reset close flag and mark dirty
    app.should_close = false;
    app.mark_dirty();
    assert!(app.is_dirty());

    // When dirty, CloseApp is intercepted with confirmation modal
    app.request_action(PendingAction::CloseApp);
    assert!(!app.should_close);
    assert_eq!(
        app.pending_confirmation_action,
        Some(PendingAction::CloseApp)
    );

    // Cancel decision clears pending action without executing
    app.pending_confirmation_action = None;
    assert!(!app.should_close);

    // DiscardAndProceed executes the action
    app.execute_confirmation_action(PendingAction::CloseApp);
    assert!(app.should_close);
}

#[test]
fn test_demo_load_interception_and_clean_reset() {
    let mut app = PhononApp::default();
    app.mark_dirty();
    assert!(app.is_dirty());

    // Loading demo when dirty intercepts
    app.request_action(PendingAction::LoadDemo(DemoCircuitKind::DiodeClipper));
    assert_eq!(
        app.pending_confirmation_action,
        Some(PendingAction::LoadDemo(DemoCircuitKind::DiodeClipper))
    );

    // Executing the demo load resets to clean baseline
    app.pending_confirmation_action = None;
    app.execute_confirmation_action(PendingAction::LoadDemo(DemoCircuitKind::DiodeClipper));
    assert!(!app.is_dirty());
    assert_eq!(app.project_title, "Diode Clipper");
    assert!(!app.components.is_empty());
}

#[test]
fn test_project_dialog_modes_and_flow() {
    let mut dialog = ProjectDialog::new();
    assert!(!dialog.is_open);

    dialog.open_for_open();
    assert!(dialog.is_open);
    assert_eq!(dialog.mode, ProjectDialogMode::Open);

    dialog.open_for_save_as("MyAmplifier");
    assert!(dialog.is_open);
    assert_eq!(dialog.mode, ProjectDialogMode::SaveAs);
    assert_eq!(dialog.project_name_buffer, "MyAmplifier");

    dialog.open_for_manager();
    assert!(dialog.is_open);
    assert_eq!(dialog.mode, ProjectDialogMode::Manager);

    dialog.close();
    assert!(!dialog.is_open);
}
