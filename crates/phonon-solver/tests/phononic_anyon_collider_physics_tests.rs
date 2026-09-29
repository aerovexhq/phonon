#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum phononic
//! non-Abelian anyon colliders and multi-qubit topological braiding interferometers.

use phonon_models::phononic_anyon_collider::PhononicAnyonColliderParams;
use phonon_solver::phononic_anyon_collider::PhononicAnyonColliderSolver;

#[test]
fn test_collision_visibility_bounds() {
    let params = PhononicAnyonColliderParams::default();
    let solver = PhononicAnyonColliderSolver::new(params);
    let vis = solver.compute_collision_visibility();

    // Target anyonic collision visibility >= 0.920 (92.0%)
    assert!(
        vis >= 0.920,
        "Anyonic collision visibility must be >= 0.920, got {:.4}",
        vis
    );
    assert!(
        vis <= 0.995,
        "Collision visibility cannot exceed physical maximum of 0.995, got {:.4}",
        vis
    );

    // Verify thermal scaling: lower temperature and dephasing yields higher visibility
    let cryo_params = PhononicAnyonColliderParams::new(4.2, 0.50, 85.0, 45.0, 280.0, 1.0, 5.0, 4);
    let cryo_solver = PhononicAnyonColliderSolver::new(cryo_params);
    assert!(
        cryo_solver.compute_collision_visibility() >= vis,
        "Lower dephasing and temperature should enhance collision visibility"
    );
}

#[test]
fn test_cross_correlation_noise_suppression_bounds() {
    let params = PhononicAnyonColliderParams::default();
    let solver = PhononicAnyonColliderSolver::new(params);
    let supp = solver.compute_cross_correlation_noise_suppression_db();

    // Target cross-correlation noise suppression >= 25.0 dB
    assert!(
        supp >= 25.0,
        "Cross-correlation noise suppression must be >= 25.0 dB, got {:.2} dB",
        supp
    );
    assert!(
        supp <= 45.0,
        "Noise suppression must remain within physical bound of 45.0 dB, got {:.2} dB",
        supp
    );

    // Verify topological gap scaling: larger gap increases noise suppression
    let high_gap_params = PhononicAnyonColliderParams::new(4.2, 0.50, 85.0, 45.0, 560.0, 5.0, 15.0, 4);
    let high_gap_solver = PhononicAnyonColliderSolver::new(high_gap_params);
    assert!(
        high_gap_solver.compute_cross_correlation_noise_suppression_db() >= supp,
        "Larger topological gap should enhance cross-correlation noise suppression"
    );
}

#[test]
fn test_braiding_phase_error_bounds() {
    let params = PhononicAnyonColliderParams::default();
    let solver = PhononicAnyonColliderSolver::new(params);
    let err = solver.compute_braiding_phase_error_rad();

    // Target braiding phase error <= 1.0e-4 rad
    assert!(
        err <= 1.0e-4,
        "Braiding phase error must be <= 1.0e-4 rad, got {:.4e} rad",
        err
    );
    assert!(
        err >= 1.0e-6,
        "Braiding phase error must be >= 1.0e-6 rad physical lower limit, got {:.4e} rad",
        err
    );

    // Verify scaling: shorter interferometer arm suppresses braiding phase error
    let short_arm_params = PhononicAnyonColliderParams::new(4.2, 0.50, 85.0, 20.0, 280.0, 5.0, 15.0, 4);
    let short_arm_solver = PhononicAnyonColliderSolver::new(short_arm_params);
    assert!(
        short_arm_solver.compute_braiding_phase_error_rad() <= err,
        "Shorter interferometer arm should decrease braiding phase error"
    );
}

#[test]
fn test_topological_parity_readout_fidelity_bounds() {
    let params = PhononicAnyonColliderParams::default();
    let solver = PhononicAnyonColliderSolver::new(params);
    let fid = solver.compute_topological_parity_readout_fidelity();

    // Target topological parity readout fidelity >= 0.998 (99.8%)
    assert!(
        fid >= 0.998,
        "Topological parity readout fidelity must be >= 0.998, got {:.5}",
        fid
    );
    assert!(
        fid <= 0.9999,
        "Readout fidelity must not exceed physical bound of 0.9999, got {:.5}",
        fid
    );

    // Verify dephasing dependence: lower dephasing yields higher readout fidelity
    let low_deph_params = PhononicAnyonColliderParams::new(4.2, 0.50, 85.0, 45.0, 280.0, 1.0, 15.0, 4);
    let low_deph_solver = PhononicAnyonColliderSolver::new(low_deph_params);
    assert!(
        low_deph_solver.compute_topological_parity_readout_fidelity() >= fid,
        "Lower dephasing rate should enhance topological parity readout fidelity"
    );
}

#[test]
fn test_anyonic_fano_factor_bounds() {
    let params = PhononicAnyonColliderParams::default();
    let solver = PhononicAnyonColliderSolver::new(params);
    let fano = solver.compute_anyonic_fano_factor();

    // Target anyonic Fano factor <= 0.35
    assert!(
        fano <= 0.35,
        "Anyonic Fano factor must be <= 0.35, got {:.4}",
        fano
    );
    assert!(
        fano >= 0.10,
        "Anyonic Fano factor must be >= 0.10, got {:.4}",
        fano
    );

    // Verify symmetry: 50:50 beam splitter achieves minimum Fano factor
    let asym_params = PhononicAnyonColliderParams::new(4.2, 0.80, 85.0, 45.0, 280.0, 5.0, 15.0, 4);
    let asym_solver = PhononicAnyonColliderSolver::new(asym_params);
    assert!(
        asym_solver.compute_anyonic_fano_factor() > fano,
        "Asymmetric beam splitter should exhibit higher Fano factor than symmetric 50:50 splitter"
    );
}

#[test]
fn test_parameter_clamping_and_defaults() {
    let def = PhononicAnyonColliderParams::default();
    assert_eq!(def.acoustic_frequency_ghz, 4.2);
    assert_eq!(def.splitter_reflectivity, 0.50);
    assert_eq!(def.anyon_wavepacket_width_ps, 85.0);
    assert_eq!(def.interferometer_arm_length_um, 45.0);
    assert_eq!(def.topological_gap_mhz, 280.0);
    assert_eq!(def.dephasing_rate_khz, 5.0);
    assert_eq!(def.operating_temp_m_k, 15.0);
    assert_eq!(def.qubit_count, 4);

    // Test out of range low clamping
    let clamped_low = PhononicAnyonColliderParams::new(0.2, 0.05, 5.0, 2.0, 20.0, 0.05, 0.5, 1);
    assert_eq!(clamped_low.acoustic_frequency_ghz, 1.0);
    assert_eq!(clamped_low.splitter_reflectivity, 0.1);
    assert_eq!(clamped_low.anyon_wavepacket_width_ps, 10.0);
    assert_eq!(clamped_low.interferometer_arm_length_um, 5.0);
    assert_eq!(clamped_low.topological_gap_mhz, 50.0);
    assert_eq!(clamped_low.dephasing_rate_khz, 0.1);
    assert_eq!(clamped_low.operating_temp_m_k, 1.0);
    assert_eq!(clamped_low.qubit_count, 2);

    // Test out of range high clamping
    let clamped_high = PhononicAnyonColliderParams::new(30.0, 0.99, 1000.0, 500.0, 2500.0, 500.0, 100.0, 64);
    assert_eq!(clamped_high.acoustic_frequency_ghz, 15.0);
    assert_eq!(clamped_high.splitter_reflectivity, 0.9);
    assert_eq!(clamped_high.anyon_wavepacket_width_ps, 500.0);
    assert_eq!(clamped_high.interferometer_arm_length_um, 200.0);
    assert_eq!(clamped_high.topological_gap_mhz, 1000.0);
    assert_eq!(clamped_high.dephasing_rate_khz, 100.0);
    assert_eq!(clamped_high.operating_temp_m_k, 50.0);
    assert_eq!(clamped_high.qubit_count, 16);
}

#[test]
fn test_full_metrics_physical_compliance() {
    let params = PhononicAnyonColliderParams::default();
    let solver = PhononicAnyonColliderSolver::new(params);
    let m = solver.evaluate_metrics();

    assert!(m.is_physically_compliant, "Default parameters must be physically compliant");
    assert!(m.collision_visibility >= 0.920);
    assert!(m.cross_correlation_noise_suppression_db >= 25.0);
    assert!(m.braiding_phase_error_rad <= 1.0e-4);
    assert!(m.topological_parity_readout_fidelity >= 0.998);
    assert!(m.anyonic_fano_factor <= 0.35);
}
