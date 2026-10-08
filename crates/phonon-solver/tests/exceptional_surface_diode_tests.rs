#![deny(unsafe_code)]

//! Analytical Unit Tests for Phase 436: Non-Hermitian Exceptional Surface
//! Chiral Phonon Diode & Unidirectional Quantum Repeater.

use phonon_solver::{
    ChiralDiodeParams, ChiralDiodeSolver, ChiralExceptionalSurfaceParams,
    ChiralExceptionalSurfaceSolver, ExceptionalSurfaceDiodeProcessor,
    QuantumRepeaterParams, QuantumRepeaterSolver,
};

#[test]
fn test_exceptional_surface_coalescence_and_spectrum() {
    let params = ChiralExceptionalSurfaceParams::default();
    let solver = ChiralExceptionalSurfaceSolver::new(params);
    let (metrics, points, fermi_arcs) = solver.solve_spectrum();

    assert!(!points.is_empty(), "Spectrum points should not be empty");
    assert!(!fermi_arcs.is_empty(), "Fermi arc segments should not be empty");
    assert!(
        metrics.es_points_count > 0,
        "Should identify points on the Exceptional Surface manifold, got {}",
        metrics.es_points_count
    );
    assert!(
        metrics.min_splitting_mhz <= 15.0,
        "Minimum splitting on ES should be small (coalescence), got {} MHz",
        metrics.min_splitting_mhz
    );
    assert!(
        metrics.fermi_arc_points_count > 0,
        "Should identify points with non-Hermitian broken PT / Fermi arc behavior"
    );
    assert!(
        metrics.max_im_energy_mhz >= 10.0,
        "Maximum imaginary energy component should be significant, got {} MHz",
        metrics.max_im_energy_mhz
    );
}

#[test]
fn test_square_root_branch_cut_scaling() {
    let params = ChiralExceptionalSurfaceParams::default();
    let solver = ChiralExceptionalSurfaceSolver::new(params);

    let dk_samples = vec![0.001, 0.004, 0.009, 0.016];
    let results = solver.evaluate_branch_cut_scaling(&dk_samples);

    assert_eq!(results.len(), 4);
    let split_1 = results[0].1;
    let split_4 = results[3].1;

    // dk ratio is 16:1, so sqrt(dk) ratio should be approx 4:1
    let ratio = split_4 / split_1;
    assert!(
        ratio >= 3.0 && ratio <= 5.5,
        "Splitting should follow square-root power-law scaling (expected ~4.0, got {})",
        ratio
    );
}

#[test]
fn test_group_velocity_asymmetry_and_fermi_arcs() {
    let params = ChiralExceptionalSurfaceParams::default();
    let solver = ChiralExceptionalSurfaceSolver::new(params);
    let (metrics, _, fermi_arcs) = solver.solve_spectrum();

    assert!(
        metrics.velocity_asymmetry_ratio >= 5.0,
        "Forward vs backward velocity asymmetry ratio should be >= 5.0, got {}",
        metrics.velocity_asymmetry_ratio
    );
    assert!(
        metrics.forward_group_velocity_ms > metrics.backward_group_velocity_ms,
        "Forward group velocity ({} m/s) must exceed backward group velocity ({} m/s)",
        metrics.forward_group_velocity_ms,
        metrics.backward_group_velocity_ms
    );

    // Verify Fermi arc properties
    for arc in &fermi_arcs {
        assert!(
            arc.forward_group_velocity > arc.backward_group_velocity,
            "Forward group velocity on Fermi arc must exceed backward group velocity"
        );
    }
}

#[test]
fn test_chiral_diode_s_parameters_and_rectification() {
    let params = ChiralDiodeParams::default();
    let solver = ChiralDiodeSolver::new(params);
    let (metrics, s_points, spatial_points) = solver.solve_diode();

    assert_eq!(metrics.center_freq_ghz, 5.0);
    assert!(
        metrics.peak_insertion_loss_db <= 0.50,
        "Forward insertion loss must be <= 0.50 dB, got {} dB",
        metrics.peak_insertion_loss_db
    );
    assert!(
        metrics.peak_isolation_db >= 35.0,
        "Backward isolation must be >= 35.0 dB, got {} dB",
        metrics.peak_isolation_db
    );
    assert!(
        metrics.rectification_contrast_db >= 35.0,
        "Rectification contrast must be >= 35.0 dB, got {} dB",
        metrics.rectification_contrast_db
    );
    assert!(
        metrics.return_loss_db >= 20.0,
        "Port return loss must be >= 20.0 dB, got {} dB",
        metrics.return_loss_db
    );

    assert!(!s_points.is_empty());
    assert!(!spatial_points.is_empty());

    // Spatial mode profiles
    let first = &spatial_points[0];
    let last = &spatial_points[spatial_points.len() - 1];

    // Forward wave enters at x=0 (amplitude 1.0) and reaches x=L with high amplitude (>0.90)
    assert!(first.forward_pressure_amplitude >= 0.99);
    assert!(
        last.forward_pressure_amplitude >= 0.90,
        "Forward wave should transmit with low attenuation, got {}",
        last.forward_pressure_amplitude
    );

    // Backward wave enters at x=L (dist=0) and is heavily attenuated reaching x=0 (dist=L)
    assert!(
        first.backward_pressure_amplitude <= 0.05,
        "Backward wave should be heavily attenuated at x=0, got {}",
        first.backward_pressure_amplitude
    );
}

#[test]
fn test_quantum_repeater_fidelity_and_backscatter_shielding() {
    let params = QuantumRepeaterParams::default();
    let solver = QuantumRepeaterSolver::new(params);

    // Unshielded node (0 dB isolation)
    let (metrics_unshielded, _) = solver.solve_repeater(0.0);

    // Shielded node (38.5 dB diode isolation)
    let (metrics_shielded, time_points) = solver.solve_repeater(38.5);

    assert!(
        metrics_shielded.bell_pair_fidelity >= 0.980,
        "Shielded Bell pair fidelity must be >= 0.980, got {}",
        metrics_shielded.bell_pair_fidelity
    );
    assert!(
        metrics_shielded.bell_pair_fidelity > metrics_unshielded.bell_pair_fidelity,
        "Shielded node must exhibit higher fidelity than unshielded"
    );
    assert!(
        metrics_shielded.quantum_memory_t2_us >= 50.0,
        "Protected quantum memory T2* must be >= 50.0 us, got {} us",
        metrics_shielded.quantum_memory_t2_us
    );
    assert!(
        metrics_shielded.secret_key_rate_kbps >= 1.0,
        "Secret key rate should be >= 1.0 kbps, got {} kbps",
        metrics_shielded.secret_key_rate_kbps
    );
    assert!(
        metrics_shielded.cryogenic_thermal_occupancy < 1e-4,
        "Cryogenic thermal occupancy at 20 mK must be negligible, got {}",
        metrics_shielded.cryogenic_thermal_occupancy
    );

    assert!(!time_points.is_empty());
}

#[test]
fn test_10_point_physics_audit_checklist_full_pass() {
    let processor = ExceptionalSurfaceDiodeProcessor::default();
    let report = processor.audit_system();

    assert!(report.exceptional_surface_coalescence_pass, "Criterion 1 failed");
    assert!(report.square_root_branch_cut_pass, "Criterion 2 failed");
    assert!(report.anisotropic_fermi_arcs_pass, "Criterion 3 failed");
    assert!(report.group_velocity_asymmetry_pass, "Criterion 4 failed");
    assert!(report.chiral_diode_insertion_loss_pass, "Criterion 5 failed");
    assert!(report.chiral_diode_isolation_pass, "Criterion 6 failed");
    assert!(report.rectification_contrast_pass, "Criterion 7 failed");
    assert!(report.return_loss_match_pass, "Criterion 8 failed");
    assert!(report.quantum_repeater_fidelity_pass, "Criterion 9 failed");
    assert!(report.cryogenic_noise_suppression_pass, "Criterion 10 failed");

    assert_eq!(report.total_score, 10, "Total score should be 10/10, got {}", report.total_score);
    assert!(report.all_passed, "All 10 physics audit criteria must pass");
}
