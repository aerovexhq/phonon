#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for cavity quantum
//! acoustomagnonic polariton condensation and chiral superfluid spin-phonon lasers.

use phonon_models::acoustomagnonic_polariton_laser::AcoustomagnonicPolaritonLaserParams;
use phonon_solver::acoustomagnonic_polariton_laser::AcoustomagnonicPolaritonLaserSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = AcoustomagnonicPolaritonLaserParams::new(
        1.0,  // below 2.0 GHz
        1.5,  // below 2.0 GHz
        3.0,  // below 5.0 MHz
        0.5,  // below 1.0 uW
        0.2,  // below 0.5 MHz
        5.0,  // below 10.0 kHz
        0.5,  // below 1.0 mK
        0.5,  // below 1.0 Hz
    );
    assert_eq!(underflow.magnon_kittel_frequency_ghz, 2.0);
    assert_eq!(underflow.acoustic_resonator_frequency_ghz, 2.0);
    assert_eq!(underflow.magnon_phonon_coupling_mhz, 5.0);
    assert_eq!(underflow.optical_microwave_pump_power_uw, 1.0);
    assert_eq!(underflow.magnon_damping_rate_mhz, 0.5);
    assert_eq!(underflow.acoustic_decay_rate_khz, 10.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.non_linear_kerr_coefficient_hz, 1.0);

    // Test values strictly above physical maximum bounds
    let overflow = AcoustomagnonicPolaritonLaserParams::new(
        22.0,  // above 18.0 GHz
        25.0,  // above 18.0 GHz
        150.0, // above 100.0 MHz
        120.0, // above 100.0 uW
        20.0,  // above 15.0 MHz
        600.0, // above 500.0 kHz
        75.0,  // above 50.0 mK
        150.0, // above 100.0 Hz
    );
    assert_eq!(overflow.magnon_kittel_frequency_ghz, 18.0);
    assert_eq!(overflow.acoustic_resonator_frequency_ghz, 18.0);
    assert_eq!(overflow.magnon_phonon_coupling_mhz, 100.0);
    assert_eq!(overflow.optical_microwave_pump_power_uw, 100.0);
    assert_eq!(overflow.magnon_damping_rate_mhz, 15.0);
    assert_eq!(overflow.acoustic_decay_rate_khz, 500.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.non_linear_kerr_coefficient_hz, 100.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = AcoustomagnonicPolaritonLaserParams::default();
    let solver = AcoustomagnonicPolaritonLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.polariton_condensation_threshold_uw <= 15.0,
        "Default condensation threshold must be <= 15.0 uW, got {:.4} uW",
        metrics.polariton_condensation_threshold_uw
    );
    assert!(
        metrics.condensate_coherence_lifetime_us >= 120.0,
        "Default coherence lifetime must be >= 120.0 us, got {:.2} us",
        metrics.condensate_coherence_lifetime_us
    );
    assert!(
        metrics.side_mode_suppression_ratio_db >= 45.0,
        "Default SMSR must be >= 45.0 dB, got {:.2} dB",
        metrics.side_mode_suppression_ratio_db
    );
    assert!(
        metrics.linewidth_narrowing_factor >= 80.0,
        "Default linewidth narrowing factor must be >= 80.0x, got {:.2}x",
        metrics.linewidth_narrowing_factor
    );
    assert!(
        metrics.polariton_superfluid_fraction >= 0.850,
        "Default polariton superfluid fraction must be >= 0.850, got {:.6}",
        metrics.polariton_superfluid_fraction
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameters must be physically compliant"
    );
}

#[test]
fn test_magnon_damping_scaling() {
    let base = AcoustomagnonicPolaritonLaserParams::default();
    let solver_base = AcoustomagnonicPolaritonLaserSolver::new(base);

    let high_damping = AcoustomagnonicPolaritonLaserParams::new(
        base.magnon_kittel_frequency_ghz,
        base.acoustic_resonator_frequency_ghz,
        base.magnon_phonon_coupling_mhz,
        base.optical_microwave_pump_power_uw,
        5.0, // increased from 2.2 MHz
        base.acoustic_decay_rate_khz,
        base.cryogenic_temperature_mk,
        base.non_linear_kerr_coefficient_hz,
    );
    let solver_high = AcoustomagnonicPolaritonLaserSolver::new(high_damping);

    assert!(
        solver_high.compute_polariton_condensation_threshold_uw()
            > solver_base.compute_polariton_condensation_threshold_uw(),
        "Higher magnon damping must increase condensation threshold"
    );
    assert!(
        solver_high.compute_condensate_coherence_lifetime_us()
            < solver_base.compute_condensate_coherence_lifetime_us(),
        "Higher magnon damping must reduce condensate coherence lifetime"
    );
    assert!(
        solver_high.compute_linewidth_narrowing_factor()
            < solver_base.compute_linewidth_narrowing_factor(),
        "Higher magnon damping must reduce linewidth narrowing factor"
    );
}

#[test]
fn test_acoustic_decay_scaling() {
    let base = AcoustomagnonicPolaritonLaserParams::default();
    let solver_base = AcoustomagnonicPolaritonLaserSolver::new(base);

    let high_decay = AcoustomagnonicPolaritonLaserParams::new(
        base.magnon_kittel_frequency_ghz,
        base.acoustic_resonator_frequency_ghz,
        base.magnon_phonon_coupling_mhz,
        base.optical_microwave_pump_power_uw,
        base.magnon_damping_rate_mhz,
        180.0, // increased from 85.0 kHz
        base.cryogenic_temperature_mk,
        base.non_linear_kerr_coefficient_hz,
    );
    let solver_high = AcoustomagnonicPolaritonLaserSolver::new(high_decay);

    assert!(
        solver_high.compute_polariton_condensation_threshold_uw()
            > solver_base.compute_polariton_condensation_threshold_uw(),
        "Higher acoustic decay rate must increase condensation threshold"
    );
    assert!(
        solver_high.compute_condensate_coherence_lifetime_us()
            < solver_base.compute_condensate_coherence_lifetime_us(),
        "Higher acoustic decay rate must reduce condensate coherence lifetime"
    );
}

#[test]
fn test_coupling_rate_scaling() {
    let base = AcoustomagnonicPolaritonLaserParams::default();
    let solver_base = AcoustomagnonicPolaritonLaserSolver::new(base);

    let high_coupling = AcoustomagnonicPolaritonLaserParams::new(
        base.magnon_kittel_frequency_ghz,
        base.acoustic_resonator_frequency_ghz,
        65.0, // increased from 38.0 MHz
        base.optical_microwave_pump_power_uw,
        base.magnon_damping_rate_mhz,
        base.acoustic_decay_rate_khz,
        base.cryogenic_temperature_mk,
        base.non_linear_kerr_coefficient_hz,
    );
    let solver_high = AcoustomagnonicPolaritonLaserSolver::new(high_coupling);

    assert!(
        solver_high.compute_polariton_condensation_threshold_uw()
            < solver_base.compute_polariton_condensation_threshold_uw(),
        "Stronger magnon-phonon coupling must lower condensation threshold"
    );
    assert!(
        solver_high.compute_condensate_coherence_lifetime_us()
            > solver_base.compute_condensate_coherence_lifetime_us(),
        "Stronger magnon-phonon coupling must enhance condensate coherence lifetime"
    );
    assert!(
        solver_high.compute_linewidth_narrowing_factor()
            > solver_base.compute_linewidth_narrowing_factor(),
        "Stronger coupling must enhance linewidth narrowing factor"
    );
    assert!(
        solver_high.compute_polariton_superfluid_fraction()
            > solver_base.compute_polariton_superfluid_fraction(),
        "Stronger coupling must increase polariton superfluid fraction"
    );
}

#[test]
fn test_pump_power_scaling() {
    let base = AcoustomagnonicPolaritonLaserParams::default();
    let solver_base = AcoustomagnonicPolaritonLaserSolver::new(base);

    let high_pump = AcoustomagnonicPolaritonLaserParams::new(
        base.magnon_kittel_frequency_ghz,
        base.acoustic_resonator_frequency_ghz,
        base.magnon_phonon_coupling_mhz,
        45.0, // increased from 22.0 uW
        base.magnon_damping_rate_mhz,
        base.acoustic_decay_rate_khz,
        base.cryogenic_temperature_mk,
        base.non_linear_kerr_coefficient_hz,
    );
    let solver_high = AcoustomagnonicPolaritonLaserSolver::new(high_pump);

    assert!(
        solver_high.compute_condensate_coherence_lifetime_us()
            > solver_base.compute_condensate_coherence_lifetime_us(),
        "Higher pump power must increase condensate coherence lifetime"
    );
    assert!(
        solver_high.compute_side_mode_suppression_ratio_db()
            > solver_base.compute_side_mode_suppression_ratio_db(),
        "Higher pump power must increase side-mode suppression ratio"
    );
    assert!(
        solver_high.compute_linewidth_narrowing_factor()
            > solver_base.compute_linewidth_narrowing_factor(),
        "Higher pump power must enhance linewidth narrowing factor"
    );
    assert!(
        solver_high.compute_polariton_superfluid_fraction()
            > solver_base.compute_polariton_superfluid_fraction(),
        "Higher pump power must increase polariton superfluid fraction"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let base = AcoustomagnonicPolaritonLaserParams::default();
    let solver_base = AcoustomagnonicPolaritonLaserSolver::new(base);

    let warmer = AcoustomagnonicPolaritonLaserParams::new(
        base.magnon_kittel_frequency_ghz,
        base.acoustic_resonator_frequency_ghz,
        base.magnon_phonon_coupling_mhz,
        base.optical_microwave_pump_power_uw,
        base.magnon_damping_rate_mhz,
        base.acoustic_decay_rate_khz,
        30.0, // increased from 15.0 mK
        base.non_linear_kerr_coefficient_hz,
    );
    let solver_warmer = AcoustomagnonicPolaritonLaserSolver::new(warmer);

    assert!(
        solver_warmer.compute_polariton_condensation_threshold_uw()
            > solver_base.compute_polariton_condensation_threshold_uw(),
        "Warmer temperature must increase condensation threshold"
    );
    assert!(
        solver_warmer.compute_condensate_coherence_lifetime_us()
            < solver_base.compute_condensate_coherence_lifetime_us(),
        "Warmer temperature must reduce condensate coherence lifetime"
    );
    assert!(
        solver_warmer.compute_side_mode_suppression_ratio_db()
            < solver_base.compute_side_mode_suppression_ratio_db(),
        "Warmer temperature must degrade side-mode suppression ratio"
    );
    assert!(
        solver_warmer.compute_polariton_superfluid_fraction()
            < solver_base.compute_polariton_superfluid_fraction(),
        "Warmer temperature must reduce polariton superfluid fraction"
    );
}

#[test]
fn test_kerr_nonlinearity_scaling() {
    let base = AcoustomagnonicPolaritonLaserParams::default();
    let solver_base = AcoustomagnonicPolaritonLaserSolver::new(base);

    let higher_kerr = AcoustomagnonicPolaritonLaserParams::new(
        base.magnon_kittel_frequency_ghz,
        base.acoustic_resonator_frequency_ghz,
        base.magnon_phonon_coupling_mhz,
        base.optical_microwave_pump_power_uw,
        base.magnon_damping_rate_mhz,
        base.acoustic_decay_rate_khz,
        base.cryogenic_temperature_mk,
        28.0, // increased from 12.0 Hz
    );
    let solver_high_kerr = AcoustomagnonicPolaritonLaserSolver::new(higher_kerr);

    assert!(
        solver_high_kerr.compute_polariton_condensation_threshold_uw()
            < solver_base.compute_polariton_condensation_threshold_uw(),
        "Higher Kerr non-linearity must lower condensation threshold"
    );
    assert!(
        solver_high_kerr.compute_condensate_coherence_lifetime_us()
            > solver_base.compute_condensate_coherence_lifetime_us(),
        "Higher Kerr non-linearity must increase coherence lifetime"
    );
    assert!(
        solver_high_kerr.compute_side_mode_suppression_ratio_db()
            > solver_base.compute_side_mode_suppression_ratio_db(),
        "Higher Kerr non-linearity must improve side-mode suppression ratio"
    );
}

#[test]
fn test_detuning_scaling() {
    let base = AcoustomagnonicPolaritonLaserParams::default();
    let solver_base = AcoustomagnonicPolaritonLaserSolver::new(base);

    let detuned = AcoustomagnonicPolaritonLaserParams::new(
        9.5, // detuned by 1.0 GHz from 8.5 GHz
        base.acoustic_resonator_frequency_ghz,
        base.magnon_phonon_coupling_mhz,
        base.optical_microwave_pump_power_uw,
        base.magnon_damping_rate_mhz,
        base.acoustic_decay_rate_khz,
        base.cryogenic_temperature_mk,
        base.non_linear_kerr_coefficient_hz,
    );
    let solver_detuned = AcoustomagnonicPolaritonLaserSolver::new(detuned);

    assert!(
        solver_detuned.compute_polariton_condensation_threshold_uw()
            > solver_base.compute_polariton_condensation_threshold_uw(),
        "Detuning must increase condensation threshold"
    );
    assert!(
        solver_detuned.compute_condensate_coherence_lifetime_us()
            < solver_base.compute_condensate_coherence_lifetime_us(),
        "Detuning must decrease condensate coherence lifetime"
    );
    assert!(
        solver_detuned.compute_side_mode_suppression_ratio_db()
            < solver_base.compute_side_mode_suppression_ratio_db(),
        "Detuning must decrease side-mode suppression ratio"
    );
    assert!(
        solver_detuned.compute_linewidth_narrowing_factor()
            < solver_base.compute_linewidth_narrowing_factor(),
        "Detuning must decrease linewidth narrowing factor"
    );
}
