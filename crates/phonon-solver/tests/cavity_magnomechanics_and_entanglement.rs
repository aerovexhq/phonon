//! Integration tests for tripartite microwave cavity magnomechanics,
//! dynamical backaction ground-state cooling, and continuous-variable quantum entanglement.

use phonon_models::cavity_magnomechanics::CavityMagnomechanicalParams;
use phonon_solver::cavity_magnomechanics::LyapunovCovarianceSolver;

#[test]
fn test_magnomechanical_coupling_and_ground_state_cooling() {
    let params = CavityMagnomechanicalParams {
        cavity_frequency_ghz: 9.0,
        magnon_frequency_ghz: 9.0,
        phonon_frequency_mhz: 20.0,
        photon_damping_mhz: 2.5,
        magnon_damping_mhz: 1.5,
        phonon_damping_hz: 100.0,
        photon_magnon_coupling_mhz: 25.0,
        single_spin_magnetostriction_hz: 1.0,
        coherent_magnon_number: 1.0e11,
        bath_temperature_mk: 20.0,
    };
    let solver = LyapunovCovarianceSolver::new(params);
    let metrics = solver.solve_magnomechanical_metrics();

    // Linearized coupling G_mb = g_mb * sqrt(N_m) ~ 0.5 * sqrt(1e11) = 0.5 * 3.16e5 = 1.58e5 Hz = 0.158 MHz
    assert!(
        metrics.effective_coupling_g_mb_mhz > 0.05,
        "Coupling {:.3} MHz should be > 0.05 MHz",
        metrics.effective_coupling_g_mb_mhz
    );

    // Initial thermal phonon number at 20 mK
    assert!(
        metrics.thermal_phonon_number > 10.0,
        "Thermal phonon number {:.1} should be > 10",
        metrics.thermal_phonon_number
    );

    // Magnon-phonon cooperativity C_mb must be large (> 1.0)
    assert!(
        metrics.magnon_phonon_cooperativity > 10.0,
        "Cooperativity {:.1} should exceed 10.0",
        metrics.magnon_phonon_cooperativity
    );

    // Dynamical backaction cooling down to quantum ground state (n_eff < 1.0)
    assert!(
        metrics.effective_phonon_occupation < 1.0,
        "Effective phonon occupation {:.4} must be < 1.0 (ground state cooling)",
        metrics.effective_phonon_occupation
    );
}

#[test]
fn test_continuous_variable_quantum_entanglement_and_steering() {
    let params = CavityMagnomechanicalParams {
        cavity_frequency_ghz: 9.0,
        magnon_frequency_ghz: 9.0,
        phonon_frequency_mhz: 25.0,
        photon_damping_mhz: 2.0,
        magnon_damping_mhz: 1.2,
        phonon_damping_hz: 80.0,
        photon_magnon_coupling_mhz: 28.0,
        single_spin_magnetostriction_hz: 0.6,
        coherent_magnon_number: 2.0e11,
        bath_temperature_mk: 15.0,
    };
    let solver = LyapunovCovarianceSolver::new(params);
    let ent = solver.solve_entanglement_metrics();

    // Magnon-phonon logarithmic negativity E_{N, mb} must be strictly positive
    assert!(
        ent.magnon_phonon_log_negativity > 0.0,
        "Magnon-phonon log negativity {:.4} must be > 0",
        ent.magnon_phonon_log_negativity
    );

    // Photon-magnon logarithmic negativity E_{N, am} must be strictly positive
    assert!(
        ent.photon_magnon_log_negativity > 0.0,
        "Photon-magnon log negativity {:.4} must be > 0",
        ent.photon_magnon_log_negativity
    );

    // EPR steering parameter must be positive
    assert!(
        ent.quantum_steering_parameter > 0.0,
        "Steering parameter {:.4} must be > 0",
        ent.quantum_steering_parameter
    );

    // Quantum state fidelity >= 90%
    assert!(
        ent.quantum_state_fidelity >= 0.90,
        "State fidelity {:.4} must be >= 0.90",
        ent.quantum_state_fidelity
    );

    // Transduction efficiency >= 50%
    assert!(
        ent.transduction_efficiency_fraction >= 0.50,
        "Transduction efficiency {:.4} must be >= 0.50",
        ent.transduction_efficiency_fraction
    );

    // High force sensitivity: S_FF^(1/2) <= 1e-14 N / sqrt(Hz)
    assert!(
        ent.force_sensitivity_n_per_rt_hz <= 1.0e-14,
        "Force sensitivity {:.2e} N/rtHz should be <= 1e-14",
        ent.force_sensitivity_n_per_rt_hz
    );
}
