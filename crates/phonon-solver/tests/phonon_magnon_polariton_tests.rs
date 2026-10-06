#![deny(unsafe_code)]

//! Integration Tests for Phonon-Magnon Polariton Transducer & Quantum Microwave-to-Acoustic Interface.

use std::f64::consts::PI;
use phonon_solver::phonon_magnon_polariton::{
    MagnetoelasticDriveEngine, MagnetoelasticDriveParams, PhononMagnonParams,
    PolaritonDispersionEngine, QuantumTransducerSolver, TransducerCouplingParams,
};

#[test]
fn test_avoided_crossing_gap_and_hopfield_normalization() {
    let params = PhononMagnonParams::default();
    let engine = PolaritonDispersionEngine::new(params.clone());

    let k_res = engine.find_resonance_wavenumber();
    assert!(k_res > 1.0e5 && k_res < 1.0e7, "Resonance wavenumber out of expected physical range: {}", k_res);

    let res_point = engine.evaluate_point(k_res);

    // Bare frequencies must be nearly equal at resonance
    let detuning = (res_point.omega_phonon - res_point.omega_magnon).abs();
    assert!(detuning < 1.0e4, "Detuning at resonance must be negligible, got {}", detuning);

    // Avoided crossing gap: Delta omega = 2 * g
    let splitting = res_point.omega_upper_polariton - res_point.omega_lower_polariton;
    let expected_gap = 2.0 * params.coupling_g_rad_s;
    let gap_error = (splitting - expected_gap).abs() / expected_gap;
    assert!(gap_error < 1.0e-4, "Splitting must match 2*g within 0.01%, error: {}", gap_error);

    // Hopfield fractions sum to 1.000 everywhere
    let hopfield_sum = res_point.hopfield_phonon_fraction + res_point.hopfield_magnon_fraction;
    assert!((hopfield_sum - 1.0).abs() < 1.0e-6, "Hopfield fractions must sum to 1.0, got {}", hopfield_sum);

    // At exact resonance, equal 50/50 phonon-magnon hybridization
    assert!(
        (res_point.hopfield_phonon_fraction - 0.5).abs() < 1.0e-3,
        "Phonon Hopfield fraction must be 0.5 at resonance, got {}",
        res_point.hopfield_phonon_fraction
    );
    assert!(
        (res_point.hopfield_magnon_fraction - 0.5).abs() < 1.0e-3,
        "Magnon Hopfield fraction must be 0.5 at resonance, got {}",
        res_point.hopfield_magnon_fraction
    );
}

#[test]
fn test_strong_coupling_cooperativity() {
    let params = PhononMagnonParams::default();
    let engine = PolaritonDispersionEngine::new(params.clone());

    let k_res = engine.find_resonance_wavenumber();
    let res_point = engine.evaluate_point(k_res);

    // Strong coupling regime requires cooperativity C >> 1
    assert!(
        res_point.cooperativity > 50.0,
        "Cooperativity must exceed 50 for coherent state transduction, got {}",
        res_point.cooperativity
    );
}

#[test]
fn test_dispersion_curve_generation_and_asymptotics() {
    let params = PhononMagnonParams::default();
    let engine = PolaritonDispersionEngine::new(params);

    let k_res = engine.find_resonance_wavenumber();
    let k_min = 0.5 * k_res;
    let k_max = 1.5 * k_res;

    let points = engine.generate_dispersion_curve(k_min, k_max, 50);
    assert_eq!(points.len(), 50);

    for pt in &points {
        assert!(pt.omega_upper_polariton >= pt.omega_lower_polariton);
        let hopfield_sum = pt.hopfield_phonon_fraction + pt.hopfield_magnon_fraction;
        assert!((hopfield_sum - 1.0).abs() < 1.0e-6);
        assert!(pt.splitting_mhz() >= 69.0, "Polariton splitting must be at least 2*g ~ 70 MHz");
    }

    // Far below resonance: lower polariton is mostly phonon, upper is mostly magnon
    let low_pt = engine.evaluate_point(0.2 * k_res);
    assert!(low_pt.hopfield_phonon_fraction > 0.85);

    // Far above resonance: lower polariton is mostly magnon, upper is mostly phonon
    let high_pt = engine.evaluate_point(3.0 * k_res);
    assert!(high_pt.hopfield_magnon_fraction > 0.85);
}

#[test]
fn test_transducer_scattering_matrix_unitarity_and_efficiency() {
    let params = TransducerCouplingParams::default();
    let solver = QuantumTransducerSolver::new(params.clone());

    let f0 = params.resonance_freq_rad_s / (2.0 * PI);
    let s_res = solver.evaluate_s_parameters(f0);

    // Physical power conservation
    let total_port1_power = s_res.s11_power + s_res.s21_power;
    assert!(
        total_port1_power <= 1.000_001,
        "Scattering matrix must conserve or dissipate power, got {}",
        total_port1_power
    );
    assert!(s_res.s22_power <= 1.000_001);

    let g_hz = params.coupling_g_rad_s / (2.0 * PI);
    let s_polariton = solver.evaluate_s_parameters(f0 + g_hz);
    let total_polariton_power = s_polariton.s11_power + s_polariton.s21_power;
    assert!(
        total_polariton_power <= 1.000_001,
        "Polariton scattering must conserve or dissipate power, got {}",
        total_polariton_power
    );

    // High peak transduction efficiency
    let peak_eta = solver.compute_peak_efficiency();
    assert!(
        peak_eta >= 0.60,
        "Peak microwave-to-acoustic conversion efficiency must be >= 60%, got {}",
        peak_eta
    );

    // Quantum state fidelity F = sqrt(eta)
    let fidelity = solver.compute_quantum_fidelity();
    assert!(
        fidelity >= 0.77,
        "Quantum state transfer fidelity must be >= 0.77, got {}",
        fidelity
    );

    // 3-dB bandwidth must be non-zero and physically reasonable (MHz order)
    let bw = solver.compute_bandwidth_hz();
    assert!(
        bw >= 1.0e6 && bw <= 1.0e8,
        "Transducer 3-dB bandwidth must be between 1 MHz and 100 MHz, got {} Hz",
        bw
    );
}

#[test]
fn test_cryogenic_quantum_noise_quanta() {
    let params = TransducerCouplingParams::default();
    let solver = QuantumTransducerSolver::new(params.clone());

    let f0 = params.resonance_freq_rad_s / (2.0 * PI);
    let g_hz = params.coupling_g_rad_s / (2.0 * PI);
    let n_add = solver.compute_added_noise_quanta(f0 + g_hz);

    // At 20 mK and 3.5 GHz, hbar * omega >> k_B * T, thermal occupancy n_th << 1
    // Added noise quanta n_add should be around 0.1 to 1.5 quanta
    assert!(
        n_add >= 0.0 && n_add < 2.0,
        "Added noise quanta at 20 mK must be near quantum limit, got {}",
        n_add
    );
}

#[test]
fn test_dynamic_magnetoelastic_drive_linearity_and_attenuation() {
    let params = MagnetoelasticDriveParams::default();
    let engine = MagnetoelasticDriveEngine::new(params.clone());

    // Linearity of effective RF field with strain
    let h1 = engine.compute_effective_rf_field_tesla(1.0e-5);
    let h2 = engine.compute_effective_rf_field_tesla(2.0e-5);
    assert!(
        (h2 - 2.0 * h1).abs() < 1.0e-12,
        "RF magnetoelastic field must scale strictly linearly with strain"
    );
    assert!(h1.abs() > 1.0e-5, "RF field must be non-zero");

    // Resonant attenuation peak
    let f_res = 3.5e9;
    let linewidth = 10.0e6;
    let delta_alpha = 15.0; // dB/cm peak absorption
    let alpha_peak = engine.compute_resonant_attenuation_db_cm(f_res, f_res, linewidth, delta_alpha);
    let alpha_off = engine.compute_resonant_attenuation_db_cm(f_res + 200.0e6, f_res, linewidth, delta_alpha);

    assert!(
        alpha_peak > alpha_off + 14.0,
        "Resonant acoustic attenuation must exhibit sharp absorption peak, peak={}, off={}",
        alpha_peak,
        alpha_off
    );

    // Track snapshot generation
    let snapshot = engine.generate_track_snapshot(5.0, 1.2, 0.5e-6, 100);
    assert_eq!(snapshot.x_positions_mm.len(), 100);
    assert_eq!(snapshot.strain_values.len(), 100);
    assert_eq!(snapshot.dynamic_magnetization.len(), 100);
    assert_eq!(snapshot.effective_field_mt.len(), 100);

    // Wavepacket envelope should have non-zero strain peak
    let max_strain = snapshot.strain_values.iter().fold(0.0f64, |acc, &x| acc.max(x.abs()));
    assert!(max_strain > 1.0, "Wavepacket peak microstrain must be non-zero, got {}", max_strain);
}
