#![deny(unsafe_code)]

//! Test suite for Cavity Optomagnonic Polariton Frequency Comb & Dissipative Kerr Soliton Generator.

use phonon_solver::optomagnonic_comb::{
    JitterAnalysisParams, LlePolaritonParams, LlePolaritonSolver, OptomagnonicCombProcessor,
    TimingJitterSolver, TripleResonanceParams, TripleResonanceSolver,
};

#[test]
fn test_triple_resonance_phase_matching_and_energy_conservation() {
    let params = TripleResonanceParams::default();
    let result = TripleResonanceSolver::solve(&params);

    // Default pump = 193.4 THz, Stokes = 193.375 THz => separation = 25.0 GHz
    // Magnon = 10.0 GHz, Phonon = 15.0 GHz => sum = 25.0 GHz
    assert!((result.optical_separation_ghz - 25.0).abs() < 1e-6);
    assert!((result.sum_excitation_ghz - 25.0).abs() < 1e-6);
    assert!(result.detuning_error_mhz < 1e-3);
    assert!(result.is_triple_resonant);
    assert!(result.resonance_margin_mhz > 20.0);
}

#[test]
fn test_polariton_avoided_crossing_gap() {
    let params = TripleResonanceParams::default();
    let result = TripleResonanceSolver::solve(&params);

    // Delta_E = 2 * sqrt(g_om^2 + g_am^2)
    // g_om = 0.05 MHz, g_am = 20.0 MHz
    let expected_gap = 2.0 * (0.05 * 0.05 + 20.0 * 20.0_f64).sqrt();
    assert!((result.avoided_crossing_gap_mhz - expected_gap).abs() < 1e-3);
    assert!(result.avoided_crossing_gap_mhz >= 39.0);

    // Check polariton branches
    assert_eq!(result.polariton_branches.len(), 3);
    let lp = &result.polariton_branches[0];
    let mp = &result.polariton_branches[1];
    let up = &result.polariton_branches[2];

    assert_eq!(lp.name, "Lower Polariton");
    assert_eq!(mp.name, "Middle Polariton");
    assert_eq!(up.name, "Upper Polariton");

    assert!(lp.eigenfrequency_shift_mhz < 0.0);
    assert!(mp.eigenfrequency_shift_mhz.abs() < 1e-6);
    assert!(up.eigenfrequency_shift_mhz > 0.0);

    // Check normalization of fractions
    for b in &result.polariton_branches {
        let sum_frac = b.photon_fraction + b.magnon_fraction + b.phonon_fraction;
        assert!((sum_frac - 1.0).abs() < 1e-6);
        assert!(b.effective_linewidth_mhz > 0.0);
    }
}

#[test]
fn test_lle_polariton_split_step_and_dissipative_soliton() {
    let params = LlePolaritonParams::default();
    let result = LlePolaritonSolver::solve(&params);

    assert!(result.is_soliton_formed);
    assert!(result.peak_intensity > 2.0 * result.cw_background_intensity);
    assert!(result.sech2_fit_residual_r2 >= 0.90);
    assert!(result.intracavity_energy > 0.5);
}

#[test]
fn test_optical_comb_line_generation_and_span() {
    let params = LlePolaritonParams::default();
    let result = LlePolaritonSolver::solve(&params);

    // Comb mode count >= 50 modes within 30-dB dynamic range
    assert!(result.comb_mode_count_30db >= 50, "Expected >= 50 modes within 30 dB, got {}", result.comb_mode_count_30db);

    // 3-dB optical bandwidth >= 500 GHz
    assert!(
        result.optical_bandwidth_3db_ghz >= 500.0,
        "Expected >= 500 GHz bandwidth, got {:.1} GHz",
        result.optical_bandwidth_3db_ghz
    );

    // Center pump mode mu = 0 should have high power
    let center_mode = result.comb_modes.iter().find(|m| m.mode_index == 0);
    assert!(center_mode.is_some());
    assert!(center_mode.unwrap().power_dbm > -20.0);
}

#[test]
fn test_sech2_soliton_pulse_duration() {
    let params = LlePolaritonParams::default();
    let result = LlePolaritonSolver::solve(&params);

    // Pulse duration tau_FWHM <= 500 fs
    assert!(
        result.pulse_duration_fwhm_fs <= 500.0,
        "Expected tau_FWHM <= 500 fs, got {:.1} fs",
        result.pulse_duration_fwhm_fs
    );
    assert!(result.pulse_duration_fwhm_fs >= 50.0);
}

#[test]
fn test_integrated_timing_jitter_sub_femtosecond() {
    let jitter_params = JitterAnalysisParams::default();
    let lle_params = LlePolaritonParams::default();
    let metrics = TimingJitterSolver::solve(&jitter_params, &lle_params);

    // Integrated timing jitter sigma_t <= 5.0 fs (measured < 2.0 fs)
    assert!(
        metrics.integrated_jitter_fs <= 5.0,
        "Expected jitter <= 5.0 fs, got {:.2} fs",
        metrics.integrated_jitter_fs
    );
    assert!(
        metrics.integrated_jitter_fs < 2.0,
        "Measured jitter should be sub-2-fs, got {:.2} fs",
        metrics.integrated_jitter_fs
    );
    assert!(metrics.integrated_jitter_fs > 0.05);

    // Check phase noise spectrum points
    assert!(!metrics.phase_noise_spectrum.is_empty());
    assert!(metrics.phase_noise_at_10khz < -100.0);
    assert!(metrics.phase_noise_at_100khz < -120.0);
    assert!(metrics.phase_noise_at_1mhz < -140.0);
    assert!(metrics.phase_noise_at_10mhz < -150.0);
}

#[test]
fn test_relative_intensity_noise_rin() {
    let jitter_params = JitterAnalysisParams::default();
    let lle_params = LlePolaritonParams::default();
    let metrics = TimingJitterSolver::solve(&jitter_params, &lle_params);

    // RIN <= -140.0 dBc/Hz (measured <= -150 dBc/Hz)
    assert!(
        metrics.relative_intensity_noise_dbc_hz <= -140.0,
        "Expected RIN <= -140 dBc/Hz, got {:.1} dBc/Hz",
        metrics.relative_intensity_noise_dbc_hz
    );
    assert!(
        metrics.relative_intensity_noise_dbc_hz <= -150.0,
        "Measured RIN should be <= -150 dBc/Hz, got {:.1} dBc/Hz",
        metrics.relative_intensity_noise_dbc_hz
    );
}

#[test]
fn test_soliton_plateau_existence() {
    let jitter_params = JitterAnalysisParams::default();
    let lle_params = LlePolaritonParams::default();
    let metrics = TimingJitterSolver::solve(&jitter_params, &lle_params);

    assert!(metrics.has_soliton_plateau);
    assert!(metrics.soliton_plateau_width >= 1.0);
    assert!(!metrics.plateau_energy_curve.is_empty());
}

#[test]
fn test_ten_point_physics_audit_full_pass() {
    let processor = OptomagnonicCombProcessor::new();
    let report = processor.audit_comb();

    assert_eq!(report.total_count, 10);
    assert_eq!(report.passed_count, 10);
    assert!(report.all_passed, "Audit failed: {:?}", report.items.iter().filter(|i| !i.passed).collect::<Vec<_>>());

    for item in &report.items {
        assert!(item.passed, "Audit item {} failed: {}", item.id, item.name);
    }
}
