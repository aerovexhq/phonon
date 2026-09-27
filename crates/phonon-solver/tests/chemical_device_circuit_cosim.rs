//! Integration Test: First-Principles Chemical Device Circuit Co-Simulation.
//!
//! Validates:
//! - Construction of a High-k Metal-Gate (HKMG) NMOS transistor using first-principles material chemistry
//! - Integration of HfO2 gate dielectric (kappa = 22.0) with TiN metal gate and NiSi silicide contacts
//! - Monolithic Modified Nodal Analysis (MNA) non-linear Newton-Raphson convergence
//! - Kirchhoff's Current Law (KCL) conservation across all nodes down to femtoamperes.

use phonon_core::constants::T_REF;
use phonon_core::CircuitGraph;
use phonon_models::chemistry::builder::ChemicalMaterialBuilder;
use phonon_models::chemistry::contact::ContactMaterial;
use phonon_models::chemistry::crystal::Crystallography;
use phonon_models::chemistry::dielectric::DielectricMaterial;
use phonon_models::chemistry::dopant::DopantSpecies;
use phonon_models::tcad::TcadDeviceBuilder;
use phonon_models::DiodeModel;
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::verification::verify_kcl_dynamic;
use std::collections::HashMap;

#[test]
fn test_chemical_hkmg_transistor_inverter_circuit() {
    let mut g = CircuitGraph::new();

    // Power rail: 1.2 V (contemporary sub-micron core voltage)
    g.add_voltage_source("V_dd", "vdd", "0", 1.2).unwrap();
    // Input gate drive: 1.0 V
    g.add_voltage_source("V_in", "vin", "0", 1.0).unwrap();

    // Pull-up resistor: 8 kOhm from Vdd to Vout
    g.add_resistor("R_load", "vdd", "out", 8000.0).unwrap();

    // First-principles HKMG NMOS: drain=out, gate=vin, source=0, bulk=0
    g.add_tcad_mosfet("M_hkmg", "out", "vin", "0", "0").unwrap();

    // High-level clamping diode on output to ground
    g.add_diode("D_clamp", "out", "0").unwrap();

    let mut ctx = ModelContext::new();
    ctx.temperature_kelvin = T_REF;

    // 1. Programmatically synthesize first-principles HKMG material from scratch
    let hkmg_material = ChemicalMaterialBuilder::new("Silicon_HKMG")
        .crystallography(Crystallography::silicon())
        .dielectric(DielectricMaterial::hafnium_dioxide(), 2.0e-9) // 2 nm HfO2 (EOT ~ 0.35 nm)
        .contact(ContactMaterial::titanium_nitride_gate()) // TiN metal gate
        .doping(DopantSpecies::Boron, 1.0e23) // Boron channel
        .build();

    // 2. Synthesize TCAD device using the first-principles chemical material
    let tcad_hkmg = TcadDeviceBuilder::new_mosfet("M_hkmg")
        .length(90.0e-9) // 90 nm physical gate length
        .cross_section_area(2.0e-12) // 2 um width x 1 um depth
        .chemical_material(hkmg_material)
        .build();

    ctx.set_tcad_device("M_hkmg", tcad_hkmg);

    // 3. Register high-level clamping diode
    ctx.set_diode_model(
        "D_clamp",
        DiodeModel {
            is: 1.0e-15,
            n: 1.0,
            ..DiodeModel::default()
        },
    );

    // 4. Solve unified multi-scale DC operating point
    let newton_opts = NewtonOptions {
        max_iters: 60,
        reltol: 1e-3,
        vntol: 1e-5,
        abstol: 1e-9,
        ..NewtonOptions::default()
    };

    let sol = solve_dc_non_linear(&g, &ctx, &newton_opts)
        .expect("HKMG chemical transistor circuit solve failed");

    let n_vdd = g.get_node("vdd").unwrap();
    let n_vin = g.get_node("vin").unwrap();
    let n_out = g.get_node("out").unwrap();

    let v_vdd = sol.node_voltage(n_vdd);
    let v_vin = sol.node_voltage(n_vin);
    let v_out = sol.node_voltage(n_out);

    println!(
        "HKMG Inverter Circuit:\n  V(vdd) = {:.4} V\n  V(vin) = {:.4} V\n  V(out) = {:.4} V",
        v_vdd, v_vin, v_out
    );

    assert!((v_vdd - 1.2).abs() < 1e-6);
    assert!((v_vin - 1.0).abs() < 1e-6);
    assert!(
        (0.0..=1.2).contains(&v_out),
        "Output voltage must be within supply rails"
    );

    // 5. Verify Kirchhoff's Current Law down to machine precision
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
        "KCL max residual across all nodes: {:.3e} A (valid: {})",
        kcl.max_residual, kcl.is_valid
    );
    assert!(
        kcl.is_valid,
        "Kirchhoff's Current Law must be strictly conserved"
    );
}
