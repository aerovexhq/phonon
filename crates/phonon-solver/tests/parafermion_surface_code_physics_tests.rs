#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological Non-Abelian
//! Parafermion Braiding Lattice & Fractional Fault-Tolerant Surface Code Engine.

use phonon_models::parafermion_surface_code::ParafermionSurfaceCodeParams;
use phonon_solver::parafermion_surface_code::ParafermionSurfaceCodeSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ParafermionSurfaceCodeParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.lattice_coupling_mev, 1.0);
    assert_eq!(underflow.topological_code_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.braiding_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_code_stabilizers_factor, 1.0);
    assert_eq!(underflow.surface_lattice_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = ParafermionSurfaceCodeParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.lattice_coupling_mev, 35.0);
    assert_eq!(overflow.topological_code_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.braiding_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_code_stabilizers_factor, 8.0);
    assert_eq!(overflow.surface_lattice_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ParafermionSurfaceCodeParams::default();
    assert_eq!(params.lattice_coupling_mev, 35.0);
    assert_eq!(params.topological_code_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.braiding_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 18.8);
    assert_eq!(params.synthetic_code_stabilizers_factor, 4.0);
    assert_eq!(params.surface_lattice_pitch_um, 17.8);

    let solver = ParafermionSurfaceCodeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.surface_code_fidelity >= 0.9980,
        "Surface code fidelity must be >= 0.9980, got {:.6}",
        metrics.surface_code_fidelity
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
        metrics.inter_node_crosstalk_isolation_db >= 55.0,
        "Inter-node crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_node_crosstalk_isolation_db
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
fn test_lattice_coupling_scaling() {
    let p_low = ParafermionSurfaceCodeParams {
        lattice_coupling_mev: 2.0,
        ..Default::default()
    };
    let p_high = ParafermionSurfaceCodeParams {
        lattice_coupling_mev: 34.0,
        ..Default::default()
    };

    let solver_low = ParafermionSurfaceCodeSolver::new(p_low);
    let solver_high = ParafermionSurfaceCodeSolver::new(p_high);

    assert!(
        solver_high.compute_surface_code_fidelity()
            > solver_low.compute_surface_code_fidelity(),
        "Higher lattice coupling energy should enhance surface code fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher lattice coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher lattice coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_code_gap_scaling() {
    let p_low = ParafermionSurfaceCodeParams {
        topological_code_gap_mev: 3.0,
        ..Default::default()
    };
    let p_high = ParafermionSurfaceCodeParams {
        topological_code_gap_mev: 44.5,
        ..Default::default()
    };

    let solver_low = ParafermionSurfaceCodeSolver::new(p_low);
    let solver_high = ParafermionSurfaceCodeSolver::new(p_high);

    assert!(
        solver_high.compute_fractional_state_retention_fraction()
            > solver_low.compute_fractional_state_retention_fraction(),
        "Wider topological code gap should improve fractional state retention"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological code gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let p_low = ParafermionSurfaceCodeParams {
        acoustic_drive_frequency_ghz: 1.5,
        ..Default::default()
    };
    let p_high = ParafermionSurfaceCodeParams {
        acoustic_drive_frequency_ghz: 11.8,
        ..Default::default()
    };

    let solver_low = ParafermionSurfaceCodeSolver::new(p_low);
    let solver_high = ParafermionSurfaceCodeSolver::new(p_high);

    assert!(
        solver_high.compute_surface_code_fidelity()
            > solver_low.compute_surface_code_fidelity(),
        "Higher acoustic drive frequency should enhance surface code fidelity"
    );
    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_braiding_dispatch_speed_scaling() {
    let p_low = ParafermionSurfaceCodeParams {
        braiding_dispatch_speed_m_per_s: 300.0,
        ..Default::default()
    };
    let p_high = ParafermionSurfaceCodeParams {
        braiding_dispatch_speed_m_per_s: 2950.0,
        ..Default::default()
    };

    let solver_low = ParafermionSurfaceCodeSolver::new(p_low);
    let solver_high = ParafermionSurfaceCodeSolver::new(p_high);

    assert!(
        solver_high.compute_fractional_state_retention_fraction()
            > solver_low.compute_fractional_state_retention_fraction(),
        "Higher braiding dispatch speed should improve state retention"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher braiding dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let p_cold = ParafermionSurfaceCodeParams {
        cryogenic_temperature_mk: 2.0,
        ..Default::default()
    };
    let p_warm = ParafermionSurfaceCodeParams {
        cryogenic_temperature_mk: 48.0,
        ..Default::default()
    };

    let solver_cold = ParafermionSurfaceCodeSolver::new(p_cold);
    let solver_warm = ParafermionSurfaceCodeSolver::new(p_warm);

    assert!(
        solver_cold.compute_surface_code_fidelity()
            > solver_warm.compute_surface_code_fidelity(),
        "Lower cryogenic temperature should improve surface code fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let p_low = ParafermionSurfaceCodeParams {
        microwave_probe_power_uw: 1.0,
        ..Default::default()
    };
    let p_high = ParafermionSurfaceCodeParams {
        microwave_probe_power_uw: 29.0,
        ..Default::default()
    };

    let solver_low = ParafermionSurfaceCodeSolver::new(p_low);
    let solver_high = ParafermionSurfaceCodeSolver::new(p_high);

    assert!(
        solver_high.compute_surface_code_fidelity()
            > solver_low.compute_surface_code_fidelity(),
        "Higher probe power should improve surface code fidelity"
    );
    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_code_stabilizers_scaling() {
    let p_low = ParafermionSurfaceCodeParams {
        synthetic_code_stabilizers_factor: 1.5,
        ..Default::default()
    };
    let p_high = ParafermionSurfaceCodeParams {
        synthetic_code_stabilizers_factor: 7.5,
        ..Default::default()
    };

    let solver_low = ParafermionSurfaceCodeSolver::new(p_low);
    let solver_high = ParafermionSurfaceCodeSolver::new(p_high);

    assert!(
        solver_high.compute_inter_node_crosstalk_isolation_db()
            > solver_low.compute_inter_node_crosstalk_isolation_db(),
        "Higher synthetic code stabilizers factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic code stabilizers factor should suppress dephasing rate"
    );
}

#[test]
fn test_surface_lattice_pitch_scaling() {
    let p_small = ParafermionSurfaceCodeParams {
        surface_lattice_pitch_um: 1.0,
        ..Default::default()
    };
    let p_large = ParafermionSurfaceCodeParams {
        surface_lattice_pitch_um: 19.0,
        ..Default::default()
    };

    let solver_small = ParafermionSurfaceCodeSolver::new(p_small);
    let solver_large = ParafermionSurfaceCodeSolver::new(p_large);

    assert!(
        solver_large.compute_inter_node_crosstalk_isolation_db()
            > solver_small.compute_inter_node_crosstalk_isolation_db(),
        "Larger surface lattice pitch should enhance crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger surface lattice pitch should increase protection gap"
    );
}
