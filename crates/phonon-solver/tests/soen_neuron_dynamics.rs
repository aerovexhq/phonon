//! Integration tests for Superconducting Optoelectronic Neuron (SOEN) dynamics.
//!
//! Validates:
//! - SNSPD hot-spot nucleation, exponential thermal relaxation, and load current diversion.
//! - Cryogenic optical emitter rate equations, threshold current, and photon pulse emission.
//! - Superconducting flux storage loop quantization and leaky memory retention.
//! - Full SOEN neuron action potential firing ($2\pi$ phase slip + self-reset) and sub-10 aJ energy accounting.

use phonon_core::FLUX_QUANTUM;
use phonon_models::superconducting::{
    CryoOpticalEmitter, SnspdModel, SoenNeuron, SuperconductingFluxLoop,
};

#[test]
fn test_snspd_microscopic_dynamics() {
    let snspd = SnspdModel::default();
    let i_bias = snspd.bias_current();
    assert!(i_bias > 20.0e-6 && i_bias < 25.0e-6);

    // Initial state before absorption: zero diverted current
    assert_eq!(snspd.diverted_current(0.0, 50.0), 0.0);

    // During hot-spot expansion (e.g. 15 ps after photon hit):
    let i_div_peak = snspd.diverted_current(15.0e-12, 50.0);
    assert!(i_div_peak > 15.0e-6);

    // After cooling (1.2 ns ~ 8 tau_th): diverted current should decay toward zero
    let i_div_late = snspd.diverted_current(1.2e-9, 50.0);
    assert!(i_div_late < 2.0e-6);
    assert!(i_div_late < 0.1 * i_div_peak);

    // Microscopic Joule dissipation during detection must be in the attojoule realm (< 10 aJ)
    let e_diss = snspd.dissipation_energy_joules(10.0);
    assert!(e_diss > 0.01e-18, "Dissipation must be positive");
    assert!(e_diss < 10.0e-18, "Dissipation must be < 10 aJ");
}

#[test]
fn test_cryo_optical_emitter_rate_equations() {
    let mut emitter = CryoOpticalEmitter::default();
    assert_eq!(emitter.carrier_density, 0.0);
    assert_eq!(emitter.photon_density, 0.0);

    // Single-photon energy at 850 nm should be ~ 1.46 eV (~ 2.34e-19 J)
    let e_ph_ev = emitter.photon_energy_ev();
    assert!((e_ph_ev - 1.458).abs() < 0.05);

    // Below threshold injection (0.5 uA): spontaneous emission dominant, low power
    let p_sub = emitter.step(1.0e-12, 0.5e-6);
    assert!(p_sub >= 0.0);

    // Driver pulse above threshold: emits an optical pulse packet
    let (pulse_energy, photons) = emitter.emit_pulse(5.0e-6, 5.0e-12);
    assert!(pulse_energy > 0.0);
    assert!(photons > 0.0);
    assert!(
        pulse_energy < 50.0e-18,
        "Pulse energy must be in attojoule range"
    );
}

#[test]
fn test_superconducting_flux_loop_retention_and_quantization() {
    let mut flux_loop = SuperconductingFluxLoop::new(50.0e-12, 0.025);
    // Leak time constant tau = L / R = 50 pH / 25 mOhm = 2.0 ns
    let tau_leak = flux_loop.leak_time_constant();
    assert!((tau_leak - 2.0e-9).abs() < 1e-11);

    // Quantized flux injection (+1 Phi_0)
    flux_loop.add_flux_quantum(1.0);
    assert_eq!(flux_loop.flux_quantum_count, 1);
    assert!((flux_loop.stored_flux - FLUX_QUANTUM).abs() < 1e-25);

    // Advance time by 0.5 ns with zero external voltage: flux decays exponentially
    let dt = 1.0e-11; // 10 ps steps
    for _ in 0..50 {
        flux_loop.step(dt, 0.0);
    }
    // After 0.5 ns (0.25 tau), current should be ~ exp(-0.25) ~ 0.778 of initial
    let expected_ratio = (-0.5 / 2.0_f64).exp();
    let initial_i = FLUX_QUANTUM / 50.0e-12;
    let actual_ratio = flux_loop.current / initial_i;
    assert!((actual_ratio - expected_ratio).abs() < 0.05);
}

#[test]
fn test_full_soen_neuron_spiking_and_energy() {
    let mut neuron = SoenNeuron::new(0, 4, 40.0e-6);
    assert_eq!(neuron.spike_count, 0);

    // Channel 0 weight = 1.0, Channel 1 weight = 0.5
    neuron.set_weight(0, 1.0);
    neuron.set_weight(1, 0.5);

    // Sub-threshold stimulus: 0.3 photons on channel 1 -> should not fire
    neuron.receive_synaptic_spikes(&[(1, 0.3)], 0.0);
    for step in 0..20 {
        let t = step as f64 * 1.0e-12;
        let fired = neuron.step(1.0e-12, t);
        assert!(fired.is_none());
    }
    assert_eq!(neuron.spike_count, 0);

    // Super-threshold stimulus: 2.0 photons on channel 0
    neuron.receive_synaptic_spikes(&[(0, 2.0)], 20.0e-12);
    let mut spike_fired = false;
    for step in 20..120 {
        let t = step as f64 * 1.0e-12;
        if let Some(energy) = neuron.step(1.0e-12, t) {
            spike_fired = true;
            assert!(energy > 0.0);
            break;
        }
    }
    assert!(
        spike_fired,
        "Neuron must fire upon super-threshold flux accumulation"
    );
    assert_eq!(neuron.spike_count, 1);

    // Energy metrics check
    let metrics = neuron.metrics();
    assert!(metrics.average_synaptic_energy_attojoules > 0.1);
    assert!(metrics.average_synaptic_energy_attojoules < 20.0);
    assert!(metrics.somatic_firing_latency_ps < 50.0);
    assert!(metrics.flux_retention_ns > 1.0);
}
