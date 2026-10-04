#![deny(unsafe_code)]

//! Test suite for Oscilloscope usability, bounds clamping, SI engineering formatters,
//! and Multi-Graph scope management with golden reference comparison.

use egui::Color32;
use phonon_gui::oscilloscope::{
    format_current_si, format_time_si, format_voltage_si, MultiGraphManager, OscilloscopePanel,
    ScopeRunMode, WaveformTrace, MAX_AMPLITUDE_SPAN, MAX_TIME_SPAN, MIN_TIME_SPAN,
};

#[test]
fn test_si_time_formatting() {
    assert_eq!(format_time_si(1e-15), "1.00 fs");
    assert_eq!(format_time_si(25e-12), "25.00 ps");
    assert_eq!(format_time_si(500e-9), "500.00 ns");
    assert_eq!(format_time_si(12.5e-6), "12.50 us");
    assert_eq!(format_time_si(3.2e-3), "3.20 ms");
    assert_eq!(format_time_si(1.5), "1.500 s");
    assert_eq!(format_time_si(-4.5e-3), "-4.50 ms");
}

#[test]
fn test_si_voltage_formatting() {
    assert_eq!(format_voltage_si(1e-9), "1.00 nV");
    assert_eq!(format_voltage_si(50e-6), "50.00 uV");
    assert_eq!(format_voltage_si(250e-3), "250.00 mV");
    assert_eq!(format_voltage_si(3.3), "3.300 V");
    assert_eq!(format_voltage_si(1200.0), "1.20 kV");
    assert_eq!(format_voltage_si(-0.75), "-750.00 mV");
}

#[test]
fn test_si_current_formatting() {
    assert_eq!(format_current_si(1e-12), "1.00 pA");
    assert_eq!(format_current_si(10e-9), "10.00 nA");
    assert_eq!(format_current_si(2.5e-6), "2.50 uA");
    assert_eq!(format_current_si(15e-3), "15.00 mA");
    assert_eq!(format_current_si(2.0), "2.000 A");
    assert_eq!(format_current_si(-50e-3), "-50.00 mA");
}

#[test]
fn test_scope_bounds_constants() {
    assert!(MIN_TIME_SPAN > 0.0);
    assert!(MAX_TIME_SPAN >= 1.0e4);
    assert!(MAX_AMPLITUDE_SPAN >= 1.0e6);
}

#[test]
fn test_oscilloscope_auto_fit_bounds() {
    let mut scope = OscilloscopePanel::new();
    assert!(scope.compute_data_bounds().is_none());

    let mut trace = WaveformTrace::new("Test Wave", Color32::RED);
    trace.push(0.001, 1.0);
    trace.push(0.002, 3.0);
    trace.push(0.003, 2.0);
    scope.add_trace(trace);

    let bounds = scope.compute_data_bounds();
    assert!(bounds.is_some());
    let (x_bounds, y_bounds) = bounds.unwrap();
    assert!(x_bounds[0] <= 0.001);
    assert!(x_bounds[1] >= 0.003);
    assert!(y_bounds[0] < 1.0);
    assert!(y_bounds[1] > 3.0);
}

#[test]
fn test_multi_graph_manager_spawning_and_detaching() {
    let mut manager = MultiGraphManager::new();
    assert!(manager.floating_scopes.is_empty());

    let id1 = manager.spawn_floating_scope("Scope A");
    assert_eq!(manager.floating_scopes.len(), 1);
    assert_eq!(manager.floating_scopes[0].id, id1);
    assert_eq!(manager.floating_scopes[0].title, "Scope A");

    // Add trace to primary scope
    let mut trace = WaveformTrace::new("Primary Trace", Color32::BLUE);
    trace.push(0.0, 0.0);
    trace.push(0.001, 5.0);
    manager.primary_scope.add_trace(trace);

    // Detach primary to new floating window
    let id2 = manager.detach_primary_to_floating();
    assert_eq!(manager.floating_scopes.len(), 2);
    let detached = manager.floating_scopes.iter().find(|s| s.id == id2).unwrap();
    assert_eq!(detached.panel.traces.len(), 1);
    assert_eq!(detached.panel.traces[0].name, "Primary Trace");
}

#[test]
fn test_multi_graph_live_vs_hold_reference_routing() {
    let mut manager = MultiGraphManager::new();
    let id = manager.spawn_floating_scope("Comparator Scope");

    // Route initial live traces
    let mut trace1 = WaveformTrace::new("V_SIG", Color32::GREEN);
    trace1.push(0.0, 1.0);
    trace1.push(0.01, 2.0);
    manager.route_simulation_traces(&[trace1]);

    // Both primary and floating scope should have 1 trace
    assert_eq!(manager.primary_scope.traces.len(), 1);
    let floating = manager.floating_scopes.iter_mut().find(|s| s.id == id).unwrap();
    assert_eq!(floating.panel.traces.len(), 1);
    assert_eq!(floating.run_mode, ScopeRunMode::Live);

    // Freeze golden reference on floating scope
    floating.freeze_reference();
    assert_eq!(floating.run_mode, ScopeRunMode::HoldReference);
    assert_eq!(floating.reference_traces.len(), 1);
    assert!(floating.reference_traces[0].name.contains("[REF]"));

    // Route updated simulation traces (new iteration)
    let mut trace2 = WaveformTrace::new("V_SIG", Color32::GREEN);
    trace2.push(0.0, 1.5);
    trace2.push(0.01, 2.5);
    manager.route_simulation_traces(&[trace2]);

    // Primary scope gets 1 live trace
    assert_eq!(manager.primary_scope.traces.len(), 1);

    // Floating scope in HoldReference mode should contain BOTH [REF] and [LIVE] overlay traces
    let floating_ref = manager.floating_scopes.iter().find(|s| s.id == id).unwrap();
    assert_eq!(floating_ref.panel.traces.len(), 2);
    let has_ref = floating_ref.panel.traces.iter().any(|t| t.name.contains("[REF]"));
    let has_live = floating_ref.panel.traces.iter().any(|t| t.name.contains("[LIVE]"));
    assert!(has_ref);
    assert!(has_live);

    // Clear reference and verify return to Live mode
    let floating_mut = manager.floating_scopes.iter_mut().find(|s| s.id == id).unwrap();
    floating_mut.clear_reference();
    assert_eq!(floating_mut.run_mode, ScopeRunMode::Live);
    assert!(floating_mut.reference_traces.is_empty());
}
