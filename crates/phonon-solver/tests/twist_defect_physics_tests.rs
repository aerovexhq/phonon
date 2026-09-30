#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! chiral topological twist-defect Majorana braiding lattices and gauge-invariant state teleporters.

use phonon_models::twist_defect_lattice::TwistDefectLatticeParams;
use phonon_solver::twist_defect_lattice::TwistDefectLatticeSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = TwistDefectLatticeParams::new(
        0.1,   // below 0.2 nm
        0.02,  // below 0.05 rad
        1.0,   // below 2.0 meV
        0.5,   // below 1.0 GHz
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 uW
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.dislocation_burgers_vector_nm, 0.2);
    assert_eq!(underflow.screw_twist_angle_rad, 0.05);
    assert_eq!(underflow.topological_pairing_gap_mev, 2.0);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.strain_shuttling_speed_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_control_power_uw, 0.5);
    assert_eq!(underflow.defect_separation_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = TwistDefectLatticeParams::new(
        8.0,    // above 5.0 nm
        1.20,   // above 0.80 rad
        60.0,   // above 45.0 meV
        15.0,   // above 12.0 GHz
        3500.0, // above 3000.0 m/s
        75.0,   // above 50.0 mK
        45.0,   // above 30.0 uW
        25.0,   // above 15.0 um
    );
    assert_eq!(overflow.dislocation_burgers_vector_nm, 5.0);
    assert_eq!(overflow.screw_twist_angle_rad, 0.80);
    assert_eq!(overflow.topological_pairing_gap_mev, 45.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.strain_shuttling_speed_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_control_power_uw, 30.0);
    assert_eq!(overflow.defect_separation_um, 15.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TwistDefectLatticeParams::default();
    assert_eq!(params.dislocation_burgers_vector_nm, 1.8);
    assert_eq!(params.screw_twist_angle_rad, 0.35);
    assert_eq!(params.topological_pairing_gap_mev, 22.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.8);
    assert_eq!(params.strain_shuttling_speed_m_per_s, 1400.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_control_power_uw, 6.2);
    assert_eq!(params.defect_separation_um, 4.5);

    let solver = TwistDefectLatticeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.teleportation_fidelity >= 0.9980,
        "Teleportation fidelity must be >= 0.9980, got {:.6}",
        metrics.teleportation_fidelity
    );
    assert!(
        metrics.twist_defect_retention_fraction >= 0.9970,
        "Twist-defect retention fraction must be >= 0.9970, got {:.6}",
        metrics.twist_defect_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 45.0,
        "Topological protection gap must be >= 45.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_defect_crosstalk_isolation_db >= 55.0,
        "Inter-defect crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.inter_defect_crosstalk_isolation_db
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
fn test_dislocation_burgers_vector_scaling() {
    let mut low_burg = TwistDefectLatticeParams::default();
    low_burg.dislocation_burgers_vector_nm = 0.5;
    let mut high_burg = TwistDefectLatticeParams::default();
    high_burg.dislocation_burgers_vector_nm = 4.5;

    let low_solver = TwistDefectLatticeSolver::new(low_burg);
    let high_solver = TwistDefectLatticeSolver::new(high_burg);

    assert!(
        high_solver.compute_teleportation_fidelity() > low_solver.compute_teleportation_fidelity(),
        "Larger Burgers vector must enhance teleportation fidelity"
    );
    assert!(
        high_solver.compute_twist_defect_retention_fraction()
            > low_solver.compute_twist_defect_retention_fraction(),
        "Larger Burgers vector must boost twist-defect retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger Burgers vector must widen topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger Burgers vector must suppress dephasing rate"
    );
}

#[test]
fn test_screw_twist_angle_scaling() {
    let mut low_twist = TwistDefectLatticeParams::default();
    low_twist.screw_twist_angle_rad = 0.10;
    let mut high_twist = TwistDefectLatticeParams::default();
    high_twist.screw_twist_angle_rad = 0.70;

    let low_solver = TwistDefectLatticeSolver::new(low_twist);
    let high_solver = TwistDefectLatticeSolver::new(high_twist);

    assert!(
        high_solver.compute_teleportation_fidelity() > low_solver.compute_teleportation_fidelity(),
        "Larger screw twist angle must enhance teleportation fidelity"
    );
    assert!(
        high_solver.compute_twist_defect_retention_fraction()
            > low_solver.compute_twist_defect_retention_fraction(),
        "Larger screw twist angle must boost twist-defect retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger screw twist angle must expand topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger screw twist angle must suppress dephasing rate"
    );
}

#[test]
fn test_topological_pairing_gap_scaling() {
    let mut low_gap = TwistDefectLatticeParams::default();
    low_gap.topological_pairing_gap_mev = 3.0;
    let mut high_gap = TwistDefectLatticeParams::default();
    high_gap.topological_pairing_gap_mev = 40.0;

    let low_solver = TwistDefectLatticeSolver::new(low_gap);
    let high_solver = TwistDefectLatticeSolver::new(high_gap);

    assert!(
        high_solver.compute_teleportation_fidelity() > low_solver.compute_teleportation_fidelity(),
        "Larger topological pairing gap must enhance teleportation fidelity"
    );
    assert!(
        high_solver.compute_twist_defect_retention_fraction()
            > low_solver.compute_twist_defect_retention_fraction(),
        "Larger topological pairing gap must boost twist-defect retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Larger topological pairing gap must widen topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger topological pairing gap must suppress dephasing rate"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_f = TwistDefectLatticeParams::default();
    low_f.acoustic_drive_frequency_ghz = 1.5;
    let mut high_f = TwistDefectLatticeParams::default();
    high_f.acoustic_drive_frequency_ghz = 11.0;

    let low_solver = TwistDefectLatticeSolver::new(low_f);
    let high_solver = TwistDefectLatticeSolver::new(high_f);

    assert!(
        high_solver.compute_teleportation_fidelity() > low_solver.compute_teleportation_fidelity(),
        "Higher acoustic drive frequency must increase teleportation fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic drive frequency must enhance topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic drive frequency must reduce dephasing"
    );
}

#[test]
fn test_strain_shuttling_speed_scaling() {
    let mut low_v = TwistDefectLatticeParams::default();
    low_v.strain_shuttling_speed_m_per_s = 300.0;
    let mut high_v = TwistDefectLatticeParams::default();
    high_v.strain_shuttling_speed_m_per_s = 2800.0;

    let low_solver = TwistDefectLatticeSolver::new(low_v);
    let high_solver = TwistDefectLatticeSolver::new(high_v);

    assert!(
        high_solver.compute_teleportation_fidelity() > low_solver.compute_teleportation_fidelity(),
        "Higher strain shuttling speed must increase teleportation fidelity"
    );
    assert!(
        high_solver.compute_twist_defect_retention_fraction()
            > low_solver.compute_twist_defect_retention_fraction(),
        "Higher strain shuttling speed must improve retention fraction"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher strain shuttling speed must suppress dephasing by shortening transit exposure"
    );
}

#[test]
fn test_defect_separation_scaling() {
    let mut low_sep = TwistDefectLatticeParams::default();
    low_sep.defect_separation_um = 1.0;
    let mut high_sep = TwistDefectLatticeParams::default();
    high_sep.defect_separation_um = 14.0;

    let low_solver = TwistDefectLatticeSolver::new(low_sep);
    let high_solver = TwistDefectLatticeSolver::new(high_sep);

    assert!(
        high_solver.compute_inter_defect_crosstalk_isolation_db()
            > low_solver.compute_inter_defect_crosstalk_isolation_db(),
        "Larger defect separation must boost crosstalk isolation"
    );
    assert!(
        high_solver.compute_twist_defect_retention_fraction()
            > low_solver.compute_twist_defect_retention_fraction(),
        "Larger defect separation must improve retention fraction"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger defect separation must reduce dephasing rate"
    );
}

#[test]
fn test_microwave_control_power_scaling() {
    let mut low_p = TwistDefectLatticeParams::default();
    low_p.microwave_control_power_uw = 1.0;
    let mut high_p = TwistDefectLatticeParams::default();
    high_p.microwave_control_power_uw = 28.0;

    let low_solver = TwistDefectLatticeSolver::new(low_p);
    let high_solver = TwistDefectLatticeSolver::new(high_p);

    assert!(
        high_solver.compute_teleportation_fidelity() > low_solver.compute_teleportation_fidelity(),
        "Higher microwave control power must increase teleportation fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave control power must improve topological gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave control power must reduce dephasing"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_t = TwistDefectLatticeParams::default();
    low_t.cryogenic_temperature_mk = 2.0;
    let mut high_t = TwistDefectLatticeParams::default();
    high_t.cryogenic_temperature_mk = 48.0;

    let low_solver = TwistDefectLatticeSolver::new(low_t);
    let high_solver = TwistDefectLatticeSolver::new(high_t);

    assert!(
        low_solver.compute_teleportation_fidelity() > high_solver.compute_teleportation_fidelity(),
        "Lower temperature must enhance teleportation fidelity due to reduced thermal noise"
    );
    assert!(
        low_solver.compute_twist_defect_retention_fraction()
            > high_solver.compute_twist_defect_retention_fraction(),
        "Lower temperature must protect twist-defect retention fraction"
    );
    assert!(
        low_solver.compute_topological_protection_gap_mhz()
            > high_solver.compute_topological_protection_gap_mhz(),
        "Lower temperature must preserve a wider topological protection gap"
    );
    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower temperature must suppress thermal dephasing rate"
    );
}
