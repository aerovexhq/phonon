#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological Floquet-Majorana engines and non-equilibrium time-translational
//! simulators.

use phonon_models::floquet_majorana_engine::FloquetMajoranaEngineParams;
use phonon_solver::floquet_majorana_engine::FloquetMajoranaEngineSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FloquetMajoranaEngineParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.05,  // below 0.1 ns
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.floquet_drive_amplitude_mev, 1.0);
    assert_eq!(underflow.topological_quasiparticle_gap_mev, 2.0);
    assert_eq!(underflow.floquet_modulation_frequency_ghz, 1.0);
    assert_eq!(underflow.stroboscopic_shuttling_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_pumping_power_uw, 0.5);
    assert_eq!(underflow.floquet_drive_period_ns, 0.1);
    assert_eq!(underflow.majorana_wire_length_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = FloquetMajoranaEngineParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        15.0,   // above 10.0 ns
        30.0,   // above 20.0 um
    );
    assert_eq!(overflow.floquet_drive_amplitude_mev, 35.0);
    assert_eq!(overflow.topological_quasiparticle_gap_mev, 45.0);
    assert_eq!(overflow.floquet_modulation_frequency_ghz, 12.0);
    assert_eq!(overflow.stroboscopic_shuttling_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_pumping_power_uw, 30.0);
    assert_eq!(overflow.floquet_drive_period_ns, 10.0);
    assert_eq!(overflow.majorana_wire_length_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FloquetMajoranaEngineParams::default();
    assert_eq!(params.floquet_drive_amplitude_mev, 16.5);
    assert_eq!(params.topological_quasiparticle_gap_mev, 22.0);
    assert_eq!(params.floquet_modulation_frequency_ghz, 5.6);
    assert_eq!(params.stroboscopic_shuttling_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_pumping_power_uw, 5.8);
    assert_eq!(params.floquet_drive_period_ns, 2.5);
    assert_eq!(params.majorana_wire_length_um, 4.8);

    let solver = FloquetMajoranaEngineSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.floquet_engine_fidelity >= 0.9980,
        "Floquet engine fidelity must be >= 0.9980, got {:.6}",
        metrics.floquet_engine_fidelity
    );
    assert!(
        metrics.floquet_majorana_state_retention_fraction >= 0.9970,
        "Floquet-Majorana state retention fraction must be >= 0.9970, got {:.6}",
        metrics.floquet_majorana_state_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_mode_crosstalk_isolation_db >= 54.0,
        "Inter-mode crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
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
fn test_floquet_drive_amplitude_scaling() {
    let mut low = FloquetMajoranaEngineParams::default();
    low.floquet_drive_amplitude_mev = 2.0;
    let mut high = FloquetMajoranaEngineParams::default();
    high.floquet_drive_amplitude_mev = 34.0;

    let low_solver = FloquetMajoranaEngineSolver::new(low);
    let high_solver = FloquetMajoranaEngineSolver::new(high);

    assert!(
        high_solver.compute_floquet_engine_fidelity()
            > low_solver.compute_floquet_engine_fidelity(),
        "Larger Floquet drive amplitude must enhance engine fidelity"
    );
    assert!(
        high_solver.compute_floquet_majorana_state_retention_fraction()
            > low_solver.compute_floquet_majorana_state_retention_fraction(),
        "Larger Floquet drive amplitude must enhance state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger Floquet drive amplitude must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_mode_crosstalk_isolation_db()
            > low_solver.compute_inter_mode_crosstalk_isolation_db(),
        "Larger Floquet drive amplitude must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger Floquet drive amplitude must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_topological_quasiparticle_gap_scaling() {
    let mut low = FloquetMajoranaEngineParams::default();
    low.topological_quasiparticle_gap_mev = 3.0;
    let mut high = FloquetMajoranaEngineParams::default();
    high.topological_quasiparticle_gap_mev = 44.0;

    let low_solver = FloquetMajoranaEngineSolver::new(low);
    let high_solver = FloquetMajoranaEngineSolver::new(high);

    assert!(
        high_solver.compute_floquet_engine_fidelity()
            > low_solver.compute_floquet_engine_fidelity(),
        "Larger topological quasiparticle gap must enhance engine fidelity"
    );
    assert!(
        high_solver.compute_floquet_majorana_state_retention_fraction()
            > low_solver.compute_floquet_majorana_state_retention_fraction(),
        "Larger topological quasiparticle gap must enhance state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger topological quasiparticle gap must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_mode_crosstalk_isolation_db()
            > low_solver.compute_inter_mode_crosstalk_isolation_db(),
        "Larger topological quasiparticle gap must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger topological quasiparticle gap must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_floquet_modulation_frequency_scaling() {
    let mut low = FloquetMajoranaEngineParams::default();
    low.floquet_modulation_frequency_ghz = 1.5;
    let mut high = FloquetMajoranaEngineParams::default();
    high.floquet_modulation_frequency_ghz = 11.5;

    let low_solver = FloquetMajoranaEngineSolver::new(low);
    let high_solver = FloquetMajoranaEngineSolver::new(high);

    assert!(
        high_solver.compute_floquet_engine_fidelity()
            > low_solver.compute_floquet_engine_fidelity(),
        "Higher Floquet modulation frequency must enhance engine fidelity"
    );
    assert!(
        high_solver.compute_floquet_majorana_state_retention_fraction()
            > low_solver.compute_floquet_majorana_state_retention_fraction(),
        "Higher Floquet modulation frequency must enhance state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher Floquet modulation frequency must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_mode_crosstalk_isolation_db()
            > low_solver.compute_inter_mode_crosstalk_isolation_db(),
        "Higher Floquet modulation frequency must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher Floquet modulation frequency must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_stroboscopic_shuttling_speed_scaling() {
    let mut low = FloquetMajoranaEngineParams::default();
    low.stroboscopic_shuttling_speed_m_per_s = 250.0;
    let mut high = FloquetMajoranaEngineParams::default();
    high.stroboscopic_shuttling_speed_m_per_s = 2900.0;

    let low_solver = FloquetMajoranaEngineSolver::new(low);
    let high_solver = FloquetMajoranaEngineSolver::new(high);

    assert!(
        high_solver.compute_floquet_engine_fidelity()
            > low_solver.compute_floquet_engine_fidelity(),
        "Faster stroboscopic shuttling speed must enhance engine fidelity"
    );
    assert!(
        high_solver.compute_floquet_majorana_state_retention_fraction()
            > low_solver.compute_floquet_majorana_state_retention_fraction(),
        "Faster stroboscopic shuttling speed must enhance state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Faster stroboscopic shuttling speed must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_mode_crosstalk_isolation_db()
            > low_solver.compute_inter_mode_crosstalk_isolation_db(),
        "Faster stroboscopic shuttling speed must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Faster stroboscopic shuttling speed must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_temp = FloquetMajoranaEngineParams::default();
    low_temp.cryogenic_temperature_mk = 2.0;
    let mut high_temp = FloquetMajoranaEngineParams::default();
    high_temp.cryogenic_temperature_mk = 48.0;

    let low_solver = FloquetMajoranaEngineSolver::new(low_temp);
    let high_solver = FloquetMajoranaEngineSolver::new(high_temp);

    // Lower cryogenic temperature suppresses thermal noise and quasiparticles
    assert!(
        low_solver.compute_floquet_engine_fidelity()
            > high_solver.compute_floquet_engine_fidelity(),
        "Lower cryogenic temperature must enhance engine fidelity"
    );
    assert!(
        low_solver.compute_floquet_majorana_state_retention_fraction()
            > high_solver.compute_floquet_majorana_state_retention_fraction(),
        "Lower cryogenic temperature must enhance state retention"
    );
    assert!(
        low_solver.compute_topological_protection_gap_mhz()
            > high_solver.compute_topological_protection_gap_mhz(),
        "Lower cryogenic temperature must expand topological protection gap"
    );
    assert!(
        low_solver.compute_inter_mode_crosstalk_isolation_db()
            > high_solver.compute_inter_mode_crosstalk_isolation_db(),
        "Lower cryogenic temperature must improve crosstalk isolation"
    );
    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_microwave_pumping_power_scaling() {
    let mut low = FloquetMajoranaEngineParams::default();
    low.microwave_pumping_power_uw = 0.8;
    let mut high = FloquetMajoranaEngineParams::default();
    high.microwave_pumping_power_uw = 29.0;

    let low_solver = FloquetMajoranaEngineSolver::new(low);
    let high_solver = FloquetMajoranaEngineSolver::new(high);

    assert!(
        high_solver.compute_floquet_engine_fidelity()
            > low_solver.compute_floquet_engine_fidelity(),
        "Higher microwave pumping power within physical bounds must enhance engine fidelity"
    );
    assert!(
        high_solver.compute_floquet_majorana_state_retention_fraction()
            > low_solver.compute_floquet_majorana_state_retention_fraction(),
        "Higher microwave pumping power must enhance state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave pumping power must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_mode_crosstalk_isolation_db()
            > low_solver.compute_inter_mode_crosstalk_isolation_db(),
        "Higher microwave pumping power must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave pumping power must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_floquet_drive_period_scaling() {
    let mut low = FloquetMajoranaEngineParams::default();
    low.floquet_drive_period_ns = 0.3;
    let mut high = FloquetMajoranaEngineParams::default();
    high.floquet_drive_period_ns = 9.5;

    let low_solver = FloquetMajoranaEngineSolver::new(low);
    let high_solver = FloquetMajoranaEngineSolver::new(high);

    assert!(
        high_solver.compute_floquet_engine_fidelity()
            > low_solver.compute_floquet_engine_fidelity(),
        "Longer Floquet drive period allows adiabatic state synthesis enhancing fidelity"
    );
    assert!(
        high_solver.compute_floquet_majorana_state_retention_fraction()
            > low_solver.compute_floquet_majorana_state_retention_fraction(),
        "Longer Floquet drive period enhances state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Longer Floquet drive period expands topological protection gap"
    );
    assert!(
        high_solver.compute_inter_mode_crosstalk_isolation_db()
            > low_solver.compute_inter_mode_crosstalk_isolation_db(),
        "Longer Floquet drive period improves crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Longer Floquet drive period suppresses topological mode dephasing rate"
    );
}

#[test]
fn test_majorana_wire_length_scaling() {
    let mut low = FloquetMajoranaEngineParams::default();
    low.majorana_wire_length_um = 1.0;
    let mut high = FloquetMajoranaEngineParams::default();
    high.majorana_wire_length_um = 19.0;

    let low_solver = FloquetMajoranaEngineSolver::new(low);
    let high_solver = FloquetMajoranaEngineSolver::new(high);

    assert!(
        high_solver.compute_floquet_engine_fidelity()
            > low_solver.compute_floquet_engine_fidelity(),
        "Larger Majorana wire length must enhance engine fidelity"
    );
    assert!(
        high_solver.compute_floquet_majorana_state_retention_fraction()
            > low_solver.compute_floquet_majorana_state_retention_fraction(),
        "Larger Majorana wire length must enhance state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger Majorana wire length must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_mode_crosstalk_isolation_db()
            > low_solver.compute_inter_mode_crosstalk_isolation_db(),
        "Larger Majorana wire length provides exponential wavefunction decay improving crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger Majorana wire length suppresses topological mode dephasing rate"
    );
}
