//! Integration tests for Phase-Change Memory (PCM / GST mushroom cell):
//! Ovonic Threshold Switching (OTS) snapback, JMAK crystallization kinetics,
//! thermal melt-quench amorphization, and non-linear MNA KCL conservation.

use phonon_core::{CircuitGraph, MemristorState};
use phonon_models::memristor::pcm::PhaseChangeMemoryModel;
use phonon_solver::mna::non_linear_solver::{solve_dc_non_linear, ModelContext, NewtonOptions};
use phonon_solver::verification::kcl_probe::verify_kcl_dynamic;
use std::collections::HashMap;

#[test]
fn test_pcm_dc_mna_and_kcl() {
    let mut graph = CircuitGraph::new();

    // V_IN (node 1) -> R_SERIES (node 1 to node 2) -> PCM (node 2 to ground)
    graph.add_voltage_source("V_IN", "1", "0", 0.8).unwrap();
    graph.add_resistor("R_SERIES", "1", "2", 2000.0).unwrap();
    graph
        .add_memristor("PCM1", "2", "0", 1e-4, 5e3, 1e6)
        .unwrap();

    let pcm_model = PhaseChangeMemoryModel::gst_mushroom_cell();
    let mut context = ModelContext::new();
    context.set_pcm_model("PCM1", pcm_model);

    let opts = NewtonOptions::default();
    let solution = solve_dc_non_linear(&graph, &context, &opts)
        .expect("Non-linear DC solver must converge for PCM circuit");

    let v1 = solution.node_voltages[graph.get_node("1").unwrap().index()];
    let v2 = solution.node_voltages[graph.get_node("2").unwrap().index()];

    assert!(
        (v1 - 0.8).abs() < 1e-6,
        "Source node 1 must be 0.8V, got {}",
        v1
    );
    assert!(
        v2 > 0.0 && v2 < 0.8,
        "PCM divider node 2 must be within (0, 0.8V), got {}",
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
        "KCL violated in PCM circuit: max residual = {} A at node {:?}",
        kcl_report.max_residual, kcl_report.worst_node
    );
}

#[test]
fn test_pcm_ovonic_threshold_switching_snapback() {
    let pcm = PhaseChangeMemoryModel::gst_mushroom_cell();
    let u_c = 0.0; // Fully amorphous HRS cell

    // Sub-threshold bias (0.5 V < V_th = 1.1 V)
    let (i_sub, g_sub) = pcm.evaluate_current_and_conductance(u_c, 0.5);
    // Supra-threshold bias (1.5 V > V_th = 1.1 V)
    let (i_supra, g_supra) = pcm.evaluate_current_and_conductance(u_c, 1.5);

    // In subthreshold, resistance is R_amorph (~800 kOhm)
    let r_sub = 0.5 / i_sub;
    assert!(
        (r_sub - pcm.r_amorph).abs() / pcm.r_amorph < 0.05,
        "Subthreshold resistance must match R_amorph, got {}",
        r_sub
    );

    // In supra-threshold, resistance snaps down to R_holding_ots (~1.5 kOhm)
    let r_supra = 1.5 / i_supra;
    assert!(
        (r_supra - pcm.r_holding_ots).abs() / pcm.r_holding_ots < 0.05,
        "Supra-threshold resistance must snap back to R_holding_ots, got {}",
        r_supra
    );

    assert!(
        g_supra > 100.0 * g_sub,
        "Supra-threshold dynamic conductance must be >100x subthreshold"
    );
}

#[test]
fn test_pcm_crystallization_and_amorphization_kinetics() {
    let pcm = PhaseChangeMemoryModel::gst_mushroom_cell();

    // 1. Crystallization window positive rate (e.g. at 750 K)
    let rate_cryst = pcm.state_derivative(0.1, 750.0);
    assert!(
        rate_cryst > 0.0,
        "Crystallization rate must be positive in the annealing window (750 K): got {}",
        rate_cryst
    );

    // 2. High-temperature melting and amorphization negative rate (> T_melt = 873 K)
    let rate_melt = pcm.state_derivative(0.8, 1000.0);
    assert!(
        rate_melt < 0.0,
        "Melting above T_melt must have negative derivative (amorphization): got {}",
        rate_melt
    );

    // 3. Melt-quench pulse amorphization using RK4 integration:
    // Starting from crystalline state (u_c = 1.0), high-voltage heating above melting point
    let state_cryst = MemristorState {
        conductance_s: 1.0 / pcm.r_cryst,
        internal_state_w: 1.0,
    };
    let dt = 1e-10;
    let mut state = state_cryst;
    for _ in 0..30 {
        state = pcm.step_rk4(state, 2.0, 300.0, dt);
    }
    assert!(
        state.internal_state_w < 0.2,
        "Crystalline cell must amorphize under intense melt pulse: got u_c = {}",
        state.internal_state_w
    );
}
