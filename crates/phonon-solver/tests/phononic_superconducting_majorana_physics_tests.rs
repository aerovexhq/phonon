#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for non-Abelian chiral Majorana bound
//! states in topological phononic superconducting junctions.

use phonon_models::phononic_superconducting_majorana::PhononicSuperconductingMajoranaParams;
use phonon_solver::phononic_superconducting_majorana::PhononicSuperconductingMajoranaSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = PhononicSuperconductingMajoranaParams::new(
        5.0,    // below 15.0 MHz
        5.0,    // below 10.0 meV*nm
        10.0,   // below 20.0 MHz
        0.1,    // below 0.5 GHz
        5.0,    // below 10.0 ppm
        0.5,    // below 1.0 mK
        0.30,   // below 0.50
        0.2,    // below 0.5 um
    );
    assert_eq!(underflow.superconducting_gap_mhz, 15.0);
    assert_eq!(underflow.spin_orbit_coupling_mev_nm, 10.0);
    assert_eq!(underflow.zeeman_splitting_mhz, 20.0);
    assert_eq!(underflow.acoustic_driving_frequency_ghz, 0.5);
    assert_eq!(underflow.acoustic_strain_amplitude_ppm, 10.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.junction_transparency, 0.50);
    assert_eq!(underflow.nanowire_length_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = PhononicSuperconductingMajoranaParams::new(
        200.0,  // above 120.0 MHz
        250.0,  // above 150.0 meV*nm
        300.0,  // above 200.0 MHz
        25.0,   // above 10.0 GHz
        800.0,  // above 500.0 ppm
        100.0,  // above 50.0 mK
        1.50,   // above 0.99
        20.0,   // above 10.0 um
    );
    assert_eq!(overflow.superconducting_gap_mhz, 120.0);
    assert_eq!(overflow.spin_orbit_coupling_mev_nm, 150.0);
    assert_eq!(overflow.zeeman_splitting_mhz, 200.0);
    assert_eq!(overflow.acoustic_driving_frequency_ghz, 10.0);
    assert_eq!(overflow.acoustic_strain_amplitude_ppm, 500.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.junction_transparency, 0.99);
    assert_eq!(overflow.nanowire_length_um, 10.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = PhononicSuperconductingMajoranaParams::default();
    let solver = PhononicSuperconductingMajoranaSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.braiding_phase_fidelity >= 0.9980,
        "Default braiding phase fidelity must be >= 0.9980, got {:.6}",
        metrics.braiding_phase_fidelity
    );
    assert!(
        metrics.topological_protection_gap_mhz >= 22.0,
        "Default topological protection gap must be >= 22.0 MHz, got {:.2} MHz",
        metrics.topological_protection_gap_mhz
    );
    assert!(
        metrics.non_adiabatic_leakage_probability <= 1.0e-5,
        "Default non-adiabatic leakage probability must be <= 1.0e-5, got {:.2e}",
        metrics.non_adiabatic_leakage_probability
    );
    assert!(
        metrics.quasiparticle_poisoning_immunity_db >= 38.0,
        "Default quasiparticle poisoning immunity must be >= 38.0 dB, got {:.2} dB",
        metrics.quasiparticle_poisoning_immunity_db
    );
    assert!(
        metrics.zero_bias_conductance_error_g0 <= 0.0020,
        "Default zero bias conductance error must be <= 0.0020 G_0, got {:.6} G_0",
        metrics.zero_bias_conductance_error_g0
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_superconducting_gap_scaling() {
    let base = PhononicSuperconductingMajoranaParams::default();
    let solver_base = PhononicSuperconductingMajoranaSolver::new(base);

    let high_gap = PhononicSuperconductingMajoranaParams::new(
        75.0, // increased from 45.0 MHz
        base.spin_orbit_coupling_mev_nm,
        base.zeeman_splitting_mhz,
        base.acoustic_driving_frequency_ghz,
        base.acoustic_strain_amplitude_ppm,
        base.cryogenic_temperature_mk,
        base.junction_transparency,
        base.nanowire_length_um,
    );
    let solver_high = PhononicSuperconductingMajoranaSolver::new(high_gap);

    let metrics_base = solver_base.evaluate_metrics();
    let metrics_high = solver_high.evaluate_metrics();

    assert!(
        metrics_high.topological_protection_gap_mhz > metrics_base.topological_protection_gap_mhz,
        "Higher superconducting gap must increase topological protection gap"
    );
    assert!(
        metrics_high.quasiparticle_poisoning_immunity_db > metrics_base.quasiparticle_poisoning_immunity_db,
        "Higher superconducting gap must increase quasiparticle poisoning immunity"
    );
}

#[test]
fn test_spin_orbit_coupling_scaling() {
    let base = PhononicSuperconductingMajoranaParams::default();
    let solver_base = PhononicSuperconductingMajoranaSolver::new(base);

    let high_soc = PhononicSuperconductingMajoranaParams::new(
        base.superconducting_gap_mhz,
        100.0, // increased from 65.0 meV*nm
        base.zeeman_splitting_mhz,
        base.acoustic_driving_frequency_ghz,
        base.acoustic_strain_amplitude_ppm,
        base.cryogenic_temperature_mk,
        base.junction_transparency,
        base.nanowire_length_um,
    );
    let solver_high = PhononicSuperconductingMajoranaSolver::new(high_soc);

    let metrics_base = solver_base.evaluate_metrics();
    let metrics_high = solver_high.evaluate_metrics();

    assert!(
        metrics_high.topological_protection_gap_mhz > metrics_base.topological_protection_gap_mhz,
        "Higher spin-orbit coupling must increase topological protection gap"
    );
    assert!(
        metrics_high.braiding_phase_fidelity >= metrics_base.braiding_phase_fidelity,
        "Higher spin-orbit coupling must maintain or improve braiding phase fidelity"
    );
}

#[test]
fn test_zeeman_splitting_scaling() {
    let base = PhononicSuperconductingMajoranaParams::default();
    let solver_base = PhononicSuperconductingMajoranaSolver::new(base);

    let high_zeeman = PhononicSuperconductingMajoranaParams::new(
        base.superconducting_gap_mhz,
        base.spin_orbit_coupling_mev_nm,
        120.0, // increased from 80.0 MHz
        base.acoustic_driving_frequency_ghz,
        base.acoustic_strain_amplitude_ppm,
        base.cryogenic_temperature_mk,
        base.junction_transparency,
        base.nanowire_length_um,
    );
    let solver_high = PhononicSuperconductingMajoranaSolver::new(high_zeeman);

    let metrics_base = solver_base.evaluate_metrics();
    let metrics_high = solver_high.evaluate_metrics();

    assert!(
        metrics_high.topological_protection_gap_mhz > metrics_base.topological_protection_gap_mhz,
        "Higher Zeeman splitting must increase topological protection gap"
    );
}

#[test]
fn test_acoustic_driving_frequency_scaling() {
    let base = PhononicSuperconductingMajoranaParams::default();
    let solver_base = PhononicSuperconductingMajoranaSolver::new(base);

    let high_freq = PhononicSuperconductingMajoranaParams::new(
        base.superconducting_gap_mhz,
        base.spin_orbit_coupling_mev_nm,
        base.zeeman_splitting_mhz,
        6.0, // increased from 3.4 GHz
        base.acoustic_strain_amplitude_ppm,
        base.cryogenic_temperature_mk,
        base.junction_transparency,
        base.nanowire_length_um,
    );
    let solver_high = PhononicSuperconductingMajoranaSolver::new(high_freq);

    let metrics_base = solver_base.evaluate_metrics();
    let metrics_high = solver_high.evaluate_metrics();

    assert!(
        metrics_high.non_adiabatic_leakage_probability > metrics_base.non_adiabatic_leakage_probability,
        "Higher acoustic driving frequency must increase non-adiabatic leakage probability"
    );
}

#[test]
fn test_acoustic_strain_amplitude_scaling() {
    let base = PhononicSuperconductingMajoranaParams::default();
    let solver_base = PhononicSuperconductingMajoranaSolver::new(base);

    let high_strain = PhononicSuperconductingMajoranaParams::new(
        base.superconducting_gap_mhz,
        base.spin_orbit_coupling_mev_nm,
        base.zeeman_splitting_mhz,
        base.acoustic_driving_frequency_ghz,
        250.0, // increased from 125.0 ppm
        base.cryogenic_temperature_mk,
        base.junction_transparency,
        base.nanowire_length_um,
    );
    let solver_high = PhononicSuperconductingMajoranaSolver::new(high_strain);

    let metrics_base = solver_base.evaluate_metrics();
    let metrics_high = solver_high.evaluate_metrics();

    assert!(
        metrics_high.topological_protection_gap_mhz > metrics_base.topological_protection_gap_mhz,
        "Higher acoustic strain amplitude must increase topological protection gap"
    );
    assert!(
        metrics_high.quasiparticle_poisoning_immunity_db > metrics_base.quasiparticle_poisoning_immunity_db,
        "Higher acoustic strain amplitude must increase quasiparticle poisoning immunity"
    );
}

#[test]
fn test_cryogenic_temperature_degradation() {
    let base = PhononicSuperconductingMajoranaParams::default();
    let solver_base = PhononicSuperconductingMajoranaSolver::new(base);

    let warm = PhononicSuperconductingMajoranaParams::new(
        base.superconducting_gap_mhz,
        base.spin_orbit_coupling_mev_nm,
        base.zeeman_splitting_mhz,
        base.acoustic_driving_frequency_ghz,
        base.acoustic_strain_amplitude_ppm,
        30.0, // increased from 12.0 mK
        base.junction_transparency,
        base.nanowire_length_um,
    );
    let solver_warm = PhononicSuperconductingMajoranaSolver::new(warm);

    let metrics_base = solver_base.evaluate_metrics();
    let metrics_warm = solver_warm.evaluate_metrics();

    assert!(
        metrics_warm.braiding_phase_fidelity < metrics_base.braiding_phase_fidelity,
        "Higher cryogenic temperature must degrade braiding phase fidelity"
    );
    assert!(
        metrics_warm.non_adiabatic_leakage_probability > metrics_base.non_adiabatic_leakage_probability,
        "Higher cryogenic temperature must increase non-adiabatic leakage probability"
    );
    assert!(
        metrics_warm.quasiparticle_poisoning_immunity_db < metrics_base.quasiparticle_poisoning_immunity_db,
        "Higher cryogenic temperature must degrade quasiparticle poisoning immunity"
    );
    assert!(
        metrics_warm.zero_bias_conductance_error_g0 > metrics_base.zero_bias_conductance_error_g0,
        "Higher cryogenic temperature must increase zero-bias conductance peak error"
    );
}

#[test]
fn test_junction_transparency_scaling() {
    let base = PhononicSuperconductingMajoranaParams::default();
    let solver_base = PhononicSuperconductingMajoranaSolver::new(base);

    let low_transparency = PhononicSuperconductingMajoranaParams::new(
        base.superconducting_gap_mhz,
        base.spin_orbit_coupling_mev_nm,
        base.zeeman_splitting_mhz,
        base.acoustic_driving_frequency_ghz,
        base.acoustic_strain_amplitude_ppm,
        base.cryogenic_temperature_mk,
        0.70, // decreased from 0.92
        base.nanowire_length_um,
    );
    let solver_low = PhononicSuperconductingMajoranaSolver::new(low_transparency);

    let metrics_base = solver_base.evaluate_metrics();
    let metrics_low = solver_low.evaluate_metrics();

    assert!(
        metrics_base.quasiparticle_poisoning_immunity_db > metrics_low.quasiparticle_poisoning_immunity_db,
        "Higher junction transparency must improve quasiparticle poisoning immunity"
    );
    assert!(
        metrics_base.zero_bias_conductance_error_g0 < metrics_low.zero_bias_conductance_error_g0,
        "Higher junction transparency must reduce zero-bias conductance peak error"
    );
}

#[test]
fn test_nanowire_length_scaling() {
    let base = PhononicSuperconductingMajoranaParams::default();
    let solver_base = PhononicSuperconductingMajoranaSolver::new(base);

    let long_wire = PhononicSuperconductingMajoranaParams::new(
        base.superconducting_gap_mhz,
        base.spin_orbit_coupling_mev_nm,
        base.zeeman_splitting_mhz,
        base.acoustic_driving_frequency_ghz,
        base.acoustic_strain_amplitude_ppm,
        base.cryogenic_temperature_mk,
        base.junction_transparency,
        5.0, // increased from 2.8 um
    );
    let solver_long = PhononicSuperconductingMajoranaSolver::new(long_wire);

    let metrics_base = solver_base.evaluate_metrics();
    let metrics_long = solver_long.evaluate_metrics();

    assert!(
        metrics_long.zero_bias_conductance_error_g0 < metrics_base.zero_bias_conductance_error_g0,
        "Longer nanowire length must reduce zero-bias conductance peak error via suppressed MZM overlap"
    );
    assert!(
        metrics_long.braiding_phase_fidelity >= metrics_base.braiding_phase_fidelity,
        "Longer nanowire length must maintain or improve braiding phase fidelity"
    );
}
