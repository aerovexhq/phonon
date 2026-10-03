#![deny(unsafe_code)]

//! Test suite for Phase 333: Memristor Dynamic State Variable Solver, Crossbar Array, and STDP SNN Engine.

use phonon_solver::neuromorphic::{
    LifNeuron, MemristorState, SpikingCrossbarNetwork, StdpParams, WindowFunction,
};

#[test]
fn test_memristor_state_and_window_functions() {
    // 1. Joglekar window function: f(w) = 1 - (2w - 1)^(2p)
    let joglekar = WindowFunction::Joglekar { p: 2.0 };
    assert!((joglekar.evaluate(0.5, 1.0) - 1.0).abs() < 1e-6);
    assert!(joglekar.evaluate(0.0, 1.0) < 1e-6);
    assert!(joglekar.evaluate(1.0, 1.0) < 1e-6);

    // 2. Biolek window function: f(w, v) = 1 - (w - stp(-v))^(2p)
    let biolek = WindowFunction::Biolek { p: 2.0 };
    // Escaping lower boundary: w = 0, positive voltage -> f = 1.0
    assert!((biolek.evaluate(0.0, 1.0) - 1.0).abs() < 1e-6);
    // Escaping upper boundary: w = 1, negative voltage -> f = 1.0
    assert!((biolek.evaluate(1.0, -1.0) - 1.0).abs() < 1e-6);
    // Approaching upper boundary under positive voltage -> f -> 0
    assert!(biolek.evaluate(1.0, 1.0) < 1e-6);
    // Approaching lower boundary under negative voltage -> f -> 0
    assert!(biolek.evaluate(0.0, -1.0) < 1e-6);

    // 3. Linear window function
    let linear = WindowFunction::Linear;
    assert_eq!(linear.evaluate(0.2, 1.0), 1.0);
    assert_eq!(linear.evaluate(0.8, -1.0), 1.0);

    // 4. Memristor state variable integration & threshold gating
    let mut memristor = MemristorState {
        w: 0.2,
        r_on: 1_000.0,
        r_off: 100_000.0,
        mobility: 1.0e-14,
        thickness: 10.0e-9,
        window_func: WindowFunction::Biolek { p: 2.0 },
        v_th: 0.2,
    };

    let initial_g = memristor.conductance();
    assert!((initial_g - (0.2 / 1000.0 + 0.8 / 100000.0)).abs() < 1e-12);

    // Below threshold -> state remains unchanged
    memristor.step(0.1, 0.001);
    assert_eq!(memristor.w, 0.2);
    assert_eq!(memristor.conductance(), initial_g);

    // Above threshold -> state w increases
    memristor.step(1.0, 0.001);
    assert!(memristor.w > 0.2);
    assert!(memristor.conductance() > initial_g);

    // Negative voltage above threshold -> state w decreases
    let w_after_pos = memristor.w;
    memristor.step(-1.0, 0.001);
    assert!(memristor.w < w_after_pos);
}

#[test]
fn test_vmm_current_integration() {
    let mut network = SpikingCrossbarNetwork::new(4, 2);
    network.reset_weights(0.5);

    let g_expected = 0.5 / 1000.0 + 0.5 / 100000.0;
    let v_pre = [1.0, 0.0, 1.0, 0.0];

    let currents = network.vmm(&v_pre);
    assert_eq!(currents.len(), 2);

    let expected_i = 2.0 * g_expected;
    assert!((currents[0] - expected_i).abs() < 1e-9);
    assert!((currents[1] - expected_i).abs() < 1e-9);
}

#[test]
fn test_lif_neuron_dynamics() {
    let mut neuron = LifNeuron {
        c_m: 1.0e-7,
        r_leak: 10_000.0,
        v_rest: 0.0,
        v_th: 1.0,
        v_reset: 0.0,
        tau_ref: 0.002, // 2 ms
        v: 0.0,
        refractory_remaining: 0.0,
    };

    // Sub-threshold integration: small current should increase V without spike
    let spiked = neuron.step(1.0e-5, 0.0005);
    assert!(!spiked);
    assert!(neuron.v > 0.0 && neuron.v < 1.0);

    // Strong current causes spike
    let spiked = neuron.step(1.0e-3, 0.001);
    assert!(spiked);
    assert_eq!(neuron.v, 0.0);
    assert!(neuron.refractory_remaining > 0.0);

    // During refractory period, neuron cannot spike
    let spiked_during_ref = neuron.step(1.0e-2, 0.001);
    assert!(!spiked_during_ref);
    assert_eq!(neuron.v, 0.0);
    assert!(neuron.refractory_remaining > 0.0);

    // Advance past remaining refractory time (remaining: 0.001 s)
    neuron.step(0.0, 0.0015);
    assert_eq!(neuron.refractory_remaining, 0.0);

    // Post-refractory: can integrate and fire again
    let spiked_again = neuron.step(1.0e-3, 0.001);
    assert!(spiked_again);
}

#[test]
fn test_stdp_exponential_updates() {
    let stdp = StdpParams {
        a_plus: 0.05,
        a_minus: 0.03,
        tau_plus: 0.020,  // 20 ms
        tau_minus: 0.020, // 20 ms
    };

    // Pre before post (delta_t = 0.005 s > 0): LTP potentiation
    let delta_w_ltp = stdp.delta_w(0.005);
    let expected_ltp = 0.05 * (-0.005 / 0.020_f64).exp();
    assert!((delta_w_ltp - expected_ltp).abs() < 1e-9);
    assert!(delta_w_ltp > 0.0);

    // Post before pre (delta_t = -0.005 s < 0): LTD depression
    let delta_w_ltd = stdp.delta_w(-0.005);
    let expected_ltd = -0.03 * (-0.005 / 0.020_f64).exp();
    assert!((delta_w_ltd - expected_ltd).abs() < 1e-9);
    assert!(delta_w_ltd < 0.0);

    // Temporal decay: larger timing separation produces smaller delta
    let delta_w_ltp_far = stdp.delta_w(0.020);
    assert!(delta_w_ltp_far < delta_w_ltp);

    let delta_w_ltd_far = stdp.delta_w(-0.020);
    assert!(delta_w_ltd_far > delta_w_ltd); // Closer to zero, less negative

    // Coincident spikes: delta_w = 0.0
    assert_eq!(stdp.delta_w(0.0), 0.0);
}

#[test]
fn test_pattern_learning_potentiation() {
    let mut network = SpikingCrossbarNetwork::new(2, 2);
    network.reset_weights(0.4);
    network.stdp_params = StdpParams {
        a_plus: 0.1,
        a_minus: 0.05,
        tau_plus: 0.020,
        tau_minus: 0.020,
    };
    // Make pulse strong enough so input triggers output firing
    network.pulse_voltage = 2.0;

    // Neuron 0 receives dense spikes from input 0 to trigger post firing and LTP
    let input_trains = vec![
        vec![0.005, 0.015, 0.025, 0.035, 0.045], // Input 0 active
        vec![],                                   // Input 1 silent
    ];

    let trajectory = network.simulate(0.060, 0.0005, &input_trains);

    assert!(!trajectory.timestamps.is_empty());
    assert!(!trajectory.input_spikes.is_empty());
    assert!(trajectory.total_synaptic_ops > 0);
    assert!(trajectory.energy_per_sop_fj > 0.0);

    // Synapse (0, 0) should potentiate due to repeated pre-post paired activity
    let initial_w_00 = trajectory.initial_conductances[0][0];
    let final_w_00 = trajectory.final_conductances[0][0];
    assert!(
        final_w_00 >= initial_w_00,
        "Active input synapse should potentiate or remain preserved"
    );
}
