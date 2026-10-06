#![deny(unsafe_code)]

use phonon_solver::pt_symmetric_acoustic::{
    InvisibilityEngine, InvisibilityParams, PtAcousticParams, PtHamiltonianSolver,
    PtPhaseClassification,
};

#[test]
fn test_pt_hamiltonian_exact_and_broken_phases() {
    // Exact PT phase: gamma < kappa
    let exact_params = PtAcousticParams {
        coupling_kappa_hz: 500.0,
        gain_loss_gamma_hz: 300.0,
        background_loss_hz: 0.0,
        ..Default::default()
    };
    let exact_solver = PtHamiltonianSolver::new(exact_params);
    assert_eq!(exact_solver.params.phase(), PtPhaseClassification::ExactPtSymmetric);
    assert!(
        (exact_solver.metrics.frequency_splitting_hz - 800.0).abs() < 1e-4,
        "Frequency splitting should be 800 Hz, got {}",
        exact_solver.metrics.frequency_splitting_hz
    );
    assert_eq!(exact_solver.metrics.decay_splitting_hz, 0.0);
    assert!(exact_solver.metrics.petermann_factor > 1.0);

    // Exceptional point: gamma = kappa
    let ep_params = PtAcousticParams {
        coupling_kappa_hz: 500.0,
        gain_loss_gamma_hz: 500.0,
        background_loss_hz: 0.0,
        ..Default::default()
    };
    let ep_solver = PtHamiltonianSolver::new(ep_params);
    assert_eq!(ep_solver.params.phase(), PtPhaseClassification::ExceptionalPoint);
    assert!(ep_solver.metrics.frequency_splitting_hz < 1e-4);
    assert!(ep_solver.metrics.petermann_factor >= 100.0);

    // Broken PT phase: gamma > kappa
    let broken_params = PtAcousticParams {
        coupling_kappa_hz: 500.0,
        gain_loss_gamma_hz: 600.0,
        background_loss_hz: 0.0,
        ..Default::default()
    };
    let broken_solver = PtHamiltonianSolver::new(broken_params);
    assert_eq!(broken_solver.params.phase(), PtPhaseClassification::BrokenPtSymmetric);
    assert_eq!(broken_solver.metrics.frequency_splitting_hz, 0.0);
    assert!(broken_solver.metrics.decay_splitting_hz > 600.0);
}

#[test]
fn test_pt_unidirectional_invisibility_scattering() {
    let ep_params = InvisibilityParams {
        pt_params: PtAcousticParams {
            coupling_kappa_hz: 500.0,
            gain_loss_gamma_hz: 500.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let engine = InvisibilityEngine::new(ep_params);

    // At exceptional point: R_L -> 0 (reflectionless from left), R_R != 0 (reflective from right)
    assert!(
        engine.metrics.center_reflection_left_db <= -30.0,
        "Left reflection at EP should be <= -30 dB, got {:.2} dB",
        engine.metrics.center_reflection_left_db
    );
    assert!(
        engine.metrics.center_reflection_right_db >= -5.0,
        "Right reflection at EP should be >= -5 dB, got {:.2} dB",
        engine.metrics.center_reflection_right_db
    );
    assert!(
        engine.metrics.unidirectional_contrast_ratio >= 0.95,
        "Unidirectional contrast ratio should be >= 95%, got {:.3}",
        engine.metrics.unidirectional_contrast_ratio
    );
    assert!(
        engine.metrics.unidirectional_isolation_db >= 25.0,
        "Directional isolation should be >= 25 dB, got {:.1} dB",
        engine.metrics.unidirectional_isolation_db
    );
}

#[test]
fn test_pt_generalized_unitarity_relation() {
    let params = InvisibilityParams {
        pt_params: PtAcousticParams {
            coupling_kappa_hz: 500.0,
            gain_loss_gamma_hz: 450.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let engine = InvisibilityEngine::new(params);

    // Verify |T - 1| = sqrt(R_L * R_R) across all frequencies
    assert!(!engine.metrics.spectrum.is_empty());
    for pt in &engine.metrics.spectrum {
        assert!(
            pt.generalized_unitarity_error < 1e-4,
            "Generalized unitarity error at {} Hz exceeds 1e-4: {}",
            pt.frequency_hz,
            pt.generalized_unitarity_error
        );
    }
}

#[test]
fn test_s_matrix_eigenvalue_coalescence_at_ep() {
    let ep_params = InvisibilityParams {
        pt_params: PtAcousticParams {
            coupling_kappa_hz: 500.0,
            gain_loss_gamma_hz: 500.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let engine = InvisibilityEngine::new(ep_params);

    let s1 = engine.metrics.s_matrix_eigenvalue_1;
    let s2 = engine.metrics.s_matrix_eigenvalue_2;
    assert!(
        (s1 - s2).abs() < 0.05,
        "S-matrix eigenvalues should coalesce at EP, got s1 = {:.4}, s2 = {:.4}",
        s1,
        s2
    );
}

#[test]
fn test_1d_acoustic_pressure_profile_asymmetry() {
    let ep_params = InvisibilityParams {
        pt_params: PtAcousticParams {
            coupling_kappa_hz: 500.0,
            gain_loss_gamma_hz: 500.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let engine = InvisibilityEngine::new(ep_params);

    assert!(!engine.spatial_field.is_empty());

    // Left incidence has nearly constant envelope in input region (negligible standing wave)
    let left_input: Vec<f64> = engine
        .spatial_field
        .iter()
        .filter(|pt| pt.position_x_mm < -25.0)
        .map(|pt| pt.intensity_left_incidence)
        .collect();
    let min_left = left_input.iter().copied().fold(f64::INFINITY, f64::min);
    let max_left = left_input.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!((max_left - min_left) < 0.2);

    // Right incidence has prominent standing wave due to strong reflection
    let right_input: Vec<f64> = engine
        .spatial_field
        .iter()
        .filter(|pt| pt.position_x_mm > 25.0)
        .map(|pt| pt.intensity_right_incidence)
        .collect();
    let min_right = right_input.iter().copied().fold(f64::INFINITY, f64::min);
    let max_right = right_input.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        max_right / min_right.max(1e-4) > 1.8,
        "Right incidence should have standing wave ratio > 1.8, got {:.2}",
        max_right / min_right.max(1e-4)
    );
}
