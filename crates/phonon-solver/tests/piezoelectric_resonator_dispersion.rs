//! Integration tests for 3D elastodynamics, piezoelectric tensor coupling,
//! Surface Acoustic Wave (SAW) and Bulk Acoustic Wave (BAW/FBAR) nanoresonators.

use phonon_models::phononic::{
    BawResonator, ElectricField, PiezoelectricMaterial, SawResonator, VoigtStrain,
};

#[test]
fn test_piezoelectric_constitutive_relations_aln() {
    let aln = PiezoelectricMaterial::aln();
    assert_eq!(aln.name, "AlN");
    assert!((aln.density - 3260.0).abs() < 1e-6);

    // Stiffened longitudinal velocity in AlN should exceed 10,000 m/s (hypersonic regime)
    let v_long = aln.longitudinal_velocity();
    let v_stiff = aln.stiffened_longitudinal_velocity();
    assert!(v_long > 10_000.0, "v_long = {} m/s", v_long);
    assert!(
        v_stiff >= v_long,
        "v_stiff ({}) should exceed v_long ({})",
        v_stiff,
        v_long
    );

    // Shear velocity ~ sqrt(116e9 / 3260) ~ 5965 m/s
    let v_shear = aln.shear_velocity();
    assert!(
        v_shear > 5500.0 && v_shear < 6500.0,
        "v_shear = {} m/s",
        v_shear
    );

    // Rayleigh wave velocity in AlN
    let v_rayleigh = aln.rayleigh_saw_velocity();
    assert!(
        v_rayleigh > 5000.0 && v_rayleigh < v_shear,
        "v_rayleigh = {} m/s",
        v_rayleigh
    );

    // Test stress computation under pure longitudinal strain S3 = 1e-4
    let mut strain = VoigtStrain::ZERO;
    strain.s[2] = 1.0e-4; // 100 microstrain
    let ef_zero = ElectricField::ZERO;
    let stress = aln.compute_stress(&strain, &ef_zero);

    // T3 = c33 * S3 = 373e9 * 1e-4 = 37.3 MPa
    assert!(
        (stress.t[2] - 37.3e6).abs() < 1e3,
        "T3 = {} Pa",
        stress.t[2]
    );

    // Test piezoelectric displacement D3 = e33 * S3
    let disp = aln.compute_displacement(&strain, &ef_zero);
    // D3 = 1.55 * 1e-4 = 1.55e-4 C/m^2
    assert!(
        (disp.d[2] - 1.55e-4).abs() < 1e-8,
        "D3 = {} C/m^2",
        disp.d[2]
    );
}

#[test]
fn test_linbo3_high_electromechanical_coupling() {
    let linbo3 = PiezoelectricMaterial::linbo3_128_yx();
    let kt2 = linbo3.electromechanical_coupling_kt2();
    // LiNbO3 exhibits strong electromechanical coupling (kt2 > 0.03)
    assert!(kt2 > 0.03, "kt2 = {}", kt2);

    let vr = linbo3.rayleigh_saw_velocity();
    // SAW velocity on 128 Y-X LiNbO3 is around 3400 - 3800 m/s
    assert!(vr > 3000.0 && vr < 4200.0, "vr = {} m/s", vr);
}

#[test]
fn test_hypersonic_saw_resonator_and_mbvd() {
    let aln = PiezoelectricMaterial::aln();
    // Nanoscale SAW with wavelength lambda = 500 nm (0.5 um)
    let wavelength = 500.0e-9;
    let aperture = 20.0e-6; // 20 um aperture
    let finger_pairs = 50;
    let quality_factor = 2000.0;

    let saw = SawResonator::new(aln, wavelength, aperture, finger_pairs, quality_factor);
    let f0 = saw.resonant_frequency();

    // f0 = v_R / lambda ~ 5600 / 500e-9 ~ 11.2 GHz (hypersonic GHz band)
    assert!(f0 > 8.0e9 && f0 < 15.0e9, "f0 = {} GHz", f0 / 1e9);

    let mbvd = saw.extract_mbvd();
    assert!(mbvd.f_s > 8.0e9);
    assert!(mbvd.f_p >= mbvd.f_s);
    assert!(mbvd.c_0 > 0.0);
    assert!(mbvd.c_m > 0.0);
    assert!(mbvd.l_m > 0.0);
    assert!(mbvd.r_m > 0.0);

    // Verify resonance dip in S11 magnitude at series resonant frequency
    let s11_at_fs = mbvd.s11_magnitude(mbvd.f_s, 50.0);
    let s11_off_resonance = mbvd.s11_magnitude(mbvd.f_s * 0.9, 50.0);
    assert!(
        s11_at_fs < s11_off_resonance,
        "S11 at fs ({}) should be lower than off-resonance ({})",
        s11_at_fs,
        s11_off_resonance
    );
}

#[test]
fn test_hypersonic_baw_fbar_resonator_and_mbvd() {
    let aln = PiezoelectricMaterial::aln();
    // Thin film thickness d = 250 nm (0.25 um), area = 50 um x 50 um
    let thickness = 250.0e-9;
    let area = 2500.0e-12; // 2500 um^2
    let quality_factor = 3500.0;

    let baw = BawResonator::new(aln, thickness, area, quality_factor);
    let fs = baw.series_frequency();
    let fp = baw.parallel_frequency();

    // fs = v / (2d) ~ 10700 / (2 * 250e-9) ~ 21.4 GHz (hypersonic K-band)
    assert!(fs > 18.0e9 && fs < 25.0e9, "fs = {} GHz", fs / 1e9);
    assert!(fp > fs, "fp ({}) must be greater than fs ({})", fp, fs);

    let mbvd = baw.extract_mbvd();
    assert_eq!(mbvd.f_s, fs);
    assert_eq!(mbvd.f_p, fp);
    assert!(mbvd.k_eff_sq > 0.05, "k_eff_sq = {}", mbvd.k_eff_sq);

    // Impedance at fs should be predominantly real and low (~ R_s + R_m)
    let (r_fs, x_fs) = mbvd.impedance(fs);
    assert!(r_fs > 0.0 && r_fs < 50.0, "r_fs = {} Ohms", r_fs);
    assert!(x_fs.abs() < 25.0, "x_fs = {} Ohms", x_fs);
}
