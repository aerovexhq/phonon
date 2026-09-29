#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum opto-electro-phononic
//! frequency translators and millimeter-wave cavity interfaces.

use phonon_models::opto_electro_phononic_translator::OptoElectroPhononicTranslatorParams;
use phonon_solver::opto_electro_phononic_translator::OptoElectroPhononicTranslatorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = OptoElectroPhononicTranslatorParams::new(
        10.0,   // below 20.0 GHz
        1400.0, // below 1500.0 nm
        1.0,    // below 5.0
        1.0,    // below 5.0
        0.05,   // below 0.1 MHz
        5.0e4,  // below 1.0e5
        0.5,    // below 1.0 mK
        0.05,   // below 0.1 mW
    );
    assert!((underflow.mmwave_frequency_ghz - 20.0).abs() < 1e-9);
    assert!((underflow.telecom_wavelength_nm - 1500.0).abs() < 1e-9);
    assert!((underflow.piezoelectric_cooperativity - 5.0).abs() < 1e-9);
    assert!((underflow.optomechanical_cooperativity - 5.0).abs() < 1e-9);
    assert!((underflow.acoustic_damping_rate_mhz - 0.1).abs() < 1e-9);
    assert!((underflow.optical_q_factor - 1.0e5).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);
    assert!((underflow.pump_laser_power_mw - 0.1).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = OptoElectroPhononicTranslatorParams::new(
        150.0,  // above 120.0 GHz
        1700.0, // above 1600.0 nm
        150.0,  // above 100.0
        150.0,  // above 100.0
        15.0,   // above 10.0 MHz
        2.0e7,  // above 1.0e7
        80.0,   // above 50.0 mK
        30.0,   // above 20.0 mW
    );
    assert!((overflow.mmwave_frequency_ghz - 120.0).abs() < 1e-9);
    assert!((overflow.telecom_wavelength_nm - 1600.0).abs() < 1e-9);
    assert!((overflow.piezoelectric_cooperativity - 100.0).abs() < 1e-9);
    assert!((overflow.optomechanical_cooperativity - 100.0).abs() < 1e-9);
    assert!((overflow.acoustic_damping_rate_mhz - 10.0).abs() < 1e-9);
    assert!((overflow.optical_q_factor - 1.0e7).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 50.0).abs() < 1e-9);
    assert!((overflow.pump_laser_power_mw - 20.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = OptoElectroPhononicTranslatorParams::default();
    let solver = OptoElectroPhononicTranslatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.transduction_efficiency >= 0.800,
        "Default transduction efficiency must be >= 0.800, got {:.5}",
        metrics.transduction_efficiency
    );
    assert!(
        metrics.added_thermal_noise_quanta <= 0.100,
        "Default added thermal noise quanta must be <= 0.100, got {:.5}",
        metrics.added_thermal_noise_quanta
    );
    assert!(
        metrics.conversion_bandwidth_mhz >= 5.0,
        "Default conversion bandwidth must be >= 5.0 MHz, got {:.3} MHz",
        metrics.conversion_bandwidth_mhz
    );
    assert!(
        metrics.quantum_state_transfer_fidelity >= 0.9850,
        "Default state transfer fidelity must be >= 0.9850, got {:.5}",
        metrics.quantum_state_transfer_fidelity
    );
    assert!(
        metrics.ground_state_cooling_occupancy <= 0.050,
        "Default cooling occupancy must be <= 0.050, got {:.5}",
        metrics.ground_state_cooling_occupancy
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_cooperativity_scaling() {
    let base = OptoElectroPhononicTranslatorParams::default();
    let solver_base = OptoElectroPhononicTranslatorSolver::new(base);
    let eta_base = solver_base.compute_transduction_efficiency();
    let bw_base = solver_base.compute_conversion_bandwidth_mhz();

    // Increased cooperativities
    let high_coop = OptoElectroPhononicTranslatorParams::new(
        base.mmwave_frequency_ghz,
        base.telecom_wavelength_nm,
        50.0, // increased piezoelectric cooperativity from 35.0
        50.0, // increased optomechanical cooperativity from 35.0
        base.acoustic_damping_rate_mhz,
        base.optical_q_factor,
        base.operating_temp_m_k,
        base.pump_laser_power_mw,
    );
    let solver_high = OptoElectroPhononicTranslatorSolver::new(high_coop);
    let eta_high = solver_high.compute_transduction_efficiency();
    let bw_high = solver_high.compute_conversion_bandwidth_mhz();

    assert!(
        eta_high >= eta_base,
        "Higher cooperativity must enhance transduction efficiency (base: {:.5}, high: {:.5})",
        eta_base,
        eta_high
    );
    assert!(
        bw_high >= bw_base,
        "Higher cooperativity must broaden conversion bandwidth (base: {:.3}, high: {:.3})",
        bw_base,
        bw_high
    );
}

#[test]
fn test_temperature_degradation() {
    let base = OptoElectroPhononicTranslatorParams::default();
    let solver_base = OptoElectroPhononicTranslatorSolver::new(base);
    let n_add_base = solver_base.compute_added_thermal_noise_quanta();
    let n_cool_base = solver_base.compute_ground_state_cooling_occupancy();
    let fid_base = solver_base.compute_quantum_state_transfer_fidelity();

    // Warmer cryogenic temperature (25 mK vs 15 mK)
    let warmer = OptoElectroPhononicTranslatorParams::new(
        base.mmwave_frequency_ghz,
        base.telecom_wavelength_nm,
        base.piezoelectric_cooperativity,
        base.optomechanical_cooperativity,
        base.acoustic_damping_rate_mhz,
        base.optical_q_factor,
        25.0, // elevated temperature
        base.pump_laser_power_mw,
    );
    let solver_warmer = OptoElectroPhononicTranslatorSolver::new(warmer);
    let n_add_warmer = solver_warmer.compute_added_thermal_noise_quanta();
    let n_cool_warmer = solver_warmer.compute_ground_state_cooling_occupancy();
    let fid_warmer = solver_warmer.compute_quantum_state_transfer_fidelity();

    assert!(
        n_add_warmer > n_add_base,
        "Warmer temperature must increase added thermal noise (base: {:.5}, warmer: {:.5})",
        n_add_base,
        n_add_warmer
    );
    assert!(
        n_cool_warmer > n_cool_base,
        "Warmer temperature must increase residual cooling occupancy (base: {:.5}, warmer: {:.5})",
        n_cool_base,
        n_cool_warmer
    );
    assert!(
        fid_warmer <= fid_base,
        "Warmer temperature must degrade quantum state transfer fidelity (base: {:.5}, warmer: {:.5})",
        fid_base,
        fid_warmer
    );
}

#[test]
fn test_added_noise_scaling() {
    let base = OptoElectroPhononicTranslatorParams::default();
    let solver_base = OptoElectroPhononicTranslatorSolver::new(base);
    let n_add_base = solver_base.compute_added_thermal_noise_quanta();

    // Higher millimeter-wave frequency (e.g., 90 GHz vs 45 GHz)
    let high_freq = OptoElectroPhononicTranslatorParams::new(
        90.0,
        base.telecom_wavelength_nm,
        base.piezoelectric_cooperativity,
        base.optomechanical_cooperativity,
        base.acoustic_damping_rate_mhz,
        base.optical_q_factor,
        base.operating_temp_m_k,
        base.pump_laser_power_mw,
    );
    let solver_high_freq = OptoElectroPhononicTranslatorSolver::new(high_freq);
    let n_add_high_freq = solver_high_freq.compute_added_thermal_noise_quanta();

    assert!(
        n_add_high_freq < n_add_base,
        "Higher mm-wave frequency must reduce added thermal noise (base: {:.5}, high-freq: {:.5})",
        n_add_base,
        n_add_high_freq
    );
}

#[test]
fn test_physical_compliance() {
    let params = OptoElectroPhononicTranslatorParams::default();
    let solver = OptoElectroPhononicTranslatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_physically_compliant);
    assert!(metrics.transduction_efficiency >= 0.800);
    assert!(metrics.added_thermal_noise_quanta <= 0.100);
    assert!(metrics.conversion_bandwidth_mhz >= 5.0);
    assert!(metrics.quantum_state_transfer_fidelity >= 0.9850);
    assert!(metrics.ground_state_cooling_occupancy <= 0.050);
}
