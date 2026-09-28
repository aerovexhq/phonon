#![allow(clippy::needless_range_loop)]
//! Integration tests for Phase 44: Spiking Liquid State Machine & STDP Memristive Plasticity.

use phonon_solver::neuromorphic::{LiquidStateMachine, LsmConfig};

#[test]
fn test_3d_cortical_lsm_connectivity_and_balance() {
    let config = LsmConfig {
        dimensions: (3, 3, 3), // 27 neurons
        num_inputs: 2,
        inhibitory_ratio: 0.25, // 25% GABAergic
        ..Default::default()
    };
    let lsm = LiquidStateMachine::new(config, 42);

    assert_eq!(lsm.num_neurons, 27);
    assert_eq!(lsm.neurons.len(), 27);
    assert_eq!(lsm.neuron_coords.len(), 27);

    // Verify distance coordinates
    assert_eq!(lsm.neuron_coords[0], (0.0, 0.0, 0.0));
    assert_eq!(lsm.neuron_coords[26], (2.0, 2.0, 2.0));

    // Verify inhibitory vs excitatory count
    let num_inhibitory = lsm.is_excitatory.iter().filter(|&&e| !e).count();
    let num_excitatory = lsm.is_excitatory.iter().filter(|&&e| e).count();
    assert!(num_inhibitory > 0, "Must have inhibitory neurons");
    assert!(
        num_excitatory > num_inhibitory,
        "Excitatory neurons must dominate"
    );

    // Recurrent weight matrix must have zero diagonal (no self-connections)
    for i in 0..27 {
        assert_eq!(
            lsm.w_rec[i][i], 0.0,
            "No self-connections allowed at index {i}"
        );
    }
}

#[test]
fn test_lsm_spiking_and_liquid_state_trajectory() {
    let config = LsmConfig {
        dimensions: (3, 3, 3),
        num_inputs: 1,
        synaptic_current_scale: 5.0e-5,
        ..Default::default()
    };
    let mut lsm = LiquidStateMachine::new(config, 101);

    // Step with strong input driving to trigger spiking activity
    let dt = 1.0e-4; // 100 us
    let mut total_spikes_seen = 0;

    for _ in 0..200 {
        let state = lsm.step(&[1.5], dt);
        assert_eq!(state.len(), 27);
        for &val in state {
            assert!(val.is_finite(), "Liquid state element must be finite");
            assert!(val >= 0.0, "Liquid state trace must be non-negative");
        }
        total_spikes_seen = lsm.total_spikes;
    }

    assert!(
        total_spikes_seen > 0,
        "LSM should emit action potentials under strong excitation, got {total_spikes_seen}"
    );
    assert!(
        lsm.cumulative_energy_joules > 0.0,
        "Cumulative energy dissipation must be positive"
    );
}

#[test]
fn test_online_stdp_synaptic_weight_adaptation() {
    let config = LsmConfig {
        dimensions: (2, 2, 2), // 8 neurons
        num_inputs: 1,
        synaptic_current_scale: 1.0e-4,
        enable_stdp: true,
        ..Default::default()
    };
    let mut lsm = LiquidStateMachine::new(config, 202);

    // Record initial recurrent weights
    let initial_w_rec = lsm.w_rec.clone();

    // Run dynamic stimulation over several ms
    let dt = 5.0e-5;
    for t in 0..400 {
        let input = [1.0 + ((t as f64) * 0.1).sin()];
        lsm.step(&input, dt);
    }

    // Verify STDP modified synaptic conductances
    let mut weights_changed = false;
    for i in 0..lsm.num_neurons {
        for j in 0..lsm.num_neurons {
            if (lsm.w_rec[i][j] - initial_w_rec[i][j]).abs() > 1e-9 {
                weights_changed = true;
                break;
            }
        }
    }

    assert!(
        weights_changed,
        "Online STDP must adapt synaptic weights during spiking activity"
    );
}

#[test]
fn test_lsm_reset_and_determinism() {
    let config = LsmConfig {
        dimensions: (2, 2, 2),
        num_inputs: 1,
        enable_stdp: false, // Freeze weights for determinism test
        ..Default::default()
    };
    let mut lsm = LiquidStateMachine::new(config, 303);

    let dt = 1.0e-4;
    let mut trajectory1 = Vec::new();
    for _ in 0..50 {
        trajectory1.push(lsm.step(&[1.0], dt).to_vec());
    }

    lsm.reset();
    assert_eq!(lsm.current_time_s, 0.0);
    for &s in &lsm.liquid_state {
        assert_eq!(s, 0.0);
    }

    let mut trajectory2 = Vec::new();
    for _ in 0..50 {
        trajectory2.push(lsm.step(&[1.0], dt).to_vec());
    }

    for (t, (s1, s2)) in trajectory1.iter().zip(trajectory2.iter()).enumerate() {
        for (i, (&v1, &v2)) in s1.iter().zip(s2.iter()).enumerate() {
            assert!(
                (v1 - v2).abs() < 1e-12,
                "LSM execution must be strictly deterministic: step {t}, neuron {i}: {v1} vs {v2}"
            );
        }
    }
}
