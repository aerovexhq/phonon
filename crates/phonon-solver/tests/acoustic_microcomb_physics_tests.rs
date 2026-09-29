//! Analytical physics validation tests for quantum acoustic frequency combs
//! and phononic microresonator soliton synthesizers (Phase 118).

#![deny(unsafe_code)]

use phonon_models::acoustic_microcomb_soliton::AcousticMicrocombParams;
use phonon_solver::acoustic_microcomb_soliton::AcousticMicrocombSolitonSolver;

#[test]
fn test_comb_repetition_rate_exceeds_threshold() {
    let params = AcousticMicrocombParams::default();
    let solver = AcousticMicrocombSolitonSolver::new(params);
    let f_rep_ghz = solver.compute_comb_repetition_rate_ghz();

    // Roadmap target: Repetition rate f_rep >= 1.0 GHz
    assert!(
        f_rep_ghz >= 1.0,
        "Comb repetition rate must be >= 1.0 GHz, got {:.4} GHz",
        f_rep_ghz
    );
    assert!(
        f_rep_ghz <= 15.0,
        "Comb repetition rate must respect clamp ceiling 15.0 GHz, got {:.4} GHz",
        f_rep_ghz
    );

    // Smaller microresonator radius increases repetition rate (FSR ~ 1 / R)
    let mut small_radius_params = params;
    small_radius_params.microresonator_radius_um = 30.0;
    let small_radius_solver = AcousticMicrocombSolitonSolver::new(small_radius_params);
    assert!(
        small_radius_solver.compute_comb_repetition_rate_ghz() >= f_rep_ghz,
        "Smaller microresonator radius must increase or preserve repetition rate"
    );

    // Larger microresonator radius decreases repetition rate
    let mut large_radius_params = params;
    large_radius_params.microresonator_radius_um = 300.0;
    let large_radius_solver = AcousticMicrocombSolitonSolver::new(large_radius_params);
    assert!(
        large_radius_solver.compute_comb_repetition_rate_ghz() <= f_rep_ghz,
        "Larger microresonator radius must decrease or preserve repetition rate"
    );
}

#[test]
fn test_comb_spacing_stability_within_quantum_limit() {
    let params = AcousticMicrocombParams::default();
    let solver = AcousticMicrocombSolitonSolver::new(params);
    let stab = solver.compute_comb_spacing_stability();

    // Roadmap target: Fractional stability <= 1.0e-11
    assert!(
        stab <= 1.0e-11,
        "Comb spacing stability must be <= 1.0e-11, got {:.4e}",
        stab
    );
    assert!(
        stab >= 1.0e-13,
        "Comb spacing stability must respect physical floor 1.0e-13, got {:.4e}",
        stab
    );

    // Higher Q factor improves stability (lower numerical value)
    let mut high_q_params = params;
    high_q_params.acoustic_quality_factor = 1.0e7;
    let high_q_solver = AcousticMicrocombSolitonSolver::new(high_q_params);
    assert!(
        high_q_solver.compute_comb_spacing_stability() <= stab,
        "Higher acoustic quality factor must improve or preserve stability"
    );

    // Colder operating temperature improves stability
    let mut cold_params = params;
    cold_params.operating_temp_m_k = 5.0;
    let cold_solver = AcousticMicrocombSolitonSolver::new(cold_params);
    assert!(
        cold_solver.compute_comb_spacing_stability() <= stab,
        "Colder operating temperature must improve or preserve stability"
    );
}

#[test]
fn test_conversion_efficiency_exceeds_threshold() {
    let params = AcousticMicrocombParams::default();
    let solver = AcousticMicrocombSolitonSolver::new(params);
    let eff = solver.compute_conversion_efficiency();

    // Roadmap target: Conversion efficiency >= 0.350 (35.0%)
    assert!(
        eff >= 0.350,
        "Conversion efficiency must be >= 0.350, got {:.4}",
        eff
    );
    assert!(
        eff <= 0.550,
        "Conversion efficiency must respect clamp ceiling 0.550, got {:.4}",
        eff
    );

    // Higher pump power increases conversion efficiency
    let mut high_pump_params = params;
    high_pump_params.pump_power_mw = 24.0;
    let high_pump_solver = AcousticMicrocombSolitonSolver::new(high_pump_params);
    assert!(
        high_pump_solver.compute_conversion_efficiency() >= eff,
        "Higher pump power must increase or preserve conversion efficiency"
    );

    // Higher laser detuning reduces conversion efficiency
    let mut high_detuning_params = params;
    high_detuning_params.laser_detuning_ratio = 5.0;
    let high_detuning_solver = AcousticMicrocombSolitonSolver::new(high_detuning_params);
    assert!(
        high_detuning_solver.compute_conversion_efficiency() <= eff,
        "Higher detuning ratio must reduce or preserve conversion efficiency"
    );
}

#[test]
fn test_phase_noise_at_10khz_within_target() {
    let params = AcousticMicrocombParams::default();
    let solver = AcousticMicrocombSolitonSolver::new(params);
    let pn = solver.compute_phase_noise_at_10khz_dbc();

    // Roadmap target: Phase noise <= -125.0 dBc/Hz
    assert!(
        pn <= -125.0,
        "Phase noise at 10 kHz must be <= -125.0 dBc/Hz, got {:.2} dBc/Hz",
        pn
    );
    assert!(
        pn >= -145.0,
        "Phase noise at 10 kHz must respect clamp floor -145.0 dBc/Hz, got {:.2} dBc/Hz",
        pn
    );

    // Higher acoustic quality factor reduces phase noise (more negative)
    let mut high_q_params = params;
    high_q_params.acoustic_quality_factor = 1.0e7;
    let high_q_solver = AcousticMicrocombSolitonSolver::new(high_q_params);
    assert!(
        high_q_solver.compute_phase_noise_at_10khz_dbc() <= pn,
        "Higher acoustic Q factor must reduce (make more negative) phase noise"
    );
}

#[test]
fn test_comb_octave_span_exceeds_threshold() {
    let params = AcousticMicrocombParams::default();
    let solver = AcousticMicrocombSolitonSolver::new(params);
    let span = solver.compute_comb_octave_span();

    // Roadmap target: Octave span >= 1.00 octaves
    assert!(
        span >= 1.00,
        "Comb octave span must be >= 1.00 octaves, got {:.4}",
        span
    );
    assert!(
        span <= 3.50,
        "Comb octave span must respect clamp ceiling 3.50, got {:.4}",
        span
    );

    // Higher anomalous dispersion D2 broadens the comb span
    let mut high_d2_params = params;
    high_d2_params.dispersion_parameter_d2_khz = 180.0;
    let high_d2_solver = AcousticMicrocombSolitonSolver::new(high_d2_params);
    assert!(
        high_d2_solver.compute_comb_octave_span() >= span,
        "Higher D2 dispersion must broaden or preserve comb octave span"
    );
}

#[test]
fn test_timing_jitter_within_femtosecond_target() {
    let params = AcousticMicrocombParams::default();
    let solver = AcousticMicrocombSolitonSolver::new(params);
    let jitter = solver.compute_timing_jitter_fs();

    // Roadmap target: Timing jitter <= 5.0 fs
    assert!(
        jitter <= 5.0,
        "Timing jitter must be <= 5.0 fs, got {:.4} fs",
        jitter
    );
    assert!(
        jitter >= 0.5,
        "Timing jitter must respect clamp floor 0.5 fs, got {:.4} fs",
        jitter
    );

    // Higher Q factor suppresses timing jitter
    let mut high_q_params = params;
    high_q_params.acoustic_quality_factor = 8.0e6;
    let high_q_solver = AcousticMicrocombSolitonSolver::new(high_q_params);
    assert!(
        high_q_solver.compute_timing_jitter_fs() <= jitter,
        "Higher acoustic Q factor must reduce or preserve timing jitter"
    );

    // Colder temperature suppresses timing jitter
    let mut cold_params = params;
    cold_params.operating_temp_m_k = 8.0;
    let cold_solver = AcousticMicrocombSolitonSolver::new(cold_params);
    assert!(
        cold_solver.compute_timing_jitter_fs() <= jitter,
        "Colder operating temperature must reduce or preserve timing jitter"
    );
}

#[test]
fn test_parameter_bounds_clamping() {
    let out_of_bounds = AcousticMicrocombParams::new(
        5.0,    // Below min 10.0 um -> clamps to 10.0
        0.1,    // Below min 0.5 GHz -> clamps to 0.5
        5.0e4,  // Below min 1.0e5 -> clamps to 1.0e5
        0.005,  // Below min 0.01 Hz -> clamps to 0.01
        0.5,    // Below min 1.0 kHz -> clamps to 1.0
        0.05,   // Below min 0.1 mW -> clamps to 0.1
        0.2,    // Below min 0.5 -> clamps to 0.5
        0.5,    // Below min 1.0 mK -> clamps to 1.0
    );

    assert_eq!(out_of_bounds.microresonator_radius_um, 10.0);
    assert_eq!(out_of_bounds.fundamental_resonance_ghz, 0.5);
    assert_eq!(out_of_bounds.acoustic_quality_factor, 1.0e5);
    assert_eq!(out_of_bounds.kerr_nonlinearity_hz, 0.01);
    assert_eq!(out_of_bounds.dispersion_parameter_d2_khz, 1.0);
    assert_eq!(out_of_bounds.pump_power_mw, 0.1);
    assert_eq!(out_of_bounds.laser_detuning_ratio, 0.5);
    assert_eq!(out_of_bounds.operating_temp_m_k, 1.0);

    let upper_bounds = AcousticMicrocombParams::new(
        800.0,   // Above max 500.0 um -> clamps to 500.0
        25.0,    // Above max 15.0 GHz -> clamps to 15.0
        5.0e8,   // Above max 1.0e8 -> clamps to 1.0e8
        100.0,   // Above max 50.0 Hz -> clamps to 50.0
        800.0,   // Above max 500.0 kHz -> clamps to 500.0
        250.0,   // Above max 100.0 mW -> clamps to 100.0
        15.0,    // Above max 10.0 -> clamps to 10.0
        2500.0,  // Above max 1000.0 mK -> clamps to 1000.0
    );

    assert_eq!(upper_bounds.microresonator_radius_um, 500.0);
    assert_eq!(upper_bounds.fundamental_resonance_ghz, 15.0);
    assert_eq!(upper_bounds.acoustic_quality_factor, 1.0e8);
    assert_eq!(upper_bounds.kerr_nonlinearity_hz, 50.0);
    assert_eq!(upper_bounds.dispersion_parameter_d2_khz, 500.0);
    assert_eq!(upper_bounds.pump_power_mw, 100.0);
    assert_eq!(upper_bounds.laser_detuning_ratio, 10.0);
    assert_eq!(upper_bounds.operating_temp_m_k, 1000.0);
}

#[test]
fn test_full_roadmap_physical_compliance() {
    let params = AcousticMicrocombParams::default();
    let solver = AcousticMicrocombSolitonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default acoustic microcomb parameter configuration must be physically compliant"
    );
    assert!(metrics.comb_repetition_rate_ghz >= 1.0);
    assert!(metrics.comb_spacing_stability <= 1.0e-11);
    assert!(metrics.conversion_efficiency >= 0.350);
    assert!(metrics.phase_noise_at_10khz_dbc <= -125.0);
    assert!(metrics.comb_octave_span >= 1.00);
    assert!(metrics.timing_jitter_fs <= 5.0);
}
