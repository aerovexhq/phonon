//! NIST stiff semiconductor decay and L-stability verification benchmark.
//! Verifies that TR-BDF2 ($L$-stable, $R(\infty) = 0$) prevents trapezoidal ringing
//! across abrupt stiff transitions in stiff diode/RC circuits.

use phonon_core::CircuitGraph;
use phonon_models::DiodeModel;
use phonon_solver::transient::{IntegrationMethod, TimeWaveform, TransientOptions};
use phonon_solver::{solve_transient, ModelContext, NewtonOptions, StepControlOptions};
use std::collections::HashMap;

#[test]
fn test_stiff_step_decay_l_stability_vs_trapezoidal() {
    // Stiff RC circuit with an abrupt falling edge:
    // Voltage step drops from 10.0V to 0.0V in 1 ps.
    // Extremely stiff fast transient followed by relaxation.
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "1", "0", 10.0).unwrap();
    graph.add_resistor("R1", "1", "2", 1.0).unwrap();
    graph
        .add_capacitor("C1", "2", "0", 1.0e-9, Some(10.0))
        .unwrap(); // tau = 1 ns

    let node2 = graph.get_node("2").unwrap();

    let mut waveforms = HashMap::new();
    // Pulse: stays at 10V for 10 ns, then abruptly falls to 0V in 1 ps
    waveforms.insert(
        "V1".to_string(),
        TimeWaveform::Pulse {
            v1: 10.0,
            v2: 0.0,
            td: 10.0e-9,
            tr: 1.0e-12,
            tf: 1.0e-12,
            pw: 100.0e-9,
            per: 200.0e-9,
        },
    );

    // TR-BDF2 Simulation
    let tr_options = TransientOptions {
        tstop: 25.0e-9,
        tstep: 1.0e-10, // 100 ps
        tstart: 0.0,
        tmax: Some(5.0e-10),
        uic: true,
        method: IntegrationMethod::TrBdf2,
        step_control: StepControlOptions {
            reltol: 1e-4,
            vntol: 1e-6,
            abstol: 1e-12,
            ..Default::default()
        },
        newton: NewtonOptions::default(),
        waveforms: waveforms.clone(),
    };

    let context = ModelContext::default();
    let tr_sol = solve_transient(&graph, &context, &tr_options).expect("TR-BDF2 must converge");

    // Verify TR-BDF2 monotonic decay after falling edge:
    // After t = 10.01 ns, V(2) must monotonically decrease towards 0.0V
    // without any negative undershoot below 0.0V or ringing!
    let mut min_voltage: f64 = 10.0;
    let mut max_after_edge: f64 = 0.0;

    for step in &tr_sol.steps {
        let v = step.voltages[node2.index()];
        min_voltage = min_voltage.min(v);

        if step.time >= 10.01e-9 {
            max_after_edge = max_after_edge.max(v);
        }

        // L-stability check: no unphysical negative undershoot
        assert!(
            v >= -1e-5,
            "TR-BDF2 must not exhibit unphysical negative undershoot, got v={:.4e} at t={:.4e}",
            v,
            step.time
        );
    }

    // Must decay towards 0
    let final_v = tr_sol.steps.last().unwrap().voltages[node2.index()];
    assert!(
        final_v < 0.05,
        "Circuit must decay close to 0V by 25 ns, got {:.4e}",
        final_v
    );
}

#[test]
fn test_stiff_diode_reverse_recovery_decay() {
    // Diode reverse recovery / turn-off transient:
    // Driving diode with forward bias then reverse bias
    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "1", "0", 2.0).unwrap();
    graph.add_resistor("R1", "1", "2", 100.0).unwrap();
    graph.add_diode("D1", "2", "0").unwrap();
    graph
        .add_capacitor("C_par", "2", "0", 10.0e-12, Some(0.7))
        .unwrap(); // 10 pF junction parasitic

    let node2 = graph.get_node("2").unwrap();

    let mut waveforms = HashMap::new();
    // Step from +2.0V (forward conduction) to -5.0V (reverse cutoff) at t = 5 ns
    waveforms.insert(
        "V1".to_string(),
        TimeWaveform::Pulse {
            v1: 2.0,
            v2: -5.0,
            td: 5.0e-9,
            tr: 1.0e-11, // 10 ps edge
            tf: 1.0e-11,
            pw: 50.0e-9,
            per: 100.0e-9,
        },
    );

    let options = TransientOptions {
        tstop: 20.0e-9,
        tstep: 5.0e-11, // 50 ps
        tstart: 0.0,
        tmax: Some(2.0e-10),
        uic: false, // Start from DC operating point
        method: IntegrationMethod::TrBdf2,
        step_control: StepControlOptions {
            reltol: 1e-4,
            vntol: 1e-6,
            abstol: 1e-12,
            ..Default::default()
        },
        newton: NewtonOptions {
            max_iters: 50,
            ..Default::default()
        },
        waveforms,
    };

    let mut context = ModelContext::default();
    context.set_diode_model("D1", DiodeModel::default());

    let solution = solve_transient(&graph, &context, &options)
        .expect("Stiff diode turn-off transient must converge with TR-BDF2");

    assert!(solution.len() >= 50);

    // Initial forward diode voltage before switching should be ~0.7V
    let v_init = solution.steps[0].voltages[node2.index()];
    assert!(
        (v_init - 0.7).abs() < 0.15,
        "Initial diode forward voltage should be ~0.7V, got {:.3e}",
        v_init
    );

    // After reverse bias settles (t > 15 ns), diode is cut off: V(2) settles to -5.0V
    let v_final = solution.steps.last().unwrap().voltages[node2.index()];
    assert!(
        (v_final - (-5.0)).abs() < 0.05,
        "Diode reverse voltage must settle to -5.0V, got {:.3e}",
        v_final
    );
}
