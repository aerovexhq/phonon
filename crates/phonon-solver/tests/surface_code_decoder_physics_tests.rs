#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological surface code anyon decoders and fault-tolerant syndrome
//! processors.

use phonon_models::surface_code_decoder::SurfaceCodeDecoderParams;
use phonon_solver::surface_code_decoder::SurfaceCodeDecoderSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SurfaceCodeDecoderParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        1.0,   // below 3.0
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.syndrome_coupling_energy_mev, 1.0);
    assert_eq!(underflow.superconducting_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_clock_frequency_ghz, 1.0);
    assert_eq!(underflow.matching_shuttling_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_readout_power_uw, 0.5);
    assert_eq!(underflow.code_distance_d, 3.0);
    assert_eq!(underflow.qubit_pitch_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SurfaceCodeDecoderParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        35.0,   // above 25.0
        20.0,   // above 15.0 um
    );
    assert_eq!(overflow.syndrome_coupling_energy_mev, 35.0);
    assert_eq!(overflow.superconducting_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_clock_frequency_ghz, 12.0);
    assert_eq!(overflow.matching_shuttling_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_readout_power_uw, 30.0);
    assert_eq!(overflow.code_distance_d, 25.0);
    assert_eq!(overflow.qubit_pitch_um, 15.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SurfaceCodeDecoderParams::default();
    assert_eq!(params.syndrome_coupling_energy_mev, 16.5);
    assert_eq!(params.superconducting_gap_mev, 22.0);
    assert_eq!(params.acoustic_clock_frequency_ghz, 5.8);
    assert_eq!(params.matching_shuttling_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_readout_power_uw, 5.6);
    assert_eq!(params.code_distance_d, 9.0);
    assert_eq!(params.qubit_pitch_um, 4.2);

    let solver = SurfaceCodeDecoderSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.decoding_fidelity >= 0.9980,
        "Decoding fidelity must be >= 0.9980, got {:.6}",
        metrics.decoding_fidelity
    );
    assert!(
        metrics.code_space_retention_fraction >= 0.9970,
        "Code space retention fraction must be >= 0.9970, got {:.6}",
        metrics.code_space_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_qubit_crosstalk_isolation_db >= 54.0,
        "Inter-qubit crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
        metrics.inter_qubit_crosstalk_isolation_db
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
fn test_syndrome_coupling_energy_scaling() {
    let mut low = SurfaceCodeDecoderParams::default();
    low.syndrome_coupling_energy_mev = 2.0;
    let mut high = SurfaceCodeDecoderParams::default();
    high.syndrome_coupling_energy_mev = 34.0;

    let low_solver = SurfaceCodeDecoderSolver::new(low);
    let high_solver = SurfaceCodeDecoderSolver::new(high);

    assert!(
        high_solver.compute_decoding_fidelity()
            > low_solver.compute_decoding_fidelity(),
        "Larger syndrome coupling energy must enhance decoding fidelity"
    );
    assert!(
        high_solver.compute_code_space_retention_fraction()
            > low_solver.compute_code_space_retention_fraction(),
        "Larger syndrome coupling energy must enhance code space retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger syndrome coupling energy must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_qubit_crosstalk_isolation_db()
            > low_solver.compute_inter_qubit_crosstalk_isolation_db(),
        "Larger syndrome coupling energy must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger syndrome coupling energy must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_superconducting_gap_scaling() {
    let mut low = SurfaceCodeDecoderParams::default();
    low.superconducting_gap_mev = 3.0;
    let mut high = SurfaceCodeDecoderParams::default();
    high.superconducting_gap_mev = 44.0;

    let low_solver = SurfaceCodeDecoderSolver::new(low);
    let high_solver = SurfaceCodeDecoderSolver::new(high);

    assert!(
        high_solver.compute_decoding_fidelity()
            > low_solver.compute_decoding_fidelity(),
        "Larger superconducting gap must enhance decoding fidelity"
    );
    assert!(
        high_solver.compute_code_space_retention_fraction()
            > low_solver.compute_code_space_retention_fraction(),
        "Larger superconducting gap must enhance code space retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger superconducting gap must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_qubit_crosstalk_isolation_db()
            > low_solver.compute_inter_qubit_crosstalk_isolation_db(),
        "Larger superconducting gap must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger superconducting gap must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_acoustic_clock_frequency_scaling() {
    let mut low = SurfaceCodeDecoderParams::default();
    low.acoustic_clock_frequency_ghz = 1.5;
    let mut high = SurfaceCodeDecoderParams::default();
    high.acoustic_clock_frequency_ghz = 11.5;

    let low_solver = SurfaceCodeDecoderSolver::new(low);
    let high_solver = SurfaceCodeDecoderSolver::new(high);

    assert!(
        high_solver.compute_decoding_fidelity()
            > low_solver.compute_decoding_fidelity(),
        "Higher acoustic clock frequency must enhance decoding fidelity"
    );
    assert!(
        high_solver.compute_code_space_retention_fraction()
            > low_solver.compute_code_space_retention_fraction(),
        "Higher acoustic clock frequency must enhance code space retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic clock frequency must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_qubit_crosstalk_isolation_db()
            > low_solver.compute_inter_qubit_crosstalk_isolation_db(),
        "Higher acoustic clock frequency must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic clock frequency must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_matching_shuttling_speed_scaling() {
    let mut low = SurfaceCodeDecoderParams::default();
    low.matching_shuttling_speed_m_per_s = 250.0;
    let mut high = SurfaceCodeDecoderParams::default();
    high.matching_shuttling_speed_m_per_s = 2900.0;

    let low_solver = SurfaceCodeDecoderSolver::new(low);
    let high_solver = SurfaceCodeDecoderSolver::new(high);

    assert!(
        high_solver.compute_decoding_fidelity()
            > low_solver.compute_decoding_fidelity(),
        "Faster matching shuttling speed must enhance decoding fidelity"
    );
    assert!(
        high_solver.compute_code_space_retention_fraction()
            > low_solver.compute_code_space_retention_fraction(),
        "Faster matching shuttling speed must enhance code space retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Faster matching shuttling speed must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_qubit_crosstalk_isolation_db()
            > low_solver.compute_inter_qubit_crosstalk_isolation_db(),
        "Faster matching shuttling speed must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Faster matching shuttling speed must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_temp = SurfaceCodeDecoderParams::default();
    low_temp.cryogenic_temperature_mk = 2.0;
    let mut high_temp = SurfaceCodeDecoderParams::default();
    high_temp.cryogenic_temperature_mk = 48.0;

    let low_solver = SurfaceCodeDecoderSolver::new(low_temp);
    let high_solver = SurfaceCodeDecoderSolver::new(high_temp);

    // Lower temperature should improve performance (suppress thermal noise)
    assert!(
        low_solver.compute_decoding_fidelity()
            > high_solver.compute_decoding_fidelity(),
        "Lower cryogenic temperature must enhance decoding fidelity"
    );
    assert!(
        low_solver.compute_code_space_retention_fraction()
            > high_solver.compute_code_space_retention_fraction(),
        "Lower cryogenic temperature must enhance code space retention"
    );
    assert!(
        low_solver.compute_topological_protection_gap_mhz()
            > high_solver.compute_topological_protection_gap_mhz(),
        "Lower cryogenic temperature must expand topological protection gap"
    );
    assert!(
        low_solver.compute_inter_qubit_crosstalk_isolation_db()
            > high_solver.compute_inter_qubit_crosstalk_isolation_db(),
        "Lower cryogenic temperature must improve crosstalk isolation"
    );
    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_microwave_readout_power_scaling() {
    let mut low = SurfaceCodeDecoderParams::default();
    low.microwave_readout_power_uw = 0.8;
    let mut high = SurfaceCodeDecoderParams::default();
    high.microwave_readout_power_uw = 29.0;

    let low_solver = SurfaceCodeDecoderSolver::new(low);
    let high_solver = SurfaceCodeDecoderSolver::new(high);

    assert!(
        high_solver.compute_decoding_fidelity()
            > low_solver.compute_decoding_fidelity(),
        "Higher microwave readout power within physical bounds must enhance decoding fidelity"
    );
    assert!(
        high_solver.compute_code_space_retention_fraction()
            > low_solver.compute_code_space_retention_fraction(),
        "Higher microwave readout power must enhance code space retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave readout power must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_qubit_crosstalk_isolation_db()
            > low_solver.compute_inter_qubit_crosstalk_isolation_db(),
        "Higher microwave readout power must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave readout power must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_code_distance_scaling() {
    let mut low = SurfaceCodeDecoderParams::default();
    low.code_distance_d = 4.0;
    let mut high = SurfaceCodeDecoderParams::default();
    high.code_distance_d = 24.0;

    let low_solver = SurfaceCodeDecoderSolver::new(low);
    let high_solver = SurfaceCodeDecoderSolver::new(high);

    assert!(
        high_solver.compute_decoding_fidelity()
            > low_solver.compute_decoding_fidelity(),
        "Larger code distance must enhance decoding fidelity"
    );
    assert!(
        high_solver.compute_code_space_retention_fraction()
            > low_solver.compute_code_space_retention_fraction(),
        "Larger code distance must enhance code space retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger code distance must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_qubit_crosstalk_isolation_db()
            > low_solver.compute_inter_qubit_crosstalk_isolation_db(),
        "Larger code distance must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger code distance must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_qubit_pitch_scaling() {
    let mut low = SurfaceCodeDecoderParams::default();
    low.qubit_pitch_um = 1.0;
    let mut high = SurfaceCodeDecoderParams::default();
    high.qubit_pitch_um = 14.0;

    let low_solver = SurfaceCodeDecoderSolver::new(low);
    let high_solver = SurfaceCodeDecoderSolver::new(high);

    assert!(
        high_solver.compute_decoding_fidelity()
            > low_solver.compute_decoding_fidelity(),
        "Larger qubit pitch must enhance decoding fidelity"
    );
    assert!(
        high_solver.compute_code_space_retention_fraction()
            > low_solver.compute_code_space_retention_fraction(),
        "Larger qubit pitch must enhance code space retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger qubit pitch must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_qubit_crosstalk_isolation_db()
            > low_solver.compute_inter_qubit_crosstalk_isolation_db(),
        "Larger qubit pitch must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger qubit pitch must suppress topological mode dephasing rate"
    );
}
