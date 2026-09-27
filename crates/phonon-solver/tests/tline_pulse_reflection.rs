//! Integration test: Distributed transmission line pulse propagation and boundary reflections.
//! Validates Branin's Method of Characteristics against analytical Telegrapher wave solutions:
//! 1. Matched load ($R_L = Z_0$): Zero reflection ($\Gamma = 0$), pure delayed pulse.
//! 2. Open circuit ($R_L = \infty$): In-phase reflection ($\Gamma = +1$), voltage doubles to $2 V_{inc}$ at load.
//! 3. Short circuit ($R_L = 0$): Out-of-phase reflection ($\Gamma = -1$), load clamped to 0 V, wave inverts.

use phonon_core::CircuitGraph;
use phonon_solver::mna::{ModelContext, NewtonOptions};
use phonon_solver::solve_transient;
use phonon_solver::transient::{
    IntegrationMethod, StepControlOptions, TimeWaveform, TransientOptions,
};

#[test]
fn test_matched_transmission_line_pulse_propagation() {
    // Topology:
    // Pulse source V1 (0 to 2V step at t=0, rise time = 0.1 ns) -> Rs = 50 Ohm -> Node 'in'
    // T1: in 0 out 0, Z0 = 50 Ohm, Td = 1.0 ns
    // Load: R_load = 50 Ohm between 'out' and 0.
    // Analytical expectation:
    // Divider at input: V_in = 2.0 * (Z0 / (Rs + Z0)) = 1.0 V.
    // Incident wave V+ = 1.0 V arrives at 'out' at t = 1.0 ns.
    // Matched load: Gamma = (50 - 50)/(50 + 50) = 0.
    // Load voltage for t >= 1.0 ns is exactly 1.0 V.
    // No reflection returns to 'in', so V_in remains 1.0 V.

    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "src", "0", 0.0).unwrap();
    graph.add_resistor("RS", "src", "in", 50.0).unwrap();
    graph
        .add_transmission_line("T1", "in", "0", "out", "0", 50.0, 1.0e-9)
        .unwrap();
    graph.add_resistor("RL", "out", "0", 50.0).unwrap();

    let n_in = graph.get_node("in").unwrap();
    let n_out = graph.get_node("out").unwrap();

    let mut waveforms = std::collections::HashMap::new();
    // Step pulse: 0V to 2V with 0.1 ns rise time
    waveforms.insert(
        "V1".to_string(),
        TimeWaveform::Pwl {
            points: vec![(0.0, 0.0), (0.1e-9, 2.0), (10.0e-9, 2.0)],
        },
    );

    let options = TransientOptions {
        tstop: 4.0e-9,
        tstep: 0.02e-9,
        method: IntegrationMethod::Trapezoidal,
        waveforms,
        step_control: StepControlOptions {
            min_step: 1e-12,
            max_step: 0.02e-9,
            reltol: 1e-3,
            abstol: 1e-6,
            ..Default::default()
        },
        newton: NewtonOptions::default(),
        ..Default::default()
    };

    let solution = solve_transient(&graph, &ModelContext::new(), &options)
        .expect("Transient simulation failed");

    let in_wave = solution.node_waveform(n_in);
    let out_wave = solution.node_waveform(n_out);

    // Before t = 1.0 ns, load voltage must be 0.0 V
    for &(t, v) in &out_wave {
        if t < 0.95e-9 {
            assert!(v.abs() < 1e-3, "Load showed early signal at t={t}: {v} V");
        }
    }

    // At t >= 1.2 ns (after pulse rise and delay), load voltage must be 1.0 V +/- 2%
    for &(t, v) in &out_wave {
        if t >= 1.2e-9 {
            assert!(
                (v - 1.0).abs() < 0.03,
                "Expected 1.0 V at matched load at t={t}, got {v} V"
            );
        }
    }

    // Input node must remain at 1.0 V after initial rise (no reflection)
    for &(t, v) in &in_wave {
        if t >= 0.15e-9 {
            assert!(
                (v - 1.0).abs() < 0.03,
                "Input node showed unexpected reflection at t={t}: {v} V"
            );
        }
    }
}

#[test]
fn test_open_circuit_transmission_line_reflection() {
    // Open circuit load: R_load = 1e8 Ohm (practically open).
    // Gamma_L = +1.0.
    // Incident wave V+ = 1.0 V travels from input (after 0.1 ns rise).
    // At t = Td = 1.0 ns, wave hits open end and reflects with Gamma = +1.
    // Load voltage doubles to V_load = V+ * (1 + Gamma) = 2.0 V!
    // Reflected wave V- = 1.0 V travels back towards source, arriving at t = 2 * Td = 2.0 ns.
    // At source, Rs = Z0 = 50 Ohm, so Gamma_S = 0 (matched source absorbs reflected wave).
    // Input node steps up to 1.0 V + 1.0 V = 2.0 V at t = 2.0 ns and stays there!

    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "src", "0", 0.0).unwrap();
    graph.add_resistor("RS", "src", "in", 50.0).unwrap();
    graph
        .add_transmission_line("T1", "in", "0", "out", "0", 50.0, 1.0e-9)
        .unwrap();
    graph.add_resistor("RL", "out", "0", 1e8).unwrap(); // Open load

    let n_in = graph.get_node("in").unwrap();
    let n_out = graph.get_node("out").unwrap();

    let mut waveforms = std::collections::HashMap::new();
    waveforms.insert(
        "V1".to_string(),
        TimeWaveform::Pwl {
            points: vec![(0.0, 0.0), (0.1e-9, 2.0), (10.0e-9, 2.0)],
        },
    );

    let options = TransientOptions {
        tstop: 4.0e-9,
        tstep: 0.02e-9,
        method: IntegrationMethod::Trapezoidal,
        waveforms,
        step_control: StepControlOptions {
            min_step: 1e-12,
            max_step: 0.02e-9,
            reltol: 1e-3,
            abstol: 1e-6,
            ..Default::default()
        },
        newton: NewtonOptions::default(),
        ..Default::default()
    };

    let solution = solve_transient(&graph, &ModelContext::new(), &options)
        .expect("Transient simulation failed");

    let in_wave = solution.node_waveform(n_in);
    let out_wave = solution.node_waveform(n_out);

    // 1. Output at open circuit before arrival (t < 0.95 ns): 0 V
    for &(t, v) in &out_wave {
        if t < 0.95e-9 {
            assert!(
                v.abs() < 1e-3,
                "Open load signal arrived early: {v} V at t={t}"
            );
        }
    }

    // 2. Output after arrival (t >= 1.2 ns): doubles to 2.0 V
    for &(t, v) in &out_wave {
        if t >= 1.2e-9 {
            assert!(
                (v - 2.0).abs() < 0.05,
                "Expected 2.0 V at open termination at t={t}, got {v} V"
            );
        }
    }

    // 3. Input between 0.15 ns and 1.95 ns: exactly 1.0 V (initial forward wave)
    for &(t, v) in &in_wave {
        if (0.15e-9..=1.95e-9).contains(&t) {
            assert!(
                (v - 1.0).abs() < 0.03,
                "Expected 1.0 V at input before reflected wave return at t={t}, got {v} V"
            );
        }
    }

    // 4. Input after 2.2 ns: reflected wave arrived, steps to 2.0 V
    for &(t, v) in &in_wave {
        if t >= 2.2e-9 {
            assert!(
                (v - 2.0).abs() < 0.05,
                "Expected 2.0 V at input after reflected wave absorbed at t={t}, got {v} V"
            );
        }
    }
}

#[test]
fn test_short_circuit_transmission_line_reflection() {
    // Short circuit load: R_load = 0.001 Ohm (~0).
    // Gamma_L = -1.0.
    // Incident wave V+ = 1.0 V.
    // At t = Td = 1.0 ns, wave hits short, V_load remains ~0 V.
    // Reflected wave V- = -1.0 V travels back to source.
    // At source, arriving at t = 2 * Td = 2.0 ns, input voltage drops:
    // V_in = 1.0 V + (-1.0 V) = 0.0 V!
    // Source current increases to V_src / Rs = 2.0 V / 50 Ohm = 40 mA.

    let mut graph = CircuitGraph::new();
    graph.add_voltage_source("V1", "src", "0", 0.0).unwrap();
    graph.add_resistor("RS", "src", "in", 50.0).unwrap();
    graph
        .add_transmission_line("T1", "in", "0", "out", "0", 50.0, 1.0e-9)
        .unwrap();
    graph.add_resistor("RL", "out", "0", 1e-4).unwrap(); // Short load

    let n_in = graph.get_node("in").unwrap();
    let n_out = graph.get_node("out").unwrap();

    let mut waveforms = std::collections::HashMap::new();
    waveforms.insert(
        "V1".to_string(),
        TimeWaveform::Pwl {
            points: vec![(0.0, 0.0), (0.1e-9, 2.0), (10.0e-9, 2.0)],
        },
    );

    let options = TransientOptions {
        tstop: 4.0e-9,
        tstep: 0.02e-9,
        method: IntegrationMethod::Trapezoidal,
        waveforms,
        step_control: StepControlOptions {
            min_step: 1e-12,
            max_step: 0.02e-9,
            reltol: 1e-3,
            abstol: 1e-6,
            ..Default::default()
        },
        newton: NewtonOptions::default(),
        ..Default::default()
    };

    let solution = solve_transient(&graph, &ModelContext::new(), &options)
        .expect("Transient simulation failed");

    let in_wave = solution.node_waveform(n_in);
    let out_wave = solution.node_waveform(n_out);

    // 1. Output is shorted, must be near 0V at all times
    for &(t, v) in &out_wave {
        assert!(
            v.abs() < 1e-3,
            "Short circuit node had non-zero voltage at t={t}: {v} V"
        );
    }

    // 2. Input between 0.15 ns and 1.95 ns: forward wave 1.0 V
    for &(t, v) in &in_wave {
        if (0.15e-9..=1.95e-9).contains(&t) {
            assert!(
                (v - 1.0).abs() < 0.03,
                "Expected 1.0 V before inverted wave returns at t={t}, got {v} V"
            );
        }
    }

    // 3. Input after 2.2 ns: inverted wave cancels forward voltage, dropping to ~0 V
    for &(t, v) in &in_wave {
        if t >= 2.2e-9 {
            assert!(
                v.abs() < 0.03,
                "Expected ~0 V after inverted wave returns at t={t}, got {v} V"
            );
        }
    }
}
