#![deny(unsafe_code)]

//! Verification test suite for Half Adder logic and Basic Logic Gates demo circuits.
//!
//! Validates:
//! 1. Half Adder circuit loading, component count, wire count, and 0 ERC diagnostics.
//! 2. Half Adder SPICE netlist compilation with XOR2, AND2, and HALF_ADDER subcircuits.
//! 3. Basic Logic Gates bench loading and 0 ERC diagnostics across all 8 gate primitives.
//! 4. Basic Logic Gates SPICE compilation with all gate subcircuits.
//! 5. Full coverage and metadata integrity across all 58 ComponentKind variants.

use phonon_gui::schematic::{
    compile_schematic, ComponentCategory, ComponentKind, ErcEngine, ErcSeverity,
};
use phonon_gui::PhononApp;

#[test]
fn test_half_adder_demo_loading_and_erc() {
    let mut app = PhononApp::default();
    app.load_half_adder_demo();

    assert_eq!(
        app.components.len(),
        10,
        "Half Adder demo must contain exactly 10 components (2 pulses, GND, XOR, AND, 2 probes, HA macro, 2 HA probes)"
    );
    assert_eq!(
        app.wires.len(),
        12,
        "Half Adder demo must contain exactly 12 Manhattan wires"
    );
    assert_eq!(
        app.project_title,
        "Half Adder Logic",
        "Project title must match demo name"
    );

    // ERC verification: strictly 0 errors and 0 warnings
    let diagnostics = ErcEngine::evaluate(&app.components, &app.wires);
    let errors_and_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == ErcSeverity::Error || d.severity == ErcSeverity::Warning)
        .collect();

    assert!(
        errors_and_warnings.is_empty(),
        "Half Adder demo must produce 0 ERC errors/warnings, found: {:?}",
        errors_and_warnings
    );
}

#[test]
fn test_half_adder_netlist_compilation() {
    let mut app = PhononApp::default();
    app.load_half_adder_demo();

    let compiled = compile_schematic(&app.components, &app.wires)
        .expect("Half Adder circuit must compile without error");

    assert!(
        !compiled.spice_netlist.is_empty(),
        "Compiled SPICE netlist must not be empty"
    );
    assert!(
        compiled.spice_netlist.contains(".SUBCKT XOR2"),
        "Netlist must define XOR2 subcircuit"
    );
    assert!(
        compiled.spice_netlist.contains(".SUBCKT AND2"),
        "Netlist must define AND2 subcircuit"
    );
    assert!(
        compiled.spice_netlist.contains(".SUBCKT HALF_ADDER"),
        "Netlist must define HALF_ADDER subcircuit"
    );
    assert!(
        compiled.spice_netlist.contains("XUXOR1"),
        "Netlist must instantiate XOR gate"
    );
    assert!(
        compiled.spice_netlist.contains("XUAND1"),
        "Netlist must instantiate AND gate"
    );
    assert!(
        compiled.spice_netlist.contains("XUHA1"),
        "Netlist must instantiate Half Adder macro"
    );
}

#[test]
fn test_basic_gates_demo_loading_and_erc() {
    let mut app = PhononApp::default();
    app.load_basic_gates_demo();

    assert_eq!(
        app.components.len(),
        19,
        "Basic Gates demo must contain 19 components (2 pulses, GND, 8 gates, 8 probes)"
    );
    assert_eq!(
        app.project_title,
        "Basic Logic Gates",
        "Project title must match demo name"
    );

    // ERC verification: strictly 0 errors and 0 warnings
    let diagnostics = ErcEngine::evaluate(&app.components, &app.wires);
    let errors_and_warnings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == ErcSeverity::Error || d.severity == ErcSeverity::Warning)
        .collect();

    assert!(
        errors_and_warnings.is_empty(),
        "Basic Gates demo must produce 0 ERC errors/warnings, found: {:?}",
        errors_and_warnings
    );
}

#[test]
fn test_basic_gates_netlist_compilation() {
    let mut app = PhononApp::default();
    app.load_basic_gates_demo();

    let compiled = compile_schematic(&app.components, &app.wires)
        .expect("Basic Gates circuit must compile without error");

    assert!(compiled.spice_netlist.contains(".SUBCKT BUF_CMOS"));
    assert!(compiled.spice_netlist.contains(".SUBCKT INV_CMOS"));
    assert!(compiled.spice_netlist.contains(".SUBCKT AND2"));
    assert!(compiled.spice_netlist.contains(".SUBCKT OR2"));
    assert!(compiled.spice_netlist.contains(".SUBCKT NAND2"));
    assert!(compiled.spice_netlist.contains(".SUBCKT NOR2"));
    assert!(compiled.spice_netlist.contains(".SUBCKT XOR2"));
    assert!(compiled.spice_netlist.contains(".SUBCKT XNOR2"));
}

#[test]
fn test_all_components_taxonomy_and_metadata() {
    assert!(
        ComponentKind::ALL_VARIANTS.len() >= 58,
        "Must have at least 58 components in ALL_VARIANTS, found {}",
        ComponentKind::ALL_VARIANTS.len()
    );
    assert_eq!(
        ComponentCategory::all_categories().len(),
        11,
        "Must have exactly 11 component categories"
    );

    for &kind in ComponentKind::ALL_VARIANTS {
        assert!(
            !kind.display_name().trim().is_empty(),
            "Component {:?} display name must not be empty",
            kind
        );
        assert!(
            !kind.description().trim().is_empty(),
            "Component {:?} description must not be empty",
            kind
        );
        assert!(
            !kind.prefix().trim().is_empty(),
            "Component {:?} prefix must not be empty",
            kind
        );
        assert!(
            !kind.pin_definitions().is_empty(),
            "Component {:?} must have at least one pin",
            kind
        );
    }
}
