//! Integration Tests for Cavity Magnon Polaritons & Strong Coupling.

use approx::assert_relative_eq;
use phonon_models::cavity_spintronics::{
    MagnonCavityCoupling, MicrowaveCavityParams, YigGeometry, YigMaterial,
};
use phonon_solver::cavity_spintronics::PolaritonSolver;
use std::f64::consts::PI;

#[test]
fn test_kittel_resonance_and_field_inversion() {
    let magnon = YigMaterial {
        saturation_magnetization: 140_000.0,
        gyromagnetic_ratio: 1.760_859_630_23e11,
        intrinsic_damping: 3.0e-5,
        geometry: YigGeometry::Sphere,
    };

    let target_f = 10.0e9; // 10 GHz
    let b0 = magnon.resonant_field(target_f);
    let f_calc = magnon.kittel_frequency_hz(b0);

    assert_relative_eq!(f_calc, target_f, epsilon = 1.0); // within 1 Hz
    assert!(b0 > 0.35 && b0 < 0.36); // ~0.3568 T

    // Dissipation rate and linewidth
    let gamma_rad = magnon.magnon_dissipation_rate_rad(b0);
    let gamma_hz = magnon.magnon_linewidth_hz(b0);
    assert_relative_eq!(gamma_rad / (2.0 * PI), gamma_hz, epsilon = 1e-6);
    // Linewidth should be 2 * alpha_0 * 10 GHz = 600 kHz
    assert_relative_eq!(gamma_hz, 600_000.0, epsilon = 100.0);
}

#[test]
fn test_microwave_cavity_rates() {
    let cavity = MicrowaveCavityParams {
        resonant_frequency_hz: 10.0e9,
        quality_factor: 5000.0,
        external_coupling_ratio: 0.5,
    };

    let kappa_tot_hz = cavity.total_decay_rate_hz();
    assert_relative_eq!(kappa_tot_hz, 2.0e6, epsilon = 1.0); // 2 MHz
    assert_relative_eq!(
        cavity.external_decay_rate_rad(),
        0.5 * cavity.total_decay_rate_rad(),
        epsilon = 1.0
    );
}

#[test]
fn test_strong_coupling_cooperativity_and_anticrossing() {
    let coupling = MagnonCavityCoupling::default();

    // Check cooperativity
    let c_mp = coupling.cooperativity();
    assert!(c_mp > 100.0, "Cooperativity should be > 100, got {}", c_mp);
    assert!(c_mp > 1000.0, "Cooperativity should be ~1333, got {}", c_mp);
    assert!(coupling.is_strong_coupling());

    // Check anti-crossing gap at zero detuning
    let gap_hz = coupling.anticrossing_gap_hz();
    assert!(
        gap_hz > 70.0e6 && gap_hz < 85.0e6,
        "Gap should be ~80 MHz, got {} Hz",
        gap_hz
    );

    // Eigenvalues
    let (f_plus, f_minus) = coupling.polariton_frequencies_hz();
    let splitting = (f_plus - f_minus).abs();
    assert_relative_eq!(splitting, gap_hz, epsilon = 1.0e3);

    // Linewidths
    let (lw_plus, lw_minus) = coupling.polariton_linewidths_hz();
    // At zero detuning, polariton linewidth is (kappa_c + gamma_m) / 2 = (2.0 + 0.6) / 2 = 1.3 MHz
    assert_relative_eq!(lw_plus, 1.3e6, epsilon = 5.0e4);
    assert_relative_eq!(lw_minus, 1.3e6, epsilon = 5.0e4);
}

#[test]
fn test_polariton_transmission_spectrum_and_branch_scan() {
    let coupling = MagnonCavityCoupling::default();
    let solver = PolaritonSolver::new();

    // Frequency sweep around 10 GHz +/- 100 MHz
    let n_pts = 401;
    let f_min = 9.9e9;
    let f_max = 10.1e9;
    let freqs: Vec<f64> = (0..n_pts)
        .map(|i| f_min + (f_max - f_min) * (i as f64) / ((n_pts - 1) as f64))
        .collect();

    let spectrum = solver.compute_transmission(&coupling, &freqs);
    assert_eq!(spectrum.frequencies_hz.len(), n_pts);

    let peaks = spectrum.peak_frequencies_hz();
    assert_eq!(
        peaks.len(),
        2,
        "Should resolve 2 polariton peaks, found {:?}",
        peaks
    );
    assert!(peaks[0] < 10.0e9);
    assert!(peaks[1] > 10.0e9);
    let peak_splitting = peaks[1] - peaks[0];
    assert!(
        (peak_splitting - 80.0e6).abs() < 5.0e6,
        "Peak splitting should be close to 80 MHz, got {} MHz",
        peak_splitting / 1e6
    );

    // Field scan across resonance
    let b0_res = coupling.bias_field;
    let fields: Vec<f64> = (0..21)
        .map(|i| b0_res * (0.95 + 0.1 * (i as f64) / 20.0))
        .collect();

    let branches = solver.scan_branches(&coupling, &fields);
    assert_eq!(branches.len(), 21);

    let min_split = PolaritonSolver::extract_minimum_splitting(&branches);
    assert!(min_split > 75.0e6 && min_split < 85.0e6);
}
