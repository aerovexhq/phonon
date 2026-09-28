//! Integration tests for Phase 44: Memristive Reservoir Computing & Echo State Property.

use phonon_models::memristor::reservoir::{
    DelayOscillatorType, DelayedFeedbackReservoir, MemristiveNonIdealityConfig,
    MemristiveReservoir, MemristorTechnology, ReservoirActivation, ReservoirRng,
};

#[test]
fn test_spectral_radius_and_rescaling_esp() {
    let technologies = [
        MemristorTechnology::FilamentaryRram,
        MemristorTechnology::PhaseChangeMemory,
        MemristorTechnology::FerroelectricFet,
    ];

    for (idx, tech) in technologies.iter().enumerate() {
        let mut res = MemristiveReservoir::new(30, 1, *tech, 0.90, 0.4, 100 + idx as u64);
        let rho = res.compute_spectral_radius();
        assert!(
            (rho - 0.90).abs() < 2e-3,
            "Target spectral radius should be ~0.90, got {rho} for {tech:?}"
        );
        assert!(rho < 1.0, "ESP requires rho < 1.0");

        // Rescale to 0.70
        res.scale_spectral_radius(0.70);
        let rho_scaled = res.compute_spectral_radius();
        assert!(
            (rho_scaled - 0.70).abs() < 2e-3,
            "Rescaled spectral radius should be ~0.70, got {rho_scaled} for {tech:?}"
        );
    }
}

#[test]
fn test_echo_state_property_asymptotic_convergence() {
    // Two identical reservoirs with different initial states must synchronize under the same input driving
    let mut res1 = MemristiveReservoir::new(
        15,
        1,
        MemristorTechnology::PhaseChangeMemory,
        0.80,
        0.5,
        123,
    );
    let mut res2 = res1.clone();

    // Perturb initial states of res2
    for s in res2.state.iter_mut() {
        *s = 0.95;
    }

    // Drive both reservoirs with identical time-varying input sequence
    for t in 0..250 {
        let u = [((t as f64) * 0.1).sin()];
        res1.step(&u);
        res2.step(&u);
    }

    // Measure Euclidean difference between states
    let mut diff_sq = 0.0;
    for i in 0..res1.num_reservoir_nodes {
        diff_sq += (res1.state[i] - res2.state[i]).powi(2);
    }
    let diff = diff_sq.sqrt();

    assert!(
        diff < 1e-2,
        "Reservoirs must asymptotically synchronize (Echo State Property), diff = {diff}"
    );
}

#[test]
fn test_fading_memory_capacity() {
    let mut rng = ReservoirRng::new(999);
    let n_steps = 500;
    let mut inputs = Vec::with_capacity(n_steps);
    for _ in 0..n_steps {
        inputs.push(rng.next_range(-0.5, 0.5));
    }

    let mut res = MemristiveReservoir::new(
        50,
        1,
        MemristorTechnology::PhaseChangeMemory,
        0.95,
        0.8,
        1234,
    );

    let mc = res.calculate_memory_capacity(&inputs, 20);
    assert!(
        mc > 5.0,
        "Fading memory capacity should exceed 5.0 for 50-node reservoir, got {mc}"
    );
}

#[test]
fn test_delayed_feedback_reservoir_dynamics() {
    let oscillator_types = [DelayOscillatorType::MackeyGlass, DelayOscillatorType::Ikeda];

    for osc in oscillator_types {
        let mut dfr = DelayedFeedbackReservoir::new(32, 10.0e-6, osc, 42);
        assert_eq!(dfr.num_virtual_nodes, 32);

        // Run steps and verify virtual state dimensions and temporal evolution
        let state1 = dfr.step(0.4);
        assert_eq!(state1.len(), 32);

        let state2 = dfr.step(0.7);
        assert_eq!(state2.len(), 32);

        assert_ne!(state1, state2, "States should evolve with input changes");
    }
}

#[test]
fn test_sub_femtojoule_energy_accounting() {
    let mut res =
        MemristiveReservoir::new(16, 1, MemristorTechnology::FerroelectricFet, 0.80, 0.5, 777);
    res.non_idealities = MemristiveNonIdealityConfig {
        read_voltage_v: 0.05,      // 50 mV
        pulse_duration_s: 5.0e-9,  // 5 ns
        d2d_variation_ratio: 0.02, // 2%
        c2c_variation_ratio: 0.01, // 1%
        enable_sneak_paths: true,
        sneak_path_conductance_s: 1.0e-9, // 1 nS
        enable_tunneling_nonlinearity: true,
    };

    let (_, step_energy) = res.step(&[0.3]);
    assert!(step_energy > 0.0, "Step energy must be positive");

    let num_synapses = (res.num_reservoir_nodes * res.num_reservoir_nodes
        + res.num_reservoir_nodes * res.num_inputs) as f64;
    let avg_synaptic_energy = step_energy / num_synapses;

    // Sub-femtojoule to few femtojoules per synapse
    assert!(
        avg_synaptic_energy < 100.0e-15,
        "Average energy per synaptic event should be < 100 fJ, got {avg_synaptic_energy:e} J"
    );
}

#[test]
fn test_activations_and_non_idealities() {
    let activations = [
        ReservoirActivation::Tanh,
        ReservoirActivation::Sigmoid,
        ReservoirActivation::Relu,
        ReservoirActivation::MemristiveNonLinear,
    ];

    for act in activations {
        let mut res =
            MemristiveReservoir::new(10, 1, MemristorTechnology::FilamentaryRram, 0.8, 0.5, 11);
        res.activation = act;
        let (state, _) = res.step(&[0.5]);
        for &s in state {
            assert!(
                s.is_finite(),
                "Activation {act:?} produced non-finite state: {s}"
            );
        }
    }
}
