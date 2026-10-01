#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven
//! Floquet-Chern Parafermion Topological Quantum Memory & Braiding Router Engine
//! (Phase 297).

use phonon_models::floquet_parafermion_memory::FloquetParafermionMemoryParams;
use phonon_solver::floquet_parafermion_memory::FloquetParafermionMemorySolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FloquetParafermionMemoryParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.5,   // below 1.0 factor
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.memory_coupling_mev, 1.0);
    assert_eq!(underflow.topological_parafermion_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.memory_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_memory_qudits_factor, 1.0);
    assert_eq!(underflow.memory_cell_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = FloquetParafermionMemoryParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        35.0,   // above 25.0 um
    );
    assert_eq!(overflow.memory_coupling_mev, 35.0);
    assert_eq!(overflow.topological_parafermion_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.memory_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_memory_qudits_factor, 8.0);
    assert_eq!(overflow.memory_cell_pitch_um, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FloquetParafermionMemoryParams::default();
    assert_eq!(params.memory_coupling_mev, 35.0);
    assert_eq!(params.topological_parafermion_gap_mev, 45.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.memory_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 29.9);
    assert_eq!(params.synthetic_memory_qudits_factor, 4.0);
    assert_eq!(params.memory_cell_pitch_um, 25.0);

    let solver = FloquetParafermionMemorySolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.memory_fidelity >= 0.9980,
        "Memory fidelity must be >= 0.9980, got {:.6}",
        metrics.memory_fidelity
    );
    assert!(
        metrics.topological_state_retention_fraction >= 0.9970,
        "Topological state retention fraction must be >= 0.9970, got {:.6}",
        metrics.topological_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_qudit_crosstalk_isolation_db >= 55.0,
        "Inter-qudit crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_qudit_crosstalk_isolation_db
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
fn test_memory_coupling_monotonicity() {
    let p_low = FloquetParafermionMemoryParams {
        memory_coupling_mev: 5.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionMemoryParams {
        memory_coupling_mev: 30.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionMemorySolver::new(p_low);
    let s_high = FloquetParafermionMemorySolver::new(p_high);

    assert!(
        s_high.compute_memory_fidelity()
            > s_low.compute_memory_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_qudit_crosstalk_isolation_db()
            > s_low.compute_inter_qudit_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_topological_parafermion_gap_monotonicity() {
    let p_low = FloquetParafermionMemoryParams {
        topological_parafermion_gap_mev: 5.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionMemoryParams {
        topological_parafermion_gap_mev: 40.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionMemorySolver::new(p_low);
    let s_high = FloquetParafermionMemorySolver::new(p_high);

    assert!(
        s_high.compute_memory_fidelity()
            > s_low.compute_memory_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_qudit_crosstalk_isolation_db()
            > s_low.compute_inter_qudit_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_acoustic_drive_frequency_monotonicity() {
    let p_low = FloquetParafermionMemoryParams {
        acoustic_drive_frequency_ghz: 2.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionMemoryParams {
        acoustic_drive_frequency_ghz: 11.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionMemorySolver::new(p_low);
    let s_high = FloquetParafermionMemorySolver::new(p_high);

    assert!(
        s_high.compute_memory_fidelity()
            > s_low.compute_memory_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_qudit_crosstalk_isolation_db()
            > s_low.compute_inter_qudit_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_memory_dispatch_speed_monotonicity() {
    let p_low = FloquetParafermionMemoryParams {
        memory_dispatch_speed_m_per_s: 500.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionMemoryParams {
        memory_dispatch_speed_m_per_s: 2800.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionMemorySolver::new(p_low);
    let s_high = FloquetParafermionMemorySolver::new(p_high);

    assert!(
        s_high.compute_memory_fidelity()
            > s_low.compute_memory_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_qudit_crosstalk_isolation_db()
            > s_low.compute_inter_qudit_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_microwave_probe_power_monotonicity() {
    let p_low = FloquetParafermionMemoryParams {
        microwave_probe_power_uw: 2.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionMemoryParams {
        microwave_probe_power_uw: 28.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionMemorySolver::new(p_low);
    let s_high = FloquetParafermionMemorySolver::new(p_high);

    assert!(
        s_high.compute_memory_fidelity()
            > s_low.compute_memory_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_qudit_crosstalk_isolation_db()
            > s_low.compute_inter_qudit_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_synthetic_memory_qudits_monotonicity() {
    let p_low = FloquetParafermionMemoryParams {
        synthetic_memory_qudits_factor: 2.0,
        ..Default::default()
    };
    let p_high = FloquetParafermionMemoryParams {
        synthetic_memory_qudits_factor: 7.0,
        ..Default::default()
    };

    let s_low = FloquetParafermionMemorySolver::new(p_low);
    let s_high = FloquetParafermionMemorySolver::new(p_high);

    assert!(
        s_high.compute_memory_fidelity()
            > s_low.compute_memory_fidelity()
    );
    assert!(
        s_high.compute_topological_state_retention_fraction()
            > s_low.compute_topological_state_retention_fraction()
    );
    assert!(
        s_high.compute_topological_protection_gap_mhz()
            > s_low.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_high.compute_inter_qudit_crosstalk_isolation_db()
            > s_low.compute_inter_qudit_crosstalk_isolation_db()
    );
    assert!(
        s_high.compute_topological_mode_dephasing_rate_hz()
            < s_low.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_cryogenic_temperature_sensitivity() {
    let p_cold = FloquetParafermionMemoryParams {
        cryogenic_temperature_mk: 5.0,
        ..Default::default()
    };
    let p_warm = FloquetParafermionMemoryParams {
        cryogenic_temperature_mk: 40.0,
        ..Default::default()
    };

    let s_cold = FloquetParafermionMemorySolver::new(p_cold);
    let s_warm = FloquetParafermionMemorySolver::new(p_warm);

    assert!(
        s_cold.compute_memory_fidelity()
            > s_warm.compute_memory_fidelity()
    );
    assert!(
        s_cold.compute_topological_state_retention_fraction()
            > s_warm.compute_topological_state_retention_fraction()
    );
    assert!(
        s_cold.compute_topological_protection_gap_mhz()
            > s_warm.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_cold.compute_inter_qudit_crosstalk_isolation_db()
            > s_warm.compute_inter_qudit_crosstalk_isolation_db()
    );
    assert!(
        s_cold.compute_topological_mode_dephasing_rate_hz()
            < s_warm.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_memory_cell_pitch_scaling() {
    let p_narrow = FloquetParafermionMemoryParams {
        memory_cell_pitch_um: 2.0,
        ..Default::default()
    };
    let p_wide = FloquetParafermionMemoryParams {
        memory_cell_pitch_um: 22.0,
        ..Default::default()
    };

    let s_narrow = FloquetParafermionMemorySolver::new(p_narrow);
    let s_wide = FloquetParafermionMemorySolver::new(p_wide);

    assert!(
        s_wide.compute_memory_fidelity()
            > s_narrow.compute_memory_fidelity()
    );
    assert!(
        s_wide.compute_topological_state_retention_fraction()
            > s_narrow.compute_topological_state_retention_fraction()
    );
    assert!(
        s_wide.compute_topological_protection_gap_mhz()
            > s_narrow.compute_topological_protection_gap_mhz()
    );
    assert!(
        s_wide.compute_inter_qudit_crosstalk_isolation_db()
            > s_narrow.compute_inter_qudit_crosstalk_isolation_db()
    );
    assert!(
        s_wide.compute_topological_mode_dephasing_rate_hz()
            < s_narrow.compute_topological_mode_dephasing_rate_hz()
    );
}

#[test]
fn test_extreme_physical_limits_compliance() {
    let min_params = FloquetParafermionMemoryParams::new(
        1.0, 2.0, 1.0, 200.0, 50.0, 0.5, 1.0, 0.5,
    );
    let solver_min = FloquetParafermionMemorySolver::new(min_params);
    let metrics_min = solver_min.evaluate_metrics();

    assert!(metrics_min.memory_fidelity >= 0.9980);
    assert!(metrics_min.topological_state_retention_fraction >= 0.9970);
    assert!(metrics_min.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_min.inter_qudit_crosstalk_isolation_db >= 55.0);
    assert!(metrics_min.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_min.is_physically_compliant);

    let max_params = FloquetParafermionMemoryParams::new(
        35.0, 45.0, 12.0, 3000.0, 1.0, 30.0, 8.0, 25.0,
    );
    let solver_max = FloquetParafermionMemorySolver::new(max_params);
    let metrics_max = solver_max.evaluate_metrics();

    assert!(metrics_max.memory_fidelity >= 0.9980);
    assert!(metrics_max.topological_state_retention_fraction >= 0.9970);
    assert!(metrics_max.topological_protection_gap_mhz >= 45.0);
    assert!(metrics_max.inter_qudit_crosstalk_isolation_db >= 55.0);
    assert!(metrics_max.topological_mode_dephasing_rate_hz <= 12.0);
    assert!(metrics_max.is_physically_compliant);
}
