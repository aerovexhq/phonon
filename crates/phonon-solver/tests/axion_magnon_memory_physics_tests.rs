#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Topological
//! Axion-Magnon Quantum Memory & Chiral Haloscope Transceiver Engine.

use phonon_models::axion_magnon_memory::AxionMagnonMemoryParams;
use phonon_solver::axion_magnon_memory::AxionMagnonMemorySolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AxionMagnonMemoryParams::new(
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
    assert_eq!(underflow.topological_memory_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.memory_dispatch_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_probe_power_uw, 0.5);
    assert_eq!(underflow.synthetic_memory_cells_factor, 1.0);
    assert_eq!(underflow.memory_cell_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = AxionMagnonMemoryParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        20.0,   // above 12.0 GHz
        4000.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        12.0,   // above 8.0 factor
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.memory_coupling_mev, 35.0);
    assert_eq!(overflow.topological_memory_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.memory_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_probe_power_uw, 30.0);
    assert_eq!(overflow.synthetic_memory_cells_factor, 8.0);
    assert_eq!(overflow.memory_cell_pitch_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AxionMagnonMemoryParams::default();
    assert_eq!(params.memory_coupling_mev, 35.0);
    assert_eq!(params.topological_memory_gap_mev, 43.5);
    assert_eq!(params.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(params.memory_dispatch_speed_m_per_s, 3000.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_probe_power_uw, 16.2);
    assert_eq!(params.synthetic_memory_cells_factor, 4.0);
    assert_eq!(params.memory_cell_pitch_um, 15.2);

    let solver = AxionMagnonMemorySolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.axion_magnon_memory_fidelity >= 0.9980,
        "Axion-magnon memory fidelity must be >= 0.9980, got {:.6}",
        metrics.axion_magnon_memory_fidelity
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
        metrics.inter_cell_crosstalk_isolation_db >= 55.0,
        "Inter-cell crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_cell_crosstalk_isolation_db
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
fn test_memory_coupling_scaling() {
    let p_low = AxionMagnonMemoryParams {
        memory_coupling_mev: 2.0,
        ..Default::default()
    };
    let p_high = AxionMagnonMemoryParams {
        memory_coupling_mev: 34.0,
        ..Default::default()
    };

    let solver_low = AxionMagnonMemorySolver::new(p_low);
    let solver_high = AxionMagnonMemorySolver::new(p_high);

    assert!(
        solver_high.compute_axion_magnon_memory_fidelity()
            > solver_low.compute_axion_magnon_memory_fidelity(),
        "Higher memory coupling energy should enhance memory fidelity"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher memory coupling energy should increase protection gap"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher memory coupling energy should suppress dephasing rate"
    );
}

#[test]
fn test_topological_memory_gap_scaling() {
    let p_low = AxionMagnonMemoryParams {
        topological_memory_gap_mev: 3.0,
        ..Default::default()
    };
    let p_high = AxionMagnonMemoryParams {
        topological_memory_gap_mev: 44.0,
        ..Default::default()
    };

    let solver_low = AxionMagnonMemorySolver::new(p_low);
    let solver_high = AxionMagnonMemorySolver::new(p_high);

    assert!(
        solver_high.compute_polariton_state_retention_fraction()
            > solver_low.compute_polariton_state_retention_fraction(),
        "Wider topological memory gap should improve polariton state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Wider topological memory gap should increase protection gap"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let p_low = AxionMagnonMemoryParams {
        acoustic_drive_frequency_ghz: 1.5,
        ..Default::default()
    };
    let p_high = AxionMagnonMemoryParams {
        acoustic_drive_frequency_ghz: 11.8,
        ..Default::default()
    };

    let solver_low = AxionMagnonMemorySolver::new(p_low);
    let solver_high = AxionMagnonMemorySolver::new(p_high);

    assert!(
        solver_high.compute_axion_magnon_memory_fidelity()
            > solver_low.compute_axion_magnon_memory_fidelity(),
        "Higher acoustic drive frequency should enhance memory fidelity"
    );
    assert!(
        solver_high.compute_inter_cell_crosstalk_isolation_db()
            > solver_low.compute_inter_cell_crosstalk_isolation_db(),
        "Higher acoustic drive frequency should improve crosstalk isolation"
    );
}

#[test]
fn test_memory_dispatch_speed_scaling() {
    let p_low = AxionMagnonMemoryParams {
        memory_dispatch_speed_m_per_s: 300.0,
        ..Default::default()
    };
    let p_high = AxionMagnonMemoryParams {
        memory_dispatch_speed_m_per_s: 2950.0,
        ..Default::default()
    };

    let solver_low = AxionMagnonMemorySolver::new(p_low);
    let solver_high = AxionMagnonMemorySolver::new(p_high);

    assert!(
        solver_high.compute_polariton_state_retention_fraction()
            > solver_low.compute_polariton_state_retention_fraction(),
        "Higher memory dispatch speed should improve polariton state retention fraction"
    );
    assert!(
        solver_high.compute_topological_protection_gap_mhz()
            > solver_low.compute_topological_protection_gap_mhz(),
        "Higher memory dispatch speed should increase protection gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let p_cold = AxionMagnonMemoryParams {
        cryogenic_temperature_mk: 2.0,
        ..Default::default()
    };
    let p_warm = AxionMagnonMemoryParams {
        cryogenic_temperature_mk: 48.0,
        ..Default::default()
    };

    let solver_cold = AxionMagnonMemorySolver::new(p_cold);
    let solver_warm = AxionMagnonMemorySolver::new(p_warm);

    assert!(
        solver_cold.compute_axion_magnon_memory_fidelity()
            > solver_warm.compute_axion_magnon_memory_fidelity(),
        "Lower cryogenic temperature should improve memory fidelity"
    );
    assert!(
        solver_cold.compute_topological_mode_dephasing_rate_hz()
            < solver_warm.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature should reduce dephasing rate"
    );
}

#[test]
fn test_microwave_probe_power_scaling() {
    let p_low = AxionMagnonMemoryParams {
        microwave_probe_power_uw: 1.0,
        ..Default::default()
    };
    let p_high = AxionMagnonMemoryParams {
        microwave_probe_power_uw: 29.0,
        ..Default::default()
    };

    let solver_low = AxionMagnonMemorySolver::new(p_low);
    let solver_high = AxionMagnonMemorySolver::new(p_high);

    assert!(
        solver_high.compute_axion_magnon_memory_fidelity()
            > solver_low.compute_axion_magnon_memory_fidelity(),
        "Higher probe power should improve memory fidelity"
    );
    assert!(
        solver_high.compute_inter_cell_crosstalk_isolation_db()
            > solver_low.compute_inter_cell_crosstalk_isolation_db(),
        "Higher probe power should improve crosstalk isolation"
    );
}

#[test]
fn test_synthetic_memory_cells_scaling() {
    let p_low = AxionMagnonMemoryParams {
        synthetic_memory_cells_factor: 1.5,
        ..Default::default()
    };
    let p_high = AxionMagnonMemoryParams {
        synthetic_memory_cells_factor: 7.5,
        ..Default::default()
    };

    let solver_low = AxionMagnonMemorySolver::new(p_low);
    let solver_high = AxionMagnonMemorySolver::new(p_high);

    assert!(
        solver_high.compute_inter_cell_crosstalk_isolation_db()
            > solver_low.compute_inter_cell_crosstalk_isolation_db(),
        "Higher synthetic memory cells factor should increase crosstalk isolation"
    );
    assert!(
        solver_high.compute_topological_mode_dephasing_rate_hz()
            < solver_low.compute_topological_mode_dephasing_rate_hz(),
        "Higher synthetic memory cells factor should suppress dephasing rate"
    );
}

#[test]
fn test_memory_cell_pitch_scaling() {
    let p_small = AxionMagnonMemoryParams {
        memory_cell_pitch_um: 1.0,
        ..Default::default()
    };
    let p_large = AxionMagnonMemoryParams {
        memory_cell_pitch_um: 19.0,
        ..Default::default()
    };

    let solver_small = AxionMagnonMemorySolver::new(p_small);
    let solver_large = AxionMagnonMemorySolver::new(p_large);

    assert!(
        solver_large.compute_inter_cell_crosstalk_isolation_db()
            > solver_small.compute_inter_cell_crosstalk_isolation_db(),
        "Larger memory cell pitch should enhance inter-cell crosstalk isolation"
    );
    assert!(
        solver_large.compute_topological_protection_gap_mhz()
            > solver_small.compute_topological_protection_gap_mhz(),
        "Larger memory cell pitch should increase protection gap"
    );
}
