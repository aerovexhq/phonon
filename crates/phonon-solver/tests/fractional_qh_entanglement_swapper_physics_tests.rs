#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic topological
//! chiral fractional quantum Hall phonon entanglement swappers and non-Abelian anyon
//! teleportation bridges.

use phonon_models::fractional_qh_entanglement_swapper::FractionalQHEntanglementSwapperParams;
use phonon_solver::fractional_qh_entanglement_swapper::FractionalQHEntanglementSwapperSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FractionalQHEntanglementSwapperParams::new(
        0.05,    // below 0.2
        1.0,     // below 2.0 meV
        200.0,   // below 500.0 m/s
        0.1,     // below 0.5 um
        0.5,     // below 1.0 mK
        0.1,     // below 0.5 uW
        5.0,     // below 8.0
        0.05,    // below 0.2 um
    );
    assert_eq!(underflow.filling_factor_nu, 0.2);
    assert_eq!(underflow.topological_tunneling_amplitude_mev, 2.0);
    assert_eq!(underflow.acoustic_edge_velocity_m_per_s, 500.0);
    assert_eq!(underflow.anyon_shuttling_distance_um, 0.5);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.microwave_drive_power_uw, 0.5);
    assert_eq!(underflow.heterostructure_dielectric_constant, 8.0);
    assert_eq!(underflow.channel_separation_um, 0.2);

    // Test values strictly above physical maximum bounds
    let overflow = FractionalQHEntanglementSwapperParams::new(
        4.0,     // above 2.5
        60.0,    // above 40.0 meV
        6000.0,  // above 4500.0 m/s
        40.0,    // above 25.0 um
        80.0,    // above 50.0 mK
        50.0,    // above 30.0 uW
        35.0,    // above 25.0
        15.0,    // above 10.0 um
    );
    assert_eq!(overflow.filling_factor_nu, 2.5);
    assert_eq!(overflow.topological_tunneling_amplitude_mev, 40.0);
    assert_eq!(overflow.acoustic_edge_velocity_m_per_s, 4500.0);
    assert_eq!(overflow.anyon_shuttling_distance_um, 25.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.microwave_drive_power_uw, 30.0);
    assert_eq!(overflow.heterostructure_dielectric_constant, 25.0);
    assert_eq!(overflow.channel_separation_um, 10.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FractionalQHEntanglementSwapperParams::default();
    assert_eq!(params.filling_factor_nu, 0.3333);
    assert_eq!(params.topological_tunneling_amplitude_mev, 18.0);
    assert_eq!(params.acoustic_edge_velocity_m_per_s, 2100.0);
    assert_eq!(params.anyon_shuttling_distance_um, 4.2);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.microwave_drive_power_uw, 6.5);
    assert_eq!(params.heterostructure_dielectric_constant, 13.1);
    assert_eq!(params.channel_separation_um, 1.8);

    let solver = FractionalQHEntanglementSwapperSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.bell_state_measurement_fidelity >= 0.9980,
        "Bell state measurement fidelity must be >= 0.9980, got {:.6}",
        metrics.bell_state_measurement_fidelity
    );
    assert!(
        metrics.entanglement_teleportation_fidelity >= 0.9980,
        "Entanglement teleportation fidelity must be >= 0.9980, got {:.6}",
        metrics.entanglement_teleportation_fidelity
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
fn test_filling_factor_nu_scaling() {
    let mut low_nu = FractionalQHEntanglementSwapperParams::default();
    low_nu.filling_factor_nu = 0.25;
    let mut high_nu = FractionalQHEntanglementSwapperParams::default();
    high_nu.filling_factor_nu = 2.40;

    let low_solver = FractionalQHEntanglementSwapperSolver::new(low_nu);
    let high_solver = FractionalQHEntanglementSwapperSolver::new(high_nu);

    assert!(
        high_solver.compute_bell_state_measurement_fidelity()
            > low_solver.compute_bell_state_measurement_fidelity(),
        "Higher filling factor nu must enhance Bell state measurement fidelity"
    );
    assert!(
        high_solver.compute_entanglement_teleportation_fidelity()
            > low_solver.compute_entanglement_teleportation_fidelity(),
        "Higher filling factor nu must boost entanglement teleportation fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher filling factor nu must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher filling factor nu must suppress the topological dephasing rate"
    );
}

#[test]
fn test_topological_tunneling_amplitude_scaling() {
    let mut low_t = FractionalQHEntanglementSwapperParams::default();
    low_t.topological_tunneling_amplitude_mev = 3.0;
    let mut high_t = FractionalQHEntanglementSwapperParams::default();
    high_t.topological_tunneling_amplitude_mev = 38.0;

    let low_solver = FractionalQHEntanglementSwapperSolver::new(low_t);
    let high_solver = FractionalQHEntanglementSwapperSolver::new(high_t);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Elevated topological tunneling amplitude must widen the protection gap"
    );
    assert!(
        high_solver.compute_bell_state_measurement_fidelity()
            > low_solver.compute_bell_state_measurement_fidelity(),
        "Elevated tunneling amplitude must improve Bell state measurement fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Elevated tunneling amplitude must suppress dephasing rate"
    );
}

#[test]
fn test_acoustic_edge_velocity_scaling() {
    let mut low_v = FractionalQHEntanglementSwapperParams::default();
    low_v.acoustic_edge_velocity_m_per_s = 600.0;
    let mut high_v = FractionalQHEntanglementSwapperParams::default();
    high_v.acoustic_edge_velocity_m_per_s = 4200.0;

    let low_solver = FractionalQHEntanglementSwapperSolver::new(low_v);
    let high_solver = FractionalQHEntanglementSwapperSolver::new(high_v);

    assert!(
        high_solver.compute_entanglement_teleportation_fidelity()
            > low_solver.compute_entanglement_teleportation_fidelity(),
        "Higher acoustic edge velocity must improve entanglement teleportation fidelity"
    );
    assert!(
        high_solver.compute_bell_state_measurement_fidelity()
            > low_solver.compute_bell_state_measurement_fidelity(),
        "Higher acoustic velocity must enhance Bell state measurement fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic velocity must suppress transit dephasing"
    );
}

#[test]
fn test_anyon_shuttling_distance_scaling() {
    let mut short_dist = FractionalQHEntanglementSwapperParams::default();
    short_dist.anyon_shuttling_distance_um = 1.0; // shorter distance
    let mut long_dist = FractionalQHEntanglementSwapperParams::default();
    long_dist.anyon_shuttling_distance_um = 22.0;

    let short_solver = FractionalQHEntanglementSwapperSolver::new(short_dist);
    let long_solver = FractionalQHEntanglementSwapperSolver::new(long_dist);

    assert!(
        short_solver.compute_entanglement_teleportation_fidelity()
            > long_solver.compute_entanglement_teleportation_fidelity(),
        "Shorter shuttling distance must enhance teleportation fidelity"
    );
    assert!(
        short_solver.compute_bell_state_measurement_fidelity()
            > long_solver.compute_bell_state_measurement_fidelity(),
        "Shorter shuttling distance must boost Bell state measurement fidelity"
    );
    assert!(
        short_solver.compute_topological_mode_dephasing_rate_hz()
            < long_solver.compute_topological_mode_dephasing_rate_hz(),
        "Shorter shuttling distance must suppress accumulated dephasing"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_temp = FractionalQHEntanglementSwapperParams::default();
    low_temp.cryogenic_temperature_mk = 2.0;
    let mut high_temp = FractionalQHEntanglementSwapperParams::default();
    high_temp.cryogenic_temperature_mk = 45.0;

    let low_solver = FractionalQHEntanglementSwapperSolver::new(low_temp);
    let high_solver = FractionalQHEntanglementSwapperSolver::new(high_temp);

    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must suppress thermal dephasing"
    );
    assert!(
        low_solver.compute_bell_state_measurement_fidelity()
            > high_solver.compute_bell_state_measurement_fidelity(),
        "Lower cryogenic temperature must improve Bell state measurement fidelity"
    );
    assert!(
        low_solver.compute_entanglement_teleportation_fidelity()
            > high_solver.compute_entanglement_teleportation_fidelity(),
        "Lower cryogenic temperature must improve teleportation fidelity"
    );
}

#[test]
fn test_microwave_drive_power_scaling() {
    let mut low_power = FractionalQHEntanglementSwapperParams::default();
    low_power.microwave_drive_power_uw = 1.0;
    let mut high_power = FractionalQHEntanglementSwapperParams::default();
    high_power.microwave_drive_power_uw = 28.0;

    let low_solver = FractionalQHEntanglementSwapperSolver::new(low_power);
    let high_solver = FractionalQHEntanglementSwapperSolver::new(high_power);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher microwave drive power must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_bell_state_measurement_fidelity()
            > low_solver.compute_bell_state_measurement_fidelity(),
        "Higher microwave power must improve Bell state measurement fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher microwave drive power must stabilize coherence and suppress dephasing"
    );
}

#[test]
fn test_heterostructure_dielectric_constant_scaling() {
    let mut low_eps = FractionalQHEntanglementSwapperParams::default();
    low_eps.heterostructure_dielectric_constant = 9.0;
    let mut high_eps = FractionalQHEntanglementSwapperParams::default();
    high_eps.heterostructure_dielectric_constant = 24.0;

    let low_solver = FractionalQHEntanglementSwapperSolver::new(low_eps);
    let high_solver = FractionalQHEntanglementSwapperSolver::new(high_eps);

    assert!(
        high_solver.compute_inter_channel_crosstalk_isolation_db()
            > low_solver.compute_inter_channel_crosstalk_isolation_db(),
        "Higher dielectric constant must increase inter-channel crosstalk isolation"
    );
    assert!(
        high_solver.compute_bell_state_measurement_fidelity()
            > low_solver.compute_bell_state_measurement_fidelity(),
        "Higher dielectric constant must enhance Bell state measurement fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher dielectric constant must reduce dephasing rate through screening"
    );
}

#[test]
fn test_channel_separation_scaling() {
    let mut narrow_sep = FractionalQHEntanglementSwapperParams::default();
    narrow_sep.channel_separation_um = 0.5;
    let mut wide_sep = FractionalQHEntanglementSwapperParams::default();
    wide_sep.channel_separation_um = 9.0;

    let narrow_solver = FractionalQHEntanglementSwapperSolver::new(narrow_sep);
    let wide_solver = FractionalQHEntanglementSwapperSolver::new(wide_sep);

    assert!(
        wide_solver.compute_inter_channel_crosstalk_isolation_db()
            > narrow_solver.compute_inter_channel_crosstalk_isolation_db(),
        "Wider channel separation must substantially boost inter-channel crosstalk isolation"
    );
    assert!(
        wide_solver.compute_bell_state_measurement_fidelity()
            > narrow_solver.compute_bell_state_measurement_fidelity(),
        "Wider channel separation must improve Bell state measurement fidelity"
    );
    assert!(
        wide_solver.compute_topological_mode_dephasing_rate_hz()
            < narrow_solver.compute_topological_mode_dephasing_rate_hz(),
        "Wider channel separation must suppress inter-channel dephasing"
    );
}
