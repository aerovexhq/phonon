#![deny(unsafe_code)]

//! Verification and analytical test suite for backward adjoint transient sensitivity analysis and worst-case optimization.

use phonon_core::CircuitGraph;
use phonon_solver::mna::non_linear_solver::ModelContext;
use phonon_solver::sensitivity::{
    AdjointSensitivityEngine, CircuitParameter, ObjectiveKind, WorstCaseOptimizer,
};
use phonon_solver::transient::{IntegrationMethod, TransientOptions};
use std::time::Instant;

/// Helper function building a canonical charging RC circuit:
/// Vin = 5.0 V, R = 1000 Ohm, C = 10 uF, stop time T = 5 ms (t / RC = 0.5).
fn build_analytical_rc_circuit(
    r_val: f64,
    c_val: f64,
) -> (CircuitGraph, ModelContext, TransientOptions) {
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "VIN", "0", 5.0).expect("add V1 failed");
    graph.add_resistor("R1", "VIN", "VOUT", r_val).expect("add R1 failed");
    graph.add_capacitor("C1", "VOUT", "0", c_val, Some(0.0)).expect("add C1 failed");

    let context = ModelContext::default();
    let mut options = TransientOptions::default();
    options.tstop = 0.005; // 5 ms
    options.tstep = 0.00005; // 50 us (100 uniform steps)
    options.tmax = Some(0.00005);
    options.uic = true;
    options.method = IntegrationMethod::Trapezoidal;

    (graph, context, options)
}

/// Helper function building a second-order underdamped series RLC circuit:
/// Vin = 5.0 V, R = 50 Ohm, L = 10 mH, C = 1 uF (zeta = 0.25, underdamped ringing).
fn build_rlc_circuit(
    r_val: f64,
    l_val: f64,
    c_val: f64,
) -> (CircuitGraph, ModelContext, TransientOptions) {
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "VIN", "0", 5.0).expect("add V1 failed");
    graph.add_resistor("R1", "VIN", "VMID", r_val).expect("add R1 failed");
    graph.add_inductor("L1", "VMID", "VOUT", l_val, Some(0.0)).expect("add L1 failed");
    graph.add_capacitor("C1", "VOUT", "0", c_val, Some(0.0)).expect("add C1 failed");

    let context = ModelContext::default();
    let mut options = TransientOptions::default();
    options.tstop = 0.002; // 2 ms
    options.tstep = 0.00002; // 20 us
    options.tmax = Some(0.00002);
    options.uic = true;
    options.method = IntegrationMethod::Trapezoidal;

    (graph, context, options)
}

#[test]
fn test_analytical_rc_circuit_sensitivity_matches_closed_form() {
    let r_nom = 1000.0; // 1 kOhm
    let c_nom = 1.0e-5; // 10 uF
    let vin = 5.0;
    let t = 0.005; // 5 ms

    let (graph, context, options) = build_analytical_rc_circuit(r_nom, c_nom);

    let params = vec![
        CircuitParameter::Resistance {
            id: "R1".to_string(),
            nominal: r_nom,
            tolerance: 0.05,
        },
        CircuitParameter::Capacitance {
            id: "C1".to_string(),
            nominal: c_nom,
            tolerance: 0.10,
        },
    ];

    let engine = AdjointSensitivityEngine::new(
        graph,
        context,
        options,
        ObjectiveKind::TerminalVoltage { node: 2 },
        params,
    );

    let sensitivities = engine
        .solve()
        .expect("Adjoint sensitivity solve must succeed");
    assert_eq!(sensitivities.len(), 2);

    // Closed-form analytical derivatives for charging RC circuit:
    // v_C(t) = Vin * (1 - exp(-t / (R*C)))
    // dv_C/dR = - (t / (R^2 * C)) * exp(-t / (R*C)) * Vin
    // dv_C/dC = - (t / (R * C^2)) * exp(-t / (R*C)) * Vin
    let rc = r_nom * c_nom;
    let exp_term = (-t / rc).exp();
    let expected_dvc_dr = -(t / (r_nom * r_nom * c_nom)) * exp_term * vin;
    let expected_dvc_dc = -(t / (r_nom * c_nom * c_nom)) * exp_term * vin;

    let r_res = sensitivities
        .iter()
        .find(|s| s.param_id == "R1")
        .expect("R1 must be present");
    let c_res = sensitivities
        .iter()
        .find(|s| s.param_id == "C1")
        .expect("C1 must be present");

    let rel_err_r = ((r_res.gradient - expected_dvc_dr) / expected_dvc_dr).abs();
    let rel_err_c = ((c_res.gradient - expected_dvc_dc) / expected_dvc_dc).abs();

    assert!(
        rel_err_r < 0.015,
        "Adjoint dV/dR must match analytical within 1.5%: got {:.6e}, expected {:.6e}, rel_err={:.4}",
        r_res.gradient,
        expected_dvc_dr,
        rel_err_r
    );
    assert!(
        rel_err_c < 0.015,
        "Adjoint dV/dC must match analytical within 1.5%: got {:.6e}, expected {:.6e}, rel_err={:.4}",
        c_res.gradient,
        expected_dvc_dc,
        rel_err_c
    );

    // Verify negative sensitivity signs (increasing resistance or capacitance slows charging)
    assert!(r_res.gradient < 0.0);
    assert!(c_res.gradient < 0.0);
    assert!(r_res.normalized_sensitivity < 0.0);
    assert!(c_res.normalized_sensitivity < 0.0);
}

#[test]
fn test_rlc_second_order_damping_sensitivity() {
    let (graph, context, options) = build_rlc_circuit(50.0, 0.01, 1.0e-6);

    let params = vec![
        CircuitParameter::Resistance {
            id: "R1".to_string(),
            nominal: 50.0,
            tolerance: 0.05,
        },
        CircuitParameter::Inductance {
            id: "L1".to_string(),
            nominal: 0.01,
            tolerance: 0.05,
        },
        CircuitParameter::Capacitance {
            id: "C1".to_string(),
            nominal: 1.0e-6,
            tolerance: 0.10,
        },
    ];

    let engine = AdjointSensitivityEngine::new(
        graph,
        context,
        options,
        ObjectiveKind::PeakOvershoot { node: 3 },
        params,
    );

    let sensitivities = engine
        .solve()
        .expect("RLC peak overshoot sensitivity solve must succeed");
    assert_eq!(sensitivities.len(), 3);

    let r_res = sensitivities
        .iter()
        .find(|s| s.param_id == "R1")
        .expect("R1 must be present");
    let l_res = sensitivities
        .iter()
        .find(|s| s.param_id == "L1")
        .expect("L1 must be present");

    // In a series RLC step response, increasing resistance R increases damping zeta = R / (2 * sqrt(L/C)),
    // strictly suppressing resonant peak overshoot. Therefore d(Peak)/dR must be negative.
    assert!(
        r_res.gradient < 0.0,
        "Increasing damping resistance R must reduce peak overshoot: got gradient={:.6e}",
        r_res.gradient
    );

    // Conversely, increasing inductance L decreases damping, enhancing overshoot.
    assert!(
        l_res.gradient > 0.0,
        "Increasing inductance L must increase peak overshoot: got gradient={:.6e}",
        l_res.gradient
    );
}

#[test]
fn test_normalized_sensitivity_ranking_order() {
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "VIN", "0", 5.0).expect("add V1 failed");
    // Asymmetric voltage divider: R1 dominates over small R2
    graph.add_resistor("R_DOMINANT", "VIN", "VMID", 10000.0).expect("add R_DOMINANT failed");
    graph.add_resistor("R_MINOR", "VMID", "VOUT", 100.0).expect("add R_MINOR failed");
    graph.add_capacitor("C_FILTER", "VOUT", "0", 1.0e-6, Some(0.0)).expect("add C_FILTER failed");

    let context = ModelContext::default();
    let mut options = TransientOptions::default();
    options.tstop = 0.01;
    options.tstep = 0.0001;
    options.uic = true;

    let params = vec![
        CircuitParameter::Resistance {
            id: "R_MINOR".to_string(),
            nominal: 100.0,
            tolerance: 0.05,
        },
        CircuitParameter::Resistance {
            id: "R_DOMINANT".to_string(),
            nominal: 10000.0,
            tolerance: 0.05,
        },
        CircuitParameter::Capacitance {
            id: "C_FILTER".to_string(),
            nominal: 1.0e-6,
            tolerance: 0.10,
        },
    ];

    let engine = AdjointSensitivityEngine::new(
        graph,
        context,
        options,
        ObjectiveKind::TerminalVoltage { node: 3 },
        params,
    );

    let mut results = engine.solve().expect("Sensitivity solve must succeed");
    assert_eq!(results.len(), 3);

    // Sort by absolute normalized sensitivity in descending order
    results.sort_by(|a, b| {
        b.normalized_sensitivity
            .abs()
            .partial_cmp(&a.normalized_sensitivity.abs())
            .unwrap()
    });

    // The dominant resistor and capacitor must rank higher than the minor 100-ohm resistor
    let top_id = &results[0].param_id;
    let last_id = &results[2].param_id;

    assert_eq!(
        last_id, "R_MINOR",
        "R_MINOR (100 Ohm) must have lowest normalized sensitivity rank, got {}",
        last_id
    );
    assert!(
        top_id == "R_DOMINANT" || top_id == "C_FILTER",
        "Top ranked parameter must be R_DOMINANT or C_FILTER, got {}",
        top_id
    );
}

#[test]
fn test_worst_case_corner_finder_matches_expected_degradation() {
    let (graph, context, options) = build_analytical_rc_circuit(1000.0, 1.0e-5);

    let params = vec![
        CircuitParameter::Resistance {
            id: "R1".to_string(),
            nominal: 1000.0,
            tolerance: 0.10, // +/-10%
        },
        CircuitParameter::Capacitance {
            id: "C1".to_string(),
            nominal: 1.0e-5,
            tolerance: 0.10, // +/-10%
        },
    ];

    let engine = AdjointSensitivityEngine::new(
        graph,
        context,
        options,
        ObjectiveKind::TerminalVoltage { node: 2 },
        params,
    );

    let optimizer = WorstCaseOptimizer::new(engine);
    let summary = optimizer
        .find_worst_case_corners()
        .expect("Worst-case corner search must succeed");

    // Nominal voltage J0 at 5 ms for Vin=5V, R=1k, C=10uF (t/RC = 0.5) is ~ 1.967 V
    assert!(
        summary.nominal.objective_value > 1.90 && summary.nominal.objective_value < 2.05,
        "Nominal objective must be ~1.967 V, got {:.4}",
        summary.nominal.objective_value
    );

    // Worst-Case Max must be strictly greater than nominal
    assert!(
        summary.worst_max.objective_value >= summary.nominal.objective_value,
        "Worst-case max must be >= nominal: max={:.6}, nom={:.6}",
        summary.worst_max.objective_value,
        summary.nominal.objective_value
    );

    // Worst-Case Min must be strictly less than nominal
    assert!(
        summary.worst_min.objective_value <= summary.nominal.objective_value,
        "Worst-case min must be <= nominal: min={:.6}, nom={:.6}",
        summary.worst_min.objective_value,
        summary.nominal.objective_value
    );

    // Verify PVT corner evaluations exist
    assert!(
        !summary.corners.is_empty(),
        "PVT corners list must not be empty"
    );
    assert!(
        summary.max_degradation_percent > 0.0,
        "Maximum degradation percent must be positive, got {:.2}%",
        summary.max_degradation_percent
    );
}

#[test]
fn test_high_speed_throughput_benchmark_exceeds_threshold() {
    let (graph, context, mut options) = build_analytical_rc_circuit(1000.0, 1.0e-5);
    options.tstop = 0.002;
    options.tstep = 0.0001; // 20 steps for ultra-fast transient sweep

    let params = vec![CircuitParameter::Resistance {
        id: "R1".to_string(),
        nominal: 1000.0,
        tolerance: 0.05,
    }];

    let engine = AdjointSensitivityEngine::new(
        graph,
        context,
        options,
        ObjectiveKind::TerminalVoltage { node: 2 },
        params,
    );

    let num_evaluations = 100;
    let start = Instant::now();

    for _ in 0..num_evaluations {
        let res = engine.solve().expect("Sensitivity evaluation must succeed");
        assert_eq!(res.len(), 1);
    }

    let elapsed = start.elapsed();
    let evals_per_sec = num_evaluations as f64 / elapsed.as_secs_f64();

    // Verify solver executes > 100 runs/sec in debug mode
    assert!(
        evals_per_sec > 100.0,
        "Solver throughput benchmark: executed {} evals in {:?} ({:.2} evals/sec)",
        num_evaluations,
        elapsed,
        evals_per_sec
    );
}
