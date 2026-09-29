//! Automated unit and physical validation tests for cavity acoustomagnonic haloscopes.

use phonon_models::cavity_acoustomagnonic::AcoustomagnonicParams;
use phonon_solver::cavity_acoustomagnonic::CavityAcoustomagnonicSolver;

#[test]
fn test_acoustomagnonic_cooperativity_scaling() {
    let mut params = AcoustomagnonicParams::default();
    let solver_nom = CavityAcoustomagnonicSolver::new(params);
    let c_nom = solver_nom.compute_acoustomagnonic_cooperativity();

    // Verify nominal cooperativity exceeds roadmap target (>= 150.0)
    assert!(
        c_nom >= 150.0,
        "Nominal cooperativity must be >= 150.0, got {:.2}",
        c_nom
    );

    // Quadratic scaling with magneto-elastic coupling g_ma: doubling g_ma quadruples cooperativity
    params.magnon_phonon_coupling_mhz = 25.0; // 2x default (12.5)
    let solver_2x = CavityAcoustomagnonicSolver::new(params);
    let c_2x = solver_2x.compute_acoustomagnonic_cooperativity();

    let ratio = c_2x / c_nom;
    assert!(
        (ratio - 4.0).abs() < 0.10,
        "Cooperativity must scale quadratically with g_ma, got ratio {:.3}",
        ratio
    );
}

#[test]
fn test_conversion_gain_and_enhancement() {
    let params = AcoustomagnonicParams::default();
    let solver = CavityAcoustomagnonicSolver::new(params);
    let gain_db = solver.compute_conversion_gain_db();

    // Roadmap requirement: G_conv >= 22.0 dB
    assert!(
        gain_db >= 22.0,
        "Conversion gain must be >= 22.0 dB, got {:.2} dB",
        gain_db
    );
    assert!(
        gain_db <= 40.0,
        "Conversion gain must be physically bounded, got {:.2} dB",
        gain_db
    );
}

#[test]
fn test_sub_kelvin_system_noise_temperature() {
    let params = AcoustomagnonicParams::default();
    let solver = CavityAcoustomagnonicSolver::new(params);
    let t_sys = solver.compute_system_noise_temperature_k();

    // Dilution refrigerator at 20 mK with quantum-limited TWPA yields T_sys < 0.40 K
    assert!(
        t_sys < 0.40,
        "Total effective noise temp must be sub-Kelvin (< 0.40 K), got {:.3} K",
        t_sys
    );
    assert!(
        t_sys > 0.10,
        "Noise temp cannot violate quantum limit (> 0.10 K at 10 GHz), got {:.3} K",
        t_sys
    );
}

#[test]
fn test_haloscope_readout_snr_and_radiometer() {
    let mut params = AcoustomagnonicParams::default();
    let solver_1s = CavityAcoustomagnonicSolver::new(params);
    let snr_1s = solver_1s.compute_haloscope_readout_snr_db();

    // Roadmap requirement: Readout SNR >= 28.0 dB
    assert!(
        snr_1s >= 28.0,
        "Readout SNR must be >= 28.0 dB, got {:.2} dB",
        snr_1s
    );

    // Dicke radiometer integration time scaling: 4x integration time increases SNR by 10*log10(sqrt(4)) = 3.01 dB
    params.integration_time_s = 4.0;
    let solver_4s = CavityAcoustomagnonicSolver::new(params);
    let snr_4s = solver_4s.compute_haloscope_readout_snr_db();

    let diff_db = snr_4s - snr_1s;
    assert!(
        (diff_db - 3.01).abs() < 0.20,
        "Radiometer SNR must improve by ~3.01 dB for 4x dwell time, got {:.2} dB",
        diff_db
    );
}

#[test]
fn test_exclusion_scan_rate_scaling() {
    let params = AcoustomagnonicParams::default();
    let solver = CavityAcoustomagnonicSolver::new(params);
    let scan_rate = solver.compute_exclusion_scan_rate_ghz_per_day();

    // Roadmap requirement: exclusion scan rate >= 1.0 GHz/day
    assert!(
        scan_rate >= 1.0,
        "Exclusion scan rate must be >= 1.0 GHz/day, got {:.3} GHz/day",
        scan_rate
    );
}

#[test]
fn test_polariton_anti_crossing_splitting() {
    let mut params = AcoustomagnonicParams::default();
    params.magnon_freq_ghz = 10.2;
    params.phonon_freq_ghz = 10.2;
    params.magnon_phonon_coupling_mhz = 12.5;

    let solver = CavityAcoustomagnonicSolver::new(params);
    let splitting = solver.compute_polariton_splitting_mhz();

    // At zero detuning, splitting = 2 * g_ma = 2 * 12.5 = 25.0 MHz
    assert!(
        (splitting - 25.0).abs() < 1.0e-5,
        "Polariton anti-crossing splitting must equal 2 * g_ma = 25.0 MHz, got {:.3}",
        splitting
    );
}

#[test]
fn test_physical_compliance_flag() {
    let params = AcoustomagnonicParams::default();
    let solver = CavityAcoustomagnonicSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be 100% physically compliant"
    );
}
