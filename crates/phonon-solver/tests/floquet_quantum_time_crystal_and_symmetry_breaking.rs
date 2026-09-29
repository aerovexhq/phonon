//! Integration tests for Floquet quantum acoustic time crystals,
//! subharmonic symmetry breaking, and spectral rigidity.

use phonon_models::quantum_time_crystal::QuantumTimeCrystalParams;
use phonon_solver::quantum_time_crystal::FloquetTimeCrystalSolver;

#[test]
fn test_subharmonic_symmetry_breaking_and_rigidity() {
    let params = QuantumTimeCrystalParams::default();
    let solver = FloquetTimeCrystalSolver::new(params);
    let metrics = solver.solve();

    // Period doubling: n = 2
    assert_eq!(
        metrics.subharmonic_period_multiplier, 2,
        "Expected subharmonic period multiplier 2 (period doubling)"
    );

    // Stroboscopic magnetization dynamics
    let dynamics = solver.evaluate_stroboscopic_magnetization(10);
    assert_eq!(dynamics.len(), 10);
    assert!(dynamics[0] > 0.0, "Z(0) must be positive");
    assert!(dynamics[1] < 0.0, "Z(1) must be negative");
    assert!(dynamics[2] > 0.0, "Z(2) must be positive");
    assert!(dynamics[3] < 0.0, "Z(3) must be negative");

    // Spectral rigidity contrast must satisfy >= 20.0 dB
    assert!(
        metrics.spectral_rigidity_contrast_db >= 20.0,
        "Spectral rigidity contrast must be >= 20.0 dB, got {:.2} dB",
        metrics.spectral_rigidity_contrast_db
    );

    // Subharmonic peak amplitude
    assert!(
        metrics.fourier_peak_amplitude >= 0.90,
        "Fourier peak amplitude must be >= 0.90, got {:.4}",
        metrics.fourier_peak_amplitude
    );

    // Subharmonic frequency: 1000 / (2 * 8.0) = 62.5 MHz
    assert!(
        (metrics.subharmonic_frequency_mhz - 62.5).abs() < 0.1,
        "Expected subharmonic frequency ~ 62.5 MHz, got {:.2} MHz",
        metrics.subharmonic_frequency_mhz
    );
}

#[test]
fn test_quantum_acoustic_memory_and_mbl_lifetime() {
    let params = QuantumTimeCrystalParams::default();
    let solver = FloquetTimeCrystalSolver::new(params);
    let metrics = solver.solve();

    // Lifetime must exceed 1000 cycles
    assert!(
        metrics.time_crystal_lifetime_cycles >= 1000.0,
        "DTC lifetime must exceed 1000 cycles, got {:.1} cycles",
        metrics.time_crystal_lifetime_cycles
    );

    // Memory fidelity after 100 cycles must satisfy >= 90.0%
    assert!(
        metrics.subharmonic_memory_fidelity >= 0.90,
        "Subharmonic memory fidelity must be >= 90.0%, got {:.2}%",
        metrics.subharmonic_memory_fidelity * 100.0
    );

    // Stability: Allan deviation floor <= 1e-11
    assert!(
        metrics.fractional_frequency_stability <= 1.0e-11,
        "Allan deviation floor must be <= 1e-11, got {:.2e}",
        metrics.fractional_frequency_stability
    );
}
