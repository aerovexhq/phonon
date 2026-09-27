//! Integration Test: Boundary Synchronization, Schmitt-Trigger Hysteresis,
//! Exact Threshold Crossing Back-tracking, and D2A Output Waveform Synthesis.

use phonon_core::{CircuitGraph, LogicLevel, NodeId};
use phonon_models::mixed_signal::{A2dBridge, D2aBridge, DigitalNetwork, PeriodicClock};
use phonon_solver::mixed_signal::{
    solve_mixed_signal, MixedSignalCircuit, MixedSignalOptions, MixedSignalSolution,
};
use phonon_solver::mna::ModelContext;
use phonon_solver::transient::{TimeWaveform, TransientOptions};

#[test]
fn test_a2d_schmitt_hysteresis_and_backtracking() {
    let mut graph = CircuitGraph::new();
    let n_in = graph.get_or_create_node("in");
    let n_gnd = NodeId::GROUND;

    // Sinusoidal voltage source: V(t) = 2.5 + 2.0 * sin(2*pi*1000*t) -> swings between 0.5V and 4.5V
    graph
        .add_voltage_source("Vsin", "in", "0", 2.5)
        .expect("Failed to add voltage source");
    graph
        .add_resistor("Rload", "in", "0", 1e6)
        .expect("Failed to add load");

    let mut digital_net = DigitalNetwork::new();
    let d_out = digital_net.add_node(Some("D_OUT"));

    // A2D Bridge with hysteresis: Vth_low = 1.5V, Vth_high = 3.5V
    let a2d = A2dBridge::new(n_in, n_gnd, d_out, 1.5, 3.5);

    let mut circuit = MixedSignalCircuit::new(graph, digital_net);
    circuit.add_a2d(a2d);

    let mut transient_opts = TransientOptions {
        tstop: 2e-3, // 2 full cycles of 1 kHz
        tstep: 2e-5, // 20 us nominal step
        ..TransientOptions::default()
    };
    transient_opts.waveforms.insert(
        "Vsin".to_string(),
        TimeWaveform::Sine {
            offset: 2.5,
            amplitude: 2.0,
            frequency: 1000.0,
            delay: 0.0,
            damping: 0.0,
        },
    );

    let options = MixedSignalOptions {
        transient: transient_opts,
        max_delta_cycles: 100,
        crossing_tolerance: 1e-10,
    };

    let context = ModelContext::new();
    let solution: MixedSignalSolution =
        solve_mixed_signal(&mut circuit, &context, &options).expect("Mixed-signal solve failed");

    // 1. Back-tracking must have occurred to land precisely on threshold crossings
    assert!(
        solution.total_backtrack_steps >= 3,
        "Expected multiple back-track steps for accurate threshold landing, got {}",
        solution.total_backtrack_steps
    );

    // 2. Extract digital waveform for D_OUT
    let d_wave = solution.digital_waveform(d_out);
    assert!(
        d_wave.len() >= 4,
        "Expected at least 4 state transitions across 2 cycles, got {}",
        d_wave.len()
    );

    // 3. Verify threshold voltages at transition timestamps
    let ana_wave = solution.analog_node_waveform(n_in);
    for (t_trans, lvl) in d_wave {
        if t_trans == 0.0 {
            continue;
        }
        // Find closest analog sample
        let v_at_t = ana_wave
            .iter()
            .min_by(|a, b| (a.0 - t_trans).abs().total_cmp(&(b.0 - t_trans).abs()))
            .map(|s| s.1)
            .unwrap();

        if lvl == LogicLevel::One {
            // Rising transition should have occurred at Vth_high = 3.5V
            assert!(
                (v_at_t - 3.5).abs() < 0.05,
                "Rising transition at t={:.6e}s occurred at V={:.3}V, expected ~3.5V",
                t_trans,
                v_at_t
            );
        } else if lvl == LogicLevel::Zero {
            // Falling transition should have occurred at Vth_low = 1.5V
            assert!(
                (v_at_t - 1.5).abs() < 0.05,
                "Falling transition at t={:.6e}s occurred at V={:.3}V, expected ~1.5V",
                t_trans,
                v_at_t
            );
        }
    }
}

#[test]
fn test_d2a_smooth_ramp_and_thevenin_loading() {
    let mut graph = CircuitGraph::new();
    let n_out = graph.get_or_create_node("out");
    let n_gnd = NodeId::GROUND;

    // Load resistor Rload = 50 Ohms to Ground
    graph
        .add_resistor("Rload", "out", "0", 50.0)
        .expect("Failed to add load");

    let mut digital_net = DigitalNetwork::new();
    let clk_node = digital_net.add_node(Some("CLK"));

    // Clock: 10 kHz (period 100 us, 50 us high, 50 us low)
    let clk = PeriodicClock::new(clk_node, 10_000.0);
    digital_net.add_clock(clk);

    // D2A Bridge: Vlow = 0V, Vhigh = 5V, rise_time = 10 us, fall_time = 10 us, Rout = 50 Ohms
    let d2a = D2aBridge::new(clk_node, n_out, n_gnd, 0.0, 5.0).with_timings(10e-6, 10e-6, 50.0);

    let mut circuit = MixedSignalCircuit::new(graph, digital_net);
    circuit.add_d2a(d2a);

    let transient_opts = TransientOptions {
        tstop: 150e-6, // 1.5 clock periods
        tstep: 1e-6,   // 1 us step to capture smooth 10 us ramp
        ..TransientOptions::default()
    };

    let options = MixedSignalOptions {
        transient: transient_opts,
        max_delta_cycles: 100,
        crossing_tolerance: 1e-10,
    };

    let context = ModelContext::new();
    let solution: MixedSignalSolution =
        solve_mixed_signal(&mut circuit, &context, &options).expect("Mixed-signal solve failed");

    // Thevenin loading: Rout = 50 Ohms, Rload = 50 Ohms
    // V_out = V_thev * 50 / (50 + 50) = 0.5 * V_thev
    // When high (5V): V_out = 2.5V. When low (0V): V_out = 0.0V.
    let ana_wave = solution.analog_node_waveform(n_out);

    // Check voltage at t=0 (initially low)
    let v_0 = ana_wave[0].1;
    assert!((v_0 - 0.0).abs() < 1e-3, "Initial voltage should be 0V");

    // Clock toggles High at t=0 (since initial_delay=0)
    // By t=10us, rise ramp finishes -> V_out should reach 2.5V
    let v_at_15us = ana_wave
        .iter()
        .min_by(|a, b| (a.0 - 15e-6).abs().total_cmp(&(b.0 - 15e-6).abs()))
        .unwrap()
        .1;
    assert!(
        (v_at_15us - 2.5).abs() < 0.05,
        "Voltage after rise time should be 2.5V, got {:.3}V",
        v_at_15us
    );

    // Halfway through rise at t=5us, V_out should be around 1.25V (smooth ramp)
    let v_at_5us = ana_wave
        .iter()
        .min_by(|a, b| (a.0 - 5e-6).abs().total_cmp(&(b.0 - 5e-6).abs()))
        .unwrap()
        .1;
    assert!(
        (v_at_5us - 1.25).abs() < 0.15,
        "Mid-ramp voltage at 5us should be ~1.25V, got {:.3}V",
        v_at_5us
    );

    // At t=50us, clock transitions Low. By t=65us, V_out should return to 0.0V
    let v_at_65us = ana_wave
        .iter()
        .min_by(|a, b| (a.0 - 65e-6).abs().total_cmp(&(b.0 - 65e-6).abs()))
        .unwrap()
        .1;
    assert!(
        (v_at_65us - 0.0).abs() < 0.05,
        "Voltage after fall time should be 0V, got {:.3}V",
        v_at_65us
    );
}
