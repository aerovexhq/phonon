#![deny(unsafe_code)]

//! Comprehensive verification suite for Phonon Studio Port-Hamiltonian Biomechanical
//! Articulatory Library & Dual-Domain Waveform Oscilloscope (Phase 324).
//!
//! Validates:
//! 1. Taxonomy registration of `ComponentCategory::PortHamiltonian`.
//! 2. Primacy, prefixes, and search indices for `PhLungs`, `PhVocalFolds`, `PhVocalTract`, `PhLipRadiation`.
//! 3. Pin geometry and terminal topology verification.
//! 4. Compact binary serialization (.phn) round-trip with discriminants 31-34.
//! 5. SPICE subcircuit generation (`.SUBCKT PH_LUNGS`, `.SUBCKT PH_VOCAL_FOLDS`, etc.) and MNA stamping.
//! 6. Dual-Domain Oscilloscope: `SignalDomain`, dB SPL calculation ($P_0 = 20\ \mu\mathrm{Pa}$), volume flow ($\mathrm{cm}^3/\mathrm{s}$).

use egui::{Color32, Pos2};
use phonon_gui::oscilloscope::{OscilloscopePanel, SignalDomain, WaveformTrace};
use phonon_gui::schematic::{
    compile_schematic, deserialize_project, serialize_project, ComponentCategory, ComponentKind,
    SchematicComponent, SchematicWire,
};
use std::collections::HashSet;

#[test]
fn test_port_hamiltonian_category_registration() {
    let cat = ComponentCategory::PortHamiltonian;
    assert_eq!(
        cat.display_name(),
        "Port-Hamiltonian Articulatory Acoustics"
    );
    assert!(!cat.description().is_empty());

    let comps = cat.components();
    assert_eq!(comps.len(), 4, "Must define exactly 4 articulatory primitives");
    assert!(comps.contains(&ComponentKind::PhLungs));
    assert!(comps.contains(&ComponentKind::PhVocalFolds));
    assert!(comps.contains(&ComponentKind::PhVocalTract));
    assert!(comps.contains(&ComponentKind::PhLipRadiation));

    for comp in comps {
        assert_eq!(comp.category(), cat);
    }
}

#[test]
fn test_port_hamiltonian_metadata_and_search_indexing() {
    let primitives = [
        (
            ComponentKind::PhLungs,
            "PhLungs",
            "XLUNG",
            "PH_LUNGS",
            "lungs",
        ),
        (
            ComponentKind::PhVocalFolds,
            "PhVocalFolds",
            "XVF",
            "PH_VOCAL_FOLDS",
            "vocal",
        ),
        (
            ComponentKind::PhVocalTract,
            "PhVocalTract",
            "XVT",
            "PH_VOCAL_TRACT",
            "tract",
        ),
        (
            ComponentKind::PhLipRadiation,
            "PhLipRadiation",
            "XRAD",
            "PH_LIP_RADIATION",
            "radiation",
        ),
    ];

    for (kind, code_name, prefix, default_val, keyword) in primitives {
        assert_eq!(kind.code_name(), code_name);
        assert_eq!(kind.prefix(), prefix);
        assert_eq!(kind.default_value(), default_val);
        assert!(kind.display_name().len() > 3);
        assert!(kind.description().len() > 10);
        assert!(
            kind.search_index().contains(keyword),
            "Search index for {:?} must contain keyword '{}'",
            kind,
            keyword
        );
    }
}

#[test]
fn test_port_hamiltonian_pin_topology() {
    // 1. PhLungs: 2 pins
    let lungs_pins = ComponentKind::PhLungs.pin_definitions();
    assert_eq!(lungs_pins.len(), 2);
    assert_eq!(lungs_pins[0].0, "P_SUB");
    assert_eq!(lungs_pins[1].0, "REF");

    // 2. PhVocalFolds: 4 pins
    let vf_pins = ComponentKind::PhVocalFolds.pin_definitions();
    assert_eq!(vf_pins.len(), 4);
    let vf_names: HashSet<&str> = vf_pins.iter().map(|(n, _)| *n).collect();
    assert!(vf_names.contains("SUB"));
    assert!(vf_names.contains("SUPRA"));
    assert!(vf_names.contains("CTRL"));
    assert!(vf_names.contains("REF"));

    // 3. PhVocalTract: 4 pins
    let vt_pins = ComponentKind::PhVocalTract.pin_definitions();
    assert_eq!(vt_pins.len(), 4);
    let vt_names: HashSet<&str> = vt_pins.iter().map(|(n, _)| *n).collect();
    assert!(vt_names.contains("IN"));
    assert!(vt_names.contains("OUT"));
    assert!(vt_names.contains("WALL"));
    assert!(vt_names.contains("CTRL"));

    // 4. PhLipRadiation: 2 pins
    let rad_pins = ComponentKind::PhLipRadiation.pin_definitions();
    assert_eq!(rad_pins.len(), 2);
    assert_eq!(rad_pins[0].0, "IN");
    assert_eq!(rad_pins[1].0, "RAD");

    // Verify all pins have finite coords
    for kind in [
        ComponentKind::PhLungs,
        ComponentKind::PhVocalFolds,
        ComponentKind::PhVocalTract,
        ComponentKind::PhLipRadiation,
    ] {
        for (name, pos) in kind.pin_definitions() {
            assert!(
                pos.x.is_finite() && pos.y.is_finite(),
                "Non-finite pin coordinate for {:?} pin {}",
                kind,
                name
            );
        }
    }
}

#[test]
fn test_port_hamiltonian_binary_round_trip() {
    let title = "Port-Hamiltonian Vocal Tract Circuit";
    let mut components = Vec::new();

    let kinds = [
        ComponentKind::PhLungs,
        ComponentKind::PhVocalFolds,
        ComponentKind::PhVocalTract,
        ComponentKind::PhLipRadiation,
    ];

    for (i, &kind) in kinds.iter().enumerate() {
        let mut comp = SchematicComponent::new(
            i + 1,
            kind,
            Pos2::new(100.0 + (i as f32) * 50.0, 200.0),
            i + 1,
        );
        comp.rotation = (i % 4) as u8;
        comp.properties
            .push(("acoustic_rho".to_string(), "1.184".to_string()));
        comp.properties
            .push(("acoustic_c".to_string(), "346.0".to_string()));
        components.push(comp);
    }

    let wires = vec![SchematicWire::manhattan_route_with_net(
        1,
        Pos2::new(100.0, 200.0),
        Pos2::new(150.0, 200.0),
        Some("P_SUB_NET".to_string()),
    )];

    let bytes = serialize_project(title, &components, &wires);
    assert!(
        bytes.len() > 100,
        "Serialized stream must contain populated component records"
    );

    let deserialized = deserialize_project(&bytes).expect("Deserialization must succeed");
    assert_eq!(deserialized.title, title);
    assert_eq!(deserialized.components.len(), 4);
    assert_eq!(deserialized.wires.len(), 1);

    for (orig, res) in components.iter().zip(deserialized.components.iter()) {
        assert_eq!(res.id, orig.id);
        assert_eq!(res.kind, orig.kind);
        assert_eq!(res.name, orig.name);
        assert_eq!(res.rotation, orig.rotation);
        assert_eq!(res.value_str, orig.value_str);
        assert_eq!(res.properties, orig.properties);
    }
}

#[test]
fn test_port_hamiltonian_circuit_compilation_and_spice_subcircuits() {
    let lungs = SchematicComponent::new(1, ComponentKind::PhLungs, Pos2::new(100.0, 100.0), 1);
    let vf = SchematicComponent::new(2, ComponentKind::PhVocalFolds, Pos2::new(200.0, 100.0), 1);
    let vt = SchematicComponent::new(3, ComponentKind::PhVocalTract, Pos2::new(300.0, 100.0), 1);
    let rad = SchematicComponent::new(4, ComponentKind::PhLipRadiation, Pos2::new(400.0, 100.0), 1);

    let components = vec![lungs, vf, vt, rad];
    let wires = Vec::new();

    let compiled =
        compile_schematic(&components, &wires).expect("Schematic compilation must succeed");

    let spice = compiled.spice_netlist;

    // Verify subcircuit templates are injected
    assert!(
        spice.contains(".SUBCKT PH_LUNGS"),
        "SPICE must include .SUBCKT PH_LUNGS"
    );
    assert!(
        spice.contains(".SUBCKT PH_VOCAL_FOLDS"),
        "SPICE must include .SUBCKT PH_VOCAL_FOLDS"
    );
    assert!(
        spice.contains(".SUBCKT PH_VOCAL_TRACT"),
        "SPICE must include .SUBCKT PH_VOCAL_TRACT"
    );
    assert!(
        spice.contains(".SUBCKT PH_LIP_RADIATION"),
        "SPICE must include .SUBCKT PH_LIP_RADIATION"
    );

    // Verify component instances
    assert!(
        spice.contains("XLUNG1"),
        "SPICE netlist must contain XLUNG1 instance"
    );
    assert!(
        spice.contains("XVF1"),
        "SPICE netlist must contain XVF1 instance"
    );
    assert!(
        spice.contains("XVT1"),
        "SPICE netlist must contain XVT1 instance"
    );
    assert!(
        spice.contains("XRAD1"),
        "SPICE netlist must contain XRAD1 instance"
    );

    // Verify internal components stamped into CircuitGraph
    assert!(compiled.graph.total_nodes() > 0);
    assert!(compiled.graph.num_components() > 0);
}

#[test]
fn test_dual_domain_oscilloscope_signal_domains_and_metrics() {
    // 1. Electrical trace
    let mut v_trace = WaveformTrace::new("V(out)", Color32::YELLOW);
    assert_eq!(v_trace.domain, SignalDomain::Electrical);
    assert_eq!(v_trace.domain.unit_symbol(), "V");
    for i in 0..200 {
        let t = i as f64 * 0.0001;
        let v = 5.0 * (2.0 * std::f64::consts::PI * 100.0 * t).sin();
        v_trace.push(t, v);
    }
    assert!((v_trace.v_pp() - 10.0).abs() < 1e-6);
    assert!((v_trace.v_rms() - (5.0 / 2.0f64.sqrt())).abs() < 0.1);

    // 2. Acoustic Pressure trace
    let mut p_trace = WaveformTrace::new_with_domain(
        "P(mouth)",
        Color32::from_rgb(0, 220, 255),
        SignalDomain::AcousticPressure,
    );
    assert_eq!(p_trace.domain, SignalDomain::AcousticPressure);
    assert_eq!(p_trace.domain.unit_symbol(), "Pa");

    // Peak acoustic pressure of 2.0 Pa -> SPL = 20 * log10(2.0 / 20e-6) = 20 * log10(100,000) = 100.0 dB SPL
    for i in 0..100 {
        let t = i as f64 * 0.0001;
        let p = 2.0 * (2.0 * std::f64::consts::PI * 440.0 * t).sin();
        p_trace.push(t, p);
    }
    let spl = p_trace.peak_spl_db();
    assert!(
        (spl - 100.0).abs() < 1.0,
        "Peak SPL must be ~100.0 dB SPL for 2.0 Pa amplitude, got {:.2}",
        spl
    );

    // 3. Acoustic Volume Flow trace
    let mut q_trace = WaveformTrace::new_with_domain(
        "U(glottis)",
        Color32::from_rgb(100, 255, 100),
        SignalDomain::AcousticFlow,
    );
    assert_eq!(q_trace.domain, SignalDomain::AcousticFlow);
    assert_eq!(q_trace.domain.unit_symbol(), "m³/s");

    // Flow of 0.0001 m^3/s -> 100 cm^3/s
    for i in 0..50 {
        q_trace.push(i as f64 * 0.001, 0.0001);
    }
    let flow_cm3 = q_trace.flow_rate_cm3_s();
    assert!(
        (flow_cm3 - 100.0).abs() < 1e-3,
        "Flow rate must be 100 cm^3/s, got {:.2}",
        flow_cm3
    );

    // 4. Multi-trace panel integration
    let mut panel = OscilloscopePanel::new();
    panel.add_trace(v_trace);
    panel.add_trace(p_trace);
    panel.add_trace(q_trace);
    assert_eq!(panel.traces.len(), 3);
}
