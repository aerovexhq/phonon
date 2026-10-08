#![deny(unsafe_code)]

//! Comprehensive Automated Verification Suite for Physical Board Card Synthesis,
//! Technology Mapping, De-clumping Auto-Placement, Trace Routing, and 3D Viewport Tabs.

use egui::Pos2;
use phonon_gui::board_synthesis::{
    AutoPlacementParams, AutoPlacer, BoardAutoRouter, BoardSynthesisEngine, CopperLayer,
    FootprintInstance, PackageType, PhysicalNetConnection, PhysicalParasiticsEngine,
    RoutedPhysicalNet, SolderMaskColor, SynthesisError, SynthesizedPhysicalCard,
    TechMappingOptions,
};
use phonon_gui::schematic::components::{ComponentKind, SchematicComponent};
use phonon_gui::viewport_tabs::{CentralTabManager, CentralViewportTab};

#[test]
fn test_physical_footprint_pad_generation() {
    let dip14 = PackageType::Dip14;
    let pads14 = dip14.generate_pads();
    assert_eq!(pads14.len(), 14, "DIP-14 must generate exactly 14 pads");
    assert!(pads14.iter().all(|p| p.is_through_hole));
    assert_eq!(pads14[0].pin_number, 1);
    assert_eq!(pads14[13].pin_number, 14);

    let dip8 = PackageType::Dip8;
    let pads8 = dip8.generate_pads();
    assert_eq!(pads8.len(), 8, "DIP-8 must generate exactly 8 pads");

    let smd = PackageType::Smd0805;
    let smd_pads = smd.generate_pads();
    assert_eq!(smd_pads.len(), 2, "SMD-0805 must generate exactly 2 pads");
    assert!(!smd_pads[0].is_through_hole);

    let to92 = PackageType::To92;
    let to92_pads = to92.generate_pads();
    assert_eq!(to92_pads.len(), 3, "TO-92 must generate exactly 3 pads");
}

#[test]
fn test_synthesis_failure_when_abstract_gate_present_without_explosion() {
    // Schematic containing an abstract XOR gate and resistor
    let xor_gate = SchematicComponent::new(1, ComponentKind::XorGate, Pos2::new(100.0, 100.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(200.0, 100.0), 1);
    let components = vec![xor_gate, r1];
    let wires = vec![];

    // Synthesis with auto_explode_gates = false MUST fail
    let options = TechMappingOptions {
        auto_explode_gates: false,
        force_universal_nand_explosion: false,
        component_clearance_mm: 2.54,
    };
    let placement = AutoPlacementParams::default();

    let result = BoardSynthesisEngine::synthesize_card(&components, &wires, &options, &placement);
    assert!(
        result.is_err(),
        "Synthesis must fail if abstract gate is present without explosion enabled"
    );

    match result.err().unwrap() {
        SynthesisError::AbstractGateNotExploded { designator, kind } => {
            assert_eq!(designator, "UXOR1");
            assert_eq!(kind, ComponentKind::XorGate);
        }
        other => panic!("Unexpected synthesis error: {:?}", other),
    }
}

#[test]
fn test_successful_technology_mapping_explosion_for_xor_and_half_adder() {
    let xor_gate = SchematicComponent::new(1, ComponentKind::XorGate, Pos2::new(100.0, 100.0), 1);
    let half_adder = SchematicComponent::new(2, ComponentKind::HalfAdder, Pos2::new(200.0, 100.0), 1);
    let r1 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(300.0, 100.0), 1);
    let components = vec![xor_gate, half_adder, r1];
    let wires = vec![];

    // Enable technology mapping explosion
    let options = TechMappingOptions {
        auto_explode_gates: true,
        force_universal_nand_explosion: false,
        component_clearance_mm: 2.54,
    };
    let placement = AutoPlacementParams::default();

    let result = BoardSynthesisEngine::synthesize_card(&components, &wires, &options, &placement);
    assert!(result.is_ok(), "Synthesis must succeed with technology mapping enabled");

    let card = result.unwrap();
    // 1 chip from XOR (74HC86), 2 chips from HalfAdder (74HC86 + 74HC08), 1 chip from R1 (SMD0805) = 4 chips
    assert_eq!(card.chips.len(), 4, "Expected 4 physical chips on synthesized card");

    // Verify original schematic was NOT modified
    assert_eq!(components.len(), 3, "Original schematic components must remain intact");
    assert_eq!(components[0].kind, ComponentKind::XorGate);
    assert_eq!(components[1].kind, ComponentKind::HalfAdder);
}

#[test]
fn test_declump_auto_placement_resolves_overlapping_chips() {
    // Create 4 DIP-14 chips placed at the EXACT same location (coincident clump)
    let c1 = FootprintInstance::new(1, "U1", "74HC00", PackageType::Dip14, Pos2::new(20.0, 20.0));
    let c2 = FootprintInstance::new(2, "U2", "74HC86", PackageType::Dip14, Pos2::new(20.0, 20.0));
    let c3 = FootprintInstance::new(3, "U3", "74HC08", PackageType::Dip14, Pos2::new(20.0, 20.0));
    let c4 = FootprintInstance::new(4, "U4", "74HC32", PackageType::Dip14, Pos2::new(20.0, 20.0));
    let clumped = vec![c1, c2, c3, c4];

    // Verify initial collisions exist
    let initial_collisions = AutoPlacer::count_collisions(&clumped, 2.54);
    assert!(initial_collisions > 0, "Initial clumped chips must have collisions");

    // Run de-clumping auto-placement
    let params = AutoPlacementParams::default();
    let placed = AutoPlacer::declump_and_place(clumped, &params);

    // Verify final collisions are completely resolved
    let final_collisions = AutoPlacer::count_collisions(&placed.chips, params.package_clearance_mm);
    assert_eq!(
        final_collisions, 0,
        "De-clumping algorithm must resolve all package bounding box overlaps"
    );

    // Verify all chips are snapped to 2.54mm grid multiples
    for chip in &placed.chips {
        let rem_x = chip.center_mm.x % params.grid_pitch_mm;
        let rem_y = chip.center_mm.y % params.grid_pitch_mm;
        assert!(
            rem_x.min(params.grid_pitch_mm - rem_x) < 1e-3,
            "Chip X coordinate must be on 2.54mm grid"
        );
        assert!(
            rem_y.min(params.grid_pitch_mm - rem_y) < 1e-3,
            "Chip Y coordinate must be on 2.54mm grid"
        );
    }
}

#[test]
fn test_copper_trace_router_generates_manhattan_routes() {
    let chip1 = FootprintInstance::new(1, "U1", "74HC00", PackageType::Dip14, Pos2::new(20.0, 20.0));
    let chip2 = FootprintInstance::new(2, "U2", "74HC86", PackageType::Dip14, Pos2::new(50.0, 20.0));
    let chips = vec![chip1, chip2];

    let conns = vec![
        PhysicalNetConnection {
            net_name: "NET_CLOCK".to_string(),
            chip_id: 1,
            pin_number: 3,
            is_power_or_gnd: false,
        },
        PhysicalNetConnection {
            net_name: "NET_CLOCK".to_string(),
            chip_id: 2,
            pin_number: 1,
            is_power_or_gnd: false,
        },
    ];

    let routes = BoardAutoRouter::route_nets(&chips, &conns);
    assert_eq!(routes.len(), 1, "Must route 1 physical net");
    let net = &routes[0];
    assert_eq!(net.net_name, "NET_CLOCK");
    assert!(!net.segments.is_empty(), "Must produce copper segments");
    assert!(net.total_length_mm > 0.0, "Route must have positive length");
}

#[test]
fn test_physical_parasitics_and_emi_disturbance_extraction() {
    // Create two parallel trace nets separated by 1.5mm running horizontally for 20mm
    let net1 = RoutedPhysicalNet {
        net_name: "NET_AGGRESSOR".to_string(),
        segments: vec![phonon_gui::board_synthesis::PhysicalTraceSegment {
            start_mm: Pos2::new(10.0, 10.0),
            end_mm: Pos2::new(30.0, 10.0),
            layer: CopperLayer::Top,
            width_mm: 0.25,
        }],
        is_power: false,
        total_length_mm: 20.0,
    };
    let net2 = RoutedPhysicalNet {
        net_name: "NET_VICTIM".to_string(),
        segments: vec![phonon_gui::board_synthesis::PhysicalTraceSegment {
            start_mm: Pos2::new(10.0, 11.5),
            end_mm: Pos2::new(30.0, 11.5),
            layer: CopperLayer::Top,
            width_mm: 0.25,
        }],
        is_power: false,
        total_length_mm: 20.0,
    };

    let nets = vec![net1, net2];
    let report = PhysicalParasiticsEngine::evaluate_board_realism(&nets);

    // Verify parasitic R, L, C extracted
    assert_eq!(report.net_parasitics.len(), 2);
    for p in &report.net_parasitics {
        assert!(p.resistance_ohms > 0.0, "Trace resistance must be positive");
        assert!(p.self_inductance_nh > 0.0, "Trace inductance must be positive");
        assert!(p.capacitance_to_gnd_pf > 0.0, "Trace capacitance must be positive");
        assert!(p.characteristic_impedance_ohms > 10.0);
    }

    // Verify electromagnetic crosstalk coupling detected
    assert!(!report.emi_disturbances.is_empty(), "Parallel traces must couple EMI disturbance");
    let emi = &report.emi_disturbances[0];
    assert_eq!(emi.aggressor_net, "NET_AGGRESSOR");
    assert_eq!(emi.victim_net, "NET_VICTIM");
    assert!(emi.mutual_inductance_nh > 0.0);
    assert!(emi.mutual_capacitance_pf > 0.0);
    assert!(emi.induced_crosstalk_noise_mv > 0.0);
    assert!(report.estimated_ground_bounce_mv > 0.0);
}

#[test]
fn test_3d_card_model_faces_and_geometry() {
    let chip = FootprintInstance::new(1, "U1", "74HC00", PackageType::Dip14, Pos2::new(25.0, 20.0));
    let card = SynthesizedPhysicalCard::new(
        60.0,
        50.0,
        vec![chip],
        vec![],
        phonon_gui::board_synthesis::PhysicalWiringRealismReport {
            net_parasitics: vec![],
            emi_disturbances: vec![],
            max_crosstalk_noise_mv: 0.0,
            estimated_ground_bounce_mv: 15.0,
        },
    );

    let faces = card.generate_3d_faces();
    assert!(
        faces.len() >= 6,
        "Card must generate at least 6 faces (top, bottom, 4 edges)"
    );

    // Test solder mask color options
    assert_ne!(
        SolderMaskColor::ObsidianMatteBlack.substrate_color(),
        SolderMaskColor::EmeraldGreen.substrate_color()
    );
}

#[test]
fn test_central_viewport_tab_manager() {
    let mut tab_mgr = CentralTabManager::new();
    assert_eq!(tab_mgr.active_tab, CentralViewportTab::Schematic);

    // Switch to 3D Physical Card tab
    tab_mgr.set_active_tab(CentralViewportTab::PhysicalCard3D);
    assert_eq!(tab_mgr.active_tab, CentralViewportTab::PhysicalCard3D);

    // Register a custom viewport tab
    tab_mgr.register_custom_tab("thermal_3d", "3D Thermal Floorplan", "[T]");
    assert_eq!(tab_mgr.custom_tabs.len(), 1);
    assert_eq!(tab_mgr.custom_tabs[0].id, "thermal_3d");
}
