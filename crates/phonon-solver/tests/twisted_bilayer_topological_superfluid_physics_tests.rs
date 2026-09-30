#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Abelian quantum acoustic
//! twisted bilayer topological superfluidity and chiral Majorana vortex networks.

use phonon_models::twisted_bilayer_topological_superfluid::TwistedBilayerTopologicalSuperfluidParams;
use phonon_solver::twisted_bilayer_topological_superfluid::TwistedBilayerTopologicalSuperfluidSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = TwistedBilayerTopologicalSuperfluidParams::new(
        0.50, // below 0.80 deg
        2.0,  // below 5.0 meV
        0.5,  // below 2.0 meV
        0.5,  // below 1.0 GHz
        0.5,  // below 1.0 mK
        5.0,  // below 10.0 nm
        0.2,  // below 0.5 um
        0.5,  // below 1.0 meV
    );
    assert_eq!(underflow.twist_angle_degrees, 0.80);
    assert_eq!(underflow.interlayer_josephson_coupling_mev, 5.0);
    assert_eq!(underflow.p_wave_pairing_amplitude_mev, 2.0);
    assert_eq!(underflow.acoustic_vortex_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.vortex_core_radius_nm, 10.0);
    assert_eq!(underflow.inter_vortex_separation_um, 0.5);
    assert_eq!(underflow.pinning_potential_barrier_mev, 1.0);

    // Test values strictly above physical maximum bounds
    let overflow = TwistedBilayerTopologicalSuperfluidParams::new(
        2.50,  // above 1.40 deg
        80.0,  // above 50.0 meV
        50.0,  // above 30.0 meV
        25.0,  // above 15.0 GHz
        80.0,  // above 50.0 mK
        250.0, // above 150.0 nm
        18.0,  // above 10.0 um
        40.0,  // above 25.0 meV
    );
    assert_eq!(overflow.twist_angle_degrees, 1.40);
    assert_eq!(overflow.interlayer_josephson_coupling_mev, 50.0);
    assert_eq!(overflow.p_wave_pairing_amplitude_mev, 30.0);
    assert_eq!(overflow.acoustic_vortex_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.vortex_core_radius_nm, 150.0);
    assert_eq!(overflow.inter_vortex_separation_um, 10.0);
    assert_eq!(overflow.pinning_potential_barrier_mev, 25.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = TwistedBilayerTopologicalSuperfluidParams::default();
    assert_eq!(params.twist_angle_degrees, 1.12);
    assert_eq!(params.interlayer_josephson_coupling_mev, 22.0);
    assert_eq!(params.p_wave_pairing_amplitude_mev, 14.5);
    assert_eq!(params.acoustic_vortex_frequency_ghz, 4.8);
    assert_eq!(params.cryogenic_temperature_mk, 12.0);
    assert_eq!(params.vortex_core_radius_nm, 45.0);
    assert_eq!(params.inter_vortex_separation_um, 2.8);
    assert_eq!(params.pinning_potential_barrier_mev, 9.2);

    let solver = TwistedBilayerTopologicalSuperfluidSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.vortex_state_fidelity >= 0.9980,
        "Vortex state fidelity must be >= 0.9980, got {:.6}",
        metrics.vortex_state_fidelity
    );
    assert!(
        metrics.topological_vortex_pinning_gap_mhz >= 40.0,
        "Topological vortex pinning gap must be >= 40.0 MHz, got {:.4} MHz",
        metrics.topological_vortex_pinning_gap_mhz
    );
    assert!(
        metrics.inter_vortex_crosstalk_isolation_db >= 52.0,
        "Inter-vortex crosstalk isolation must be >= 52.0 dB, got {:.4} dB",
        metrics.inter_vortex_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_vortex_dephasing_rate_hz <= 18.0,
        "Topological vortex dephasing rate must be <= 18.0 Hz, got {:.4} Hz",
        metrics.topological_vortex_dephasing_rate_hz
    );
    assert!(
        metrics.chiral_majorana_mode_purity >= 0.992,
        "Chiral Majorana mode purity must be >= 0.992, got {:.6}",
        metrics.chiral_majorana_mode_purity
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_twist_angle_scaling() {
    let mut tuned = TwistedBilayerTopologicalSuperfluidParams::default();
    tuned.twist_angle_degrees = 1.12; // magic angle
    let solver_tuned = TwistedBilayerTopologicalSuperfluidSolver::new(tuned);
    let metrics_tuned = solver_tuned.evaluate_metrics();

    let mut detuned = TwistedBilayerTopologicalSuperfluidParams::default();
    detuned.twist_angle_degrees = 0.85; // away from magic angle
    let solver_detuned = TwistedBilayerTopologicalSuperfluidSolver::new(detuned);
    let metrics_detuned = solver_detuned.evaluate_metrics();

    // Magic angle optimizes flat-band localization, enhancing fidelity, pinning gap, and purity
    assert!(
        metrics_tuned.vortex_state_fidelity > metrics_detuned.vortex_state_fidelity
    );
    assert!(
        metrics_tuned.topological_vortex_pinning_gap_mhz
            > metrics_detuned.topological_vortex_pinning_gap_mhz
    );
    assert!(
        metrics_tuned.chiral_majorana_mode_purity
            > metrics_detuned.chiral_majorana_mode_purity
    );
    assert!(
        metrics_tuned.topological_vortex_dephasing_rate_hz
            < metrics_detuned.topological_vortex_dephasing_rate_hz
    );
}

#[test]
fn test_interlayer_josephson_coupling_scaling() {
    let mut low_j = TwistedBilayerTopologicalSuperfluidParams::default();
    low_j.interlayer_josephson_coupling_mev = 8.0;
    let solver_low = TwistedBilayerTopologicalSuperfluidSolver::new(low_j);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_j = TwistedBilayerTopologicalSuperfluidParams::default();
    high_j.interlayer_josephson_coupling_mev = 45.0;
    let solver_high = TwistedBilayerTopologicalSuperfluidSolver::new(high_j);
    let metrics_high = solver_high.evaluate_metrics();

    // Stronger interlayer Josephson coupling boosts pinning gap and crosstalk isolation
    assert!(
        metrics_high.topological_vortex_pinning_gap_mhz
            > metrics_low.topological_vortex_pinning_gap_mhz
    );
    assert!(
        metrics_high.inter_vortex_crosstalk_isolation_db
            > metrics_low.inter_vortex_crosstalk_isolation_db
    );
    assert!(
        metrics_high.topological_vortex_dephasing_rate_hz
            < metrics_low.topological_vortex_dephasing_rate_hz
    );
}

#[test]
fn test_p_wave_pairing_amplitude_scaling() {
    let mut low_delta = TwistedBilayerTopologicalSuperfluidParams::default();
    low_delta.p_wave_pairing_amplitude_mev = 4.0;
    let solver_low = TwistedBilayerTopologicalSuperfluidSolver::new(low_delta);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_delta = TwistedBilayerTopologicalSuperfluidParams::default();
    high_delta.p_wave_pairing_amplitude_mev = 28.0;
    let solver_high = TwistedBilayerTopologicalSuperfluidSolver::new(high_delta);
    let metrics_high = solver_high.evaluate_metrics();

    // Higher p-wave pairing amplitude increases chiral Majorana mode purity and pinning gap
    assert!(
        metrics_high.chiral_majorana_mode_purity
            > metrics_low.chiral_majorana_mode_purity
    );
    assert!(
        metrics_high.topological_vortex_pinning_gap_mhz
            > metrics_low.topological_vortex_pinning_gap_mhz
    );
    assert!(
        metrics_high.vortex_state_fidelity > metrics_low.vortex_state_fidelity
    );
}

#[test]
fn test_acoustic_vortex_frequency_scaling() {
    let mut low_f = TwistedBilayerTopologicalSuperfluidParams::default();
    low_f.acoustic_vortex_frequency_ghz = 1.5;
    let solver_low = TwistedBilayerTopologicalSuperfluidSolver::new(low_f);
    let metrics_low = solver_low.evaluate_metrics();

    let mut high_f = TwistedBilayerTopologicalSuperfluidParams::default();
    high_f.acoustic_vortex_frequency_ghz = 13.5;
    let solver_high = TwistedBilayerTopologicalSuperfluidSolver::new(high_f);
    let metrics_high = solver_high.evaluate_metrics();

    // Higher frequency increases pinning protection gap and inter-vortex isolation
    assert!(
        metrics_high.topological_vortex_pinning_gap_mhz
            > metrics_low.topological_vortex_pinning_gap_mhz
    );
    assert!(
        metrics_high.inter_vortex_crosstalk_isolation_db
            > metrics_low.inter_vortex_crosstalk_isolation_db
    );
    assert!(
        metrics_high.topological_vortex_dephasing_rate_hz
            < metrics_low.topological_vortex_dephasing_rate_hz
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut cold = TwistedBilayerTopologicalSuperfluidParams::default();
    cold.cryogenic_temperature_mk = 2.0;
    let solver_cold = TwistedBilayerTopologicalSuperfluidSolver::new(cold);
    let metrics_cold = solver_cold.evaluate_metrics();

    let mut warm = TwistedBilayerTopologicalSuperfluidParams::default();
    warm.cryogenic_temperature_mk = 45.0;
    let solver_warm = TwistedBilayerTopologicalSuperfluidSolver::new(warm);
    let metrics_warm = solver_warm.evaluate_metrics();

    // Cooler temperatures enhance fidelity, expand gap, and suppress thermal dephasing
    assert!(
        metrics_cold.vortex_state_fidelity > metrics_warm.vortex_state_fidelity
    );
    assert!(
        metrics_cold.topological_vortex_pinning_gap_mhz
            > metrics_warm.topological_vortex_pinning_gap_mhz
    );
    assert!(
        metrics_cold.chiral_majorana_mode_purity
            > metrics_warm.chiral_majorana_mode_purity
    );
    assert!(
        metrics_cold.topological_vortex_dephasing_rate_hz
            < metrics_warm.topological_vortex_dephasing_rate_hz
    );
}

#[test]
fn test_vortex_core_radius_scaling() {
    let mut small_core = TwistedBilayerTopologicalSuperfluidParams::default();
    small_core.vortex_core_radius_nm = 15.0;
    let solver_small = TwistedBilayerTopologicalSuperfluidSolver::new(small_core);
    let metrics_small = solver_small.evaluate_metrics();

    let mut large_core = TwistedBilayerTopologicalSuperfluidParams::default();
    large_core.vortex_core_radius_nm = 135.0;
    let solver_large = TwistedBilayerTopologicalSuperfluidSolver::new(large_core);
    let metrics_large = solver_large.evaluate_metrics();

    // Tightly localized core radius enhances crosstalk isolation and Majorana purity
    assert!(
        metrics_small.inter_vortex_crosstalk_isolation_db
            > metrics_large.inter_vortex_crosstalk_isolation_db
    );
    assert!(
        metrics_small.chiral_majorana_mode_purity
            > metrics_large.chiral_majorana_mode_purity
    );
    assert!(
        metrics_small.topological_vortex_dephasing_rate_hz
            < metrics_large.topological_vortex_dephasing_rate_hz
    );
}

#[test]
fn test_inter_vortex_separation_scaling() {
    let mut close = TwistedBilayerTopologicalSuperfluidParams::default();
    close.inter_vortex_separation_um = 1.0;
    let solver_close = TwistedBilayerTopologicalSuperfluidSolver::new(close);
    let metrics_close = solver_close.evaluate_metrics();

    let mut far = TwistedBilayerTopologicalSuperfluidParams::default();
    far.inter_vortex_separation_um = 8.5;
    let solver_far = TwistedBilayerTopologicalSuperfluidSolver::new(far);
    let metrics_far = solver_far.evaluate_metrics();

    // Larger separation substantially improves crosstalk isolation and lowers dephasing
    assert!(
        metrics_far.inter_vortex_crosstalk_isolation_db
            > metrics_close.inter_vortex_crosstalk_isolation_db
    );
    assert!(
        metrics_far.vortex_state_fidelity > metrics_close.vortex_state_fidelity
    );
    assert!(
        metrics_far.topological_vortex_dephasing_rate_hz
            < metrics_close.topological_vortex_dephasing_rate_hz
    );
}

#[test]
fn test_pinning_potential_barrier_scaling() {
    let mut shallow_pin = TwistedBilayerTopologicalSuperfluidParams::default();
    shallow_pin.pinning_potential_barrier_mev = 2.0;
    let solver_shallow = TwistedBilayerTopologicalSuperfluidSolver::new(shallow_pin);
    let metrics_shallow = solver_shallow.evaluate_metrics();

    let mut deep_pin = TwistedBilayerTopologicalSuperfluidParams::default();
    deep_pin.pinning_potential_barrier_mev = 22.0;
    let solver_deep = TwistedBilayerTopologicalSuperfluidSolver::new(deep_pin);
    let metrics_deep = solver_deep.evaluate_metrics();

    // Deeper pinning potential barrier widens pinning protection gap and suppresses dephasing
    assert!(
        metrics_deep.topological_vortex_pinning_gap_mhz
            > metrics_shallow.topological_vortex_pinning_gap_mhz
    );
    assert!(
        metrics_deep.inter_vortex_crosstalk_isolation_db
            > metrics_shallow.inter_vortex_crosstalk_isolation_db
    );
    assert!(
        metrics_deep.topological_vortex_dephasing_rate_hz
            < metrics_shallow.topological_vortex_dephasing_rate_hz
    );
}
