#![deny(unsafe_code)]

//! Integration tests for Phase 429: Phonon Studio Topological Acoustic Floquet Chiral
//! Magnon-Phonon Entanglement Router & Continuous-Variable Quantum Key Distribution (CV-QKD) Engine.

use phonon_solver::floquet_cv_qkd::{
    ChiralPolaritonRouter, CvQkdEngine, CvQkdParams, FloquetCvQkdProcessor,
    FloquetRouterParams,
};

#[test]
fn test_chiral_polariton_router_telemetry_and_isolation() {
    let router = ChiralPolaritonRouter::new(FloquetRouterParams::default());
    let tele = router.evaluate_telemetry();

    assert!(
        tele.chiral_isolation_db >= 30.0,
        "Chiral isolation must be >= 30.0 dB (got {:.2} dB)",
        tele.chiral_isolation_db
    );
    assert!(
        tele.forward_transmittance >= 0.90,
        "Forward transmittance must be >= 90% (got {:.1}%)",
        tele.forward_transmittance * 100.0
    );
    assert!(
        tele.forward_transmission_db >= -0.50,
        "Insertion loss must be <= 0.50 dB (got {:.2} dB)",
        tele.forward_transmission_db
    );
    assert!(
        tele.backward_transmission_db <= -30.0,
        "Backward transmission must be <= -30.0 dB (got {:.2} dB)",
        tele.backward_transmission_db
    );
    assert!(
        tele.nonreciprocal_phase_deg >= 45.0,
        "Non-reciprocal phase shift must be >= 45 deg (got {:.1} deg)",
        tele.nonreciprocal_phase_deg
    );
    assert!(
        tele.port_directivity_db >= 25.0,
        "Port directivity must be >= 25.0 dB (got {:.2} dB)",
        tele.port_directivity_db
    );
}

#[test]
fn test_polariton_spectrum_and_routing_matrix() {
    let router = ChiralPolaritonRouter::new(FloquetRouterParams::default());
    let spectrum = router.generate_spectrum();

    assert_eq!(spectrum.len(), 80);
    // Find resonant peak point
    let center_point = spectrum
        .iter()
        .min_by(|a, b| {
            (a.freq_ghz - 2.4)
                .abs()
                .partial_cmp(&(b.freq_ghz - 2.4).abs())
                .unwrap()
        })
        .unwrap();

    assert!(center_point.isolation_db >= 30.0);
    assert!(center_point.s21_db >= -0.50);

    // Verify 4x4 scattering power matrix
    let matrix = router.evaluate_routing_matrix();
    let forward_leak = matrix[0][3];
    let backward_leak = matrix[2][0];
    assert!(forward_leak >= 0.90, "Alice In -> Bob Out must be >= 0.90");
    assert!(backward_leak <= 0.001, "Bob In -> Alice In must be strongly isolated (<= 0.001)");
}

#[test]
fn test_gaussian_covariance_matrix_and_duan_entanglement() {
    let engine = CvQkdEngine::new(CvQkdParams::default());
    let cov = engine.evaluate_covariance_matrix();

    assert!(
        cov.squeezing_db >= 8.0,
        "Squeezing level must be >= 8.0 dB (got {:.2} dB)",
        cov.squeezing_db
    );
    assert!(
        cov.symplectic_eigenvalue < 1.0,
        "Symplectic eigenvalue nu must be strictly < 1.0 for EPR entanglement (got {:.4})",
        cov.symplectic_eigenvalue
    );
    assert!(
        cov.duan_witness < 2.0,
        "Duan inseparability witness must be < 2.0 (got {:.4})",
        cov.duan_witness
    );

    // Verify matrix symmetries
    assert_eq!(cov.m[0][0], cov.m[1][1]);
    assert_eq!(cov.m[2][2], cov.m[3][3]);
    assert_eq!(cov.m[0][2], cov.m[2][0]);
    assert_eq!(cov.m[1][3], cov.m[3][1]);
    assert_eq!(cov.m[0][2], -cov.m[1][3]); // Squeezed cross-correlations
}

#[test]
fn test_cv_qkd_telemetry_and_secret_key_rate() {
    let engine = CvQkdEngine::new(CvQkdParams::default());
    let tele = engine.evaluate_telemetry();

    assert!(
        tele.secret_key_rate_mbps >= 5.0,
        "Secret key rate must be >= 5.0 Mbps (got {:.2} Mbps)",
        tele.secret_key_rate_mbps
    );
    assert!(
        tele.mutual_information_bits_pulse > tele.holevo_bound_bits_pulse,
        "Mutual info I_AB must exceed Holevo bound chi_BE (I_AB={:.3}, chi_BE={:.3})",
        tele.mutual_information_bits_pulse,
        tele.holevo_bound_bits_pulse
    );
    assert!(
        tele.channel_transmittance >= 0.70,
        "Channel transmittance must be >= 70% at 10 m (got {:.1}%)",
        tele.channel_transmittance * 100.0
    );

    // Key rate vs distance curve
    let curve = engine.generate_key_rate_vs_distance();
    assert_eq!(curve.len(), 50);
    assert!(curve[0].secret_key_rate_mbps > curve[49].secret_key_rate_mbps);

    // Wigner function slice
    let wigner_slice = engine.generate_wigner_slice();
    assert_eq!(wigner_slice.len(), 80);
    let peak_wigner = wigner_slice
        .iter()
        .max_by(|a, b| a.wigner_density.partial_cmp(&b.wigner_density).unwrap())
        .unwrap();
    assert!(peak_wigner.x.abs() < 0.2); // Centered at zero
}

#[test]
fn test_10_point_physics_audit_checklist_pass() {
    let processor = FloquetCvQkdProcessor::default();
    let audit = processor.audit_processor();

    for item in &audit.items {
        assert!(
            item.passed,
            "Audit item '{}' failed: measured {}, criterion {}",
            item.name, item.measured_value, item.target_criterion
        );
    }

    assert_eq!(audit.total_score, 10, "Audit score must be 10/10");
    assert!(audit.all_passed, "Audit all_passed must be true");
}
