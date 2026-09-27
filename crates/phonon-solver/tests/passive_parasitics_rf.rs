//! Integration test: Real component parasitics and high-frequency RF modeling.
//! Validates:
//! 1. Real capacitor with ESR, ESL, and Self-Resonant Frequency (SRF) impedance dip.
//! 2. Real inductor with magnetic core saturation $L(I)$ and parallel winding capacitance $C_p$.

use phonon_core::CircuitGraph;
use phonon_models::parasitics::{RealCapacitorModel, RealInductorModel};
use phonon_solver::mna::{solve_dc_linear, SolverOptions};

#[test]
fn test_real_capacitor_srf_and_impedance() {
    let c_val = 100e-9; // 100 nF
    let esr = 0.05; // 50 mOhm
    let esl = 1.5e-9; // 1.5 nH

    let cap_model = RealCapacitorModel::new(c_val, esr, esl);

    // Analytical SRF: f_srf = 1 / (2*pi*sqrt(C*ESL))
    let expected_srf = 1.0 / (2.0 * std::f64::consts::PI * (c_val * esl).sqrt());
    let computed_srf = cap_model.self_resonant_frequency();

    assert!(
        (computed_srf - expected_srf).abs() / expected_srf < 1e-6,
        "Computed SRF {computed_srf} Hz did not match analytical {expected_srf} Hz"
    );
    // Should be approximately 12.99 MHz
    assert!(computed_srf > 12.5e6 && computed_srf < 13.5e6);

    // At SRF: reactances cancel -> |Z| = ESR exactly
    let z_srf = cap_model.impedance_magnitude(computed_srf);
    assert!(
        (z_srf - esr).abs() < 1e-6,
        "Expected |Z(SRF)| == ESR ({esr}), got {z_srf}"
    );

    // Below SRF (1 MHz): capacitive regime dominates
    let z_1mhz = cap_model.impedance_magnitude(1.0e6);
    let expected_zc_1mhz = 1.0 / (2.0 * std::f64::consts::PI * 1e6 * c_val);
    assert!(
        (z_1mhz - expected_zc_1mhz).abs() / expected_zc_1mhz < 0.01,
        "Expected capacitive impedance at 1MHz, got {z_1mhz} vs {expected_zc_1mhz}"
    );

    // Above SRF (100 MHz): inductive ESL dominates
    let z_100mhz = cap_model.impedance_magnitude(100.0e6);
    let expected_zl_100mhz = 2.0 * std::f64::consts::PI * 100e6 * esl;
    assert!(
        (z_100mhz - expected_zl_100mhz).abs() / expected_zl_100mhz < 0.02,
        "Expected inductive impedance at 100MHz, got {z_100mhz} vs {expected_zl_100mhz}"
    );

    // Synthesize into a circuit graph and check DC operating point
    let mut graph = CircuitGraph::new();
    cap_model
        .synthesize_subcircuit(&mut graph, "C_REAL", "in", "0")
        .expect("Synthesis failed");

    // Add 10V DC source through 1k resistor to charge the capacitor
    graph.add_voltage_source("V1", "src", "0", 10.0).unwrap();
    graph.add_resistor("R1", "src", "in", 1000.0).unwrap();

    let dc_sol = solve_dc_linear(&graph, &SolverOptions::default()).expect("DC solve failed");
    let in_node = graph.get_node("in").unwrap();
    let v_in = dc_sol.node_voltage(in_node);
    assert!(
        (v_in - 10.0).abs() < 1e-4,
        "Capacitor did not charge to 10V in DC, got {v_in}"
    );
}

#[test]
fn test_real_inductor_saturation_curve_and_synthesis() {
    let l0 = 10e-6; // 10 uH
    let i_sat = 2.0; // 2.0 A
    let r_dc = 0.1; // 100 mOhm
    let c_p = 10e-12; // 10 pF

    let ind_model = RealInductorModel::new(l0, r_dc, c_p).with_saturation(i_sat);

    // 1. Zero current: L(0) == L0
    let l_zero = ind_model.inductance_at_current(0.0);
    assert!((l_zero - l0).abs() < 1e-12);

    // 2. Saturation current: L(I_sat) == L0 / 2
    let l_sat = ind_model.inductance_at_current(i_sat);
    assert!((l_sat - l0 * 0.5).abs() < 1e-12);

    // 3. Double saturation current: L(2*I_sat) == L0 / (1 + 4) = L0 / 5
    let l_2sat = ind_model.inductance_at_current(2.0 * i_sat);
    assert!((l_2sat - l0 * 0.2).abs() < 1e-12);

    // 4. Flux linkage Phi(I) = L0 * I_sat * arctan(I / I_sat)
    let phi_zero = ind_model.flux_linkage(0.0);
    assert_eq!(phi_zero, 0.0);

    let phi_sat = ind_model.flux_linkage(i_sat);
    let expected_phi_sat = l0 * i_sat * (1.0f64).atan(); // L0 * Isat * pi / 4
    assert!((phi_sat - expected_phi_sat).abs() < 1e-12);

    // 5. Synthesize into CircuitGraph and verify DC steady state
    let mut graph = CircuitGraph::new();
    ind_model
        .synthesize_subcircuit(&mut graph, "L_REAL", "in", "0")
        .expect("Synthesis failed");

    // Add 5V DC source through 4.9 Ohm resistor: total resistance = 4.9 + Rdc(0.1) = 5.0 Ohm
    graph.add_voltage_source("V1", "src", "0", 5.0).unwrap();
    graph.add_resistor("R_EXT", "src", "in", 4.9).unwrap();

    let dc_sol = solve_dc_linear(&graph, &SolverOptions::default()).expect("DC solve failed");
    let in_node = graph.get_node("in").unwrap();
    // Steady state current = 5V / 5 Ohm = 1.0 A.
    // Node 'in' voltage = 1.0 A * Rdc = 1.0 * 0.1 = 0.1 V.
    let v_in = dc_sol.node_voltage(in_node);
    assert!(
        (v_in - 0.1).abs() < 1e-5,
        "Expected 0.1V at inductor in DC steady state, got {v_in} V"
    );
}
