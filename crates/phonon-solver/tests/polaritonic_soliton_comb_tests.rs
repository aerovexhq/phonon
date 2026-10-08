#![deny(unsafe_code)]

//! Automated verification tests for Phase 434: Polaritonic Soliton Frequency Comb &
//! Dissipative Kerr Squeezed State Generator.

use phonon_solver::polaritonic_soliton_comb::{
    KagomePolaritonParams, KagomePolaritonSolver, LugiatoLefeverSoliton,
    PolaritonicSolitonCombProcessor, QuantumNoiseQuadratures, SolitonCombParams,
    SqueezingParams,
};

#[test]
fn test_kagome_flat_band_and_dirac_point() {
    let params = KagomePolaritonParams::default();
    let solver = KagomePolaritonSolver::new(params.clone());
    let band_struct = solver.evaluate_band_structure();

    // 1. Verify 3 bands evaluated across high symmetry path
    assert!(!band_struct.points.is_empty());
    assert!(band_struct.points.len() >= 140);

    // 2. Verify flat-band flatness across the Brillouin zone
    assert!(
        band_struct.relative_flatness < 1e-4,
        "Flat band relative variation ({:e}) must be < 1e-4",
        band_struct.relative_flatness
    );

    // 3. Verify Dirac degeneracy at K-point (splitting < 1 kHz)
    assert!(
        band_struct.dirac_degeneracy_split_hz < 1000.0,
        "Dirac point splitting at K ({:.2} Hz) must be < 1000 Hz",
        band_struct.dirac_degeneracy_split_hz
    );

    // 4. Verify anomalous group velocity dispersion (D2 > 0)
    assert!(
        band_struct.gvd_d2_khz > 0.0,
        "GVD D2 ({:.2} kHz) must be anomalous (D2 > 0)",
        band_struct.gvd_d2_khz
    );
}

#[test]
fn test_dissipative_kerr_soliton_and_comb_spectrum() {
    let params = SolitonCombParams::default();
    let solver = LugiatoLefeverSoliton::new(params);
    let (metrics, profile, comb_modes) = solver.solve_soliton();

    // 1. Soliton profile validation
    assert!(!profile.is_empty());
    assert!(metrics.soliton_regime_valid);
    assert!(
        metrics.pulse_duration_fwhm_ps > 10.0 && metrics.pulse_duration_fwhm_ps < 100.0,
        "FWHM pulse duration ({:.2} ps) out of expected range",
        metrics.pulse_duration_fwhm_ps
    );

    // 2. High contrast ratio over CW background (>= 20 dB)
    assert!(
        metrics.contrast_ratio_db >= 20.0,
        "Soliton contrast ratio ({:.2} dB) must be >= 20 dB",
        metrics.contrast_ratio_db
    );

    // 3. Multi-octave frequency comb with >= 30 active lines above -40 dBc
    assert!(
        metrics.active_comb_lines >= 30,
        "Active comb line count ({}) must be >= 30",
        metrics.active_comb_lines
    );
    assert_eq!(comb_modes.len(), 65);

    // 4. Repetition rate equidistance and sub-10 fs jitter
    assert!(
        metrics.repetition_jitter_fs < 10.0,
        "Repetition jitter ({:.2} fs) must be < 10 fs",
        metrics.repetition_jitter_fs
    );
}

#[test]
fn test_quantum_noise_squeezing_and_sub_poissonian_stats() {
    let params = SqueezingParams::default();
    let solver = QuantumNoiseQuadratures::new(params);
    let (metrics, scan_points, wigner) = solver.evaluate_noise_and_wigner();

    // 1. Quadrature noise reduction below SQL (>= 6.0 dB)
    assert!(
        metrics.squeezing_db >= 6.0,
        "Squeezing level ({:.2} dB) must be >= 6.0 dB",
        metrics.squeezing_db
    );
    assert!(metrics.delta_x_min_sq < 0.5);

    // 2. Anti-squeezing along orthogonal quadrature
    assert!(metrics.delta_x_max_sq > 0.5);

    // 3. Sub-Poissonian phonon statistics (g^(2)(0) < 1.0)
    assert!(
        metrics.g2_zero < 1.0,
        "Second-order correlation g^(2)(0) ({:.3}) must be < 1.0",
        metrics.g2_zero
    );

    // 4. Negative Mandel Q parameter (Q_M < 0)
    assert!(
        metrics.mandel_q < 0.0,
        "Mandel Q ({:.3}) must be strictly negative",
        metrics.mandel_q
    );

    // 5. Wigner function phase-space ellipticity (ratio >= 2.0)
    assert!(
        metrics.wigner_ellipticity >= 2.0,
        "Wigner ellipticity ({:.2}) must be >= 2.0",
        metrics.wigner_ellipticity
    );

    assert_eq!(scan_points.len(), 120);
    assert_eq!(wigner.grid_size, 51);
    assert_eq!(wigner.values.len(), 51);
    assert_eq!(wigner.values[0].len(), 51);
}

#[test]
fn test_ten_point_physics_audit_full_pass() {
    let processor = PolaritonicSolitonCombProcessor::default();
    let report = processor.audit_processor();

    assert!(report.flat_band_flatness_pass, "Flat-band flatness failed");
    assert!(report.anomalous_gvd_pass, "Anomalous GVD failed");
    assert!(report.soliton_stability_pass, "Soliton stability failed");
    assert!(report.comb_line_count_pass, "Comb line count failed");
    assert!(report.repetition_equidistance_pass, "Repetition equidistance failed");
    assert!(report.quadrature_squeezing_pass, "Quadrature squeezing failed");
    assert!(report.sub_poissonian_stats_pass, "Sub-Poissonian stats failed");
    assert!(report.negative_mandel_q_pass, "Negative Mandel Q failed");
    assert!(report.wigner_ellipticity_pass, "Wigner ellipticity failed");
    assert!(report.soliton_contrast_pass, "Soliton contrast failed");

    assert_eq!(report.total_score, 10, "Total audit score must be 10/10");
    assert!(report.all_passed, "All 10 physics audit criteria must pass");
}
