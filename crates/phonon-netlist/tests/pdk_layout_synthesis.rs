//! Open-Source PDK profiles (SkyWater 130nm, GF180MCU) and LEF layout geometry synthesis tests.

use phonon_core::CircuitGraph;
use phonon_netlist::geometry::{LefLibrary, PinDirection, TransistorLayout};
use phonon_netlist::pdk::{Gf180McuPdk, Sky130Pdk};
use phonon_solver::mna::{solve_dc_linear, SolverOptions};

#[test]
fn test_sky130_and_gf180_device_generation() {
    // SkyWater 130nm models
    let nfet_130 = Sky130Pdk::nfet_01v8(1.0e-6, 0.15e-6);
    assert_eq!(nfet_130.w, 1.0e-6);
    assert_eq!(nfet_130.l, 0.15e-6);
    assert_eq!(nfet_130.vth0, 0.45);
    assert_eq!(nfet_130.tox, 4.13e-9);

    let pfet_130 = Sky130Pdk::pfet_01v8(2.0e-6, 0.15e-6);
    assert_eq!(pfet_130.vth0, -0.55);

    let nfet_io = Sky130Pdk::nfet_g5v0(2.5e-6, 0.5e-6);
    assert_eq!(nfet_io.vth0, 0.70);
    assert_eq!(nfet_io.tox, 12.5e-9);

    // GF180MCU models
    let nfet_180 = Gf180McuPdk::nfet_03v3(1.2e-6, 0.28e-6);
    assert_eq!(nfet_180.vth0, 0.60);
    assert_eq!(nfet_180.tox, 6.8e-9);

    let pfet_180 = Gf180McuPdk::pfet_03v3(2.4e-6, 0.28e-6);
    assert_eq!(pfet_180.vth0, -0.65);
}

#[test]
fn test_lef_parser_macro_extraction() {
    let mock_lef = r#"
VERSION 5.8 ;
MACRO inv_1x
  CLASS CORE ;
  ORIGIN 0 0 ;
  SIZE 1.38 BY 2.72 ;
  PIN A
    DIRECTION INPUT ;
    USE SIGNAL ;
    PORT
      LAYER met1 ;
      RECT 0.2 0.5 0.4 1.2 ;
    END
  END A
  PIN Y
    DIRECTION OUTPUT ;
    USE SIGNAL ;
    PORT
      LAYER met1 ;
      RECT 0.8 0.6 1.0 1.5 ;
    END
  END Y
  PIN VPWR
    DIRECTION INOUT ;
    USE POWER ;
    PORT
      LAYER met1 ;
      RECT 0 2.5 1.38 2.72 ;
    END
  END VPWR
  PIN VGND
    DIRECTION INOUT ;
    USE GROUND ;
    PORT
      LAYER met1 ;
      RECT 0 0 1.38 0.22 ;
    END
  END VGND
END inv_1x
"#;

    let library = LefLibrary::parse_from_str(mock_lef).expect("LEF parsing must succeed");

    assert_eq!(library.macros.len(), 1);
    let inv = &library.macros[0];
    assert_eq!(inv.name, "inv_1x");
    assert!((inv.width - 1.38).abs() < 1e-4);
    assert!((inv.height - 2.72).abs() < 1e-4);
    assert_eq!(inv.pins.len(), 4);

    let pin_a = inv.pins.iter().find(|p| p.name == "A").unwrap();
    assert_eq!(pin_a.direction, PinDirection::Input);
    assert_eq!(pin_a.rects.len(), 1);
}

#[test]
fn test_transistor_layout_parasitic_synthesis_into_circuit_graph() {
    let layout = TransistorLayout::new("MN1", 2.0e-6, 0.15e-6);

    let c_drain = layout.drain_capacitance();
    let c_source = layout.source_capacitance();
    assert!(
        c_drain > 0.0,
        "Drain parasitic capacitance must be positive"
    );
    assert!(
        c_source > 0.0,
        "Source parasitic capacitance must be positive"
    );

    let r_diff = layout.diffusion_series_resistance();
    assert!(r_diff > 0.0, "Diffusion series resistance must be positive");

    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("VDD", "vdd", "0", 1.8).unwrap();
    graph.add_resistor("RLOAD", "vdd", "out", 5000.0).unwrap();

    // Synthesize transistor and diffusion parasitics
    layout
        .synthesize_into_graph(&mut graph, "out", "in", "0", "0")
        .expect("Synthesis into graph should succeed");

    // Check that parasitic capacitors were added
    assert!(graph.get_component("MN1_Cdiff_d").is_some());
    assert!(graph.get_component("MN1_Cdiff_s").is_some());

    // Validate topology
    graph.validate_topology().expect("Topology must be valid");

    // Ensure linear MNA assembler recognizes the circuit correctly
    let solver_opts = SolverOptions::default();
    let linear_result = solve_dc_linear(&graph, &solver_opts);
    // Should return error because MN1 is non-linear MOSFET
    assert!(linear_result.is_err());
}
