#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Abelian quantum acoustic
//! anyonic braiding in moire skyrmion crystals and chiral topological spin-Peierls transducers.

use phonon_models::moire_skyrmion_anyon_braiding::MoireSkyrmionAnyonBraidingParams;
use phonon_solver::moire_skyrmion_anyon_braiding::MoireSkyrmionAnyonBraidingSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = MoireSkyrmionAnyonBraidingParams::new(
        0.50, // below 0.80 deg
        0.05, // below 0.10
        0.50, // below 1.0 meV
        2.0,  // below 5.0 meV
        0.5,  // below 1.0 GHz
        0.5,  // below 1.0 mK
        15.0, // below 30.0 nm
        0.2,  // below 0.5 um
    );
    assert_eq!(underflow.twist_angle_degrees, 0.80);
    assert_eq!(underflow.spin_peierls_coupling_constant, 0.10);
    assert_eq!(underflow.dzyaloshinskii_moriya_interaction_mev, 1.0);
    assert_eq!(underflow.heisenberg_exchange_coupling_j_mev, 5.0);
    assert_eq!(underflow.surface_acoustic_wave_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.inter_skyrmion_pitch_nm, 30.0);
    assert_eq!(underflow.braiding_path_length_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = MoireSkyrmionAnyonBraidingParams::new(
        3.50,  // above 2.50 deg
        1.20,  // above 0.95
        25.0,  // above 15.0 meV
        60.0,  // above 40.0 meV
        20.0,  // above 15.0 GHz
        80.0,  // above 50.0 mK
        450.0, // above 300.0 nm
        12.0,  // above 8.0 um
    );
    assert_eq!(overflow.twist_angle_degrees, 2.50);
    assert_eq!(overflow.spin_peierls_coupling_constant, 0.95);
    assert_eq!(overflow.dzyaloshinskii_moriya_interaction_mev, 15.0);
    assert_eq!(overflow.heisenberg_exchange_coupling_j_mev, 40.0);
    assert_eq!(overflow.surface_acoustic_wave_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.inter_skyrmion_pitch_nm, 300.0);
    assert_eq!(overflow.braiding_path_length_um, 8.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = MoireSkyrmionAnyonBraidingParams::default();
    assert_eq!(params.twist_angle_degrees, 1.25);
    assert_eq!(params.spin_peierls_coupling_constant, 0.65);
    assert_eq!(params.dzyaloshinskii_moriya_interaction_mev, 5.8);
    assert_eq!(params.heisenberg_exchange_coupling_j_mev, 18.0);
    assert_eq!(params.surface_acoustic_wave_frequency_ghz, 4.5);
    assert_eq!(params.cryogenic_temperature_mk, 12.0);
    assert_eq!(params.inter_skyrmion_pitch_nm, 110.0);
    assert_eq!(params.braiding_path_length_um, 2.2);

    let solver = MoireSkyrmionAnyonBraidingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.anyonic_braiding_phase_fidelity >= 0.9980,
        "Anyonic braiding phase fidelity must be >= 0.9980, got {:.6}",
        metrics.anyonic_braiding_phase_fidelity
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 42.0,
        "Topological protection gap must be >= 42.0 MHz, got {:.4} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.skyrmion_topological_stability_fraction >= 0.9970,
        "Skyrmion topological stability fraction must be >= 0.9970, got {:.6}",
        metrics.skyrmion_topological_stability_fraction
    );
    assert!(
        metrics.inter_skyrmion_crosstalk_isolation_db >= 53.0,
        "Inter-skyrmion crosstalk isolation must be >= 53.0 dB, got {:.4} dB",
        metrics.inter_skyrmion_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 16.0,
        "Topological mode dephasing rate must be <= 16.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_twist_angle_scaling() {
    let mut low_params = MoireSkyrmionAnyonBraidingParams::default();
    low_params.twist_angle_degrees = 0.85;
    let mut high_params = MoireSkyrmionAnyonBraidingParams::default();
    high_params.twist_angle_degrees = 2.40;

    let low_solver = MoireSkyrmionAnyonBraidingSolver::new(low_params);
    let high_solver = MoireSkyrmionAnyonBraidingSolver::new(high_params);

    assert!(
        high_solver.compute_anyonic_braiding_phase_fidelity()
            >= low_solver.compute_anyonic_braiding_phase_fidelity(),
        "Higher twist angle modulation should improve anyonic braiding fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            >= low_solver.compute_topological_protection_gap_mhz(),
        "Higher twist angle should enhance the topological protection gap"
    );
}

#[test]
fn test_spin_peierls_coupling_scaling() {
    let mut low_params = MoireSkyrmionAnyonBraidingParams::default();
    low_params.spin_peierls_coupling_constant = 0.15;
    let mut high_params = MoireSkyrmionAnyonBraidingParams::default();
    high_params.spin_peierls_coupling_constant = 0.90;

    let low_solver = MoireSkyrmionAnyonBraidingSolver::new(low_params);
    let high_solver = MoireSkyrmionAnyonBraidingSolver::new(high_params);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Stronger spin-Peierls coupling must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_anyonic_braiding_phase_fidelity()
            > low_solver.compute_anyonic_braiding_phase_fidelity(),
        "Stronger spin-Peierls coupling must enhance braiding fidelity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Stronger spin-Peierls coupling must suppress topological dephasing rate"
    );
}

#[test]
fn test_dzyaloshinskii_moriya_scaling() {
    let mut low_params = MoireSkyrmionAnyonBraidingParams::default();
    low_params.dzyaloshinskii_moriya_interaction_mev = 1.5;
    let mut high_params = MoireSkyrmionAnyonBraidingParams::default();
    high_params.dzyaloshinskii_moriya_interaction_mev = 14.0;

    let low_solver = MoireSkyrmionAnyonBraidingSolver::new(low_params);
    let high_solver = MoireSkyrmionAnyonBraidingSolver::new(high_params);

    assert!(
        high_solver.compute_skyrmion_topological_stability_fraction()
            > low_solver.compute_skyrmion_topological_stability_fraction(),
        "Stronger DMI must increase skyrmion topological stability fraction"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Stronger DMI must increase the topological protection energy gap"
    );
}

#[test]
fn test_heisenberg_exchange_scaling() {
    let mut low_params = MoireSkyrmionAnyonBraidingParams::default();
    low_params.heisenberg_exchange_coupling_j_mev = 6.0;
    let mut high_params = MoireSkyrmionAnyonBraidingParams::default();
    high_params.heisenberg_exchange_coupling_j_mev = 38.0;

    let low_solver = MoireSkyrmionAnyonBraidingSolver::new(low_params);
    let high_solver = MoireSkyrmionAnyonBraidingSolver::new(high_params);

    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Stronger Heisenberg exchange J must boost the topological protection gap"
    );
    assert!(
        high_solver.compute_skyrmion_topological_stability_fraction()
            > low_solver.compute_skyrmion_topological_stability_fraction(),
        "Stronger Heisenberg exchange J must enhance skyrmion stability"
    );
}

#[test]
fn test_saw_frequency_scaling() {
    let mut low_params = MoireSkyrmionAnyonBraidingParams::default();
    low_params.surface_acoustic_wave_frequency_ghz = 1.5;
    let mut high_params = MoireSkyrmionAnyonBraidingParams::default();
    high_params.surface_acoustic_wave_frequency_ghz = 14.0;

    let low_solver = MoireSkyrmionAnyonBraidingSolver::new(low_params);
    let high_solver = MoireSkyrmionAnyonBraidingSolver::new(high_params);

    assert!(
        high_solver.compute_anyonic_braiding_phase_fidelity()
            >= low_solver.compute_anyonic_braiding_phase_fidelity(),
        "Higher SAW driving frequency should enhance braiding phase fidelity"
    );
    assert!(
        high_solver.compute_topological_protection_gap_mhz()
            > low_solver.compute_topological_protection_gap_mhz(),
        "Higher SAW frequency should increase the effective dynamic topological gap"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_params = MoireSkyrmionAnyonBraidingParams::default();
    low_params.cryogenic_temperature_mk = 2.0;
    let mut high_params = MoireSkyrmionAnyonBraidingParams::default();
    high_params.cryogenic_temperature_mk = 48.0;

    let low_solver = MoireSkyrmionAnyonBraidingSolver::new(low_params);
    let high_solver = MoireSkyrmionAnyonBraidingSolver::new(high_params);

    assert!(
        low_solver.compute_anyonic_braiding_phase_fidelity()
            > high_solver.compute_anyonic_braiding_phase_fidelity(),
        "Lower cryogenic temperature must preserve higher braiding phase fidelity"
    );
    assert!(
        low_solver.compute_skyrmion_topological_stability_fraction()
            > high_solver.compute_skyrmion_topological_stability_fraction(),
        "Lower temperature must enhance skyrmion topological stability"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            > low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher cryogenic temperature must increase the topological mode dephasing rate"
    );
}

#[test]
fn test_inter_skyrmion_pitch_scaling() {
    let mut low_params = MoireSkyrmionAnyonBraidingParams::default();
    low_params.inter_skyrmion_pitch_nm = 35.0;
    let mut high_params = MoireSkyrmionAnyonBraidingParams::default();
    high_params.inter_skyrmion_pitch_nm = 280.0;

    let low_solver = MoireSkyrmionAnyonBraidingSolver::new(low_params);
    let high_solver = MoireSkyrmionAnyonBraidingSolver::new(high_params);

    assert!(
        high_solver.compute_inter_skyrmion_crosstalk_isolation_db()
            > low_solver.compute_inter_skyrmion_crosstalk_isolation_db(),
        "Larger inter-skyrmion pitch must significantly increase crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Larger pitch must reduce parasitic dephasing rate"
    );
}

#[test]
fn test_braiding_path_length_scaling() {
    let mut low_params = MoireSkyrmionAnyonBraidingParams::default();
    low_params.braiding_path_length_um = 0.6;
    let mut high_params = MoireSkyrmionAnyonBraidingParams::default();
    high_params.braiding_path_length_um = 7.5;

    let low_solver = MoireSkyrmionAnyonBraidingSolver::new(low_params);
    let high_solver = MoireSkyrmionAnyonBraidingSolver::new(high_params);

    assert!(
        high_solver.compute_anyonic_braiding_phase_fidelity()
            >= low_solver.compute_anyonic_braiding_phase_fidelity(),
        "Well-defined braiding paths should support high adiabatic fidelity"
    );
    assert!(
        high_solver.compute_inter_skyrmion_crosstalk_isolation_db()
            >= low_solver.compute_inter_skyrmion_crosstalk_isolation_db(),
        "Longer braiding paths maintain high acoustic spatial isolation"
    );
}
