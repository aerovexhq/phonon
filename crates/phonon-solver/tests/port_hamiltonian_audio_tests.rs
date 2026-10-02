#![deny(unsafe_code)]

//! Analytical Integration & Multi-Physics Validation Test Suite for the Phonon
//! Port-Hamiltonian Audio-Acoustic Multi-Physics Engine & Symplectic MNA Stamp Library.

use phonon_models::port_hamiltonian::PortHamiltonianAcousticParams;
use phonon_solver::port_hamiltonian::{
    DiracInterconnection, HiranoVocalFold, PortHamiltonianAudioEngine, PortHamiltonianMnaStamp,
    RiccatiWebsterHorn,
};

/// 1. Verifies skew-symmetry of the Dirac interconnection matrix J = -J^T and x^T * J * x = 0.
#[test]
fn test_dirac_interconnection_skew_symmetry() {
    let mut dirac = DiracInterconnection::new(4, 1);
    let j_data = vec![
        0.0, 1.5, -2.0, 0.5,
        -1.5, 0.0, 3.2, -1.1,
        2.0, -3.2, 0.0, 0.8,
        -0.5, 1.1, -0.8, 0.0,
    ];
    dirac.set_j(&j_data);

    assert!(dirac.is_skew_symmetric(1e-12));

    // Test across several orthogonal and non-zero vectors
    let test_vectors = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![1.0, 2.0, -3.0, 4.0],
        vec![-2.5, 0.8, 1.4, -3.1],
        vec![10.0, -5.0, 2.0, 7.5],
    ];

    for v in &test_vectors {
        let quad = dirac.quadratic_form_j(v);
        assert!(
            quad.abs() < 1e-12,
            "Quadratic form x^T * J * x must vanish identically, got {}",
            quad
        );
    }
}

/// 2. Verifies positive semi-definiteness of the dissipation matrix R >= 0.
#[test]
fn test_dissipation_positive_semidefinite() {
    let mut dirac = DiracInterconnection::new(4, 1);
    // Symmetric positive semi-definite matrix: R = M^T * M
    let r_data = vec![
        2.5, 0.5, 0.0, 0.0,
        0.5, 1.8, 0.2, 0.0,
        0.0, 0.2, 3.0, 0.1,
        0.0, 0.0, 0.1, 1.2,
    ];
    dirac.set_r(&r_data);

    assert!(dirac.is_dissipation_positive_semidefinite(1e-12));

    let test_vectors = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.0, 0.0],
        vec![0.0, 0.0, 1.0, 0.0],
        vec![0.0, 0.0, 0.0, 1.0],
        vec![1.5, -2.0, 0.5, 3.2],
        vec![-1.0, 1.0, -1.0, 1.0],
        vec![10.0, 20.0, -5.0, -8.0],
    ];

    for v in &test_vectors {
        let quad = dirac.quadratic_form_r(v);
        assert!(
            quad >= -1e-12,
            "Dissipation quadratic form x^T * R * x must be non-negative, got {}",
            quad
        );
    }
}

/// 3. Verifies symplectic conservation (dH/dt = 0 when R = 0) and monotonic dissipation (dH/dt <= 0 when R > 0).
#[test]
fn test_hamiltonian_energy_passivity_conservation() {
    let mut dirac = DiracInterconnection::new(2, 1);
    // Canonical harmonic oscillator: dq/dt = p/m, dp/dt = -k*q
    let j_data = vec![
        0.0, 1.0,
        -1.0, 0.0,
    ];
    dirac.set_j(&j_data);

    // Q matrix with stiffness k = 200.0 N/m and mass m = 0.001 kg (Q_22 = 1/m = 1000.0)
    let q_data = vec![
        200.0, 0.0,
        0.0, 1000.0,
    ];
    dirac.set_q(&q_data);

    let dt = 1.0 / 48000.0;
    let u_zero = vec![0.0];

    // Case A: Zero dissipation (R = 0), unforced
    let mut x_state = vec![0.001, 0.0]; // Initial position 1 mm, zero momentum
    let h0 = dirac.hamiltonian(&x_state);
    assert!(h0 > 0.0);

    for _ in 0..1000 {
        x_state = dirac.discrete_step_midpoint(&x_state, &u_zero, dt);
    }

    let h_final_conservative = dirac.hamiltonian(&x_state);
    let drift_fraction = (h_final_conservative - h0).abs() / h0;
    assert!(
        drift_fraction < 1e-6,
        "Symplectic midpoint rule must conserve Hamiltonian energy within 1e-6, drift was {}",
        drift_fraction
    );

    // Case B: Positive dissipation (R > 0), unforced
    let r_data = vec![
        0.0, 0.0,
        0.0, 0.05, // Viscous damping on momentum coordinate
    ];
    dirac.set_r(&r_data);

    let mut x_damped = vec![0.001, 0.0];
    let mut prev_energy = dirac.hamiltonian(&x_damped);

    for _ in 0..1000 {
        x_damped = dirac.discrete_step_midpoint(&x_damped, &u_zero, dt);
        let curr_energy = dirac.hamiltonian(&x_damped);
        assert!(
            curr_energy <= prev_energy + 1e-12,
            "Energy must monotonically decrease under dissipation: curr={}, prev={}",
            curr_energy,
            prev_energy
        );
        prev_energy = curr_energy;
    }

    assert!(prev_energy < h0);
}

/// 4. Verifies vertical bottom-up mucosal traveling wave delay tau_m = T_h / c_m approx 2.8 ms.
#[test]
fn test_hirano_mucosal_wave_phase_delay() {
    let params = PortHamiltonianAcousticParams::default();
    assert_eq!(params.vocal_fold_thickness_m, 0.003);
    assert_eq!(params.mucosal_wave_velocity_m_s, 1.07);

    let vf = HiranoVocalFold::new(&params);
    let tau_m = vf.mucosal_wave_delay_s();

    // 0.003 m / 1.07 m/s = 0.002803738 s = 2.8037 ms
    let tau_ms = tau_m * 1000.0;
    assert!(
        (tau_ms - 2.80).abs() < 0.05,
        "Mucosal wave delay must be approximately 2.8 ms, got {} ms",
        tau_ms
    );
}

/// 5. Verifies sustained self-sustained periodic limit cycle oscillation with F0 in [100.0, 250.0] Hz
/// and open quotient in [0.4, 0.7] under constant subglottal pressure P_sub = 1000 Pa.
#[test]
fn test_vocal_fold_sustained_self_oscillation() {
    let params = PortHamiltonianAcousticParams::default();
    let mut engine = PortHamiltonianAudioEngine::new(params);

    // Phonate for 0.25 seconds under constant 1000 Pa lung drive
    let metrics = engine.evaluate_metrics(0.25);

    assert!(
        metrics.fundamental_frequency_hz >= 100.0 && metrics.fundamental_frequency_hz <= 250.0,
        "Extracted F0 must be in [100.0, 250.0] Hz, got {} Hz",
        metrics.fundamental_frequency_hz
    );

    assert!(
        metrics.open_quotient >= 0.4 && metrics.open_quotient <= 0.7,
        "Open quotient must be in [0.4, 0.7], got {}",
        metrics.open_quotient
    );

    assert!(metrics.is_physically_passive);
    assert!(metrics.peak_lip_pressure_pa > 0.0);
}

/// 6. Verifies that von Karman-Pohlhausen dynamic flow detachment prevents unphysical negative pressures in divergent glottis.
#[test]
fn test_von_karman_pohlhausen_flow_separation() {
    let params = PortHamiltonianAcousticParams::default();
    let mut vf = HiranoVocalFold::new(&params);

    // Configure divergent glottal shape: lower margin x1 < upper margin x2
    vf.x1 = 0.0001; // narrower entrance
    vf.delay_buffer.fill(0.0006); // upper margin is delayed at 0.0006 (wider exit)
    vf.volume_velocity = 0.0001; // active flow

    let p_sub = 1000.0;
    let p_supra = 0.0;

    // Execute step to update separation point
    let dt = 1.0 / 48000.0;
    vf.step(dt, p_sub, p_supra);

    // Separation must occur upstream of the exit (x_s < T_h)
    assert!(
        vf.separation_point < vf.thickness,
        "Separation point {} must be less than thickness {}",
        vf.separation_point,
        vf.thickness
    );

    // Test pressure profile across depths: downstream of separation, pressure must not be negative
    let n_points = 20;
    for i in 0..=n_points {
        let z = (i as f64) * vf.thickness / (n_points as f64);
        let p_z = vf.pressure_at_depth(z, p_sub, p_supra);

        if z >= vf.separation_point {
            assert!(
                p_z >= p_supra - 1e-6,
                "Downstream of separation, pressure {} must be bounded by supraglottal pressure {}",
                p_z,
                p_supra
            );
        }
        assert!(
            p_z >= -1e-6,
            "Intraglottal pressure must not exhibit unphysical negative suction spikes, got {}",
            p_z
        );
    }
}

/// 7. Verifies that continuous Riccati Webster-horn resonances match quarter-wave theory for length 17.5 cm (F1 approx 500 Hz, F2 approx 1500 Hz).
#[test]
fn test_continuous_riccati_webster_horn_resonance() {
    let params = PortHamiltonianAcousticParams::default();
    let horn = RiccatiWebsterHorn::new(&params);
    assert_eq!(horn.length, 0.175);

    // Quarter wave theoretical resonances: F_n = (2n - 1) * c / (4 * L)
    // For c = 350.0 m/s and L = 0.175 m:
    // F1 = 350 / (4 * 0.175) = 500.0 Hz
    // F2 = 3 * 350 / (4 * 0.175) = 1500.0 Hz

    let mut freqs = Vec::new();
    let mut magnitudes = Vec::new();

    // Frequency sweep from 100 Hz to 2200 Hz with 5 Hz resolution
    let mut f = 100.0;
    while f <= 2200.0 {
        let z_in = horn.solve_riccati_input_impedance(f);
        freqs.push(f);
        magnitudes.push(z_in.norm());
        f += 5.0;
    }

    // Find local maxima
    let mut peak_freqs = Vec::new();
    for i in 1..(magnitudes.len() - 1) {
        if magnitudes[i] > magnitudes[i - 1] && magnitudes[i] > magnitudes[i + 1] {
            peak_freqs.push(freqs[i]);
        }
    }

    assert!(
        peak_freqs.len() >= 2,
        "Must identify at least two quarter-wave resonant formants, found {:?}",
        peak_freqs
    );

    let f1 = peak_freqs[0];
    let f2 = peak_freqs[1];

    assert!(
        (f1 - 500.0).abs() <= 35.0,
        "First formant F1 must be approximately 500 Hz, got {} Hz",
        f1
    );

    assert!(
        (f2 - 1500.0).abs() <= 65.0,
        "Second formant F2 must be approximately 1500 Hz, got {} Hz",
        f2
    );
}

/// 8. Verifies that real acoustic radiation resistance scales with omega^2, yielding +6 dB/octave differentiation.
#[test]
fn test_lip_radiation_impedance_high_frequency_boost() {
    let params = PortHamiltonianAcousticParams::default();
    let horn = RiccatiWebsterHorn::new(&params);

    let omega1 = 2.0 * std::f64::consts::PI * 1000.0; // 1 kHz
    let omega2 = 2.0 * std::f64::consts::PI * 2000.0; // 2 kHz (1 octave above)

    let r_rad1 = horn.lip_radiation_resistance(omega1);
    let r_rad2 = horn.lip_radiation_resistance(omega2);

    let power_ratio = r_rad2 / r_rad1;
    assert!(
        (power_ratio - 4.0).abs() < 1e-9,
        "Radiation resistance must scale as omega^2 (ratio 4.0 for 1 octave), got {}",
        power_ratio
    );

    let boost_db = 10.0 * power_ratio.log10();
    assert!(
        (boost_db - 6.0206).abs() < 1e-3,
        "High-frequency differentiation boost must be +6.02 dB/octave, got {} dB",
        boost_db
    );
}

/// 9. Verifies that the bilinear companion MNA stamp solution matches direct Port-Hamiltonian implicit midpoint integration.
#[test]
fn test_symplectic_bilinear_mna_stamp_equivalence() {
    let mut dirac = DiracInterconnection::new(4, 1);
    let j_data = vec![
        0.0, 2.0, -1.0, 0.5,
        -2.0, 0.0, 1.5, -0.8,
        1.0, -1.5, 0.0, 0.3,
        -0.5, 0.8, -0.3, 0.0,
    ];
    let r_data = vec![
        0.1, 0.0, 0.0, 0.0,
        0.0, 0.2, 0.0, 0.0,
        0.0, 0.0, 0.05, 0.0,
        0.0, 0.0, 0.0, 0.15,
    ];
    let q_data = vec![
        100.0, 0.0, 0.0, 0.0,
        0.0, 50.0, 0.0, 0.0,
        0.0, 0.0, 200.0, 0.0,
        0.0, 0.0, 0.0, 80.0,
    ];
    let b_data = vec![1.0, 0.0, 0.5, 0.0];

    dirac.set_j(&j_data);
    dirac.set_r(&r_data);
    dirac.set_q(&q_data);
    dirac.set_b(&b_data);

    let dt = 1.0 / 48000.0;
    let mut stamp = PortHamiltonianMnaStamp::new(&dirac, dt);

    let x0 = vec![0.005, -0.002, 0.001, 0.003];
    let u_mid = vec![1.5];

    // Method 1: Direct Port-Hamiltonian implicit midpoint integration
    let x_midpoint = dirac.discrete_step_midpoint(&x0, &u_mid, dt);

    // Method 2: Companion MNA stamp solution
    stamp.update_history(&dirac, &x0, &u_mid);
    let x_stamp = stamp.solve();

    assert_eq!(x_midpoint.len(), x_stamp.len());
    for i in 0..x_midpoint.len() {
        let diff = (x_midpoint[i] - x_stamp[i]).abs();
        assert!(
            diff < 1e-12,
            "MNA stamp solution must match direct midpoint step at index {}, diff = {}",
            i,
            diff
        );
    }
}

/// 10. Verifies audio engine 48 kHz synthesis real-time throughput (> 1,000,000 samples/sec, > 20x real-time).
#[test]
fn test_audio_engine_48khz_realtime_throughput() {
    let params = PortHamiltonianAcousticParams::default();
    let mut engine = PortHamiltonianAudioEngine::new(params);

    let n_samples = 48_000;
    let t_start = std::time::Instant::now();
    let samples = engine.synthesize_samples(n_samples);
    let elapsed = t_start.elapsed().as_secs_f64();

    let throughput = (n_samples as f64) / elapsed;

    assert_eq!(samples.len(), n_samples);
    for s in &samples {
        assert!(!s.is_nan());
        assert!(!s.is_infinite());
    }

    assert!(
        throughput > 1_000_000.0,
        "Audio engine throughput must exceed 1,000,000 samples/sec, achieved {} samples/sec ({:.1}x real-time)",
        throughput,
        throughput / 48000.0
    );
}

/// 11. Verifies numerical stability and passivity under extreme plosive closure and release transients.
#[test]
fn test_port_hamiltonian_extreme_plosive_transient_stability() {
    let params = PortHamiltonianAcousticParams::default();
    let mut engine = PortHamiltonianAudioEngine::new(params);

    // Elevate respiratory drive
    engine.set_subglottal_pressure(2500.0);

    // Phase 1: Complete tract occlusion (bilabial stop closure /p/)
    engine.set_occlusion_factor(0.0);
    let closure_samples = engine.synthesize_samples(2400); // 50 ms occlusion

    for s in &closure_samples {
        assert!(!s.is_nan());
        assert!(!s.is_infinite());
    }

    // Phase 2: Instantaneous abrupt occlusion release (plosive burst release)
    engine.set_occlusion_factor(1.0);
    let release_samples = engine.synthesize_samples(4800); // 100 ms release

    for s in &release_samples {
        assert!(!s.is_nan());
        assert!(!s.is_infinite());
        assert!(
            s.abs() < 1e5,
            "Acoustic pressure must remain bounded during plosive transient, got {}",
            s
        );
    }
}
