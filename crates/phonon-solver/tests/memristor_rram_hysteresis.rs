//! Integration tests for Filamentary Resistive RAM (RRAM) memristors:
//! Pinched hysteresis loop, SET/RESET switching dynamics, compliance current limiting,
//! and non-linear MNA KCL conservation.

use phonon_core::{CircuitGraph, MemristorState};
use phonon_models::memristor::rram::FilamentaryRramModel;
use phonon_solver::mna::non_linear_solver::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::verification::kcl_probe::verify_kcl_dynamic;
use std::collections::HashMap;

#[test]
fn test_rram_dc_mna_and_kcl() {
    let mut graph = CircuitGraph::new();

    // V_IN (node 1) -> R_SERIES (node 1 to node 2) -> RRAM (node 2 to ground)
    graph.add_voltage_source("V_IN", "1", "0", 1.5).unwrap();
    graph.add_resistor("R_SERIES", "1", "2", 1000.0).unwrap();
    graph
        .add_memristor("MEM1", "2", "0", 1e-4, 1e3, 1e6)
        .unwrap();

    let mut rram_model = FilamentaryRramModel::hfo2_synaptic();
    rram_model.compliance_current_a = 5e-3;

    let mut context = ModelContext::new();
    context.set_rram_model("MEM1", rram_model);

    let opts = NewtonOptions::default();
    let solution = solve_dc_non_linear(&graph, &context, &opts)
        .expect("Non-linear DC solver must converge for RRAM circuit");

    let v1 = solution.node_voltages[graph.get_node("1").unwrap().index()];
    let v2 = solution.node_voltages[graph.get_node("2").unwrap().index()];

    assert!(
        (v1 - 1.5).abs() < 1e-6,
        "Source node 1 must be 1.5V, got {}",
        v1
    );
    assert!(
        v2 > 0.0 && v2 < 1.5,
        "Memristor divider node 2 must be within (0, 1.5V), got {}",
        v2
    );

    // Verify KCL
    let empty_cap = HashMap::new();
    let kcl_report = verify_kcl_dynamic(
        &graph,
        &solution.node_voltages,
        &solution.branch_currents,
        &empty_cap,
        Some(&context),
        1e-3,
        1e-6,
    );

    assert!(
        kcl_report.is_valid,
        "KCL violated in RRAM circuit: max residual = {} A at node {:?}",
        kcl_report.max_residual, kcl_report.worst_node
    );
}

#[test]
fn test_rram_pinched_hysteresis_and_zero_crossing() {
    let rram = FilamentaryRramModel::hfo2_synaptic();
    let mut state = MemristorState {
        conductance_s: 1.0 / rram.r_off,
        internal_state_w: 0.1, // initial HRS state (w near 0)
    };
    let temp_k = 300.0;

    // Perform a full bipolar voltage excitation cycle: 0 -> +2.0V -> 0 -> -2.0V -> 0
    let n_steps = 400;
    let dt = 1e-4; // 100 microseconds per step
    let mut currents = Vec::with_capacity(n_steps);
    let mut voltages = Vec::with_capacity(n_steps);

    for i in 0..n_steps {
        let t_norm = (i as f64) / (n_steps as f64) * 2.0 * std::f64::consts::PI;
        let v = 2.0 * t_norm.sin();

        // Evaluate I-V before updating internal state
        let (i_curr, _) = rram.evaluate_current_and_conductance(state.internal_state_w, v);
        currents.push(i_curr);
        voltages.push(v);

        // Update filament protrusion state via RK4 ionic drift
        state = rram.step_rk4(state, v, temp_k, dt);
    }

    // 1. Strictly verify pinched hysteresis law: whenever V = 0, I must be 0
    for (v, i) in voltages.iter().zip(currents.iter()) {
        if v.abs() < 1e-6 {
            assert!(
                i.abs() < 1e-9,
                "Pinched hysteresis violated: at V = {}, I = {} A",
                v,
                i
            );
        }
    }

    // 2. Verify that state w remains physically bounded in [0, 1]
    assert!(
        state.internal_state_w >= 0.0 && state.internal_state_w <= 1.0,
        "State variable w must remain in [0, 1]: {}",
        state.internal_state_w
    );
}

#[test]
fn test_rram_compliance_limiting() {
    let mut rram = FilamentaryRramModel::hfo2_synaptic();
    rram.compliance_current_a = 1.0e-3; // 1 mA compliance

    // In LRS (w = 1.0), R_on = 1 kOhm. At 5.0 V, raw current would be 5 mA.
    let (i_clamped, _) = rram.evaluate_current_and_conductance(1.0, 5.0);
    assert!(
        (i_clamped - 1.0e-3).abs() < 1e-8,
        "Current compliance failed: expected 1.0 mA, got {} mA",
        i_clamped * 1e3
    );

    // Zero-crossing check
    let (i_zero, _) = rram.evaluate_current_and_conductance(1.0, 0.0);
    assert_eq!(i_zero, 0.0, "Current at V=0 must be exactly zero");
}
