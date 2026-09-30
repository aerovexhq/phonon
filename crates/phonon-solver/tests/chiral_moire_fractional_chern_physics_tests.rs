#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for chiral acoustic moire
//! fractional Chern insulators and anyonic interferometric braiding networks.

use phonon_models::chiral_moire_fractional_chern::ChiralMoireFractionalChernParams;
use phonon_solver::chiral_moire_fractional_chern::ChiralMoireFractionalChernSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = ChiralMoireFractionalChernParams::new(
        0.10, // below 0.50 deg
        0.50, // below 2.0 meV
        0.05, // below 0.10
        1,    // below 2
        5.0,  // below 10.0 kHz
        0.5,  // below 1.0 GHz
        0.5,  // below 1.0 mK
        0.5,  // below 1.0 Hz
    );
    assert_eq!(underflow.twist_angle_deg, 0.50);
    assert_eq!(underflow.moire_potential_depth_mev, 2.0);
    assert_eq!(underflow.fractional_filling_factor, 0.10);
    assert_eq!(underflow.acoustic_interferometer_arms, 2);
    assert_eq!(underflow.topological_flatband_width_khz, 10.0);
    assert_eq!(underflow.braiding_drive_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.chiral_damping_rate_hz, 1.0);

    // Test values strictly above physical maximum bounds
    let overflow = ChiralMoireFractionalChernParams::new(
        15.0,  // above 10.0 deg
        80.0,  // above 50.0 meV
        1.5,   // above 1.0
        12,    // above 8
        800.0, // above 500.0 kHz
        25.0,  // above 15.0 GHz
        80.0,  // above 50.0 mK
        150.0, // above 100.0 Hz
    );
    assert_eq!(overflow.twist_angle_deg, 10.0);
    assert_eq!(overflow.moire_potential_depth_mev, 50.0);
    assert_eq!(overflow.fractional_filling_factor, 1.0);
    assert_eq!(overflow.acoustic_interferometer_arms, 8);
    assert_eq!(overflow.topological_flatband_width_khz, 500.0);
    assert_eq!(overflow.braiding_drive_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.chiral_damping_rate_hz, 100.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = ChiralMoireFractionalChernParams::default();
    let solver = ChiralMoireFractionalChernSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.anyonic_braiding_phase_fidelity >= 0.9980,
        "Default anyonic braiding phase fidelity must be >= 0.9980, got {:.6}",
        metrics.anyonic_braiding_phase_fidelity
    );
    assert!(
        metrics.moire_flatband_coherence_ms >= 15.0,
        "Default moire flatband coherence must be >= 15.0 ms, got {:.2} ms",
        metrics.moire_flatband_coherence_ms
    );
    assert!(
        metrics.non_adiabatic_braiding_leakage <= 1.0e-5,
        "Default non-adiabatic braiding leakage must be <= 1.0e-5, got {:.4e}",
        metrics.non_adiabatic_braiding_leakage
    );
    assert!(
        metrics.quasiparticle_parity_poisoning_immunity_db >= 42.0,
        "Default quasiparticle parity poisoning immunity must be >= 42.0 dB, got {:.2} dB",
        metrics.quasiparticle_parity_poisoning_immunity_db
    );
    assert!(
        metrics.braiding_phase_stability_error_rad <= 0.0020,
        "Default braiding phase stability error must be <= 0.0020 rad, got {:.6} rad",
        metrics.braiding_phase_stability_error_rad
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_twist_angle_scaling() {
    let base = ChiralMoireFractionalChernParams::default(); // twist_angle_deg = 1.08
    let solver_base = ChiralMoireFractionalChernSolver::new(base);

    let mut detuned = base;
    detuned.twist_angle_deg = 3.50; // Detuned far from magic angle
    let solver_detuned = ChiralMoireFractionalChernSolver::new(detuned);

    let fidelity_base = solver_base.compute_anyonic_braiding_phase_fidelity();
    let fidelity_detuned = solver_detuned.compute_anyonic_braiding_phase_fidelity();
    assert!(
        fidelity_base > fidelity_detuned,
        "Magic angle must provide higher braiding fidelity: base {:.6} vs detuned {:.6}",
        fidelity_base, fidelity_detuned
    );

    let coh_base = solver_base.compute_moire_flatband_coherence_ms();
    let coh_detuned = solver_detuned.compute_moire_flatband_coherence_ms();
    assert!(
        coh_base > coh_detuned,
        "Magic angle must provide longer flatband coherence: base {:.2} vs detuned {:.2}",
        coh_base, coh_detuned
    );

    let err_base = solver_base.compute_braiding_phase_stability_error_rad();
    let err_detuned = solver_detuned.compute_braiding_phase_stability_error_rad();
    assert!(
        err_base < err_detuned,
        "Magic angle must have smaller phase stability error: base {:.6} vs detuned {:.6}",
        err_base, err_detuned
    );
}

#[test]
fn test_moire_potential_depth_scaling() {
    let base = ChiralMoireFractionalChernParams::default();

    let mut shallow = base;
    shallow.moire_potential_depth_mev = 6.0;
    let solver_shallow = ChiralMoireFractionalChernSolver::new(shallow);

    let mut deep = base;
    deep.moire_potential_depth_mev = 36.0;
    let solver_deep = ChiralMoireFractionalChernSolver::new(deep);

    let coh_shallow = solver_shallow.compute_moire_flatband_coherence_ms();
    let coh_deep = solver_deep.compute_moire_flatband_coherence_ms();
    assert!(
        coh_deep > coh_shallow,
        "Deeper moire potential must increase coherence: deep {:.2} vs shallow {:.2}",
        coh_deep, coh_shallow
    );

    let imm_shallow = solver_shallow.compute_quasiparticle_parity_poisoning_immunity_db();
    let imm_deep = solver_deep.compute_quasiparticle_parity_poisoning_immunity_db();
    assert!(
        imm_deep > imm_shallow,
        "Deeper moire potential must improve poisoning immunity: deep {:.2} vs shallow {:.2}",
        imm_deep, imm_shallow
    );

    let leak_shallow = solver_shallow.compute_non_adiabatic_braiding_leakage();
    let leak_deep = solver_deep.compute_non_adiabatic_braiding_leakage();
    assert!(
        leak_deep < leak_shallow,
        "Deeper moire potential must reduce non-adiabatic leakage: deep {:.4e} vs shallow {:.4e}",
        leak_deep, leak_shallow
    );
}

#[test]
fn test_fractional_filling_factor_scaling() {
    let base = ChiralMoireFractionalChernParams::default(); // nu = 1/3
    let solver_base = ChiralMoireFractionalChernSolver::new(base);

    let mut detuned = base;
    detuned.fractional_filling_factor = 0.65;
    let solver_detuned = ChiralMoireFractionalChernSolver::new(detuned);

    let fidelity_base = solver_base.compute_anyonic_braiding_phase_fidelity();
    let fidelity_detuned = solver_detuned.compute_anyonic_braiding_phase_fidelity();
    assert!(
        fidelity_base > fidelity_detuned,
        "Optimal nu=1/3 must yield higher braiding fidelity: base {:.6} vs detuned {:.6}",
        fidelity_base, fidelity_detuned
    );

    let imm_base = solver_base.compute_quasiparticle_parity_poisoning_immunity_db();
    let imm_detuned = solver_detuned.compute_quasiparticle_parity_poisoning_immunity_db();
    assert!(
        imm_base > imm_detuned,
        "Optimal nu=1/3 must improve poisoning immunity: base {:.2} vs detuned {:.2}",
        imm_base, imm_detuned
    );
}

#[test]
fn test_acoustic_interferometer_arms_scaling() {
    let base = ChiralMoireFractionalChernParams::default();

    let mut few_arms = base;
    few_arms.acoustic_interferometer_arms = 2;
    let solver_few = ChiralMoireFractionalChernSolver::new(few_arms);

    let mut many_arms = base;
    many_arms.acoustic_interferometer_arms = 6;
    let solver_many = ChiralMoireFractionalChernSolver::new(many_arms);

    let coh_few = solver_few.compute_moire_flatband_coherence_ms();
    let coh_many = solver_many.compute_moire_flatband_coherence_ms();
    assert!(
        coh_many > coh_few,
        "More interferometer arms must increase coherence: many {:.2} vs few {:.2}",
        coh_many, coh_few
    );

    let imm_few = solver_few.compute_quasiparticle_parity_poisoning_immunity_db();
    let imm_many = solver_many.compute_quasiparticle_parity_poisoning_immunity_db();
    assert!(
        imm_many > imm_few,
        "More interferometer arms must improve parity immunity: many {:.2} vs few {:.2}",
        imm_many, imm_few
    );

    let err_few = solver_few.compute_braiding_phase_stability_error_rad();
    let err_many = solver_many.compute_braiding_phase_stability_error_rad();
    assert!(
        err_many < err_few,
        "More arms must reduce phase stability error: many {:.6} vs few {:.6}",
        err_many, err_few
    );
}

#[test]
fn test_topological_flatband_width_scaling() {
    let base = ChiralMoireFractionalChernParams::default();

    let mut narrow = base;
    narrow.topological_flatband_width_khz = 25.0;
    let solver_narrow = ChiralMoireFractionalChernSolver::new(narrow);

    let mut broad = base;
    broad.topological_flatband_width_khz = 350.0;
    let solver_broad = ChiralMoireFractionalChernSolver::new(broad);

    let coh_narrow = solver_narrow.compute_moire_flatband_coherence_ms();
    let coh_broad = solver_broad.compute_moire_flatband_coherence_ms();
    assert!(
        coh_narrow > coh_broad,
        "Narrower flatband must increase coherence: narrow {:.2} vs broad {:.2}",
        coh_narrow, coh_broad
    );

    let leak_narrow = solver_narrow.compute_non_adiabatic_braiding_leakage();
    let leak_broad = solver_broad.compute_non_adiabatic_braiding_leakage();
    assert!(
        leak_narrow < leak_broad,
        "Narrower flatband must reduce non-adiabatic leakage: narrow {:.4e} vs broad {:.4e}",
        leak_narrow, leak_broad
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let base = ChiralMoireFractionalChernParams::default();

    let mut cold = base;
    cold.cryogenic_temperature_mk = 4.0;
    let solver_cold = ChiralMoireFractionalChernSolver::new(cold);

    let mut warm = base;
    warm.cryogenic_temperature_mk = 35.0;
    let solver_warm = ChiralMoireFractionalChernSolver::new(warm);

    let coh_cold = solver_cold.compute_moire_flatband_coherence_ms();
    let coh_warm = solver_warm.compute_moire_flatband_coherence_ms();
    assert!(
        coh_cold > coh_warm,
        "Lower temperature must extend coherence: cold {:.2} vs warm {:.2}",
        coh_cold, coh_warm
    );

    let err_cold = solver_cold.compute_braiding_phase_stability_error_rad();
    let err_warm = solver_warm.compute_braiding_phase_stability_error_rad();
    assert!(
        err_cold < err_warm,
        "Lower temperature must improve phase stability: cold {:.6} vs warm {:.6}",
        err_cold, err_warm
    );
}

#[test]
fn test_chiral_damping_rate_scaling() {
    let base = ChiralMoireFractionalChernParams::default();

    let mut low_damping = base;
    low_damping.chiral_damping_rate_hz = 5.0;
    let solver_low = ChiralMoireFractionalChernSolver::new(low_damping);

    let mut high_damping = base;
    high_damping.chiral_damping_rate_hz = 60.0;
    let solver_high = ChiralMoireFractionalChernSolver::new(high_damping);

    let coh_low = solver_low.compute_moire_flatband_coherence_ms();
    let coh_high = solver_high.compute_moire_flatband_coherence_ms();
    assert!(
        coh_low > coh_high,
        "Lower damping must extend coherence: low {:.2} vs high {:.2}",
        coh_low, coh_high
    );

    let leak_low = solver_low.compute_non_adiabatic_braiding_leakage();
    let leak_high = solver_high.compute_non_adiabatic_braiding_leakage();
    assert!(
        leak_low < leak_high,
        "Lower damping must reduce non-adiabatic leakage: low {:.4e} vs high {:.4e}",
        leak_low, leak_high
    );
}
