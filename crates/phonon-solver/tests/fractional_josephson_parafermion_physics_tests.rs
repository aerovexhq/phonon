#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for topological acoustic
//! parafermionic fractional Josephson interconnects and non-Abelian quantum logic.

use phonon_models::fractional_josephson_parafermion::FractionalJosephsonParafermionParams;
use phonon_solver::fractional_josephson_parafermion::FractionalJosephsonParafermionSolver;
use std::f64::consts::PI;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FractionalJosephsonParafermionParams::new(
        -1.0,  // below 0.0 rad
        0.05,  // below 0.10
        2.0,   // below 5.0 MHz
        0.5,   // below 1.0 GHz
        0.40,  // below 0.50
        100.0, // below 200.0 m/s
        0.5,   // below 1.0 mK
        0.2,   // below 0.5 um
    );
    assert_eq!(underflow.superconducting_phase_difference_rad, 0.0);
    assert_eq!(underflow.fractional_filling_factor_nu, 0.10);
    assert_eq!(underflow.induced_pairing_gap_mhz, 5.0);
    assert_eq!(underflow.acoustic_wavepacket_frequency_ghz, 1.0);
    assert_eq!(underflow.junction_barrier_transparency, 0.50);
    assert_eq!(underflow.parafermion_braiding_velocity_mps, 200.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.heterostructure_length_um, 0.5);

    // Test values strictly above physical maximum bounds
    let overflow = FractionalJosephsonParafermionParams::new(
        25.0,   // above 6.0 * PI (~18.8495 rad)
        1.50,   // above 1.0
        100.0,  // above 80.0 MHz
        15.0,   // above 12.0 GHz
        1.05,   // above 0.99
        3000.0, // above 2500.0 m/s
        80.0,   // above 50.0 mK
        15.0,   // above 10.0 um
    );
    assert_eq!(overflow.superconducting_phase_difference_rad, 6.0 * PI);
    assert_eq!(overflow.fractional_filling_factor_nu, 1.0);
    assert_eq!(overflow.induced_pairing_gap_mhz, 80.0);
    assert_eq!(overflow.acoustic_wavepacket_frequency_ghz, 12.0);
    assert_eq!(overflow.junction_barrier_transparency, 0.99);
    assert_eq!(overflow.parafermion_braiding_velocity_mps, 2500.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.heterostructure_length_um, 10.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FractionalJosephsonParafermionParams::default();
    let solver = FractionalJosephsonParafermionSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.fractional_braiding_phase_fidelity >= 0.9970,
        "Default fractional braiding phase fidelity must be >= 0.9970, got {:.6}",
        metrics.fractional_braiding_phase_fidelity
    );
    assert!(
        metrics.fractional_josephson_coherence_ms >= 10.0,
        "Default fractional Josephson coherence must be >= 10.0 ms, got {:.2} ms",
        metrics.fractional_josephson_coherence_ms
    );
    assert!(
        metrics.non_adiabatic_excitation_leakage <= 1.0e-5,
        "Default non-adiabatic leakage must be <= 1.0e-5, got {:.4e}",
        metrics.non_adiabatic_excitation_leakage
    );
    assert!(
        metrics.quasiparticle_parity_poisoning_immunity_db >= 40.0,
        "Default quasiparticle poisoning immunity must be >= 40.0 dB, got {:.2} dB",
        metrics.quasiparticle_parity_poisoning_immunity_db
    );
    assert!(
        metrics.fractional_conductance_quantization_error <= 0.0030,
        "Default fractional conductance error must be <= 0.0030, got {:.6}",
        metrics.fractional_conductance_quantization_error
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_pairing_gap_scaling() {
    let base = FractionalJosephsonParafermionParams::default();
    let solver_base = FractionalJosephsonParafermionSolver::new(base);

    let high_gap = FractionalJosephsonParafermionParams::new(
        base.superconducting_phase_difference_rad,
        base.fractional_filling_factor_nu,
        50.0, // increased from 28.0 MHz
        base.acoustic_wavepacket_frequency_ghz,
        base.junction_barrier_transparency,
        base.parafermion_braiding_velocity_mps,
        base.cryogenic_temperature_mk,
        base.heterostructure_length_um,
    );
    let solver_high = FractionalJosephsonParafermionSolver::new(high_gap);

    let fid_base = solver_base.compute_fractional_braiding_phase_fidelity();
    let fid_high = solver_high.compute_fractional_braiding_phase_fidelity();
    assert!(
        fid_high > fid_base,
        "Higher pairing gap must increase braiding phase fidelity: {:.6} vs {:.6}",
        fid_high,
        fid_base
    );

    let coh_base = solver_base.compute_fractional_josephson_coherence_ms();
    let coh_high = solver_high.compute_fractional_josephson_coherence_ms();
    assert!(
        coh_high > coh_base,
        "Higher pairing gap must increase coherence: {:.2} ms vs {:.2} ms",
        coh_high,
        coh_base
    );

    let leak_base = solver_base.compute_non_adiabatic_excitation_leakage();
    let leak_high = solver_high.compute_non_adiabatic_excitation_leakage();
    assert!(
        leak_high < leak_base,
        "Higher pairing gap must suppress non-adiabatic leakage: {:.4e} vs {:.4e}",
        leak_high,
        leak_base
    );

    let imm_base = solver_base.compute_quasiparticle_parity_poisoning_immunity_db();
    let imm_high = solver_high.compute_quasiparticle_parity_poisoning_immunity_db();
    assert!(
        imm_high > imm_base,
        "Higher pairing gap must improve poisoning immunity: {:.2} dB vs {:.2} dB",
        imm_high,
        imm_base
    );
}

#[test]
fn test_barrier_transparency_scaling() {
    let base = FractionalJosephsonParafermionParams::default();
    let solver_base = FractionalJosephsonParafermionSolver::new(base);

    let high_trans = FractionalJosephsonParafermionParams::new(
        base.superconducting_phase_difference_rad,
        base.fractional_filling_factor_nu,
        base.induced_pairing_gap_mhz,
        base.acoustic_wavepacket_frequency_ghz,
        0.98, // increased from 0.93
        base.parafermion_braiding_velocity_mps,
        base.cryogenic_temperature_mk,
        base.heterostructure_length_um,
    );
    let solver_high = FractionalJosephsonParafermionSolver::new(high_trans);

    let coh_base = solver_base.compute_fractional_josephson_coherence_ms();
    let coh_high = solver_high.compute_fractional_josephson_coherence_ms();
    assert!(
        coh_high > coh_base,
        "Higher barrier transparency must increase coherence lifetime: {:.2} ms vs {:.2} ms",
        coh_high,
        coh_base
    );

    let err_base = solver_base.compute_fractional_conductance_quantization_error();
    let err_high = solver_high.compute_fractional_conductance_quantization_error();
    assert!(
        err_high < err_base,
        "Higher barrier transparency must reduce conductance error: {:.6} vs {:.6}",
        err_high,
        err_base
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let base = FractionalJosephsonParafermionParams::default();
    let solver_base = FractionalJosephsonParafermionSolver::new(base);

    let warm_temp = FractionalJosephsonParafermionParams::new(
        base.superconducting_phase_difference_rad,
        base.fractional_filling_factor_nu,
        base.induced_pairing_gap_mhz,
        base.acoustic_wavepacket_frequency_ghz,
        base.junction_barrier_transparency,
        base.parafermion_braiding_velocity_mps,
        28.0, // increased from 12.0 mK
        base.heterostructure_length_um,
    );
    let solver_warm = FractionalJosephsonParafermionSolver::new(warm_temp);

    let fid_base = solver_base.compute_fractional_braiding_phase_fidelity();
    let fid_warm = solver_warm.compute_fractional_braiding_phase_fidelity();
    assert!(
        fid_warm < fid_base,
        "Higher temperature must degrade braiding phase fidelity: {:.6} vs {:.6}",
        fid_warm,
        fid_base
    );

    let coh_base = solver_base.compute_fractional_josephson_coherence_ms();
    let coh_warm = solver_warm.compute_fractional_josephson_coherence_ms();
    assert!(
        coh_warm < coh_base,
        "Higher temperature must degrade coherence lifetime: {:.2} ms vs {:.2} ms",
        coh_warm,
        coh_base
    );

    let leak_base = solver_base.compute_non_adiabatic_excitation_leakage();
    let leak_warm = solver_warm.compute_non_adiabatic_excitation_leakage();
    assert!(
        leak_warm > leak_base,
        "Higher temperature must increase leakage: {:.4e} vs {:.4e}",
        leak_warm,
        leak_base
    );
}

#[test]
fn test_heterostructure_length_scaling() {
    let base = FractionalJosephsonParafermionParams::default();
    let solver_base = FractionalJosephsonParafermionSolver::new(base);

    let long_channel = FractionalJosephsonParafermionParams::new(
        base.superconducting_phase_difference_rad,
        base.fractional_filling_factor_nu,
        base.induced_pairing_gap_mhz,
        base.acoustic_wavepacket_frequency_ghz,
        base.junction_barrier_transparency,
        base.parafermion_braiding_velocity_mps,
        base.cryogenic_temperature_mk,
        6.5, // increased from 3.5 um
    );
    let solver_long = FractionalJosephsonParafermionSolver::new(long_channel);

    let imm_base = solver_base.compute_quasiparticle_parity_poisoning_immunity_db();
    let imm_long = solver_long.compute_quasiparticle_parity_poisoning_immunity_db();
    assert!(
        imm_long > imm_base,
        "Longer channel length must increase quasiparticle poisoning immunity: {:.2} dB vs {:.2} dB",
        imm_long,
        imm_base
    );

    let leak_base = solver_base.compute_non_adiabatic_excitation_leakage();
    let leak_long = solver_long.compute_non_adiabatic_excitation_leakage();
    assert!(
        leak_long < leak_base,
        "Longer channel length must reduce non-adiabatic leakage: {:.4e} vs {:.4e}",
        leak_long,
        leak_base
    );
}

#[test]
fn test_braiding_velocity_scaling() {
    let base = FractionalJosephsonParafermionParams::default();
    let solver_base = FractionalJosephsonParafermionSolver::new(base);

    let fast_braid = FractionalJosephsonParafermionParams::new(
        base.superconducting_phase_difference_rad,
        base.fractional_filling_factor_nu,
        base.induced_pairing_gap_mhz,
        base.acoustic_wavepacket_frequency_ghz,
        base.junction_barrier_transparency,
        2000.0, // increased from 1150.0 m/s
        base.cryogenic_temperature_mk,
        base.heterostructure_length_um,
    );
    let solver_fast = FractionalJosephsonParafermionSolver::new(fast_braid);

    let leak_base = solver_base.compute_non_adiabatic_excitation_leakage();
    let leak_fast = solver_fast.compute_non_adiabatic_excitation_leakage();
    assert!(
        leak_fast > leak_base,
        "Higher braiding velocity must increase non-adiabatic leakage: {:.4e} vs {:.4e}",
        leak_fast,
        leak_base
    );
}

#[test]
fn test_acoustic_frequency_scaling() {
    let base = FractionalJosephsonParafermionParams::default();
    let solver_base = FractionalJosephsonParafermionSolver::new(base);

    let high_freq = FractionalJosephsonParafermionParams::new(
        base.superconducting_phase_difference_rad,
        base.fractional_filling_factor_nu,
        base.induced_pairing_gap_mhz,
        8.5, // increased from 4.2 GHz
        base.junction_barrier_transparency,
        base.parafermion_braiding_velocity_mps,
        base.cryogenic_temperature_mk,
        base.heterostructure_length_um,
    );
    let solver_high = FractionalJosephsonParafermionSolver::new(high_freq);

    let fid_base = solver_base.compute_fractional_braiding_phase_fidelity();
    let fid_high = solver_high.compute_fractional_braiding_phase_fidelity();
    assert!(
        fid_high > fid_base,
        "Higher acoustic frequency must increase braiding phase fidelity: {:.6} vs {:.6}",
        fid_high,
        fid_base
    );

    let coh_base = solver_base.compute_fractional_josephson_coherence_ms();
    let coh_high = solver_high.compute_fractional_josephson_coherence_ms();
    assert!(
        coh_high > coh_base,
        "Higher acoustic frequency must increase coherence lifetime: {:.2} ms vs {:.2} ms",
        coh_high,
        coh_base
    );
}

#[test]
fn test_fractional_filling_factor_scaling() {
    let base = FractionalJosephsonParafermionParams::default(); // nu = 0.333333
    let solver_base = FractionalJosephsonParafermionSolver::new(base);

    let detuned_nu = FractionalJosephsonParafermionParams::new(
        base.superconducting_phase_difference_rad,
        0.85, // detuned from 1/3 plateau
        base.induced_pairing_gap_mhz,
        base.acoustic_wavepacket_frequency_ghz,
        base.junction_barrier_transparency,
        base.parafermion_braiding_velocity_mps,
        base.cryogenic_temperature_mk,
        base.heterostructure_length_um,
    );
    let solver_detuned = FractionalJosephsonParafermionSolver::new(detuned_nu);

    let imm_base = solver_base.compute_quasiparticle_parity_poisoning_immunity_db();
    let imm_detuned = solver_detuned.compute_quasiparticle_parity_poisoning_immunity_db();
    assert!(
        imm_base > imm_detuned,
        "Plateau filling factor (nu = 1/3) must provide higher immunity than detuned: {:.2} dB vs {:.2} dB",
        imm_base,
        imm_detuned
    );
}

#[test]
fn test_phase_difference_conductance_modulation() {
    let base = FractionalJosephsonParafermionParams::default();
    let solver_base = FractionalJosephsonParafermionSolver::new(base);

    let zero_phase = FractionalJosephsonParafermionParams::new(
        0.0, // phase = 0.0
        base.fractional_filling_factor_nu,
        base.induced_pairing_gap_mhz,
        base.acoustic_wavepacket_frequency_ghz,
        base.junction_barrier_transparency,
        base.parafermion_braiding_velocity_mps,
        base.cryogenic_temperature_mk,
        base.heterostructure_length_um,
    );
    let solver_zero = FractionalJosephsonParafermionSolver::new(zero_phase);

    let err_base = solver_base.compute_fractional_conductance_quantization_error();
    let err_zero = solver_zero.compute_fractional_conductance_quantization_error();
    assert!(
        err_zero <= err_base,
        "Phase modulation at sin(0) = 0 must give minimal conductance error: {:.6} vs {:.6}",
        err_zero,
        err_base
    );
    assert!(
        err_zero <= 0.0030 && err_base <= 0.0030,
        "Conductance error must remain within target <= 0.0030 across phase difference values"
    );
}
