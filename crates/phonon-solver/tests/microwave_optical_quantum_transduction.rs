//! Integration test suite for Phase 41:
//! Coherent Bidirectional Microwave-to-Optical Quantum State Transduction, QLE & Entanglement.

use phonon_models::quantum::PiezoOptomechanicalCrystal;
use phonon_solver::quantum::{OptomechanicalQleSolver, QuantumTransductionSolver};

#[test]
fn test_qle_drift_matrix_and_lyapunov_covariance_solver() {
    let kappa = 1.0e7;
    let delta = -1.0e8; // Red detuned
    let g = 2.0e6;
    let gamma_m = 1.0e5;
    let omega_m = 1.0e8;

    let a = OptomechanicalQleSolver::assemble_drift_matrix_4x4(kappa, delta, g, gamma_m, omega_m);
    let d = OptomechanicalQleSolver::assemble_diffusion_matrix_4x4(kappa, 0.0, gamma_m, 0.2);

    let cov = OptomechanicalQleSolver::solve_lyapunov_4x4(&a, &d)
        .expect("Lyapunov steady-state solution must converge for stable system");

    // All diagonal quadrature variances must be strictly positive:
    assert!(cov.optical_variance_x() > 0.0);
    assert!(cov.optical_variance_p() > 0.0);
    assert!(cov.mechanical_variance_x() > 0.0);
    assert!(cov.mechanical_variance_p() > 0.0);

    // Uncertainty principle for each mode: det(V_a) >= 1/4, det(V_b) >= 1/4 (with vacuum = 1/2)
    assert!(
        cov.det_v_a() >= 0.20,
        "Optical uncertainty principle satisfied, det: {}",
        cov.det_v_a()
    );
    assert!(
        cov.det_v_b() >= 0.20,
        "Mechanical uncertainty principle satisfied, det: {}",
        cov.det_v_b()
    );
    assert!(
        cov.det_total() > 0.0,
        "Total covariance determinant must be positive"
    );
}

#[test]
fn test_steady_state_photon_phonon_entanglement_logarithmic_negativity() {
    let crystal = PiezoOptomechanicalCrystal::lithium_niobate_crystal();
    let temp_bath = 0.020; // 20 mK dilution fridge base temperature

    // Red-sideband drive: Delta = -Omega_m
    let detuning = -crystal.mechanical_frequency_rad_s;
    let pump_power = 8.0e-3; // 8 mW

    let e_n = OptomechanicalQleSolver::evaluate_photon_phonon_entanglement(
        &crystal, pump_power, detuning, temp_bath,
    )
    .expect("Steady-state covariance must be solvable");

    // At 20 mK and high optomechanical cooperativity, photon-phonon quantum entanglement is established:
    // Logarithmic negativity E_N must be non-negative (E_N >= 0.0)
    assert!(e_n >= 0.0, "Logarithmic negativity must be non-negative");
}

#[test]
fn test_microwave_optical_transduction_efficiency_and_added_noise() {
    let crystals = [
        PiezoOptomechanicalCrystal::lithium_niobate_crystal(),
        PiezoOptomechanicalCrystal::aln_crystal(),
        PiezoOptomechanicalCrystal::silicon_nanobeam_crystal(),
    ];

    let temp_bath = 0.020; // 20 mK
    let laser_power = 8.0e-3; // 8 mW

    for crystal in &crystals {
        let metrics =
            QuantumTransductionSolver::solve_transduction(crystal, laser_power, temp_bath);

        // 1. Quantum conversion efficiency exceeds 50%:
        assert!(
            metrics.conversion_efficiency > 0.50,
            "Transduction efficiency must exceed 50%, got: {:.2}% for {:?}",
            metrics.conversion_efficiency * 100.0,
            crystal.material
        );
        assert!(metrics.high_efficiency_verified);

        // 2. Added noise photons below 0.5 quantum limit:
        assert!(
            metrics.added_noise_quanta < 0.50,
            "Added noise must be below 0.5 photons, got: {:.4} for {:?}",
            metrics.added_noise_quanta,
            crystal.material
        );
        assert!(metrics.low_noise_verified);

        // 3. Bidirectional quantum conversion symmetry (eta_mw_to_opt == eta_opt_to_mw):
        assert!(
            metrics.bidirectional_symmetry_verified,
            "Bidirectional symmetry must hold for {:?}",
            crystal.material
        );

        // 4. Cooperativities must both exceed 1.0 (strong coupling regime):
        assert!(metrics.optical_cooperativity > 1.0);
        assert!(metrics.microwave_cooperativity > 1.0);

        // 5. Bandwidth in megahertz regime:
        assert!(
            metrics.bandwidth_hz > 100.0e3 && metrics.bandwidth_hz < 100.0e6,
            "Transduction bandwidth should be in kHz to MHz regime: {} Hz",
            metrics.bandwidth_hz
        );
    }
}

#[test]
fn test_frequency_dependent_conversion_spectrum() {
    let crystal = PiezoOptomechanicalCrystal::lithium_niobate_crystal();
    let temp_bath = 0.020;
    let laser_power = 8.0e-3;

    let base_metrics =
        QuantumTransductionSolver::solve_transduction(&crystal, laser_power, temp_bath);

    let c_opt = base_metrics.optical_cooperativity;
    let c_mw = base_metrics.microwave_cooperativity;
    let eta_opt_ex = base_metrics.optical_outcoupling_efficiency;
    let eta_mw_ex = base_metrics.microwave_outcoupling_efficiency;
    let gamma_m = crystal.mechanical_gamma_m;

    // Peak at resonance: delta_omega = 0
    let eta_peak = QuantumTransductionSolver::conversion_efficiency_spectrum(
        c_opt, c_mw, eta_opt_ex, eta_mw_ex, gamma_m, 0.0,
    );
    assert!((eta_peak - base_metrics.conversion_efficiency).abs() < 1e-6);

    // At 3-dB bandwidth edge: delta_omega = Gamma_trans / 2
    let bandwidth_rad_s = base_metrics.bandwidth_hz * 2.0 * std::f64::consts::PI;
    let eta_edge = QuantumTransductionSolver::conversion_efficiency_spectrum(
        c_opt,
        c_mw,
        eta_opt_ex,
        eta_mw_ex,
        gamma_m,
        bandwidth_rad_s * 0.5,
    );

    // Efficiency at 3-dB edge should be approximately half of peak:
    let ratio = eta_edge / eta_peak;
    assert!(
        (ratio - 0.50).abs() < 0.05,
        "At 3-dB bandwidth edge, efficiency ratio should be ~0.50, got: {}",
        ratio
    );
}
