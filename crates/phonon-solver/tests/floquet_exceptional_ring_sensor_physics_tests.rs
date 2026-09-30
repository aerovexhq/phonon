#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic
//! non-Hermitian Floquet exceptional-ring synthesizers and chiral skin sensors.

use phonon_models::floquet_exceptional_ring_sensor::FloquetExceptionalRingSensorParams;
use phonon_solver::floquet_exceptional_ring_sensor::FloquetExceptionalRingSensorSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = FloquetExceptionalRingSensorParams::new(
        1.0,  // below 5.0 MHz
        0.5,  // below 1.0 GHz
        0.05, // below 0.10
        10.0, // below 20.0 kHz
        4,    // below 8 elements
        5.0,  // below 10.0 Hz
        0.5,  // below 1.0 mK
        5.0,  // below 10.0 dB
    );
    assert_eq!(underflow.floquet_drive_amplitude_mhz, 5.0);
    assert_eq!(underflow.floquet_modulation_frequency_ghz, 1.0);
    assert_eq!(underflow.non_reciprocal_hopping_asymmetry, 0.10);
    assert_eq!(underflow.cavity_loss_contrast_khz, 20.0);
    assert_eq!(underflow.sensor_array_elements, 8);
    assert_eq!(underflow.perturbation_coupling_strength_hz, 10.0);
    assert_eq!(underflow.operating_temperature_mk, 1.0);
    assert_eq!(underflow.piezoelectric_gain_db, 10.0);

    // Test values strictly above physical maximum bounds
    let overflow = FloquetExceptionalRingSensorParams::new(
        120.0,  // above 80.0 MHz
        20.0,   // above 12.0 GHz
        1.20,   // above 0.95
        800.0,  // above 500.0 kHz
        100,    // above 64 elements
        2000.0, // above 1000.0 Hz
        100.0,  // above 50.0 mK
        60.0,   // above 45.0 dB
    );
    assert_eq!(overflow.floquet_drive_amplitude_mhz, 80.0);
    assert_eq!(overflow.floquet_modulation_frequency_ghz, 12.0);
    assert_eq!(overflow.non_reciprocal_hopping_asymmetry, 0.95);
    assert_eq!(overflow.cavity_loss_contrast_khz, 500.0);
    assert_eq!(overflow.sensor_array_elements, 64);
    assert_eq!(overflow.perturbation_coupling_strength_hz, 1000.0);
    assert_eq!(overflow.operating_temperature_mk, 50.0);
    assert_eq!(overflow.piezoelectric_gain_db, 45.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = FloquetExceptionalRingSensorParams::default();
    let solver = FloquetExceptionalRingSensorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.skin_mode_localization_ratio >= 0.940,
        "Default skin mode localization ratio must be >= 0.940, got {:.6}",
        metrics.skin_mode_localization_ratio
    );
    assert!(
        metrics.sensitivity_enhancement_factor >= 85.0,
        "Default sensitivity enhancement factor must be >= 85.0, got {:.4}",
        metrics.sensitivity_enhancement_factor
    );
    assert!(
        metrics.reverse_backscattering_suppression_db >= 52.0,
        "Default reverse backscattering suppression must be >= 52.0 dB, got {:.4} dB",
        metrics.reverse_backscattering_suppression_db
    );
    assert!(
        metrics.sensor_noise_figure_db <= 0.45,
        "Default sensor noise figure must be <= 0.45 dB, got {:.4} dB",
        metrics.sensor_noise_figure_db
    );
    assert!(
        metrics.exceptional_ring_topological_charge >= 0.990,
        "Default exceptional ring topological charge must be >= 0.990, got {:.6}",
        metrics.exceptional_ring_topological_charge
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_floquet_drive_amplitude_scaling() {
    let base = FloquetExceptionalRingSensorParams::default();
    let solver_base = FloquetExceptionalRingSensorSolver::new(base);

    let mut enhanced = base;
    enhanced.floquet_drive_amplitude_mhz = 70.0;
    let solver_enhanced = FloquetExceptionalRingSensorSolver::new(enhanced);

    let charge_base = solver_base.compute_exceptional_ring_topological_charge();
    let charge_enhanced = solver_enhanced.compute_exceptional_ring_topological_charge();

    let sensitivity_base = solver_base.compute_sensitivity_enhancement_factor();
    let sensitivity_enhanced = solver_enhanced.compute_sensitivity_enhancement_factor();

    assert!(
        charge_enhanced >= charge_base,
        "Higher Floquet drive amplitude must improve topological charge: enhanced {:.6} >= base {:.6}",
        charge_enhanced,
        charge_base
    );
    assert!(
        sensitivity_enhanced >= sensitivity_base,
        "Higher Floquet drive amplitude must improve sensitivity enhancement: enhanced {:.4} >= base {:.4}",
        sensitivity_enhanced,
        sensitivity_base
    );
}

#[test]
fn test_non_reciprocal_hopping_asymmetry_scaling() {
    let base = FloquetExceptionalRingSensorParams::default();
    let solver_base = FloquetExceptionalRingSensorSolver::new(base);

    let mut enhanced = base;
    enhanced.non_reciprocal_hopping_asymmetry = 0.88;
    let solver_enhanced = FloquetExceptionalRingSensorSolver::new(enhanced);

    let localization_base = solver_base.compute_skin_mode_localization_ratio();
    let localization_enhanced = solver_enhanced.compute_skin_mode_localization_ratio();

    let isolation_base = solver_base.compute_reverse_backscattering_suppression_db();
    let isolation_enhanced = solver_enhanced.compute_reverse_backscattering_suppression_db();

    assert!(
        localization_enhanced >= localization_base,
        "Higher non-reciprocal asymmetry must enhance skin mode localization: enhanced {:.6} >= base {:.6}",
        localization_enhanced,
        localization_base
    );
    assert!(
        isolation_enhanced >= isolation_base,
        "Higher non-reciprocal asymmetry must increase backscattering suppression: enhanced {:.4} >= base {:.4}",
        isolation_enhanced,
        isolation_base
    );
}

#[test]
fn test_cavity_loss_contrast_scaling() {
    let base = FloquetExceptionalRingSensorParams::default();
    let solver_base = FloquetExceptionalRingSensorSolver::new(base);

    let mut enhanced = base;
    enhanced.cavity_loss_contrast_khz = 380.0;
    let solver_enhanced = FloquetExceptionalRingSensorSolver::new(enhanced);

    let sensitivity_base = solver_base.compute_sensitivity_enhancement_factor();
    let sensitivity_enhanced = solver_enhanced.compute_sensitivity_enhancement_factor();

    assert!(
        sensitivity_enhanced >= sensitivity_base,
        "Higher cavity loss contrast must increase EP sensitivity enhancement: enhanced {:.4} >= base {:.4}",
        sensitivity_enhanced,
        sensitivity_base
    );
}

#[test]
fn test_sensor_array_elements_scaling() {
    let base = FloquetExceptionalRingSensorParams::default();
    let solver_base = FloquetExceptionalRingSensorSolver::new(base);

    let mut enhanced = base;
    enhanced.sensor_array_elements = 48;
    let solver_enhanced = FloquetExceptionalRingSensorSolver::new(enhanced);

    let localization_base = solver_base.compute_skin_mode_localization_ratio();
    let localization_enhanced = solver_enhanced.compute_skin_mode_localization_ratio();

    let isolation_base = solver_base.compute_reverse_backscattering_suppression_db();
    let isolation_enhanced = solver_enhanced.compute_reverse_backscattering_suppression_db();

    assert!(
        localization_enhanced >= localization_base,
        "More array elements must amplify boundary skin mode localization: enhanced {:.6} >= base {:.6}",
        localization_enhanced,
        localization_base
    );
    assert!(
        isolation_enhanced >= isolation_base,
        "More array elements must improve cumulative reverse isolation: enhanced {:.4} >= base {:.4}",
        isolation_enhanced,
        isolation_base
    );
}

#[test]
fn test_perturbation_coupling_strength_scaling() {
    let base = FloquetExceptionalRingSensorParams::default();
    let solver_base = FloquetExceptionalRingSensorSolver::new(base);

    let mut perturbed = base;
    perturbed.perturbation_coupling_strength_hz = 800.0;
    let solver_perturbed = FloquetExceptionalRingSensorSolver::new(perturbed);

    let sensitivity_base = solver_base.compute_sensitivity_enhancement_factor();
    let sensitivity_perturbed = solver_perturbed.compute_sensitivity_enhancement_factor();

    assert!(
        sensitivity_perturbed <= sensitivity_base,
        "Stronger perturbation moves further from sharp EP singularity cusp: perturbed {:.4} <= base {:.4}",
        sensitivity_perturbed,
        sensitivity_base
    );
}

#[test]
fn test_operating_temperature_scaling() {
    let base = FloquetExceptionalRingSensorParams::default();
    let solver_base = FloquetExceptionalRingSensorSolver::new(base);

    let mut warm = base;
    warm.operating_temperature_mk = 40.0;
    let solver_warm = FloquetExceptionalRingSensorSolver::new(warm);

    let noise_base = solver_base.compute_sensor_noise_figure_db();
    let noise_warm = solver_warm.compute_sensor_noise_figure_db();

    assert!(
        noise_warm >= noise_base,
        "Higher operating temperature must increase sensor noise figure: warm {:.4} >= base {:.4}",
        noise_warm,
        noise_base
    );
}

#[test]
fn test_piezoelectric_gain_scaling() {
    let base = FloquetExceptionalRingSensorParams::default();
    let solver_base = FloquetExceptionalRingSensorSolver::new(base);

    let mut high_gain = base;
    high_gain.piezoelectric_gain_db = 40.0;
    let solver_high_gain = FloquetExceptionalRingSensorSolver::new(high_gain);

    let noise_base = solver_base.compute_sensor_noise_figure_db();
    let noise_high_gain = solver_high_gain.compute_sensor_noise_figure_db();

    let sensitivity_base = solver_base.compute_sensitivity_enhancement_factor();
    let sensitivity_high_gain = solver_high_gain.compute_sensitivity_enhancement_factor();

    assert!(
        noise_high_gain <= noise_base,
        "Higher piezoelectric readout gain must reduce noise figure: high gain {:.4} <= base {:.4}",
        noise_high_gain,
        noise_base
    );
    assert!(
        sensitivity_high_gain >= sensitivity_base,
        "Higher piezoelectric readout gain must improve sensitivity: high gain {:.4} >= base {:.4}",
        sensitivity_high_gain,
        sensitivity_base
    );
}

#[test]
fn test_floquet_modulation_frequency_resonance() {
    let base = FloquetExceptionalRingSensorParams::default(); // 4.8 GHz
    let solver_base = FloquetExceptionalRingSensorSolver::new(base);

    let mut detuned = base;
    detuned.floquet_modulation_frequency_ghz = 10.5;
    let solver_detuned = FloquetExceptionalRingSensorSolver::new(detuned);

    let noise_base = solver_base.compute_sensor_noise_figure_db();
    let noise_detuned = solver_detuned.compute_sensor_noise_figure_db();

    let sensitivity_base = solver_base.compute_sensitivity_enhancement_factor();
    let sensitivity_detuned = solver_detuned.compute_sensitivity_enhancement_factor();

    assert!(
        noise_detuned >= noise_base,
        "Detuned modulation frequency must degrade noise figure: detuned {:.4} >= base {:.4}",
        noise_detuned,
        noise_base
    );
    assert!(
        sensitivity_detuned <= sensitivity_base,
        "Detuned modulation frequency must reduce sensitivity enhancement: detuned {:.4} <= base {:.4}",
        sensitivity_detuned,
        sensitivity_base
    );
}
