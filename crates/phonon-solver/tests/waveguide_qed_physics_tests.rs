//! Automated unit and physical validation tests for quantum acoustic waveguide QED.

use phonon_models::quantum_acoustic_waveguide::WaveguideQedParams;
use phonon_solver::quantum_acoustic_waveguide::QuantumAcousticWaveguideSolver;

#[test]
fn test_chiral_acoustic_emission_directionality() {
    let params = WaveguideQedParams::default();
    let solver = QuantumAcousticWaveguideSolver::new(params);
    let directionality = solver.compute_chiral_acoustic_directionality();

    // Target directionality >= 95.0%
    assert!(
        directionality >= 0.950,
        "Directionality must be >= 95.0%, got {:.3}%",
        directionality * 100.0
    );
    assert!(
        directionality <= 1.0,
        "Directionality cannot exceed unity, got {:.3}",
        directionality
    );

    let (gamma_r, gamma_l) = solver.compute_directional_emission_rates_mhz();
    assert!(
        gamma_r > 10.0 * gamma_l,
        "Rightward emission must dominate leftward by > 10x, got Gamma_R={:.2}, Gamma_L={:.2}",
        gamma_r,
        gamma_l
    );
}

#[test]
fn test_waveguide_purcell_enhancement_factor() {
    let params = WaveguideQedParams::default();
    let solver = QuantumAcousticWaveguideSolver::new(params);
    let purcell = solver.compute_waveguide_purcell_factor();

    // Target Purcell factor >= 80.0
    assert!(
        purcell >= 80.0,
        "Purcell enhancement factor must be >= 80.0, got {:.2}",
        purcell
    );
}

#[test]
fn test_bound_state_in_continuum_lifetime_extension() {
    let params = WaveguideQedParams::default();
    let solver = QuantumAcousticWaveguideSolver::new(params);
    let bic_ext = solver.compute_bound_state_lifetime_extension();

    // Target lifetime extension >= 50.0x
    assert!(
        bic_ext >= 50.0,
        "Bound state lifetime extension must be >= 50.0x, got {:.2}x",
        bic_ext
    );
}

#[test]
fn test_multi_qubit_acoustic_entanglement_concurrence() {
    let params = WaveguideQedParams::default();
    let solver = QuantumAcousticWaveguideSolver::new(params);
    let concurrence = solver.compute_acoustic_entanglement_concurrence();

    // Target concurrence >= 0.90
    assert!(
        concurrence >= 0.900,
        "Acoustic entanglement concurrence must be >= 0.900, got {:.3}",
        concurrence
    );
    assert!(
        concurrence <= 1.0,
        "Concurrence cannot exceed 1.0, got {:.3}",
        concurrence
    );
}

#[test]
fn test_non_markovian_retardation_delay() {
    let params = WaveguideQedParams::default();
    let solver = QuantumAcousticWaveguideSolver::new(params);
    let delay_ns = solver.compute_acoustic_retardation_delay_ns();

    // For L = 180 um, v_saw = 3488 m/s: tau = 180e-6 / 3488 = ~51.6 ns
    assert!(
        delay_ns > 30.0 && delay_ns < 100.0,
        "Acoustic retardation delay must be ~51.6 ns, got {:.2} ns",
        delay_ns
    );
}

#[test]
fn test_physical_compliance_flag() {
    let params = WaveguideQedParams::default();
    let solver = QuantumAcousticWaveguideSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be 100% physically compliant"
    );
}
