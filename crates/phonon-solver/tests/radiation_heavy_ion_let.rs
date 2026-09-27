//! Integration test: Cosmic Heavy-Ion particle strike, Linear Energy Transfer (LET),
//! transient current pulse charge conservation, and Single-Event Upset (SEU) bit-flip.

use phonon_core::CircuitGraph;
use phonon_models::{HeavyIonStrikeModel, StandardSramCell};
use phonon_solver::mna::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::verification::verify_kcl_dynamic;
use std::collections::HashMap;

#[test]
fn test_let_charge_generation_and_integral_conservation() {
    let strike = HeavyIonStrikeModel::typical_30nm_heavy_ion(1.0e-9, 30.0);

    // Verify charge density: 30 MeV*cm^2/mg -> ~310 fC/um
    let dq_dx_c_per_m = strike.linear_charge_density_c_per_m();
    let dq_dx_fc_per_um = dq_dx_c_per_m * 1.0e15 * 1.0e-6;
    assert!(
        (dq_dx_fc_per_um - 310.8).abs() < 5.0,
        "Expected ~310 fC/um for LET=30, got {}",
        dq_dx_fc_per_um
    );

    // Verify collection depth: 200 nm dep + 800 nm funnel = 1.0 um
    let l_eff = strike.effective_collection_depth_m();
    assert!((l_eff - 1.0e-6).abs() < 1e-12);

    let q_coll_coulombs = strike.total_collected_charge_c();
    let q_coll_fc = q_coll_coulombs * 1.0e15;
    assert!(q_coll_fc > 250.0 && q_coll_fc < 350.0);

    // Numerically integrate I_SET(t) from t0 to t0 + 10 * tau_fall
    let dt = 0.5e-12; // 0.5 ps
    let mut t = strike.strike_time_s;
    let t_end = strike.strike_time_s + 10.0 * strike.tau_fall_s;
    let mut integrated_charge = 0.0;

    while t <= t_end {
        let current = strike.current_at_time(t);
        integrated_charge += current * dt;
        t += dt;
    }

    let rel_err = ((integrated_charge - q_coll_coulombs) / q_coll_coulombs).abs();
    assert!(
        rel_err < 1.0e-3,
        "Current pulse integral must equal Q_coll within 0.1%: got {} vs analytical {}",
        integrated_charge,
        q_coll_coulombs
    );
}

#[test]
fn test_mna_transient_radiation_injection_and_kcl() {
    let mut graph = CircuitGraph::new();

    // Circuit: 1.2V source on VDD, 1 kOhm pull-up to V_DRAIN, strike injected on V_DRAIN
    graph.add_voltage_source("V1", "VDD", "0", 1.2).unwrap();
    graph
        .add_resistor("R_PULLUP", "VDD", "V_DRAIN", 1000.0)
        .unwrap();
    graph
        .add_radiation_strike("STRIKE1", "V_DRAIN", 20.0, 100.0e-12)
        .unwrap();

    let n1 = graph.get_node("V_DRAIN").unwrap();

    let mut context = ModelContext::new();
    let strike_model = HeavyIonStrikeModel::typical_30nm_heavy_ion(100.0e-12, 20.0);
    context.set_heavy_ion_strike("STRIKE1", strike_model);

    // Test at t = 0 (before strike): voltage should be VDD = 1.2V
    context.set_current_time(0.0);
    let sol_t0 = solve_dc_non_linear(&graph, &context, &NewtonOptions::default())
        .expect("DC solve at t=0 must succeed");
    assert!((sol_t0.node_voltages[n1.index()] - 1.2).abs() < 1e-3);

    // Test at peak of pulse: t = t0 + t_peak (~25 ps after strike)
    let t_peak = 100.0e-12 + 25.0e-12;
    context.set_current_time(t_peak);
    let sol_peak = solve_dc_non_linear(&graph, &context, &NewtonOptions::default())
        .expect("DC solve at pulse peak must succeed");

    let v_struck = sol_peak.node_voltages[n1.index()];
    assert!(v_struck.is_finite());

    // Verify KCL dynamic conservation
    let cap_map = HashMap::new();
    let kcl_report = verify_kcl_dynamic(
        &graph,
        &sol_peak.node_voltages,
        &sol_peak.branch_currents,
        &cap_map,
        Some(&context),
        1e-3,
        1e-6,
    );
    assert!(
        kcl_report.is_valid,
        "KCL must hold at strike peak: max residual = {} A (worst node {:?})",
        kcl_report.max_residual, kcl_report.worst_node
    );
}

#[test]
fn test_sram_cell_bit_flip_seu_threshold() {
    let mut sram = StandardSramCell::new(true, 1.0);
    let q_crit = sram.critical_charge_fc();

    // Strike below threshold (0.5 * Qcrit)
    let outcome_sub = sram.strike(q_crit * 0.5);
    assert!(!outcome_sub.upset_occurred);
    assert!(outcome_sub.self_restored);
    assert!(sram.state_q); // Bit preserved

    // Strike above threshold (1.5 * Qcrit)
    let outcome_super = sram.strike(q_crit * 1.5);
    assert!(outcome_super.upset_occurred);
    assert!(!outcome_super.self_restored);
    assert!(!sram.state_q); // Bit flipped!
}
