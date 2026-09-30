#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Abelian
//! topological defect Majorana-Kramers pair network processors and time-reversal-symmetric
//! phononic braiding engines.

use phonon_models::majorana_kramers_network::MajoranaKramersNetworkParams;
use phonon_solver::majorana_kramers_network::MajoranaKramersNetworkSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = MajoranaKramersNetworkParams::new(
        1.0,     // below 2.0 meV
        0.5,     // below 1.5 meV
        0.5,     // below 1.0 GHz
        100.0,   // below 200.0 m/s
        0.5,     // below 1.0 mK
        0.1,     // below 0.5 uW
        0.2,     // below 0.5 um
        0.05,    // below 0.10
    );
    assert_eq!(underflow.spin_orbit_phononic_coupling_mev, 2.0);
    assert_eq!(underflow.time_reversal_pairing_gap_mev, 1.5);
    assert_eq!(underflow.acoustic_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.shuttling_velocity_m_per_s, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_control_power_uw, 0.5);
    assert_eq!(underflow.defect_separation_distance_um, 0.5);
    assert_eq!(underflow.substrate_piezoelectric_coupling, 0.10);

    // Test values strictly above physical maximum bounds
    let overflow = MajoranaKramersNetworkParams::new(
        50.0,    // above 40.0 meV
        45.0,    // above 30.0 meV
        18.0,    // above 12.0 GHz
        4000.0,  // above 3000.0 m/s
        80.0,    // above 50.0 mK
        50.0,    // above 30.0 uW
        25.0,    // above 15.0 um
        1.05,    // above 0.95
    );
    assert_eq!(overflow.spin_orbit_phononic_coupling_mev, 40.0);
    assert_eq!(overflow.time_reversal_pairing_gap_mev, 30.0);
    assert_eq!(overflow.acoustic_drive_frequency_ghz, 12.0);
    assert_eq!(overflow.shuttling_velocity_m_per_s, 3000.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_control_power_uw, 30.0);
    assert_eq!(overflow.defect_separation_distance_um, 15.0);
    assert_eq!(overflow.substrate_piezoelectric_coupling, 0.95);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = MajoranaKramersNetworkParams::default();
    assert_eq!(params.spin_orbit_phononic_coupling_mev, 18.5);
    assert_eq!(params.time_reversal_pairing_gap_mev, 14.0);
    assert_eq!(params.acoustic_drive_frequency_ghz, 5.5);
    assert_eq!(params.shuttling_velocity_m_per_s, 1350.0);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_control_power_uw, 5.5);
    assert_eq!(params.defect_separation_distance_um, 3.5);
    assert_eq!(params.substrate_piezoelectric_coupling, 0.65);

    let solver = MajoranaKramersNetworkSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.braiding_fidelity >= 0.9980,
        "Braiding fidelity must be >= 0.9980, got {:.6}",
        metrics.braiding_fidelity
    );
    assert!(
        metrics.kramers_pair_retention_fraction >= 0.9970,
        "Kramers pair retention fraction must be >= 0.9970, got {:.6}",
        metrics.kramers_pair_retention_fraction
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 46.0,
        "Topological protection gap must be >= 46.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.inter_defect_crosstalk_isolation_db >= 54.0,
        "Inter-defect crosstalk isolation must be >= 54.0 dB, got {:.4} dB",
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
fn test_spin_orbit_phononic_coupling_scaling() {
    let mut low_so = MajoranaKramersNetworkParams::default();
    low_so.spin_orbit_phononic_coupling_mev = 3.0;
    let mut high_so = MajoranaKramersNetworkParams::default();
    high_so.spin_orbit_phononic_coupling_mev = 38.0;

    let low_solver = MajoranaKramersNetworkSolver::new(low_so);
    let high_solver = MajoranaKramersNetworkSolver::new(high_so);

    assert!(
        high_solver.compute_braiding_fidelity()
            > low_solver.compute_braiding_fidelity(),
        "Higher spin-orbit phononic coupling must enhance braiding fidelity"
    );
    assert!(
        high_solver.compute_kramers_pair_retention_fraction()
            > low_solver.compute_kramers_pair_retention_fraction(),
        "Higher spin-orbit phononic coupling must boost Kramers pair retention fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher spin-orbit phononic coupling must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher spin-orbit phononic coupling must suppress dephasing rate"
    );
}

#[test]
fn test_time_reversal_pairing_gap_scaling() {
    let mut low_tr = MajoranaKramersNetworkParams::default();
    low_tr.time_reversal_pairing_gap_mev = 2.0;
    let mut high_tr = MajoranaKramersNetworkParams::default();
    high_tr.time_reversal_pairing_gap_mev = 28.0;

    let low_solver = MajoranaKramersNetworkSolver::new(low_tr);
    let high_solver = MajoranaKramersNetworkSolver::new(high_tr);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher time-reversal pairing gap must widen topological protection gap"
    );
    assert!(
        high_solver.compute_braiding_fidelity()
            > low_solver.compute_braiding_fidelity(),
        "Higher time-reversal pairing gap must improve braiding fidelity"
    );
    assert!(
        high_solver.compute_kramers_pair_retention_fraction()
            > low_solver.compute_kramers_pair_retention_fraction(),
        "Higher time-reversal pairing gap must enhance retention fraction"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher time-reversal pairing gap must suppress dephasing rate"
    );
}

#[test]
fn test_substrate_piezoelectric_coupling_scaling() {
    let mut low_piezo = MajoranaKramersNetworkParams::default();
    low_piezo.substrate_piezoelectric_coupling = 0.15;
    let mut high_piezo = MajoranaKramersNetworkParams::default();
    high_piezo.substrate_piezoelectric_coupling = 0.90;

    let low_solver = MajoranaKramersNetworkSolver::new(low_piezo);
    let high_solver = MajoranaKramersNetworkSolver::new(high_piezo);

    assert!(
        high_solver.compute_braiding_fidelity()
            > low_solver.compute_braiding_fidelity(),
        "Higher piezoelectric coupling must improve braiding fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher piezoelectric coupling must increase topological protection gap"
    );
    assert!(
        high_solver.compute_inter_defect_crosstalk_isolation_db()
            > low_solver.compute_inter_defect_crosstalk_isolation_db(),
        "Higher piezoelectric coupling must increase crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher piezoelectric coupling must decrease dephasing rate"
    );
}

#[test]
fn test_defect_separation_distance_scaling() {
    let mut low_dist = MajoranaKramersNetworkParams::default();
    low_dist.defect_separation_distance_um = 1.0;
    let mut high_dist = MajoranaKramersNetworkParams::default();
    high_dist.defect_separation_distance_um = 14.0;

    let low_solver = MajoranaKramersNetworkSolver::new(low_dist);
    let high_solver = MajoranaKramersNetworkSolver::new(high_dist);

    assert!(
        high_solver.compute_inter_defect_crosstalk_isolation_db()
            > low_solver.compute_inter_defect_crosstalk_isolation_db(),
        "Greater defect separation must substantially increase crosstalk isolation"
    );
    assert!(
        high_solver.compute_braiding_fidelity()
            > low_solver.compute_braiding_fidelity(),
        "Greater defect separation must reduce inter-defect stray coupling, improving fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Greater defect separation must suppress inter-defect dephasing"
    );
}

#[test]
fn test_shuttling_velocity_scaling() {
    let mut low_v = MajoranaKramersNetworkParams::default();
    low_v.shuttling_velocity_m_per_s = 300.0;
    let mut high_v = MajoranaKramersNetworkParams::default();
    high_v.shuttling_velocity_m_per_s = 2800.0;

    let low_solver = MajoranaKramersNetworkSolver::new(low_v);
    let high_solver = MajoranaKramersNetworkSolver::new(high_v);

    assert!(
        high_solver.compute_braiding_fidelity()
            > low_solver.compute_braiding_fidelity(),
        "Higher shuttling velocity within adiabatic bounds reduces exposure time, improving fidelity"
    );
    assert!(
        high_solver.compute_kramers_pair_retention_fraction()
            > low_solver.compute_kramers_pair_retention_fraction(),
        "Higher shuttling velocity enhances Kramers pair retention fraction"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Faster shuttling suppresses cumulative phase dephasing"
    );
}

#[test]
fn test_acoustic_drive_frequency_scaling() {
    let mut low_f = MajoranaKramersNetworkParams::default();
    low_f.acoustic_drive_frequency_ghz = 1.5;
    let mut high_f = MajoranaKramersNetworkParams::default();
    high_f.acoustic_drive_frequency_ghz = 11.0;

    let low_solver = MajoranaKramersNetworkSolver::new(low_f);
    let high_solver = MajoranaKramersNetworkSolver::new(high_f);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher acoustic drive frequency increases effective Floquet-topological protection gap"
    );
    assert!(
        high_solver.compute_braiding_fidelity()
            > low_solver.compute_braiding_fidelity(),
        "Higher acoustic drive frequency enhances braiding gate fidelity"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold = MajoranaKramersNetworkParams::default();
    cold.cryogenic_temperature_mk = 2.0;
    let mut warm = MajoranaKramersNetworkParams::default();
    warm.cryogenic_temperature_mk = 45.0;

    let cold_solver = MajoranaKramersNetworkSolver::new(cold);
    let warm_solver = MajoranaKramersNetworkSolver::new(warm);

    assert!(
        cold_solver.compute_braiding_fidelity()
            > warm_solver.compute_braiding_fidelity(),
        "Colder cryogenic temperature must increase braiding fidelity"
    );
    assert!(
        cold_solver.compute_kramers_pair_retention_fraction()
            > warm_solver.compute_kramers_pair_retention_fraction(),
        "Colder temperature must improve Kramers pair retention fraction"
    );
    assert!(
        cold_solver.compute_topological_protection_gap_mhz()
            > warm_solver.compute_topological_protection_gap_mhz(),
        "Colder temperature must preserve a larger topological protection gap"
    );
    assert!(
        cold_solver.compute_topological_mode_dephasing_rate_hz()
            < warm_solver.compute_topological_mode_dephasing_rate_hz(),
        "Colder temperature must reduce topological mode dephasing rate"
    );
}

#[test]
fn test_microwave_control_power_scaling() {
    let mut low_p = MajoranaKramersNetworkParams::default();
    low_p.microwave_control_power_uw = 1.0;
    let mut high_p = MajoranaKramersNetworkParams::default();
    high_p.microwave_control_power_uw = 28.0;

    let low_solver = MajoranaKramersNetworkSolver::new(low_p);
    let high_solver = MajoranaKramersNetworkSolver::new(high_p);

    assert!(
        high_solver.compute_braiding_fidelity()
            > low_solver.compute_braiding_fidelity(),
        "Higher microwave control power accelerates gate execution, improving fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher control power strengthens topological drive gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave power reduces cumulative dephasing per gate operation"
    );
}
