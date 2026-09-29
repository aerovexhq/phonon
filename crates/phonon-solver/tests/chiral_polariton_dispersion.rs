//! Integration tests for Chiral Phonon-Magnon Polariton Dispersion & Selection Rules.

use phonon_models::chiral_polariton::{AcousticPolarizationChirality, MagnetoElasticMedium};
use phonon_solver::chiral_polariton::PolaritonEigensolver;

#[test]
fn test_magneto_elastic_sound_velocity_and_resonance() {
    let medium = MagnetoElasticMedium::default();
    assert_eq!(medium.mass_density_kg_per_m3, 5170.0);

    let vt = medium.transverse_sound_velocity_m_per_s();
    assert!(
        vt > 3800.0 && vt < 3900.0,
        "Transverse sound velocity should be ~3844 m/s, got {:.1}",
        vt
    );

    let wm = medium.kittel_magnon_frequency_rad_per_s();
    assert!(
        wm > 1.5e10 && wm < 2.0e10,
        "Kittel frequency at 0.1 T should be ~1.76e10 rad/s, got {:e}",
        wm
    );

    let k_res = medium.resonance_wavevector_per_m();
    assert!(
        k_res > 4.0e6 && k_res < 5.0e6,
        "Resonance crossing wavevector should be ~4.58e6 m^-1, got {:e}",
        k_res
    );

    // Anti-crossing splitting
    let split_rad = medium.polariton_splitting_rad_per_s(k_res);
    let split_mhz = (split_rad / (2.0 * std::f64::consts::PI)) / 1e6;
    assert!(
        (10.0..=100.0).contains(&split_mhz),
        "Splitting should be 10-100 MHz, got {:.2} MHz",
        split_mhz
    );
}

#[test]
fn test_chiral_polariton_selection_rule_and_hybridization() {
    let medium = MagnetoElasticMedium::default();
    let solver = PolaritonEigensolver::new();

    let k_res = medium.resonance_wavevector_per_m();
    let pt = solver.solve_at_wavevector(&medium, k_res);

    // Right-handed coupling splits the modes
    assert!(pt.splitting_rad > 0.0);
    assert!(pt.upper_branch_freq_rad > pt.bare_acoustic_freq_rad);
    assert!(pt.lower_branch_freq_rad < pt.bare_acoustic_freq_rad);

    // Left-handed mode remains completely uncoupled: frequency equals bare acoustic frequency
    assert!((pt.uncoupled_left_handed_freq_rad - pt.bare_acoustic_freq_rad).abs() < 1e-6);

    // Hybridization at exact crossing: 50% magnon, 50% phonon
    assert!(
        (pt.magnon_fraction_upper - 0.50).abs() < 0.05,
        "Magnon fraction should be ~50%, got {:.3}",
        pt.magnon_fraction_upper
    );
    assert!(
        (pt.phonon_fraction_upper - 0.50).abs() < 0.05,
        "Phonon fraction should be ~50%, got {:.3}",
        pt.phonon_fraction_upper
    );
    assert!((pt.magnon_fraction_upper + pt.phonon_fraction_upper - 1.0).abs() < 1e-6);
}

#[test]
fn test_acoustic_angular_momentum_conservation() {
    let medium = MagnetoElasticMedium::default();
    let u0 = 1.0e-11; // 10 pm displacement amplitude
    let w = medium.kittel_magnon_frequency_rad_per_s();

    let j_rh = medium.acoustic_angular_momentum_density(
        u0,
        w,
        AcousticPolarizationChirality::RightHandedCircular,
    );
    let j_lh = medium.acoustic_angular_momentum_density(
        u0,
        w,
        AcousticPolarizationChirality::LeftHandedCircular,
    );
    let j_lin =
        medium.acoustic_angular_momentum_density(u0, w, AcousticPolarizationChirality::Linear);

    assert!(
        j_rh > 0.0,
        "RH circular phonon must carry positive angular momentum"
    );
    assert!(
        j_lh < 0.0,
        "LH circular phonon must carry negative angular momentum"
    );
    assert!(
        (j_rh + j_lh).abs() < 1e-25,
        "RH and LH angular momenta must be opposite"
    );
    assert_eq!(
        j_lin, 0.0,
        "Linear phonon carries zero net angular momentum"
    );
}
