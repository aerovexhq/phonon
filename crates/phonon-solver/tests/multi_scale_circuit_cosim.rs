//! Integration Test: Multi-Scale Circuit Co-Simulation.
//!
//! Validates:
//! - Seamless co-existence of microscopic structural TCAD devices and macro-scale analytical compact components
//! - Simultaneous monolithic MNA stamping of both first-principles TCAD devices and BSIM/Shockley models
//! - Robust non-linear Newton-Raphson convergence through coupled multi-scale physics
//! - Kirchhoff's Current Law (KCL) conservation across all multi-scale circuit nodes.

use phonon_core::constants::T_REF;
use phonon_core::CircuitGraph;
use phonon_models::tcad::TcadDeviceBuilder;
use phonon_models::{DiodeModel, MosfetModel};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::verification::verify_kcl_dynamic;
use std::collections::HashMap;

#[test]
fn test_multi_scale_dual_tier_diode_circuit() {
    let mut g = CircuitGraph::new();

    // DC Voltage source: 5.0 V
    g.add_voltage_source("V_in", "in", "0", 5.0).unwrap();
    // Current limiting resistor
    g.add_resistor("R_limit", "in", "mid", 5000.0).unwrap();
    // Low-level structural TCAD physical diode: mid -> out
    g.add_tcad_diode("D_tcad", "mid", "out").unwrap();
    // High-level analytical compact diode: out -> 0
    g.add_diode("D_compact", "out", "0").unwrap();

    let mut ctx = ModelContext::new();
    ctx.temperature_kelvin = T_REF;

    // Register low-level TCAD device
    let tcad_diode = TcadDeviceBuilder::new_pn_junction("D_tcad")
        .length(2.0e-6)
        .cross_section_area(1.0e-7)
        .build();
    ctx.set_tcad_device("D_tcad", tcad_diode);

    // Register high-level analytical compact diode
    ctx.set_diode_model(
        "D_compact",
        DiodeModel {
            is: 1.0e-14,
            n: 1.0,
            ..DiodeModel::default()
        },
    );

    // Solve unified multi-scale DC operating point
    let newton_opts = NewtonOptions {
        max_iters: 60,
        reltol: 1e-3,
        vntol: 1e-5,
        abstol: 1e-9,
        ..NewtonOptions::default()
    };

    let sol = solve_dc_non_linear(&g, &ctx, &newton_opts)
        .expect("Multi-scale non-linear DC solve failed");

    let n_in = g.get_node("in").unwrap();
    let n_mid = g.get_node("mid").unwrap();
    let n_out = g.get_node("out").unwrap();

    let v_in = sol.node_voltage(n_in);
    let v_mid = sol.node_voltage(n_mid);
    let v_out = sol.node_voltage(n_out);

    println!(
        "Multi-Scale Diode String:\n  V(in)  = {:.4} V\n  V(mid) = {:.4} V\n  V(out) = {:.4} V",
        v_in, v_mid, v_out
    );

    assert!((v_in - 5.0).abs() < 1e-6, "Input voltage must equal 5.0V");
    assert!(
        v_mid < v_in,
        "V(mid) must be less than V(in) due to resistor drop"
    );
    assert!(
        v_out < v_mid,
        "V(out) must be less than V(mid) due to TCAD diode drop"
    );
    assert!(v_out > 0.0, "V(out) must be positive above ground");

    // Verify physical KCL conservation
    let kcl = verify_kcl_dynamic(
        &g,
        &sol.node_voltages,
        &sol.branch_currents,
        &HashMap::new(),
        Some(&ctx),
        1e-2,
        1e-5,
    );
    println!(
        "KCL max residual across all multi-scale nodes: {:.3e} A (valid: {})",
        kcl.max_residual, kcl.is_valid
    );
    assert!(
        kcl.is_valid,
        "Kirchhoff's Current Law must be conserved at all nodes"
    );
}

#[test]
fn test_multi_scale_transistor_inverter_buffer() {
    let mut g = CircuitGraph::new();

    // 1.8 V supply rail
    g.add_voltage_source("V_dd", "vdd", "0", 1.8).unwrap();
    // Input control signal: 1.5 V (High logic)
    g.add_voltage_source("V_in", "vin", "0", 1.5).unwrap();

    // Stage 1: Low-level TCAD NMOS inverter
    // Pull-up resistor: vdd -> v_out1
    g.add_resistor("R_pullup", "vdd", "out1", 10_000.0).unwrap();
    // TCAD physical NMOS: drain=out1, gate=vin, source=0, bulk=0
    g.add_tcad_mosfet("M_tcad", "out1", "vin", "0", "0")
        .unwrap();

    // Stage 2: High-level analytical compact NMOS inverter
    // Pull-up resistor: vdd -> v_out2
    g.add_resistor("R_pullup2", "vdd", "out2", 10_000.0)
        .unwrap();
    // Analytical compact NMOS: drain=out2, gate=out1, source=0, bulk=0
    g.add_mosfet("M_compact", "out2", "out1", "0", "0").unwrap();

    let mut ctx = ModelContext::new();
    ctx.temperature_kelvin = T_REF;

    // Register physical TCAD NMOS
    let tcad_nmos = TcadDeviceBuilder::new_mosfet("M_tcad")
        .length(180.0e-9)
        .cross_section_area(2.0e-12)
        .oxide_thickness(3.5e-9)
        .build();
    ctx.set_tcad_device("M_tcad", tcad_nmos);

    // Register analytical compact NMOS
    ctx.set_mosfet_model(
        "M_compact",
        MosfetModel {
            vth0: 0.45,
            w: 5.0e-6,
            l: 0.18e-6,
            ..MosfetModel::default()
        },
    );

    let newton_opts = NewtonOptions {
        max_iters: 60,
        reltol: 1e-3,
        vntol: 1e-5,
        abstol: 1e-9,
        ..NewtonOptions::default()
    };

    let sol = solve_dc_non_linear(&g, &ctx, &newton_opts)
        .expect("Multi-scale inverter buffer solve failed");

    let n_vdd = g.get_node("vdd").unwrap();
    let n_vin = g.get_node("vin").unwrap();
    let n_out1 = g.get_node("out1").unwrap();
    let n_out2 = g.get_node("out2").unwrap();

    let v_vdd = sol.node_voltage(n_vdd);
    let v_vin = sol.node_voltage(n_vin);
    let v_out1 = sol.node_voltage(n_out1);
    let v_out2 = sol.node_voltage(n_out2);

    println!(
        "Multi-Scale Inverter Buffer:\n  V(vdd)  = {:.4} V\n  V(vin)  = {:.4} V\n  V(out1) = {:.4} V (TCAD Stage)\n  V(out2) = {:.4} V (Compact Stage)",
        v_vdd, v_vin, v_out1, v_out2
    );

    assert!((v_vdd - 1.8).abs() < 1e-6);
    assert!((v_vin - 1.5).abs() < 1e-6);
    assert!((0.0..=1.8).contains(&v_out1));
    assert!((0.0..=1.8).contains(&v_out2));

    // Verify physical KCL conservation
    let kcl = verify_kcl_dynamic(
        &g,
        &sol.node_voltages,
        &sol.branch_currents,
        &HashMap::new(),
        Some(&ctx),
        1e-2,
        1e-5,
    );
    println!(
        "KCL max residual across all multi-scale nodes: {:.3e} A (valid: {})",
        kcl.max_residual, kcl.is_valid
    );
    assert!(
        kcl.is_valid,
        "Kirchhoff's Current Law must be conserved at all nodes"
    );
}
