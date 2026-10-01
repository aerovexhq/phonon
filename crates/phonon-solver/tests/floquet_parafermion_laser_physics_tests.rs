#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Floquet-Chern Parafermion Laser & Coherent Soliton Router Engine
//! (Phase 282).

use phonon_models::floquet_parafermion_laser::FloquetParafermionLaserParams;
use phonon_solver::floquet_parafermion_laser::FloquetParafermionLaserSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FloquetParafermionLaserParams::new(
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
    assert_eq!(underflow.topological_parafermion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.soliton_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.optical_pump_power_uw, 0.5);
    assert_eq!(underflow.synthetic_laser_modes_factor, 1.0);
    assert_eq!(underflow.laser_cavity_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = FloquetParafermionLaserParams::new(
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
    assert_eq!(overflow.topological_parafermion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.soliton_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.optical_pump_power_uw, 30.0);
    assert_eq!(overflow.synthetic_laser_modes_factor, 8.0);
    assert_eq!(overflow.laser_cavity_pitch_um, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FloquetParafermionLaserParams::default();
    assert_eq!(params.laser_coupling_mev, 35.0);
    assert_eq!(params.topological_parafermion_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.soliton_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.optical_pump_power_uw, 29.0);
    assert_eq!(params.synthetic_laser_modes_factor, 4.0);
    assert_eq!(params.laser_cavity_pitch_um, 25.0);

    let solver = FloquetParafermionLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.laser_fidelity >= 0.9980,
        "Laser fidelity must be >= 0.9980, got {:.6}",
        metrics.laser_fidelity
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
        metrics.inter_mode_crosstalk_isolation_db >= 55.0,
        "Inter-mode crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_mode_crosstalk_isolation_db
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
    let p_low = FloquetParafermionLaserParams {
        laser_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionLaserParams {
        laser_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionLaserSolver::new(p_low);
    let s_high = FloquetParafermionLaserSolver::new(p_high);

    assert!(
        s_high.compute_laser_fidelity()
            > s_low.compute_laser_fidelity()
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
        s_high.compute_inter_mode_crosstalk_isolation_db()
            > s_low.compute_inter_mode_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_parafermion_gap_monotonicity() {
    let p_low = FloquetParafermionLaserParams {
        topological_parafermion_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionLaserParams {
        topological_parafermion_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionLaserSolver::new(p_low);
    let s_high = FloquetParafermionLaserSolver::new(p_high);

    assert!(
        s_high.compute_laser_fidelity()
            > s_low.compute_laser_fidelity()
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
        s_high.compute_inter_mode_crosstalk_isolation_db()
            > s_low.compute_inter_mode_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = FloquetParafermionLaserParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionLaserParams {
        acoustic_drive_frequency_ghz: 11.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionLaserSolver::new(p_low);
    let s_high = FloquetParafermionLaserSolver::new(p_high);

    assert!(
        s_high.compute_laser_fidelity()
            > s_low.compute_laser_fidelity()
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
        s_high.compute_inter_mode_crosstalk_isolation_db()
            > s_low.compute_inter_mode_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_soliton_dispatch_speed_monotonicity() {
    let p_low = FloquetParafermionLaserParams {
        soliton_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionLaserParams {
        soliton_dispatch_speed_m_per_s: 2800.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionLaserSolver::new(p_low);
    let s_high = FloquetParafermionLaserSolver::new(p_high);

    assert!(
        s_high.compute_laser_fidelity()
            > s_low.compute_laser_fidelity()
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
        s_high.compute_inter_mode_crosstalk_isolation_db()
            > s_low.compute_inter_mode_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_optical_pump_power_monotonicity() {
    let p_low = FloquetParafermionLaserParams {
        optical_pump_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionLaserParams {
        optical_pump_power_uw: 28.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionLaserSolver::new(p_low);
    let s_high = FloquetParafermionLaserSolver::new(p_high);

    assert!(
        s_high.compute_laser_fidelity()
            > s_low.compute_laser_fidelity()
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
        s_high.compute_inter_mode_crosstalk_isolation_db()
            > s_low.compute_inter_mode_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_laser_modes_monotonicity() {
    let p_low = FloquetParafermionLaserParams {
        synthetic_laser_modes_factor: 2.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionLaserParams {
        synthetic_laser_modes_factor: 7.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionLaserSolver::new(p_low);
    let s_high = FloquetParafermionLaserSolver::new(p_high);

    assert!(
        s_high.compute_laser_fidelity()
            > s_low.compute_laser_fidelity()
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
        s_high.compute_inter_mode_crosstalk_isolation_db()
            > s_low.compute_inter_mode_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = FloquetParafermionLaserParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = FloquetParafermionLaserParams {
        cryogenic_temperature_mk: 40.0,
        ..Default::default()
    };

    let s_cold = FloquetParafermionLaserSolver::new(p_cold);
    let s_warm = FloquetParafermionLaserSolver::new(p_warm);

    assert!(
        s_cold.compute_laser_fidelity()
            > s_warm.compute_laser_fidelity()
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
        s_cold.compute_inter_mode_crosstalk_isolation_db()
            > s_warm.compute_inter_mode_crosstalk_isolation_db()
    );
    assert!(
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_laser_cavity_pitch_scaling() {
    let p_narrow = FloquetParafermionLaserParams {
        laser_cavity_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = FloquetParafermionLaserParams {
        laser_cavity_pitch_um: 22.0,
        ..Default::default()
    };

    let s_narrow = FloquetParafermionLaserSolver::new(p_narrow);
    let s_wide = FloquetParafermionLaserSolver::new(p_wide);

    assert!(
        s_wide.compute_laser_fidelity()
            > s_narrow.compute_laser_fidelity()
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
        s_wide.compute_inter_mode_crosstalk_isolation_db()
            > s_narrow.compute_inter_mode_crosstalk_isolation_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_narrow.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_extreme_physical_limits_compliance() {
    let min_params = FloquetParafermionLaserParams::new(
        1.0, 2.0, 1.0, 200.0, 50.0, 0.5, 1.0, 0.5,
    );
    let solver_min = FloquetParafermionLaserSolver::new(min_params);
    let metrics_min = solver_min.evaluate_metrics();

    assert!(metrics_min.laser_fidelity >= 0.9980);
    assert!(metrics_min.coherent_soliton_retention_fraction >= 0.9970);
    assert!(metrics_min.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_min.inter_mode_crosstalk_isolation_db >= 55.0);
    assert!(metrics_min.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_min.is_physically_compliant);

    let max_params = FloquetParafermionLaserParams::new(
        35.0, 45.0, 12.0, 3000.0, 1.0, 30.0, 8.0, 25.0,
    );
    let solver_max = FloquetParafermionLaserSolver::new(max_params);
    let metrics_max = solver_max.evaluate_metrics();

    assert!(metrics_max.laser_fidelity >= 0.9980);
    assert!(metrics_max.coherent_soliton_retention_fraction >= 0.9970);
    assert!(metrics_max.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_max.inter_mode_crosstalk_isolation_db >= 55.0);
    assert!(metrics_max.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_max.is_physically_compliant);
}
