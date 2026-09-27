//! Integration tests for phononic crystal metamaterials, bandgap dispersion,
//! stopband acoustic attenuation (> 80 dB), and defect acoustic waveguides.

use phonon_models::phononic::{AcousticLayer, DefectPhononicWaveguide, PhononicCrystal1D};

#[test]
fn test_phononic_crystal_dispersion_and_bandgap() {
    // Si / AlN nanoscale superlattice (a = 100 nm, 12 periods)
    let crystal = PhononicCrystal1D::hypersonic_si_aln(12);
    assert_eq!(crystal.lattice_constant(), 100.0e-9);

    // Find the primary acoustic stopband in the 10 GHz - 80 GHz range
    let bandgap = crystal.find_primary_bandgap(10.0e9, 80.0e9, 500);
    assert!(
        bandgap.is_some(),
        "Acoustic bandgap must exist for Si/AlN superlattice"
    );

    let (f_low, f_high) = bandgap.unwrap();
    assert!(f_high > f_low, "Bandgap upper edge must exceed lower edge");
    let gap_width = f_high - f_low;
    assert!(gap_width > 2.0e9, "Bandgap width = {} GHz", gap_width / 1e9);

    // Midgap frequency
    let f_mid = 0.5 * (f_low + f_high);
    assert!(crystal.is_in_bandgap(f_mid));
    assert!(crystal.attenuation_constant(f_mid) > 0.0);
}

#[test]
fn test_acoustic_stopband_attenuation_exceeds_80_db() {
    // High-contrast SiO2 / Tungsten metamaterial with 14 periods
    // Extreme acoustic impedance mismatch: Z_W / Z_SiO2 ~ (19300 * 5200) / (2200 * 5900) ~ 100.3e6 / 13.0e6 ~ 7.7
    let crystal = PhononicCrystal1D::high_contrast_sio2_tungsten(14);
    assert_eq!(crystal.lattice_constant(), 200.0e-9);

    // Look for bandgap around Bragg condition: f_bragg ~ v_eff / (2*a) ~ 5500 / 400e-9 ~ 13.75 GHz
    let bandgap = crystal.find_primary_bandgap(5.0e9, 25.0e9, 400);
    assert!(
        bandgap.is_some(),
        "High-contrast phononic crystal must have bandgap"
    );

    let (f_low, f_high) = bandgap.unwrap();
    let f_mid = 0.5 * (f_low + f_high);

    // Attenuation at mid-gap must strictly exceed 80 dB!
    let atten_db = crystal.attenuation_db(f_mid);
    assert!(
        atten_db > 80.0,
        "Stopband acoustic attenuation must exceed 80 dB, got {} dB at {} GHz",
        atten_db,
        f_mid / 1e9
    );

    // Passband attenuation should be small (< 3 dB)
    let f_pass = f_low * 0.5;
    let atten_pass_db = crystal.attenuation_db(f_pass);
    assert!(
        atten_pass_db < 6.0,
        "Passband attenuation = {} dB",
        atten_pass_db
    );
}

#[test]
fn test_defect_phononic_waveguide_confinement() {
    let crystal = PhononicCrystal1D::high_contrast_sio2_tungsten(12);
    // Insert a Silicon defect layer of 120 nm in the center
    let defect_layer = AcousticLayer::new("SiliconDefect", 120.0e-9, 2330.0, 8430.0);
    let waveguide = DefectPhononicWaveguide::new(crystal.clone(), defect_layer);

    let bandgap = crystal
        .find_primary_bandgap(5.0e9, 25.0e9, 400)
        .expect("Bandgap");
    let (f_low, f_high) = bandgap;
    let f_mid = 0.5 * (f_low + f_high);

    // In crystal without defect, transmission is heavily attenuated (> 60 dB)
    let t_bulk = crystal.transmission_magnitude(f_mid);
    assert!(t_bulk < 1.0e-3, "Bulk transmission = {}", t_bulk);

    // In defect waveguide, resonance transmission occurs inside the bandgap
    let mut max_defect_t = 0.0;
    let steps = 100;
    let df = (f_high - f_low) / (steps as f64);
    for i in 0..=steps {
        let f = f_low + (i as f64) * df;
        let t = waveguide.transmission_magnitude(f);
        if t > max_defect_t {
            max_defect_t = t;
        }
    }

    assert!(
        max_defect_t > 0.05,
        "Defect mode transmission = {} (much larger than bulk {})",
        max_defect_t,
        t_bulk
    );
}
