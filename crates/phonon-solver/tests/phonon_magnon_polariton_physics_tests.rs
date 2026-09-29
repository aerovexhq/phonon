#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for coherent quantum
//! phonon-magnon-polariton transducers and chiral spin-acoustic interfaces.

use phonon_models::phonon_magnon_polariton::PhononMagnonPolaritonParams;
use phonon_solver::phonon_magnon_polariton::PhononMagnonPolaritonSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values below physical minimum bounds
    let underflow = PhononMagnonPolaritonParams::new(
        1.0,    // below 2.0 GHz
        1.0,    // below 2.0 GHz
        5.0,    // below 10.0 MHz
        5.0,    // below 10.0 nm
        0.01,   // below 0.05 MHz
        1.0e-6, // below 1.0e-5
        0.30,   // below 0.50
        0.5,    // below 1.0 mK
    );
    assert!((underflow.spin_wave_frequency_ghz - 2.0).abs() < 1e-9);
    assert!((underflow.acoustic_frequency_ghz - 2.0).abs() < 1e-9);
    assert!((underflow.magnetoelastic_coupling_mhz - 10.0).abs() < 1e-9);
    assert!((underflow.yig_film_thickness_nm - 10.0).abs() < 1e-9);
    assert!((underflow.piezo_acoustic_loss_mhz - 0.05).abs() < 1e-9);
    assert!((underflow.magnon_damping_alpha - 1.0e-5).abs() < 1e-12);
    assert!((underflow.chiral_asymmetry_factor - 0.50).abs() < 1e-9);
    assert!((underflow.operating_temp_m_k - 1.0).abs() < 1e-9);

    // Test values above physical maximum bounds
    let overflow = PhononMagnonPolaritonParams::new(
        25.0,   // above 20.0 GHz
        25.0,   // above 20.0 GHz
        200.0,  // above 150.0 MHz
        600.0,  // above 500.0 nm
        10.0,   // above 5.0 MHz
        5.0e-3, // above 1.0e-3
        1.20,   // above 0.99
        150.0,  // above 100.0 mK
    );
    assert!((overflow.spin_wave_frequency_ghz - 20.0).abs() < 1e-9);
    assert!((overflow.acoustic_frequency_ghz - 20.0).abs() < 1e-9);
    assert!((overflow.magnetoelastic_coupling_mhz - 150.0).abs() < 1e-9);
    assert!((overflow.yig_film_thickness_nm - 500.0).abs() < 1e-9);
    assert!((overflow.piezo_acoustic_loss_mhz - 5.0).abs() < 1e-9);
    assert!((overflow.magnon_damping_alpha - 1.0e-3).abs() < 1e-12);
    assert!((overflow.chiral_asymmetry_factor - 0.99).abs() < 1e-9);
    assert!((overflow.operating_temp_m_k - 100.0).abs() < 1e-9);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = PhononMagnonPolaritonParams::default();
    let solver = PhononMagnonPolaritonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical targets for default parameters
    assert!(
        metrics.polariton_cooperativity >= 50.0,
        "Default polariton cooperativity must be >= 50.0, got {:.3}",
        metrics.polariton_cooperativity
    );
    assert!(
        metrics.bidirectional_transduction_efficiency >= 0.850,
        "Default transduction efficiency must be >= 0.850, got {:.5}",
        metrics.bidirectional_transduction_efficiency
    );
    assert!(
        metrics.spin_wave_dephasing_rate_mhz <= 1.00,
        "Default spin-wave dephasing rate must be <= 1.00 MHz, got {:.3} MHz",
        metrics.spin_wave_dephasing_rate_mhz
    );
    assert!(
        metrics.chiral_isolation_db >= 30.0,
        "Default chiral isolation must be >= 30.0 dB, got {:.3} dB",
        metrics.chiral_isolation_db
    );
    assert!(
        metrics.single_quantum_conversion_fidelity >= 0.990,
        "Default single-quantum conversion fidelity must be >= 0.990, got {:.5}",
        metrics.single_quantum_conversion_fidelity
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_coupling_scaling() {
    let base = PhononMagnonPolaritonParams::default();
    let solver_base = PhononMagnonPolaritonSolver::new(base);
    let coop_base = solver_base.compute_polariton_cooperativity();
    let eff_base = solver_base.compute_bidirectional_transduction_efficiency();
    let fid_base = solver_base.compute_single_quantum_conversion_fidelity();

    let high_coupling = PhononMagnonPolaritonParams::new(
        base.spin_wave_frequency_ghz,
        base.acoustic_frequency_ghz,
        85.0, // increased from 65.0 MHz
        base.yig_film_thickness_nm,
        base.piezo_acoustic_loss_mhz,
        base.magnon_damping_alpha,
        base.chiral_asymmetry_factor,
        base.operating_temp_m_k,
    );
    let solver_high = PhononMagnonPolaritonSolver::new(high_coupling);
    let coop_high = solver_high.compute_polariton_cooperativity();
    let eff_high = solver_high.compute_bidirectional_transduction_efficiency();
    let fid_high = solver_high.compute_single_quantum_conversion_fidelity();

    assert!(
        coop_high > coop_base,
        "Higher magnetoelastic coupling must increase cooperativity: {} > {}",
        coop_high,
        coop_base
    );
    assert!(
        eff_high > eff_base,
        "Higher magnetoelastic coupling must increase transduction efficiency: {} > {}",
        eff_high,
        eff_base
    );
    assert!(
        fid_high > fid_base,
        "Higher magnetoelastic coupling must increase conversion fidelity: {} > {}",
        fid_high,
        fid_base
    );
}

#[test]
fn test_damping_scaling() {
    let base = PhononMagnonPolaritonParams::default();
    let solver_base = PhononMagnonPolaritonSolver::new(base);
    let deph_base = solver_base.compute_spin_wave_dephasing_rate_mhz();
    let coop_base = solver_base.compute_polariton_cooperativity();

    let high_damping = PhononMagnonPolaritonParams::new(
        base.spin_wave_frequency_ghz,
        base.acoustic_frequency_ghz,
        base.magnetoelastic_coupling_mhz,
        base.yig_film_thickness_nm,
        base.piezo_acoustic_loss_mhz,
        3.0e-4, // increased from 1.5e-4
        base.chiral_asymmetry_factor,
        base.operating_temp_m_k,
    );
    let solver_high_damping = PhononMagnonPolaritonSolver::new(high_damping);
    let deph_high = solver_high_damping.compute_spin_wave_dephasing_rate_mhz();
    let coop_high = solver_high_damping.compute_polariton_cooperativity();

    assert!(
        deph_high > deph_base,
        "Higher Gilbert damping must increase spin-wave dephasing rate: {} > {}",
        deph_high,
        deph_base
    );
    assert!(
        coop_high < coop_base,
        "Higher Gilbert damping must decrease polariton cooperativity: {} < {}",
        coop_high,
        coop_base
    );
}

#[test]
fn test_chiral_asymmetry_scaling() {
    let base = PhononMagnonPolaritonParams::default();
    let solver_base = PhononMagnonPolaritonSolver::new(base);
    let iso_base = solver_base.compute_chiral_isolation_db();

    let high_chiral = PhononMagnonPolaritonParams::new(
        base.spin_wave_frequency_ghz,
        base.acoustic_frequency_ghz,
        base.magnetoelastic_coupling_mhz,
        base.yig_film_thickness_nm,
        base.piezo_acoustic_loss_mhz,
        base.magnon_damping_alpha,
        0.96, // increased from 0.88
        base.operating_temp_m_k,
    );
    let solver_high_chiral = PhononMagnonPolaritonSolver::new(high_chiral);
    let iso_high = solver_high_chiral.compute_chiral_isolation_db();

    assert!(
        iso_high > iso_base,
        "Higher chiral asymmetry factor must increase non-reciprocal isolation: {} > {}",
        iso_high,
        iso_base
    );
}

#[test]
fn test_temperature_scaling() {
    let cold = PhononMagnonPolaritonParams::new(
        8.5, 8.5, 65.0, 100.0, 0.45, 1.5e-4, 0.88, 15.0, // 15 mK
    );
    let warm = PhononMagnonPolaritonParams::new(
        8.5, 8.5, 65.0, 100.0, 0.45, 1.5e-4, 0.88, 30.0, // 30 mK
    );

    let solver_cold = PhononMagnonPolaritonSolver::new(cold);
    let solver_warm = PhononMagnonPolaritonSolver::new(warm);

    let metrics_cold = solver_cold.evaluate_metrics();
    let metrics_warm = solver_warm.evaluate_metrics();

    assert!(
        metrics_cold.polariton_cooperativity > metrics_warm.polariton_cooperativity,
        "Warmer temperature must decrease cooperativity: {} > {}",
        metrics_cold.polariton_cooperativity,
        metrics_warm.polariton_cooperativity
    );
    assert!(
        metrics_cold.bidirectional_transduction_efficiency
            > metrics_warm.bidirectional_transduction_efficiency,
        "Warmer temperature must decrease transduction efficiency: {} > {}",
        metrics_cold.bidirectional_transduction_efficiency,
        metrics_warm.bidirectional_transduction_efficiency
    );
    assert!(
        metrics_cold.spin_wave_dephasing_rate_mhz < metrics_warm.spin_wave_dephasing_rate_mhz,
        "Warmer temperature must increase spin-wave dephasing rate: {} < {}",
        metrics_cold.spin_wave_dephasing_rate_mhz,
        metrics_warm.spin_wave_dephasing_rate_mhz
    );
    assert!(
        metrics_cold.chiral_isolation_db > metrics_warm.chiral_isolation_db,
        "Warmer temperature must decrease chiral isolation: {} > {}",
        metrics_cold.chiral_isolation_db,
        metrics_warm.chiral_isolation_db
    );
    assert!(
        metrics_cold.single_quantum_conversion_fidelity
            > metrics_warm.single_quantum_conversion_fidelity,
        "Warmer temperature must decrease single-quantum conversion fidelity: {} > {}",
        metrics_cold.single_quantum_conversion_fidelity,
        metrics_warm.single_quantum_conversion_fidelity
    );
}

#[test]
fn test_physical_compliance() {
    let nominal = PhononMagnonPolaritonParams::default();
    let solver_nominal = PhononMagnonPolaritonSolver::new(nominal);
    let metrics_nominal = solver_nominal.evaluate_metrics();
    assert!(
        metrics_nominal.is_physically_compliant,
        "Nominal parameters must be physically compliant"
    );
    assert!(metrics_nominal.polariton_cooperativity >= 50.0);
    assert!(metrics_nominal.bidirectional_transduction_efficiency >= 0.850);
    assert!(metrics_nominal.spin_wave_dephasing_rate_mhz <= 1.00);
    assert!(metrics_nominal.chiral_isolation_db >= 30.0);
    assert!(metrics_nominal.single_quantum_conversion_fidelity >= 0.990);

    // Test across alternative parameter design regimes
    let high_perf = PhononMagnonPolaritonParams::new(
        10.0, 10.0, 80.0, 150.0, 0.40, 1.2e-4, 0.92, 15.0,
    );
    let solver_high_perf = PhononMagnonPolaritonSolver::new(high_perf);
    let metrics_high_perf = solver_high_perf.evaluate_metrics();
    assert!(
        metrics_high_perf.is_physically_compliant,
        "High-performance parameters must be physically compliant"
    );
    assert!(metrics_high_perf.polariton_cooperativity >= 50.0);
    assert!(metrics_high_perf.bidirectional_transduction_efficiency >= 0.850);
    assert!(metrics_high_perf.spin_wave_dephasing_rate_mhz <= 1.00);
    assert!(metrics_high_perf.chiral_isolation_db >= 30.0);
    assert!(metrics_high_perf.single_quantum_conversion_fidelity >= 0.990);
}
