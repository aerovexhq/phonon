//! Integration tests for Cryogenic Optical Interconnects and Multi-Neuron SOEN Networks.
//!
//! Validates:
//! - Optical waveguide delay scaling ($\tau = n_{eff} L / c$).
//! - Zero electronic crosstalk between adjacent optical waveguides ($> 70\text{ dB}$ isolation).
//! - Massive optical fanout capability ($> 1000$) without capacitive charging penalties.
//! - Multi-threaded recurrent SOEN network simulation using Rayon.

use phonon_solver::superconducting::{OpticalInterconnect, SoenNetwork};

#[test]
fn test_waveguide_propagation_delay_and_zero_crosstalk() {
    // 500 um long Si3N4 waveguide (n_eff = 2.0)
    let conn = OpticalInterconnect::new(8, 500.0e-6);

    // Theoretical delay: 2.0 * 500e-6 / 3e8 = 3.333 ps
    let expected_delay = (2.0 * 500.0e-6) / 299_792_458.0;
    assert!((conn.propagation_delay_s - expected_delay).abs() < 1e-13);

    // Dielectric waveguides have essentially zero electromagnetic crosstalk
    assert!(conn.crosstalk_isolation_db >= 75.0);

    // Optical fanout limit > 1000
    assert!(conn.max_fanout >= 1000);
}

#[test]
fn test_massive_optical_fanout_distribution() {
    let num_neurons = 16;
    let mut network = SoenNetwork::new(num_neurons, 100.0e-6);

    // Fanout: neuron 0 connects to all other 15 neurons
    for dst in 1..num_neurons {
        network.interconnect.set_weight(0, dst, 0.9);
    }

    // Fire neuron 0 with a strong input stimulus
    network.inject_stimulus(0, 0, 3.5);

    // Run 60 simulation steps (1 ps step)
    network.simulate(60, 1.0e-12);

    // Neuron 0 must have fired
    assert!(network.neurons[0].spike_count >= 1);

    // Downstream neurons should have received the fanned-out optical pulses
    let receiving_neurons_count = network.neurons[1..]
        .iter()
        .filter(|n| n.synaptic_events_processed > 0)
        .count();
    assert_eq!(
        receiving_neurons_count, 15,
        "All 15 fanout destinations must receive optical spikes"
    );
}

#[test]
fn test_multi_layer_feedforward_soen_network_simulation() {
    // 3-layer network: Layer 0 (neurons 0..2) -> Layer 1 (neurons 2..4) -> Layer 2 (neuron 4)
    let mut network = SoenNetwork::new(5, 120.0e-6);

    // Connect Layer 0 (0, 1) -> Layer 1 (2, 3)
    network.interconnect.connect_layers(0..2, 2..4, 1.0);
    // Connect Layer 1 (2, 3) -> Layer 2 (4)
    network.interconnect.connect_layers(2..4, 4..5, 1.0);

    // Stimulate Layer 0 neurons
    network.inject_stimulus(0, 0, 3.0);
    network.inject_stimulus(1, 1, 3.0);

    // Simulate for 150 ps (150 steps of 1 ps)
    network.simulate(150, 1.0e-12);

    // Verify propagation through layers
    assert!(network.neurons[0].spike_count >= 1);
    assert!(network.neurons[1].spike_count >= 1);
    assert!(
        network.total_spikes() >= 3,
        "Signal should propagate through intermediate and output layers"
    );

    let metrics = network.network_metrics();
    assert!(metrics.average_synaptic_energy_attojoules < 25.0);
    assert!(metrics.flux_retention_ns > 0.5);
}
