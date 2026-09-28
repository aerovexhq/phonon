//! Integration tests for Parity-Time (PT) symmetry and exceptional point degeneracies.

use phonon_models::non_hermitian::{PtDimerParams, PtPhaseRegime};

#[test]
fn test_pt_symmetric_exact_phase() {
    // gamma = 10 GHz, kappa = 20 GHz (Exact phase)
    let dimer = PtDimerParams::new(193.4e12, 20.0e9, 10.0e9);
    assert_eq!(dimer.phase_regime(), PtPhaseRegime::Exact);

    let ((w1_re, w1_im), (w2_re, w2_im)) = dimer.eigenfrequencies_hz();
    assert_eq!(w1_im, 0.0, "Im(w1) must be 0 in exact PT phase");
    assert_eq!(w2_im, 0.0, "Im(w2) must be 0 in exact PT phase");
    assert!(
        w1_re > w2_re,
        "Eigenvalues should split symmetrically around w0"
    );

    let split = w1_re - dimer.bare_frequency_hz;
    let expected = (20.0e9_f64.powi(2) - 10.0e9_f64.powi(2)).sqrt();
    assert!((split - expected).abs() < 1e3);

    let k_petermann = dimer.petermann_factor();
    assert!(k_petermann > 1.0 && k_petermann < 2.0);
}

#[test]
fn test_exceptional_point_coalescence() {
    // gamma = kappa = 20 GHz (Exceptional Point)
    let ep = PtDimerParams::standard_exceptional_point_dimer();
    assert_eq!(ep.phase_regime(), PtPhaseRegime::ExceptionalPoint);

    let ((w1_re, w1_im), (w2_re, w2_im)) = ep.eigenfrequencies_hz();
    assert_eq!(w1_im, 0.0);
    assert_eq!(w2_im, 0.0);
    assert!(
        (w1_re - w2_re).abs() < 1e-3,
        "Eigenvalues must coalesce at EP"
    );
    assert_eq!(w1_re, ep.bare_frequency_hz);

    let k_petermann = ep.petermann_factor();
    assert!(
        k_petermann > 1000.0,
        "Petermann factor must diverge at EP, got {}",
        k_petermann
    );
}

#[test]
fn test_pt_broken_phase() {
    // gamma = 30 GHz, kappa = 20 GHz (Broken phase)
    let broken = PtDimerParams::new(193.4e12, 20.0e9, 30.0e9);
    assert_eq!(broken.phase_regime(), PtPhaseRegime::Broken);

    let ((w1_re, w1_im), (w2_re, w2_im)) = broken.eigenfrequencies_hz();
    assert_eq!(w1_re, broken.bare_frequency_hz);
    assert_eq!(w2_re, broken.bare_frequency_hz);

    assert!(
        w1_im > 0.0,
        "Amplified mode must have positive imaginary part"
    );
    assert!(
        w2_im < 0.0,
        "Decaying mode must have negative imaginary part"
    );
    assert!(
        (w1_im + w2_im).abs() < 1e-6,
        "Im(w1) and Im(w2) must be opposite"
    );
}
