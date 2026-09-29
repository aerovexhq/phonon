#![deny(unsafe_code)]

//! Integration Tests for Non-Local Hydrodynamic Drude SPP Dispersion & Feibelman Shifts.

use approx::assert_relative_eq;
use phonon_models::quantum_plasmonics::{NobleMetal, SppHydrodynamicModel, SPEED_OF_LIGHT};

#[test]
fn test_noble_metal_drude_permittivity_and_hydrodynamic_velocity() {
    let metals = [
        NobleMetal::Silver,
        NobleMetal::Gold,
        NobleMetal::Aluminum,
        NobleMetal::Copper,
    ];

    for &metal in &metals {
        let wp = metal.plasma_frequency();
        assert!(wp > 1.0e16, "Plasma frequency {} rad/s is too low", wp);

        let vf = metal.fermi_velocity();
        assert!(
            (1.0e6..=3.0e6).contains(&vf),
            "Fermi velocity {} m/s out of range",
            vf
        );

        let beta = metal.hydrodynamic_velocity();
        let expected_beta = (0.6f64).sqrt() * vf;
        assert_relative_eq!(beta, expected_beta, epsilon = 1e-6);

        // At optical frequencies (omega ~ wp / 4, near infrared/visible)
        let omega = wp * 0.25;
        let (eps_re, eps_im) = metal.dielectric_permittivity(omega);
        assert!(
            eps_re < 0.0,
            "Real permittivity should be negative (metallic), got {}",
            eps_re
        );
        assert!(
            eps_im > 0.0,
            "Imaginary permittivity must be positive (dissipative), got {}",
            eps_im
        );
    }
}

#[test]
fn test_hydrodynamic_spp_quantum_blueshift_and_feibelman_shift() {
    let eps_d = 2.25; // Silica substrate
    let gap_m = 5.0e-9; // 5 nm gap
    let d_perp = 0.2e-9; // 0.2 nm Feibelman parameter

    let model = SppHydrodynamicModel::new(NobleMetal::Silver, eps_d, gap_m, d_perp);

    let w_local = model.local_surface_plasmon_resonance();
    assert!(w_local > 1e15 && w_local < 1e16);

    let w_nl = model.non_local_surface_plasmon_resonance();
    assert!(
        w_nl > w_local,
        "Quantum electron pressure must blueshift the SPP resonance: w_nl={}, w_local={}",
        w_nl,
        w_local
    );

    let blueshift = model.resonance_blueshift();
    assert!(blueshift > 0.0);
    let blueshift_ghz = blueshift / (2.0 * std::f64::consts::PI * 1e9);
    assert!(
        blueshift_ghz >= 1.0,
        "Expected significant non-local blueshift (>= 1 GHz) at 5 nm, got {} GHz",
        blueshift_ghz
    );

    // Feibelman shift should counteract the blueshift
    let shift_feibelman = model.feibelman_centroid_frequency_shift();
    assert!(shift_feibelman < 0.0, "Feibelman shift must be negative");

    // Propagation length at 800 nm wavelength
    let lambda0 = 800.0e-9;
    let omega = (2.0 * std::f64::consts::PI * SPEED_OF_LIGHT) / lambda0;
    let l_prop = model.propagation_length(omega);
    assert!(
        l_prop > 1.0e-6,
        "SPP propagation length should exceed 1 um, got {} m",
        l_prop
    );

    let n_eff = model.effective_index(omega);
    assert!(
        n_eff > 1.0,
        "Effective mode index must exceed 1.0, got {}",
        n_eff
    );
}
