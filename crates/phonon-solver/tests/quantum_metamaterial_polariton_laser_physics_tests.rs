#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Quantum Metamaterial Polariton Laser & Coherent Soliton Engine.

use phonon_models::quantum_metamaterial_polariton_laser::QuantumMetamaterialPolaritonLaserParams;
use phonon_solver::quantum_metamaterial_polariton_laser::QuantumMetamaterialPolaritonLaserSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = QuantumMetamaterialPolaritonLaserParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.laser_coupling_mev, 1.0);
    assert_eq!(underflow.topological_polariton_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.soliton_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.optical_pump_power_uw, 0.5);
    assert_eq!(underflow.synthetic_metamaterial_cavities_factor, 1.0);
    assert_eq!(underflow.metamaterial_lattice_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = QuantumMetamaterialPolaritonLaserParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        35.0,   // above 25.0 um
    );
    assert_eq!(overflow.laser_coupling_mev, 35.0);
    assert_eq!(overflow.topological_polariton_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.soliton_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.optical_pump_power_uw, 30.0);
    assert_eq!(overflow.synthetic_metamaterial_cavities_factor, 8.0);
    assert_eq!(overflow.metamaterial_lattice_pitch_um, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = QuantumMetamaterialPolaritonLaserParams::default();
    assert_eq!(params.laser_coupling_mev, 35.0);
    assert_eq!(params.topological_polariton_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.soliton_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.optical_pump_power_uw, 23.5);
    assert_eq!(params.synthetic_metamaterial_cavities_factor, 4.0);
    assert_eq!(params.metamaterial_lattice_pitch_um, 22.5);

    let solver = QuantumMetamaterialPolaritonLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.polariton_laser_fidelity >= 0.9980,
        "Polariton laser fidelity must be >= 0.9980, got {:.6}",
        metrics.polariton_laser_fidelity
    );
    assert!(
        metrics.coherent_soliton_retention_fraction >= 0.9970,
        "Coherent soliton retention fraction must be >= 0.9970, got {:.6}",
        metrics.coherent_soliton_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_cavity_crosstalk_isolation_db >= 55.0,
        "Inter-cavity crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_cavity_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 12.0,
        "Topological mode dephasing rate must be <= 12.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_laser_coupling_monotonicity() {
    let p_low = QuantumMetamaterialPolaritonLaserParams {
        laser_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialPolaritonLaserParams {
        laser_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialPolaritonLaserSolver::new(p_low);
    let s_high = QuantumMetamaterialPolaritonLaserSolver::new(p_high);

    assert!(
        s_high.compute_polariton_laser_fidelity()
            > s_low.compute_polariton_laser_fidelity()
    );
    assert!(
        s_high.compute_coherent_soliton_retention_fraction()
            > s_low.compute_coherent_soliton_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_cavity_crosstalk_isolation_db()
            > s_low.compute_inter_cavity_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_polariton_gap_monotonicity() {
    let p_low = QuantumMetamaterialPolaritonLaserParams {
        topological_polariton_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialPolaritonLaserParams {
        topological_polariton_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialPolaritonLaserSolver::new(p_low);
    let s_high = QuantumMetamaterialPolaritonLaserSolver::new(p_high);

    assert!(
        s_high.compute_polariton_laser_fidelity()
            > s_low.compute_polariton_laser_fidelity()
    );
    assert!(
        s_high.compute_coherent_soliton_retention_fraction()
            > s_low.compute_coherent_soliton_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_cavity_crosstalk_isolation_db()
            > s_low.compute_inter_cavity_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = QuantumMetamaterialPolaritonLaserParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialPolaritonLaserParams {
        acoustic_drive_frequency_ghz: 10.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialPolaritonLaserSolver::new(p_low);
    let s_high = QuantumMetamaterialPolaritonLaserSolver::new(p_high);

    assert!(
        s_high.compute_polariton_laser_fidelity()
            > s_low.compute_polariton_laser_fidelity()
    );
    assert!(
        s_high.compute_coherent_soliton_retention_fraction()
            > s_low.compute_coherent_soliton_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_cavity_crosstalk_isolation_db()
            > s_low.compute_inter_cavity_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_soliton_dispatch_speed_monotonicity() {
    let p_low = QuantumMetamaterialPolaritonLaserParams {
        soliton_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialPolaritonLaserParams {
        soliton_dispatch_speed_m_per_s: 2500.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialPolaritonLaserSolver::new(p_low);
    let s_high = QuantumMetamaterialPolaritonLaserSolver::new(p_high);

    assert!(
        s_high.compute_polariton_laser_fidelity()
            > s_low.compute_polariton_laser_fidelity()
    );
    assert!(
        s_high.compute_coherent_soliton_retention_fraction()
            > s_low.compute_coherent_soliton_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_cavity_crosstalk_isolation_db()
            > s_low.compute_inter_cavity_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_optical_pump_power_monotonicity() {
    let p_low = QuantumMetamaterialPolaritonLaserParams {
        optical_pump_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialPolaritonLaserParams {
        optical_pump_power_uw: 25.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialPolaritonLaserSolver::new(p_low);
    let s_high = QuantumMetamaterialPolaritonLaserSolver::new(p_high);

    assert!(
        s_high.compute_polariton_laser_fidelity()
            > s_low.compute_polariton_laser_fidelity()
    );
    assert!(
        s_high.compute_coherent_soliton_retention_fraction()
            > s_low.compute_coherent_soliton_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_cavity_crosstalk_isolation_db()
            > s_low.compute_inter_cavity_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_metamaterial_cavities_monotonicity() {
    let p_low = QuantumMetamaterialPolaritonLaserParams {
        synthetic_metamaterial_cavities_factor: 2.0,
        ..Default::default()
    };
    let p_high = QuantumMetamaterialPolaritonLaserParams {
        synthetic_metamaterial_cavities_factor: 6.0,
        ..Default::default()
    };

    let s_low = QuantumMetamaterialPolaritonLaserSolver::new(p_low);
    let s_high = QuantumMetamaterialPolaritonLaserSolver::new(p_high);

    assert!(
        s_high.compute_polariton_laser_fidelity()
            > s_low.compute_polariton_laser_fidelity()
    );
    assert!(
        s_high.compute_coherent_soliton_retention_fraction()
            > s_low.compute_coherent_soliton_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_cavity_crosstalk_isolation_db()
            > s_low.compute_inter_cavity_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = QuantumMetamaterialPolaritonLaserParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = QuantumMetamaterialPolaritonLaserParams {
        cryogenic_temperature_mk: 40.0,
        ..Default::default()
    };

    let s_cold = QuantumMetamaterialPolaritonLaserSolver::new(p_cold);
    let s_warm = QuantumMetamaterialPolaritonLaserSolver::new(p_warm);

    assert!(
        s_cold.compute_polariton_laser_fidelity()
            > s_warm.compute_polariton_laser_fidelity()
    );
    assert!(
        s_cold.compute_coherent_soliton_retention_fraction()
            > s_warm.compute_coherent_soliton_retention_fraction()
    );
    assert!(
        s_cold.compute_topological_protection_gap_mhz()
            > s_warm.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_cold.compute_inter_cavity_crosstalk_isolation_db()
            > s_warm.compute_inter_cavity_crosstalk_isolation_db()
    );
    assert!(
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_metamaterial_lattice_pitch_scaling() {
    let p_narrow = QuantumMetamaterialPolaritonLaserParams {
        metamaterial_lattice_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = QuantumMetamaterialPolaritonLaserParams {
        metamaterial_lattice_pitch_um: 22.0,
        ..Default::default()
    };

    let s_narrow = QuantumMetamaterialPolaritonLaserSolver::new(p_narrow);
    let s_wide = QuantumMetamaterialPolaritonLaserSolver::new(p_wide);

    assert!(
        s_wide.compute_polariton_laser_fidelity()
            > s_narrow.compute_polariton_laser_fidelity()
    );
    assert!(
        s_wide.compute_coherent_soliton_retention_fraction()
            > s_narrow.compute_coherent_soliton_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_narrow.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_inter_cavity_crosstalk_isolation_db()
            > s_narrow.compute_inter_cavity_crosstalk_isolation_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_narrow.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_extreme_physical_limits_compliance() {
    let min_params = QuantumMetamaterialPolaritonLaserParams::new(
        1.0, 2.0, 1.0, 200.0, 50.0, 0.5, 1.0, 0.5,
    );
    let solver_min = QuantumMetamaterialPolaritonLaserSolver::new(min_params);
    let metrics_min = solver_min.evaluate_metrics();

    assert!(metrics_min.polariton_laser_fidelity >= 0.9980);
    assert!(metrics_min.coherent_soliton_retention_fraction >= 0.9970);
    assert!(metrics_min.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_min.inter_cavity_crosstalk_isolation_db >= 55.0);
    assert!(metrics_min.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_min.is_physically_compliant);

    let max_params = QuantumMetamaterialPolaritonLaserParams::new(
        35.0, 45.0, 12.0, 3000.0, 1.0, 30.0, 8.0, 25.0,
    );
    let solver_max = QuantumMetamaterialPolaritonLaserSolver::new(max_params);
    let metrics_max = solver_max.evaluate_metrics();

    assert!(metrics_max.polariton_laser_fidelity >= 0.9980);
    assert!(metrics_max.coherent_soliton_retention_fraction >= 0.9970);
    assert!(metrics_max.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_max.inter_cavity_crosstalk_isolation_db >= 55.0);
    assert!(metrics_max.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_max.is_physically_compliant);
}
