#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological
//! Non-Abelian Parafermion Braiding Router & Fractional Quantum Logic Engine.

use phonon_models::parafermion_braiding_router::ParafermionBraidingRouterParams;
use phonon_solver::parafermion_braiding_router::ParafermionBraidingRouterSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ParafermionBraidingRouterParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.parafermion_coupling_mev, 1.0);
    assert_eq!(underflow.topological_parafermion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.routing_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_routing_channels_factor, 1.0);
    assert_eq!(underflow.parafermion_channel_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = ParafermionBraidingRouterParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.parafermion_coupling_mev, 35.0);
    assert_eq!(overflow.topological_parafermion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.routing_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_routing_channels_factor, 8.0);
    assert_eq!(overflow.parafermion_channel_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ParafermionBraidingRouterParams::default();
    assert_eq!(params.parafermion_coupling_mev, 35.0);
    assert_eq!(params.topological_parafermion_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.routing_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 17.8);
    assert_eq!(params.synthetic_routing_channels_factor, 4.0);
    assert_eq!(params.parafermion_channel_pitch_um, 16.8);

    let solver = ParafermionBraidingRouterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.parafermion_braiding_fidelity >= 0.9980,
        "Parafermion braiding fidelity must be >= 0.9980, got {:.6}",
        metrics.parafermion_braiding_fidelity
    );
    assert!(
        metrics.fractional_state_retention_fraction >= 0.9970,
        "Fractional state retention fraction must be >= 0.9970, got {:.6}",
        metrics.fractional_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_channel_crosstalk_isolation_db >= 55.0,
        "Inter-channel crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_channel_crosstalk_isolation_db
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
fn test_parafermion_coupling_scaling() {
    let p_low = ParafermionBraidingRouterParams {
        parafermion_coupling_mev: 2.0,
        ..Default::default()
    };
    let p_high = ParafermionBraidingRouterParams {
        parafermion_coupling_mev: 34.0,
        ..Default::default()
    };

    let solver_low = ParafermionBraidingRouterSolver::new(p_low);
    let solver_high = ParafermionBraidingRouterSolver::new(p_high);

    assert!(
        solver_high.compute_parafermion_braiding_fidelity()
            > solver_low.compute_parafermion_braiding_fidelity(),
        "Higher parafermion coupling energy should enhance braiding fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher parafermion coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher parafermion coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_parafermion_gap_scaling() {
    let p_low = ParafermionBraidingRouterParams {
        topological_parafermion_gap_mev: 3.0,
        ..Default::default()
    };
    let p_high = ParafermionBraidingRouterParams {
        topological_parafermion_gap_mev: 44.5,
        ..Default::default()
    };

    let solver_low = ParafermionBraidingRouterSolver::new(p_low);
    let solver_high = ParafermionBraidingRouterSolver::new(p_high);

    assert!(
        solver_high.compute_parafermion_braiding_fidelity()
            > solver_low.compute_parafermion_braiding_fidelity(),
        "Higher topological parafermion gap should improve braiding fidelity"
    );
    assert!(
        solver_high.compute_fractional_state_retention_fraction()
            > solver_low.compute_fractional_state_retention_fraction(),
        "Higher topological parafermion gap should improve fractional state retention"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher topological parafermion gap should widen protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let p_low = ParafermionBraidingRouterParams {
        acoustic_drive_frequency_ghz: 1.5,
        ..Default::default()
    };
    let p_high = ParafermionBraidingRouterParams {
        acoustic_drive_frequency_ghz: 11.5,
        ..Default::default()
    };

    let solver_low = ParafermionBraidingRouterSolver::new(p_low);
    let solver_high = ParafermionBraidingRouterSolver::new(p_high);

    assert!(
        solver_high.compute_parafermion_braiding_fidelity()
            > solver_low.compute_parafermion_braiding_fidelity(),
        "Higher acoustic drive frequency should enhance braiding fidelity"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic drive frequency should decrease dephasing rate"
    );
}

#[test]
fn test_routing_dispatch_speed_scaling() {
    let p_low = ParafermionBraidingRouterParams {
        routing_dispatch_speed_m_per_s: 300.0,
        ..Default::default()
    };
    let p_high = ParafermionBraidingRouterParams {
        routing_dispatch_speed_m_per_s: 2900.0,
        ..Default::default()
    };

    let solver_low = ParafermionBraidingRouterSolver::new(p_low);
    let solver_high = ParafermionBraidingRouterSolver::new(p_high);

    assert!(
        solver_high.compute_parafermion_braiding_fidelity()
            > solver_low.compute_parafermion_braiding_fidelity(),
        "Faster routing dispatch speed should enhance braiding fidelity"
    );
    assert!(
        solver_high.compute_fractional_state_retention_fraction()
            > solver_low.compute_fractional_state_retention_fraction(),
        "Faster routing dispatch speed should enhance fractional state retention"
    );
}

#[test]
fn test_cryogenic_temperature_damping() {
    let p_cold = ParafermionBraidingRouterParams {
        cryogenic_temperature_mk: 1.5,
        ..Default::default()
    };
    let p_warm = ParafermionBraidingRouterParams {
        cryogenic_temperature_mk: 48.0,
        ..Default::default()
    };

    let solver_cold = ParafermionBraidingRouterSolver::new(p_cold);
    let solver_warm = ParafermionBraidingRouterSolver::new(p_warm);

    assert!(
        solver_cold.compute_parafermion_braiding_fidelity()
            > solver_warm.compute_parafermion_braiding_fidelity(),
        "Lower cryogenic temperature should yield higher braiding fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should suppress thermal dephasing"
    );
    assert!(
        solver_cold.compute_topological_protection_gap_mhz()
            > solver_warm.compute_topological_protection_gap_mhz(),
        "Lower cryogenic temperature should preserve wider effective protection gap"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let p_low = ParafermionBraidingRouterParams {
        microwave_probe_power_uw: 1.0,
        ..Default::default()
    };
    let p_high = ParafermionBraidingRouterParams {
        microwave_probe_power_uw: 28.0,
        ..Default::default()
    };

    let solver_low = ParafermionBraidingRouterSolver::new(p_low);
    let solver_high = ParafermionBraidingRouterSolver::new(p_high);

    assert!(
        solver_high.compute_parafermion_braiding_fidelity()
            > solver_low.compute_parafermion_braiding_fidelity(),
        "Optimal microwave probe power enhances readout signal and braiding fidelity"
    );
    assert!(
        solver_high.compute_inter_channel_crosstalk_isolation_db()
            > solver_low.compute_inter_channel_crosstalk_isolation_db(),
        "Optimal microwave probe power improves crosstalk isolation"
    );
}

#[test]
fn test_synthetic_routing_channels_scaling() {
    let p_low = ParafermionBraidingRouterParams {
        synthetic_routing_channels_factor: 1.2,
        ..Default::default()
    };
    let p_high = ParafermionBraidingRouterParams {
        synthetic_routing_channels_factor: 7.8,
        ..Default::default()
    };

    let solver_low = ParafermionBraidingRouterSolver::new(p_low);
    let solver_high = ParafermionBraidingRouterSolver::new(p_high);

    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher synthetic routing channels factor increases effective protection gap"
    );
    assert!(
        solver_high.compute_inter_channel_crosstalk_isolation_db()
            > solver_low.compute_inter_channel_crosstalk_isolation_db(),
        "Higher synthetic routing channels factor improves channel isolation"
    );
}

#[test]
fn test_parafermion_channel_pitch_scaling() {
    let p_dense = ParafermionBraidingRouterParams {
        parafermion_channel_pitch_um: 1.0,
        ..Default::default()
    };
    let p_sparse = ParafermionBraidingRouterParams {
        parafermion_channel_pitch_um: 19.0,
        ..Default::default()
    };

    let solver_dense = ParafermionBraidingRouterSolver::new(p_dense);
    let solver_sparse = ParafermionBraidingRouterSolver::new(p_sparse);

    assert!(
        solver_sparse.compute_inter_channel_crosstalk_isolation_db()
            > solver_dense.compute_inter_channel_crosstalk_isolation_db(),
        "Wider channel pitch must significantly improve crosstalk isolation"
    );
    assert!(
        solver_sparse.compute_topological_mode_dephasing_rate_hz()
            < solver_dense.compute_topological_mode_dephasing_rate_hz(),
        "Wider channel pitch reduces capacitive/parasitic dephasing"
    );
}
