//! Integration tests for RK4 coupled-mode spatial propagation and quantum noise analysis.

use phonon_models::jtwpa::ParametricProcessParams;
use phonon_solver::jtwpa::{CoupledModeSolver, QuantumNoiseSolver};

#[test]
fn test_coupled_mode_rk4_propagation_and_manley_rowe() {
    let params = ParametricProcessParams::standard_3wm_amplifier();
    let res = CoupledModeSolver::solve(&params, 100);

    assert!(
        res.signal_gain_db > 18.0,
        "Signal gain should approach/exceed 20 dB, got {} dB",
        res.signal_gain_db
    );
    assert!(res.signal_gain_linear > 50.0);
    assert!(res.idler_gain_linear > 49.0);

    assert!(
        res.manley_rowe_error < 1.0e-6,
        "Manley-Rowe photon balance |G_s - G_i - 1| must be < 1e-6, got {}",
        res.manley_rowe_error
    );
}

#[test]
fn test_quantum_noise_and_caves_limit() {
    let params = ParametricProcessParams::standard_3wm_amplifier();
    let mode_res = CoupledModeSolver::solve(&params, 100);

    let noise = QuantumNoiseSolver::evaluate(&params, mode_res.signal_gain_linear);

    assert!(
        noise.commutator_preservation_error < 1.0e-10,
        "Bosonic field commutator must be strictly preserved"
    );

    assert!(
        noise.added_noise_quanta >= noise.caves_limit_quanta,
        "Added noise quanta must satisfy Caves theorem N_add >= N_caves"
    );

    assert!(
        noise.added_noise_quanta < 0.505,
        "At 20 mK and high gain, added noise should be near quantum limit ~0.5 quanta, got {}",
        noise.added_noise_quanta
    );

    assert!(noise.added_noise_temperature_k > 0.0);
    assert!(
        (noise.noise_figure_db - 3.01).abs() < 0.1,
        "Noise figure at quantum limit should be ~3.0 dB, got {} dB",
        noise.noise_figure_db
    );
}
