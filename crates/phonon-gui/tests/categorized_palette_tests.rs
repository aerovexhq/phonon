#![deny(unsafe_code)]

//! Verification suite for Phonon Studio Categorized Component Architecture (Phase 308):
//! Multi-tier hierarchical component taxonomy, component-category bijection,
//! pin coordinate validity, live search filtering, circuit compilation,
//! and palette filtering throughput benchmarking.

use egui::Pos2;
use phonon_gui::schematic::{
    compile_schematic, ComponentCategory, ComponentKind, SchematicComponent,
};
use phonon_gui::ComponentPalette;
use std::collections::HashSet;
use std::time::Instant;

#[test]
fn test_all_categories_coverage() {
    let categories = ComponentCategory::all_categories();
    assert_eq!(
        categories.len(),
        7,
        "Hierarchical taxonomy must define exactly 7 distinct categories"
    );

    let mut seen = HashSet::new();
    for &cat in categories {
        assert!(
            seen.insert(cat),
            "Duplicate category detected in all_categories: {:?}",
            cat
        );

        let name = cat.display_name();
        assert!(
            !name.trim().is_empty(),
            "Category {:?} display_name must not be empty",
            cat
        );

        let desc = cat.description();
        assert!(
            !desc.trim().is_empty(),
            "Category {:?} description must not be empty",
            cat
        );

        let comps = cat.components();
        assert!(
            !comps.is_empty(),
            "Category {:?} must provide a non-empty list of components",
            cat
        );
    }
}

#[test]
fn test_component_category_mapping_bijection() {
    let categories = ComponentCategory::all_categories();
    let mut all_comps = Vec::new();
    let mut seen_comps = HashSet::new();

    for &cat in categories {
        let comps = cat.components();
        for comp in comps {
            assert_eq!(
                comp.category(),
                cat,
                "Component {:?} must map back to designated category {:?}",
                comp,
                cat
            );

            assert!(
                seen_comps.insert(comp),
                "Component {:?} appears in multiple categories",
                comp
            );
            all_comps.push(comp);
        }
    }

    assert_eq!(
        all_comps.len(),
        31,
        "Total categorized primitives must equal 31"
    );
}

#[test]
fn test_pin_definitions_validity_all_components() {
    for &cat in ComponentCategory::all_categories() {
        for comp in cat.components() {
            let pins = comp.pin_definitions();
            assert!(
                !pins.is_empty(),
                "Component {:?} must define at least 1 pin",
                comp
            );

            let mut pin_names = HashSet::new();
            for (pin_name, offset) in pins {
                assert!(
                    !pin_name.trim().is_empty(),
                    "Component {:?} has empty pin name",
                    comp
                );
                assert!(
                    offset.x.is_finite(),
                    "Component {:?} pin '{}' has non-finite x coordinate",
                    comp,
                    pin_name
                );
                assert!(
                    offset.y.is_finite(),
                    "Component {:?} pin '{}' has non-finite y coordinate",
                    comp,
                    pin_name
                );

                assert!(
                    pin_names.insert(pin_name),
                    "Component {:?} defines duplicate pin name '{}'",
                    comp,
                    pin_name
                );
            }
        }
    }
}

#[test]
fn test_palette_live_search_filtering() {
    let palette = ComponentPalette::new();

    // Empty query returns empty vec
    let empty_res = palette.filter_components("");
    assert!(empty_res.is_empty());
    let ws_res = palette.filter_components("   ");
    assert!(ws_res.is_empty());

    // Query "diode"
    let diode_matches = palette.filter_components("diode");
    assert!(
        diode_matches.contains(&ComponentKind::Diode),
        "Query 'diode' must match standard Diode"
    );
    assert!(
        diode_matches.contains(&ComponentKind::ZenerDiode),
        "Query 'diode' must match ZenerDiode"
    );
    assert!(
        diode_matches.contains(&ComponentKind::SchottkyDiode),
        "Query 'diode' must match SchottkyDiode"
    );

    // Case insensitivity: "DIODE" == "diode"
    let upper_diode = palette.filter_components("DIODE");
    assert_eq!(diode_matches, upper_diode);

    // Query "op"
    let op_matches = palette.filter_components("op");
    assert!(
        op_matches.contains(&ComponentKind::OpAmp),
        "Query 'op' must match OpAmp"
    );

    // Query "transistor"
    let transistor_matches = palette.filter_components("transistor");
    assert!(
        transistor_matches.contains(&ComponentKind::Nmos),
        "Query 'transistor' must match NMOS"
    );
    assert!(
        transistor_matches.contains(&ComponentKind::Pmos),
        "Query 'transistor' must match PMOS"
    );
    assert!(
        transistor_matches.contains(&ComponentKind::BjtNpn),
        "Query 'transistor' must match NPN BJT"
    );
    assert!(
        transistor_matches.contains(&ComponentKind::BjtPnp),
        "Query 'transistor' must match PNP BJT"
    );

    // Query "gate"
    let gate_matches = palette.filter_components("gate");
    assert!(
        gate_matches.contains(&ComponentKind::NandGate),
        "Query 'gate' must match NAND Gate"
    );
    assert!(
        gate_matches.contains(&ComponentKind::NorGate),
        "Query 'gate' must match NOR Gate"
    );

    // Query "saw"
    let saw_matches = palette.filter_components("saw");
    assert!(
        saw_matches.contains(&ComponentKind::SawIdt),
        "Query 'saw' must match SAW IDT Filter"
    );
}

#[test]
fn test_new_components_circuit_compilation() {
    let mut components = Vec::new();
    let mut id = 1;

    // Instantiate representative primitives across all 7 categories
    components.push(SchematicComponent::new(
        id,
        ComponentKind::AcVoltageSource,
        Pos2::new(100.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::Transformer,
        Pos2::new(200.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::ZenerDiode,
        Pos2::new(300.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::Led,
        Pos2::new(350.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::SchottkyDiode,
        Pos2::new(400.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::FinFet,
        Pos2::new(450.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::GaaNanosheet,
        Pos2::new(500.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::OpAmp,
        Pos2::new(550.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::Inverter,
        Pos2::new(600.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::NandGate,
        Pos2::new(650.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::NorGate,
        Pos2::new(700.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::Mux2to1,
        Pos2::new(750.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::StrainGauge,
        Pos2::new(800.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::TactileMatrix,
        Pos2::new(850.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::Imu9Dof,
        Pos2::new(900.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::SawIdt,
        Pos2::new(950.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::MajoranaJunction,
        Pos2::new(1000.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::ParafermionicCavity,
        Pos2::new(1050.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::SkyrmionRouter,
        Pos2::new(1100.0, 200.0),
        1,
    ));
    id += 1;

    components.push(SchematicComponent::new(
        id,
        ComponentKind::Ground,
        Pos2::new(100.0, 300.0),
        1,
    ));

    let wires = vec![];
    let result = compile_schematic(&components, &wires);
    assert!(result.is_ok(), "Schematic compilation must succeed: {:?}", result.err());

    let compiled = result.unwrap();
    assert!(compiled.spice_netlist.contains(".SUBCKT FINFET_3NM"));
    assert!(compiled.spice_netlist.contains(".SUBCKT GAA_2NM"));
    assert!(compiled.spice_netlist.contains(".SUBCKT OPAMP_IDEAL"));
    assert!(compiled.spice_netlist.contains(".SUBCKT INV_CMOS"));
    assert!(compiled.spice_netlist.contains(".SUBCKT NAND2"));
    assert!(compiled.spice_netlist.contains(".SUBCKT NOR2"));
    assert!(compiled.spice_netlist.contains(".SUBCKT MUX21"));
    assert!(compiled.spice_netlist.contains(".SUBCKT TACTILE_8X8"));
    assert!(compiled.spice_netlist.contains(".SUBCKT IMU_6DOF_9DOF"));
    assert!(compiled.spice_netlist.contains(".SUBCKT SAW_1GHZ"));
    assert!(compiled.spice_netlist.contains(".SUBCKT TOPOMAJ_1"));
    assert!(compiled.spice_netlist.contains(".SUBCKT PARAFERM_RES"));
    assert!(compiled.spice_netlist.contains(".SUBCKT SKYRMION_RT"));
    assert!(compiled.spice_netlist.contains(".OP"));
    assert!(compiled.spice_netlist.contains(".END"));
    assert!(!compiled.net_names.is_empty());
}

#[test]
fn test_palette_rendering_throughput() {
    let palette = ComponentPalette::new();
    let queries = [
        "diode",
        "op",
        "transistor",
        "gate",
        "saw",
        "resistor",
        "source",
        "fet",
        "sensor",
        "cavity",
    ];

    let iterations = 10_000;
    let start = Instant::now();
    let mut total_matches = 0;

    for i in 0..iterations {
        let q = queries[i % queries.len()];
        let results = palette.filter_components(q);
        total_matches += results.len();
    }

    let elapsed = start.elapsed();
    let ops_per_sec = (iterations as f64) / elapsed.as_secs_f64();

    println!(
        "Palette filter throughput: {:.2} ops/sec ({} iterations in {:?}, total matches = {})",
        ops_per_sec, iterations, elapsed, total_matches
    );

    assert!(total_matches > 0);
    assert!(
        elapsed.as_millis() < 15,
        "10,000 palette filter operations took {:?} (exceeding 15ms target)",
        elapsed
    );
    assert!(
        ops_per_sec > 650_000.0,
        "Throughput {:.2} ops/sec below 650,000 ops/sec threshold",
        ops_per_sec
    );
}
