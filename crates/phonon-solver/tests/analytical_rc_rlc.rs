//! Analytical closed-form verification tests for 1st-order RC and 2nd-order RLC circuits.

use phonon_core::CircuitGraph;
use phonon_solver::transient::{IntegrationMethod, TimeWaveform, TransientOptions};
use phonon_solver::verification::verify_transient_kcl;
use phonon_solver::{solve_transient, ModelContext, NewtonOptions, StepControlOptions};
use std::collections::HashMap;

#[test]
fn test_analytical_first_order_rc_step_response() {
    let mut graph = CircuitGraph::new();

    // V1 from node 1 to 0 (step from 0 to 5.0V at t = 0)
    let _v_src = graph.add_voltage_source("V1", "1", "0", 5.0).unwrap();
    // R1 between node 1 and 2: R = 10k
    graph.add_resistor("R1", "1", "2", 10_000.0).unwrap();
    // C1 between node 2 and 0: C = 100nF (tau = R*C = 1.0 ms)
    graph
        .add_capacitor("C1", "2", "0", 100e-9, Some(0.0))
        .unwrap();

    let node2 = graph.get_node("2").unwrap();

    let tau = 10_000.0 * 100e-9; // 1.0 ms
    let tstop = 0.005; // 5.0 ms (5 time constants)

    let mut waveforms = HashMap::new();
    waveforms.insert(
        "V1".to_string(),
        TimeWaveform::Pulse {
            v1: 0.0,
            v2: 5.0,
            td: 0.0,
            tr: 1e-9, // Instantaneous step
            tf: 1e-9,
            pw: 10.0,
            per: 20.0,
        },
    );

    let options = TransientOptions {
        tstop,
        tstep: 2e-5, // 20 us
        tstart: 0.0,
        tmax: Some(5e-5),
        uic: true,
        method: IntegrationMethod::TrBdf2,
        step_control: StepControlOptions {
            reltol: 1e-4,
            vntol: 1e-6,
            abstol: 1e-12,
            ..Default::default()
        },
        newton: NewtonOptions::default(),
        waveforms,
    };

    let context = ModelContext::default();
    let solution =
        solve_transient(&graph, &context, &options).expect("Transient solve must succeed");

    assert!(solution.len() >= 50);

    // Compute L2 error against exact analytical solution: V(t) = 5.0 * (1 - exp(-t / tau))
    let mut sum_sq_err = 0.0;
    let mut count = 0;

    for step in &solution.steps {
        if step.time > 1e-8 {
            let t = step.time;
            let v_exact = 5.0 * (1.0 - (-t / tau).exp());
            let v_sim = step.voltages[node2.index()];
            let err = v_sim - v_exact;
            sum_sq_err += err * err;
            count += 1;

            // Also verify KCL holds at every accepted step
            let kcl = verify_transient_kcl(&graph, step, 1e-3, 1e-6);
            assert!(
                kcl.is_valid,
                "KCL violated at t={}: max residual = {}",
                t, kcl.max_residual
            );
        }
    }

    let l2_error = (sum_sq_err / count as f64).sqrt();
    assert!(
        l2_error < 1e-3,
        "L2 error against analytical RC solution must be < 1mV, got {:.3e} V",
        l2_error
    );

    // Final voltage after 5 tau must be >= 4.96 V (99.3% of 5.0V)
    let final_v = solution.steps.last().unwrap().voltages[node2.index()];
    assert!(
        (final_v - 5.0).abs() < 0.05,
        "Final voltage should approach 5.0V, got {}",
        final_v
    );
}

#[test]
fn test_analytical_second_order_rlc_oscillator_regimes() {
    // Underdamped RLC circuit (zeta < 1):
    // L = 10 mH, C = 100 nF, R = 100 ohm
    // omega_0 = 1 / sqrt(LC) = 31622.77 rad/s
    // zeta = (R/2) * sqrt(C/L) = 50 * sqrt(1e-5) = 0.1581
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "1", "0", 5.0).unwrap();
    graph.add_resistor("R1", "1", "2", 100.0).unwrap();
    graph
        .add_inductor("L1", "2", "3", 10e-3, Some(0.0))
        .unwrap();
    graph
        .add_capacitor("C1", "3", "0", 100e-9, Some(0.0))
        .unwrap();

    let node3 = graph.get_node("3").unwrap();

    let mut waveforms = HashMap::new();
    waveforms.insert(
        "V1".to_string(),
        TimeWaveform::Pulse {
            v1: 0.0,
            v2: 5.0,
            td: 0.0,
            tr: 1e-9,
            tf: 1e-9,
            pw: 10.0,
            per: 20.0,
        },
    );

    let options = TransientOptions {
        tstop: 0.001, // 1.0 ms
        tstep: 5e-6,  // 5 us
        tstart: 0.0,
        tmax: Some(1e-5),
        uic: true,
        method: IntegrationMethod::TrBdf2,
        step_control: StepControlOptions::default(),
        newton: NewtonOptions::default(),
        waveforms,
    };

    let context = ModelContext::default();
    let solution =
        solve_transient(&graph, &context, &options).expect("Transient solve must succeed");

    // Underdamped response must overshoot 5.0V
    let max_v = solution
        .steps
        .iter()
        .map(|s| s.voltages[node3.index()])
        .fold(0.0f64, f64::max);

    assert!(
        max_v > 5.5,
        "Underdamped RLC must exhibit overshoot above 5.5V, got {:.3} V",
        max_v
    );

    // KCL must hold at all time points
    for step in &solution.steps {
        let kcl = verify_transient_kcl(&graph, step, 1e-3, 1e-5);
        assert!(
            kcl.is_valid,
            "KCL violated during RLC transient: {}",
            kcl.max_residual
        );
    }
}
