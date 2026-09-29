#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic chiral
//! spin-mechanical frequency-bin entanglement and phononic Bell state analyzers.

use phonon_models::chiral_frequency_bin_bell_analyzer::ChiralFrequencyBinBellAnalyzerParams;
use phonon_solver::chiral_frequency_bin_bell_analyzer::ChiralFrequencyBinBellAnalyzerSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ChiralFrequencyBinBellAnalyzerParams::new(
        1.0,   // below 2.0 MHz
        5.0,   // below 10.0 MHz
        0.5,   // below 1.0 MHz
        5.0,   // below 10.0 kHz
        0.50,  // below 0.70
        10.0,  // below 20.0 dB
        0.5,   // below 1.0 mK
        0.05,  // below 0.1 us
    );
    assert_eq!(underflow.parametric_pump_amplitude_mhz, 2.0);
    assert_eq!(underflow.bin_frequency_separation_mhz, 10.0);
    assert_eq!(underflow.spin_acoustic_coupling_mhz, 1.0);
    assert_eq!(underflow.cavity_decay_rate_khz, 10.0);
    assert_eq!(underflow.detector_quantum_efficiency, 0.70);
    assert_eq!(underflow.chiral_isolation_db, 20.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.measurement_window_us, 0.1);

    // Test values strictly above physical maximum bounds
    let overflow = ChiralFrequencyBinBellAnalyzerParams::new(
        100.0, // above 50.0 MHz
        300.0, // above 200.0 MHz
        40.0,  // above 25.0 MHz
        500.0, // above 300.0 kHz
        1.05,  // above 0.99
        80.0,  // above 60.0 dB
        100.0, // above 50.0 mK
        20.0,  // above 10.0 us
    );
    assert_eq!(overflow.parametric_pump_amplitude_mhz, 50.0);
    assert_eq!(overflow.bin_frequency_separation_mhz, 200.0);
    assert_eq!(overflow.spin_acoustic_coupling_mhz, 25.0);
    assert_eq!(overflow.cavity_decay_rate_khz, 300.0);
    assert_eq!(overflow.detector_quantum_efficiency, 0.99);
    assert_eq!(overflow.chiral_isolation_db, 60.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.measurement_window_us, 10.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralFrequencyBinBellAnalyzerParams::default();
    let solver = ChiralFrequencyBinBellAnalyzerSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.bell_state_measurement_fidelity >= 0.9950,
        "Default Bell state measurement fidelity must be >= 0.9950, got {:.6}",
        metrics.bell_state_measurement_fidelity
    );
    assert!(
        metrics.frequency_bin_mode_indistinguishability >= 0.9980,
        "Default frequency-bin mode indistinguishability must be >= 0.9980, got {:.6}",
        metrics.frequency_bin_mode_indistinguishability
    );
    assert!(
        metrics.crosstalk_quantum_dephasing_rate_hz <= 120.0,
        "Default cross-talk dephasing rate must be <= 120.0 Hz, got {:.2} Hz",
        metrics.crosstalk_quantum_dephasing_rate_hz
    );
    assert!(
        metrics.dark_count_probability <= 1.0e-5,
        "Default dark count probability must be <= 1.0e-5, got {:.4e}",
        metrics.dark_count_probability
    );
    assert!(
        metrics.two_phonon_entanglement_concurrence >= 0.980,
        "Default two-phonon entanglement concurrence must be >= 0.980, got {:.6}",
        metrics.two_phonon_entanglement_concurrence
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_detector_efficiency_scaling() {
    let base = ChiralFrequencyBinBellAnalyzerParams::default();
    let solver_base = ChiralFrequencyBinBellAnalyzerSolver::new(base);

    let high_eff = ChiralFrequencyBinBellAnalyzerParams::new(
        base.parametric_pump_amplitude_mhz,
        base.bin_frequency_separation_mhz,
        base.spin_acoustic_coupling_mhz,
        base.cavity_decay_rate_khz,
        0.98, // increased from 0.94
        base.chiral_isolation_db,
        base.cryogenic_temperature_mk,
        base.measurement_window_us,
    );
    let solver_high = ChiralFrequencyBinBellAnalyzerSolver::new(high_eff);

    let fid_base = solver_base.compute_bell_state_measurement_fidelity();
    let fid_high = solver_high.compute_bell_state_measurement_fidelity();
    assert!(
        fid_high > fid_base,
        "Higher detector efficiency must increase Bell state measurement fidelity: {:.6} vs {:.6}",
        fid_high,
        fid_base
    );

    let conc_base = solver_base.compute_two_phonon_entanglement_concurrence();
    let conc_high = solver_high.compute_two_phonon_entanglement_concurrence();
    assert!(
        conc_high > conc_base,
        "Higher detector efficiency must increase two-phonon concurrence: {:.6} vs {:.6}",
        conc_high,
        conc_base
    );
}

#[test]
fn test_chiral_isolation_scaling() {
    let base = ChiralFrequencyBinBellAnalyzerParams::default();
    let solver_base = ChiralFrequencyBinBellAnalyzerSolver::new(base);

    let high_iso = ChiralFrequencyBinBellAnalyzerParams::new(
        base.parametric_pump_amplitude_mhz,
        base.bin_frequency_separation_mhz,
        base.spin_acoustic_coupling_mhz,
        base.cavity_decay_rate_khz,
        base.detector_quantum_efficiency,
        48.0, // increased from 38.0 dB
        base.cryogenic_temperature_mk,
        base.measurement_window_us,
    );
    let solver_high = ChiralFrequencyBinBellAnalyzerSolver::new(high_iso);

    let deph_base = solver_base.compute_crosstalk_quantum_dephasing_rate_hz();
    let deph_high = solver_high.compute_crosstalk_quantum_dephasing_rate_hz();
    assert!(
        deph_high < deph_base,
        "Higher chiral isolation must suppress cross-talk dephasing: {:.2} Hz vs {:.2} Hz",
        deph_high,
        deph_base
    );

    let dark_base = solver_base.compute_dark_count_probability();
    let dark_high = solver_high.compute_dark_count_probability();
    assert!(
        dark_high < dark_base,
        "Higher chiral isolation must reduce dark count probability: {:.4e} vs {:.4e}",
        dark_high,
        dark_base
    );

    let indist_base = solver_base.compute_frequency_bin_mode_indistinguishability();
    let indist_high = solver_high.compute_frequency_bin_mode_indistinguishability();
    assert!(
        indist_high > indist_base,
        "Higher chiral isolation must improve frequency-bin mode indistinguishability: {:.6} vs {:.6}",
        indist_high,
        indist_base
    );
}

#[test]
fn test_cavity_decay_scaling() {
    let base = ChiralFrequencyBinBellAnalyzerParams::default();
    let solver_base = ChiralFrequencyBinBellAnalyzerSolver::new(base);

    let high_decay = ChiralFrequencyBinBellAnalyzerParams::new(
        base.parametric_pump_amplitude_mhz,
        base.bin_frequency_separation_mhz,
        base.spin_acoustic_coupling_mhz,
        150.0, // increased from 75.0 kHz
        base.detector_quantum_efficiency,
        base.chiral_isolation_db,
        base.cryogenic_temperature_mk,
        base.measurement_window_us,
    );
    let solver_high = ChiralFrequencyBinBellAnalyzerSolver::new(high_decay);

    let deph_base = solver_base.compute_crosstalk_quantum_dephasing_rate_hz();
    let deph_high = solver_high.compute_crosstalk_quantum_dephasing_rate_hz();
    assert!(
        deph_high > deph_base,
        "Higher cavity decay rate must increase cross-talk dephasing: {:.2} Hz vs {:.2} Hz",
        deph_high,
        deph_base
    );

    let conc_base = solver_base.compute_two_phonon_entanglement_concurrence();
    let conc_high = solver_high.compute_two_phonon_entanglement_concurrence();
    assert!(
        conc_high < conc_base,
        "Higher cavity decay rate must degrade concurrence: {:.6} vs {:.6}",
        conc_high,
        conc_base
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let base = ChiralFrequencyBinBellAnalyzerParams::default();
    let solver_base = ChiralFrequencyBinBellAnalyzerSolver::new(base);

    let high_temp = ChiralFrequencyBinBellAnalyzerParams::new(
        base.parametric_pump_amplitude_mhz,
        base.bin_frequency_separation_mhz,
        base.spin_acoustic_coupling_mhz,
        base.cavity_decay_rate_khz,
        base.detector_quantum_efficiency,
        base.chiral_isolation_db,
        25.0, // increased from 10.0 mK
        base.measurement_window_us,
    );
    let solver_high = ChiralFrequencyBinBellAnalyzerSolver::new(high_temp);

    let dark_base = solver_base.compute_dark_count_probability();
    let dark_high = solver_high.compute_dark_count_probability();
    assert!(
        dark_high > dark_base,
        "Elevated temperature must increase dark count probability: {:.4e} vs {:.4e}",
        dark_high,
        dark_base
    );

    let fid_base = solver_base.compute_bell_state_measurement_fidelity();
    let fid_high = solver_high.compute_bell_state_measurement_fidelity();
    assert!(
        fid_high < fid_base,
        "Elevated temperature must degrade Bell state measurement fidelity: {:.6} vs {:.6}",
        fid_high,
        fid_base
    );
}

#[test]
fn test_bin_frequency_separation_scaling() {
    let base = ChiralFrequencyBinBellAnalyzerParams::default();
    let solver_base = ChiralFrequencyBinBellAnalyzerSolver::new(base);

    let wide_bins = ChiralFrequencyBinBellAnalyzerParams::new(
        base.parametric_pump_amplitude_mhz,
        120.0, // increased from 65.0 MHz
        base.spin_acoustic_coupling_mhz,
        base.cavity_decay_rate_khz,
        base.detector_quantum_efficiency,
        base.chiral_isolation_db,
        base.cryogenic_temperature_mk,
        base.measurement_window_us,
    );
    let solver_wide = ChiralFrequencyBinBellAnalyzerSolver::new(wide_bins);

    let deph_base = solver_base.compute_crosstalk_quantum_dephasing_rate_hz();
    let deph_wide = solver_wide.compute_crosstalk_quantum_dephasing_rate_hz();
    assert!(
        deph_wide < deph_base,
        "Wider frequency-bin separation must suppress cross-talk dephasing: {:.2} Hz vs {:.2} Hz",
        deph_wide,
        deph_base
    );

    let indist_base = solver_base.compute_frequency_bin_mode_indistinguishability();
    let indist_wide = solver_wide.compute_frequency_bin_mode_indistinguishability();
    assert!(
        indist_wide > indist_base,
        "Wider frequency-bin separation must improve indistinguishability: {:.6} vs {:.6}",
        indist_wide,
        indist_base
    );
}

#[test]
fn test_measurement_window_scaling() {
    let base = ChiralFrequencyBinBellAnalyzerParams::default();
    let solver_base = ChiralFrequencyBinBellAnalyzerSolver::new(base);

    let long_window = ChiralFrequencyBinBellAnalyzerParams::new(
        base.parametric_pump_amplitude_mhz,
        base.bin_frequency_separation_mhz,
        base.spin_acoustic_coupling_mhz,
        base.cavity_decay_rate_khz,
        base.detector_quantum_efficiency,
        base.chiral_isolation_db,
        base.cryogenic_temperature_mk,
        5.0, // increased from 2.2 us
    );
    let solver_long = ChiralFrequencyBinBellAnalyzerSolver::new(long_window);

    let dark_base = solver_base.compute_dark_count_probability();
    let dark_long = solver_long.compute_dark_count_probability();
    assert!(
        dark_long > dark_base,
        "Longer measurement window must increase dark count probability: {:.4e} vs {:.4e}",
        dark_long,
        dark_base
    );
}

#[test]
fn test_parametric_pump_amplitude_scaling() {
    let base = ChiralFrequencyBinBellAnalyzerParams::default();
    let solver_base = ChiralFrequencyBinBellAnalyzerSolver::new(base);

    let strong_pump = ChiralFrequencyBinBellAnalyzerParams::new(
        30.0, // increased from 16.5 MHz
        base.bin_frequency_separation_mhz,
        base.spin_acoustic_coupling_mhz,
        base.cavity_decay_rate_khz,
        base.detector_quantum_efficiency,
        base.chiral_isolation_db,
        base.cryogenic_temperature_mk,
        base.measurement_window_us,
    );
    let solver_strong = ChiralFrequencyBinBellAnalyzerSolver::new(strong_pump);

    let conc_base = solver_base.compute_two_phonon_entanglement_concurrence();
    let conc_strong = solver_strong.compute_two_phonon_entanglement_concurrence();
    assert!(
        conc_strong > conc_base,
        "Stronger pump amplitude must increase concurrence: {:.6} vs {:.6}",
        conc_strong,
        conc_base
    );
}

#[test]
fn test_spin_acoustic_coupling_scaling() {
    let base = ChiralFrequencyBinBellAnalyzerParams::default();
    let solver_base = ChiralFrequencyBinBellAnalyzerSolver::new(base);

    let strong_coupling = ChiralFrequencyBinBellAnalyzerParams::new(
        base.parametric_pump_amplitude_mhz,
        base.bin_frequency_separation_mhz,
        15.0, // increased from 6.2 MHz
        base.cavity_decay_rate_khz,
        base.detector_quantum_efficiency,
        base.chiral_isolation_db,
        base.cryogenic_temperature_mk,
        base.measurement_window_us,
    );
    let solver_strong = ChiralFrequencyBinBellAnalyzerSolver::new(strong_coupling);

    let fid_base = solver_base.compute_bell_state_measurement_fidelity();
    let fid_strong = solver_strong.compute_bell_state_measurement_fidelity();
    assert!(
        fid_strong > fid_base,
        "Stronger spin-acoustic coupling must increase Bell state measurement fidelity: {:.6} vs {:.6}",
        fid_strong,
        fid_base
    );
}
