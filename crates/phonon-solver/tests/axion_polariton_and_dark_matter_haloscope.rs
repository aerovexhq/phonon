//! Integration tests for quantum axion electrodynamics,
//! axion-polariton dispersion, Witten effect anomalous Hall conductance, and dark-matter haloscopes.

use phonon_models::axion_electrodynamics::{
    constants::QUANTUM_CONDUCTANCE, AxionElectrodynamicsParams,
};
use phonon_solver::axion_electrodynamics::AxionPolaritonSolver;

#[test]
fn test_axion_resonance_and_witten_conductance() {
    let params = AxionElectrodynamicsParams {
        axion_mass_uev: 20.0,
        theta_angle_rad: std::f64::consts::PI,
        ..Default::default()
    };

    let solver = AxionPolaritonSolver::new(params);
    let freq_ghz = solver.compute_resonance_frequency_ghz();
    let bwidth_khz = solver.compute_virial_bandwidth_khz();
    let hall_cond = solver.compute_witten_hall_conductance_siemens();
    let charge_frac = solver.compute_witten_monopole_charge_fraction();

    // 20 ueV corresponds to ~ 4.836 GHz
    assert!(
        (freq_ghz - 4.836).abs() < 0.05,
        "Expected resonance frequency ~ 4.836 GHz, got {:.4} GHz",
        freq_ghz
    );
    assert!(
        (bwidth_khz - 4.836).abs() < 0.05,
        "Expected virial bandwidth ~ 4.836 kHz, got {:.4} kHz",
        bwidth_khz
    );

    // Half-quantized Witten effect conductance for theta = pi: sigma_xy = 0.5 * G_0
    let expected_cond = 0.5 * QUANTUM_CONDUCTANCE;
    assert!(
        (hall_cond - expected_cond).abs() < 1.0e-9,
        "Expected half-quantized Hall conductance {:.6e} S, got {:.6e} S",
        expected_cond,
        hall_cond
    );
    assert!(
        (charge_frac - 0.5).abs() < 1.0e-6,
        "Expected fractional charge q/e = 0.5, got {:.4}",
        charge_frac
    );

    // For theta = 2*pi, fully quantized
    let params_2pi = AxionElectrodynamicsParams {
        theta_angle_rad: 2.0 * std::f64::consts::PI,
        ..params
    };
    let solver_2pi = AxionPolaritonSolver::new(params_2pi);
    let hall_cond_2pi = solver_2pi.compute_witten_hall_conductance_siemens();
    assert!(
        (hall_cond_2pi - QUANTUM_CONDUCTANCE).abs() < 1.0e-9,
        "Expected quantized Hall conductance G_0, got {:.6e}",
        hall_cond_2pi
    );
}

#[test]
fn test_dark_matter_haloscope_conversion_and_snr() {
    let params = AxionElectrodynamicsParams::default();
    let solver = AxionPolaritonSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.conversion_power_watts > 1.0e-20,
        "Converted power must be positive and non-negligible, got {:.3e} W",
        metrics.conversion_power_watts
    );
    assert!(
        metrics.snr_db >= 15.0,
        "Haloscope SNR must be >= 15.0 dB, got {:.2} dB",
        metrics.snr_db
    );
    assert!(
        metrics.polariton_gap_ghz >= 0.5,
        "Axion-polariton anti-crossing gap must be >= 0.5 GHz, got {:.2} GHz",
        metrics.polariton_gap_ghz
    );

    // Dispersion branch anti-crossing
    let (upper, lower) = solver.evaluate_polariton_branches(100.0);
    assert!(
        upper > lower,
        "Upper branch ({:.3} GHz) must exceed lower branch ({:.3} GHz)",
        upper,
        lower
    );
    assert!(
        upper - lower >= metrics.polariton_gap_ghz * 0.99,
        "Branch splitting ({:.3} GHz) must be at least the anti-crossing gap ({:.3} GHz)",
        upper - lower,
        metrics.polariton_gap_ghz
    );
}
