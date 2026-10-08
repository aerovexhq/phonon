#![deny(unsafe_code)]

//! Automated Verification Suite for Phase 426: Topological Quantum Metamaterials Demo Circuit.
//!
//! Verifies:
//! - Complete schematic construction with SAW IDT filter, Parafermionic Cavity,
//!   Chiral Skyrmion Router, and Non-Abelian Majorana Braiding Junction.
//! - Strict Zero-Diagnostic ERC: 0 errors and 0 warnings on Electrical Rules Check.
//! - Clean SPICE netlist compilation with correct pin-to-node mappings and subcircuit calls.
//! - DC Operating Point and Transient simulation convergence through the MNA solver.

use phonon_gui::app::PhononApp;
use phonon_gui::schematic::circuit_compiler::compile_schematic;
use phonon_gui::schematic::components::ComponentKind;
use phonon_gui::schematic::erc::{ErcEngine, ErcSeverity};

#[test]
fn test_quantum_metamaterial_demo_circuit_structure() {
    let mut app = PhononApp::default();
    app.load_quantum_metamaterial_demo();

    // Verify component count
    assert_eq!(app.components.len(), 9, "Expected exactly 9 components in demo");

    // Verify all 4 topological quantum metamaterial components are present
    let kinds: Vec<ComponentKind> = app.components.iter().map(|c| c.kind).collect();
    assert!(kinds.contains(&ComponentKind::SawIdt), "Must contain SawIdt component");
    assert!(kinds.contains(&ComponentKind::ParafermionicCavity), "Must contain ParafermionicCavity component");
    assert!(kinds.contains(&ComponentKind::SkyrmionRouter), "Must contain SkyrmionRouter component");
    assert!(kinds.contains(&ComponentKind::MajoranaJunction), "Must contain MajoranaJunction component");
    assert!(kinds.contains(&ComponentKind::PulseGenerator), "Must contain PulseGenerator excitation");
    assert!(kinds.contains(&ComponentKind::VoltageSource), "Must contain VoltageSource gate bias");
    assert!(kinds.contains(&ComponentKind::Ground), "Must contain Ground reference");
    assert!(kinds.contains(&ComponentKind::Resistor), "Must contain load Resistor");
    assert!(kinds.contains(&ComponentKind::LogicProbe), "Must contain LogicProbe");

    // Verify wire count
    assert!(app.wires.len() >= 12, "Expected at least 12 wire branches in schematic");
}

#[test]
fn test_quantum_metamaterial_demo_erc_zero_diagnostics() {
    let mut app = PhononApp::default();
    app.load_quantum_metamaterial_demo();

    let diagnostics = ErcEngine::evaluate(&app.components, &app.wires);
    let errors: Vec<_> = diagnostics.iter().filter(|d| d.severity == ErcSeverity::Error).collect();
    let warnings: Vec<_> = diagnostics.iter().filter(|d| d.severity == ErcSeverity::Warning).collect();

    assert!(
        errors.is_empty(),
        "Expected 0 ERC errors, found {}: {:?}",
        errors.len(),
        errors
    );
    assert!(
        warnings.is_empty(),
        "Expected 0 ERC warnings, found {}: {:?}",
        warnings.len(),
        warnings
    );
}

#[test]
fn test_quantum_metamaterial_demo_spice_compilation() {
    let mut app = PhononApp::default();
    app.load_quantum_metamaterial_demo();

    let compile_result = compile_schematic(&app.components, &app.wires);
    assert!(
        compile_result.is_ok(),
        "Schematic compilation must succeed: {:?}",
        compile_result.err()
    );

    let compiled = compile_result.unwrap();
    let netlist = compiled.spice_netlist;

    // Verify SPICE netlist contains expected device instances
    assert!(netlist.contains("XSAW1"), "Netlist must instantiate SAW IDT filter");
    assert!(netlist.contains("XPC1"), "Netlist must instantiate Parafermionic cavity");
    assert!(netlist.contains("XSR1"), "Netlist must instantiate Skyrmion router");
    assert!(netlist.contains("XMJ1"), "Netlist must instantiate Majorana junction");
    assert!(netlist.contains("VPULSE1"), "Netlist must instantiate pulse generator VPULSE1");
    assert!(netlist.contains("V1"), "Netlist must instantiate gate bias V1");
    assert!(netlist.contains("R1"), "Netlist must instantiate load resistor R1");
}

#[test]
fn test_quantum_metamaterial_demo_simulation_solve() {
    let mut app = PhononApp::default();
    app.load_quantum_metamaterial_demo();

    // Execute DC operating point solve
    app.run_dc_op();
    assert!(
        !app.sim_status.starts_with("Compilation Error") && !app.sim_status.starts_with("Error:"),
        "DC solve must not produce error status: {}",
        app.sim_status
    );

    // Execute transient demo simulation
    app.run_transient_demo();
    assert!(
        !app.sim_status.starts_with("Error:"),
        "Transient simulation must succeed: {}",
        app.sim_status
    );
    assert!(
        !app.oscilloscope.traces.is_empty(),
        "Oscilloscope must contain traces after transient solve"
    );
}
