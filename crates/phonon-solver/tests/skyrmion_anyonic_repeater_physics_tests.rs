#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological skyrmion-lattice anyonic quantum repeaters and entanglement
//! distillation nodes.

use phonon_models::skyrmion_anyonic_repeater::SkyrmionAnyonicRepeaterParams;
use phonon_solver::skyrmion_anyonic_repeater::SkyrmionAnyonicRepeaterSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = SkyrmionAnyonicRepeaterParams::new(
        0.5,   // below 1.0 meV
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        15.0,  // below 30.0 nm
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.dzyaloshinskii_moriya_energy_mev, 1.0);
    assert_eq!(underflow.superconducting_pairing_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_carrier_frequency_ghz, 1.0);
    assert_eq!(underflow.distillation_shuttling_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_pump_power_uw, 0.5);
    assert_eq!(underflow.skyrmion_lattice_constant_nm, 30.0);
    assert_eq!(underflow.node_separation_distance_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = SkyrmionAnyonicRepeaterParams::new(
        50.0,   // above 35.0 meV
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        300.0,  // above 250.0 nm
        25.0,   // above 20.0 um
    );
    assert_eq!(overflow.dzyaloshinskii_moriya_energy_mev, 35.0);
    assert_eq!(overflow.superconducting_pairing_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_carrier_frequency_ghz, 12.0);
    assert_eq!(overflow.distillation_shuttling_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_pump_power_uw, 30.0);
    assert_eq!(overflow.skyrmion_lattice_constant_nm, 250.0);
    assert_eq!(overflow.node_separation_distance_um, 20.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = SkyrmionAnyonicRepeaterParams::default();
    assert_eq!(params.dzyaloshinskii_moriya_energy_mev, 16.5);
    assert_eq!(params.superconducting_pairing_gap_mev, 21.0);
    assert_eq!(params.acoustic_carrier_frequency_ghz, 5.6);
    assert_eq!(params.distillation_shuttling_speed_m_per_s, 1350.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_pump_power_uw, 5.5);
    assert_eq!(params.skyrmion_lattice_constant_nm, 90.0);
    assert_eq!(params.node_separation_distance_um, 5.0);

    let solver = SkyrmionAnyonicRepeaterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.repeater_fidelity >= 0.9980,
        "Repeater fidelity must be >= 0.9980, got {:.6}",
        metrics.repeater_fidelity
    );
    assert!(
        metrics.anyon_state_retention_fraction >= 0.9970,
        "Anyon state retention fraction must be >= 0.9970, got {:.6}",
        metrics.anyon_state_retention_fraction
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
fn test_dzyaloshinskii_moriya_energy_scaling() {
    let mut low_dmi = SkyrmionAnyonicRepeaterParams::default();
    low_dmi.dzyaloshinskii_moriya_energy_mev = 2.0;
    let mut high_dmi = SkyrmionAnyonicRepeaterParams::default();
    high_dmi.dzyaloshinskii_moriya_energy_mev = 34.0;

    let low_solver = SkyrmionAnyonicRepeaterSolver::new(low_dmi);
    let high_solver = SkyrmionAnyonicRepeaterSolver::new(high_dmi);

    assert!(
        high_solver.compute_repeater_fidelity()
            > low_solver.compute_repeater_fidelity(),
        "Larger DMI energy must enhance repeater fidelity"
    );
    assert!(
        high_solver.compute_anyon_state_retention_fraction()
            > low_solver.compute_anyon_state_retention_fraction(),
        "Larger DMI energy must enhance anyon state retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger DMI energy must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_node_crosstalk_isolation_db()
            > low_solver.compute_inter_node_crosstalk_isolation_db(),
        "Larger DMI energy must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger DMI energy must suppress topological mode dephasing rate"
    );
}

#[test]
fn test_superconducting_pairing_gap_scaling() {
    let mut low_sc = SkyrmionAnyonicRepeaterParams::default();
    low_sc.superconducting_pairing_gap_mev = 3.0;
    let mut high_sc = SkyrmionAnyonicRepeaterParams::default();
    high_sc.superconducting_pairing_gap_mev = 42.0;

    let low_solver = SkyrmionAnyonicRepeaterSolver::new(low_sc);
    let high_solver = SkyrmionAnyonicRepeaterSolver::new(high_sc);

    assert!(
        high_solver.compute_repeater_fidelity()
            > low_solver.compute_repeater_fidelity(),
        "Larger superconducting pairing gap must enhance repeater fidelity"
    );
    assert!(
        high_solver.compute_anyon_state_retention_fraction()
            > low_solver.compute_anyon_state_retention_fraction(),
        "Larger superconducting pairing gap must enhance anyon retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger superconducting pairing gap must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_node_crosstalk_isolation_db()
            > low_solver.compute_inter_node_crosstalk_isolation_db(),
        "Larger superconducting pairing gap must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger superconducting pairing gap must suppress dephasing rate"
    );
}

#[test]
fn test_acoustic_carrier_frequency_scaling() {
    let mut low_freq = SkyrmionAnyonicRepeaterParams::default();
    low_freq.acoustic_carrier_frequency_ghz = 1.5;
    let mut high_freq = SkyrmionAnyonicRepeaterParams::default();
    high_freq.acoustic_carrier_frequency_ghz = 11.5;

    let low_solver = SkyrmionAnyonicRepeaterSolver::new(low_freq);
    let high_solver = SkyrmionAnyonicRepeaterSolver::new(high_freq);

    assert!(
        high_solver.compute_repeater_fidelity()
            > low_solver.compute_repeater_fidelity(),
        "Higher acoustic carrier frequency must enhance repeater fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic carrier frequency must expand topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic carrier frequency must suppress dephasing rate"
    );
}

#[test]
fn test_distillation_shuttling_speed_scaling() {
    let mut low_speed = SkyrmionAnyonicRepeaterParams::default();
    low_speed.distillation_shuttling_speed_m_per_s = 300.0;
    let mut high_speed = SkyrmionAnyonicRepeaterParams::default();
    high_speed.distillation_shuttling_speed_m_per_s = 2800.0;

    let low_solver = SkyrmionAnyonicRepeaterSolver::new(low_speed);
    let high_solver = SkyrmionAnyonicRepeaterSolver::new(high_speed);

    assert!(
        high_solver.compute_repeater_fidelity()
            > low_solver.compute_repeater_fidelity(),
        "Higher distillation shuttling speed must enhance repeater fidelity"
    );
    assert!(
        high_solver.compute_anyon_state_retention_fraction()
            > low_solver.compute_anyon_state_retention_fraction(),
        "Higher distillation shuttling speed must enhance anyon retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher distillation shuttling speed must expand topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher distillation shuttling speed must suppress dephasing rate"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_temp = SkyrmionAnyonicRepeaterParams::default();
    low_temp.cryogenic_temperature_mk = 2.0;
    let mut high_temp = SkyrmionAnyonicRepeaterParams::default();
    high_temp.cryogenic_temperature_mk = 48.0;

    let low_solver = SkyrmionAnyonicRepeaterSolver::new(low_temp);
    let high_solver = SkyrmionAnyonicRepeaterSolver::new(high_temp);

    assert!(
        low_solver.compute_repeater_fidelity()
            > high_solver.compute_repeater_fidelity(),
        "Lower cryogenic temperature must enhance repeater fidelity"
    );
    assert!(
        low_solver.compute_anyon_state_retention_fraction()
            > high_solver.compute_anyon_state_retention_fraction(),
        "Lower cryogenic temperature must preserve anyon retention"
    );
    assert!(
        low_solver.compute_topological_protection_gap_mhz()
            > high_solver.compute_topological_protection_gap_mhz(),
        "Lower cryogenic temperature must sustain larger topological protection gap"
    );
    assert!(
        low_solver.compute_inter_node_crosstalk_isolation_db()
            > high_solver.compute_inter_node_crosstalk_isolation_db(),
        "Lower cryogenic temperature must improve crosstalk isolation"
    );
    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must reduce topological mode dephasing rate"
    );
}

#[test]
fn test_microwave_pump_power_scaling() {
    let mut low_power = SkyrmionAnyonicRepeaterParams::default();
    low_power.microwave_pump_power_uw = 1.0;
    let mut high_power = SkyrmionAnyonicRepeaterParams::default();
    high_power.microwave_pump_power_uw = 28.0;

    let low_solver = SkyrmionAnyonicRepeaterSolver::new(low_power);
    let high_solver = SkyrmionAnyonicRepeaterSolver::new(high_power);

    assert!(
        high_solver.compute_repeater_fidelity()
            > low_solver.compute_repeater_fidelity(),
        "Higher microwave pump power must enhance repeater fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave pump power must expand topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave pump power must suppress dephasing rate"
    );
}

#[test]
fn test_skyrmion_lattice_constant_scaling() {
    let mut low_lat = SkyrmionAnyonicRepeaterParams::default();
    low_lat.skyrmion_lattice_constant_nm = 40.0;
    let mut high_lat = SkyrmionAnyonicRepeaterParams::default();
    high_lat.skyrmion_lattice_constant_nm = 220.0;

    let low_solver = SkyrmionAnyonicRepeaterSolver::new(low_lat);
    let high_solver = SkyrmionAnyonicRepeaterSolver::new(high_lat);

    assert!(
        high_solver.compute_repeater_fidelity()
            > low_solver.compute_repeater_fidelity(),
        "Optimal skyrmion lattice constant must enhance repeater fidelity"
    );
    assert!(
        high_solver.compute_anyon_state_retention_fraction()
            > low_solver.compute_anyon_state_retention_fraction(),
        "Optimal skyrmion lattice constant must enhance anyon retention"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Optimal skyrmion lattice constant must expand topological protection gap"
    );
    assert!(
        high_solver.compute_inter_node_crosstalk_isolation_db()
            > low_solver.compute_inter_node_crosstalk_isolation_db(),
        "Optimal skyrmion lattice constant must improve crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Optimal skyrmion lattice constant must suppress dephasing rate"
    );
}

#[test]
fn test_node_separation_distance_scaling() {
    let mut low_sep = SkyrmionAnyonicRepeaterParams::default();
    low_sep.node_separation_distance_um = 1.0;
    let mut high_sep = SkyrmionAnyonicRepeaterParams::default();
    high_sep.node_separation_distance_um = 18.0;

    let low_solver = SkyrmionAnyonicRepeaterSolver::new(low_sep);
    let high_solver = SkyrmionAnyonicRepeaterSolver::new(high_sep);

    assert!(
        high_solver.compute_inter_node_crosstalk_isolation_db()
            > low_solver.compute_inter_node_crosstalk_isolation_db(),
        "Larger node separation distance must substantially improve crosstalk acoustic isolation"
    );
    assert!(
        high_solver.compute_repeater_fidelity()
            > low_solver.compute_repeater_fidelity(),
        "Larger node separation distance provides spatial isolation enhancing fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger node separation distance suppresses parasitic coupling dephasing rate"
    );
}
