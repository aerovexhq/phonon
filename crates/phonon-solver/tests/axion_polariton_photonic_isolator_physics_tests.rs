#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Topological Axion-Polariton Photonic Isolator & Quantum Routing Engine.

use phonon_models::axion_polariton_photonic_isolator::AxionPolaritonPhotonicIsolatorParams;
use phonon_solver::axion_polariton_photonic_isolator::AxionPolaritonPhotonicIsolatorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AxionPolaritonPhotonicIsolatorParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.isolator_coupling_mev, 1.0);
    assert_eq!(underflow.topological_axion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.routing_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.optical_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_router_ports_factor, 1.0);
    assert_eq!(underflow.waveguide_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = AxionPolaritonPhotonicIsolatorParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.isolator_coupling_mev, 35.0);
    assert_eq!(overflow.topological_axion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.routing_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.optical_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_router_ports_factor, 8.0);
    assert_eq!(overflow.waveguide_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AxionPolaritonPhotonicIsolatorParams::default();
    assert_eq!(params.isolator_coupling_mev, 35.0);
    assert_eq!(params.topological_axion_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.routing_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.optical_probe_power_uw, 22.0);
    assert_eq!(params.synthetic_router_ports_factor, 4.0);
    assert_eq!(params.waveguide_pitch_um, 21.0);

    let solver = AxionPolaritonPhotonicIsolatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.isolator_fidelity >= 0.9980,
        "Isolator fidelity must be >= 0.9980, got {:.6}",
        metrics.isolator_fidelity
    );
    assert!(
        metrics.polariton_state_retention_fraction >= 0.9970,
        "Polariton state retention fraction must be >= 0.9970, got {:.6}",
        metrics.polariton_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.isolation_contrast_db >= 55.0,
        "Isolation contrast must be >= 55.0 dB, got {:.4} dB",
        metrics.isolation_contrast_db
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
fn test_isolator_coupling_monotonicity() {
    let p_low = AxionPolaritonPhotonicIsolatorParams {
        isolator_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonPhotonicIsolatorParams {
        isolator_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonPhotonicIsolatorSolver::new(p_low);
    let s_high = AxionPolaritonPhotonicIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_axion_gap_monotonicity() {
    let p_low = AxionPolaritonPhotonicIsolatorParams {
        topological_axion_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonPhotonicIsolatorParams {
        topological_axion_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonPhotonicIsolatorSolver::new(p_low);
    let s_high = AxionPolaritonPhotonicIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = AxionPolaritonPhotonicIsolatorParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonPhotonicIsolatorParams {
        acoustic_drive_frequency_ghz: 10.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonPhotonicIsolatorSolver::new(p_low);
    let s_high = AxionPolaritonPhotonicIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_routing_dispatch_speed_monotonicity() {
    let p_low = AxionPolaritonPhotonicIsolatorParams {
        routing_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonPhotonicIsolatorParams {
        routing_dispatch_speed_m_per_s: 2500.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonPhotonicIsolatorSolver::new(p_low);
    let s_high = AxionPolaritonPhotonicIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_optical_probe_power_monotonicity() {
    let p_low = AxionPolaritonPhotonicIsolatorParams {
        optical_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonPhotonicIsolatorParams {
        optical_probe_power_uw: 25.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonPhotonicIsolatorSolver::new(p_low);
    let s_high = AxionPolaritonPhotonicIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_router_ports_monotonicity() {
    let p_low = AxionPolaritonPhotonicIsolatorParams {
        synthetic_router_ports_factor: 2.0,
        ..Default::default()
    };
    let p_high = AxionPolaritonPhotonicIsolatorParams {
        synthetic_router_ports_factor: 6.0,
        ..Default::default()
    };

    let s_low = AxionPolaritonPhotonicIsolatorSolver::new(p_low);
    let s_high = AxionPolaritonPhotonicIsolatorSolver::new(p_high);

    assert!(
        s_high.compute_isolator_fidelity()
            > s_low.compute_isolator_fidelity()
    );
    assert!(
        s_high.compute_polariton_state_retention_fraction()
            > s_low.compute_polariton_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_isolation_contrast_db()
            > s_low.compute_isolation_contrast_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = AxionPolaritonPhotonicIsolatorParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = AxionPolaritonPhotonicIsolatorParams {
        cryogenic_temperature_mk: 40.0,
        ..Default::default()
    };

    let s_cold = AxionPolaritonPhotonicIsolatorSolver::new(p_cold);
    let s_warm = AxionPolaritonPhotonicIsolatorSolver::new(p_warm);

    assert!(
        s_cold.compute_isolator_fidelity()
            > s_warm.compute_isolator_fidelity()
    );
    assert!(
        s_cold.compute_polariton_state_retention_fraction()
            > s_warm.compute_polariton_state_retention_fraction()
    );
    assert!(
        s_cold.compute_topological_protection_gap_mhz()
            > s_warm.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_cold.compute_isolation_contrast_db()
            > s_warm.compute_isolation_contrast_db()
    );
    assert!(
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_waveguide_pitch_scaling() {
    let p_narrow = AxionPolaritonPhotonicIsolatorParams {
        waveguide_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = AxionPolaritonPhotonicIsolatorParams {
        waveguide_pitch_um: 18.0,
        ..Default::default()
    };

    let s_narrow = AxionPolaritonPhotonicIsolatorSolver::new(p_narrow);
    let s_wide = AxionPolaritonPhotonicIsolatorSolver::new(p_wide);

    assert!(
        s_wide.compute_isolator_fidelity()
            > s_narrow.compute_isolator_fidelity()
    );
    assert!(
        s_wide.compute_polariton_state_retention_fraction()
            > s_narrow.compute_polariton_state_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_narrow.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_isolation_contrast_db()
            > s_narrow.compute_isolation_contrast_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_narrow.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_extreme_physical_limits_compliance() {
    let min_params = AxionPolaritonPhotonicIsolatorParams::new(
        1.0, 2.0, 1.0, 200.0, 50.0, 0.5, 1.0, 0.5,
    );
    let solver_min = AxionPolaritonPhotonicIsolatorSolver::new(min_params);
    let metrics_min = solver_min.evaluate_metrics();

    assert!(metrics_min.isolator_fidelity >= 0.9980);
    assert!(metrics_min.polariton_state_retention_fraction >= 0.9970);
    assert!(metrics_min.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_min.isolation_contrast_db >= 55.0);
    assert!(metrics_min.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_min.is_physically_compliant);

    let max_params = AxionPolaritonPhotonicIsolatorParams::new(
        35.0, 45.0, 12.0, 3000.0, 1.0, 30.0, 8.0, 20.0,
    );
    let solver_max = AxionPolaritonPhotonicIsolatorSolver::new(max_params);
    let metrics_max = solver_max.evaluate_metrics();

    assert!(metrics_max.isolator_fidelity >= 0.9980);
    assert!(metrics_max.polariton_state_retention_fraction >= 0.9970);
    assert!(metrics_max.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_max.isolation_contrast_db >= 55.0);
    assert!(metrics_max.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_max.is_physically_compliant);
}
