#![deny(unsafe_code)]

//! Test suite for Phase 342: Twisted Bilayer Moiré Phonon Polariton Superlattice Engine.
//!
//! Verifies:
//! - Moiré lattice period scaling L_M proportional to 1/theta.
//! - Atomic relaxation shrinkage of AA domain area fraction from ~33.3% to < 18.0%.
//! - Magic-angle flat band formation: bandwidth Delta_E < 1.0 meV and group velocity quenching v_g / v_0 <= 0.05 at theta ~ 1.08 deg.
//! - Giant Van Hove singularity DOS peak enhancement at magic angle.
//! - Localized acoustic soliton confinement xi < 0.5 * L_M.

use phonon_solver::twisted_moire_superlattice::{
    AtomicRelaxationField, BilayerLatticeParams, DensityOfStates, LocalizedAcousticSoliton,
    MoireBandStructure, StackingClassification,
};
use std::f64::consts::PI;

#[test]
fn test_moire_lattice_period_scaling() {
    let params_magic = BilayerLatticeParams::default();
    assert_eq!(params_magic.theta_deg, 1.08);
    let lm_magic = params_magic.moire_period();

    // L_M = a_0 / (2 * sin(theta / 2)) approx a_0 / theta_rad
    let theta_rad = 1.08_f64.to_radians();
    let approx_lm = params_magic.a_0 / theta_rad;
    assert!(
        (lm_magic - approx_lm).abs() / lm_magic < 0.01,
        "L_M should match small-angle approximation a_0 / theta: got {} vs {}",
        lm_magic,
        approx_lm
    );
    assert!(
        (lm_magic - 13.05).abs() < 0.1,
        "Magic angle L_M for graphene (a_0 ~ 0.246 nm) should be ~13.05 nm: got {}",
        lm_magic
    );

    // Test scaling with 1 / theta:
    // If angle doubles (e.g. 1.0 deg vs 2.0 deg), period must halve
    let params_1deg = BilayerLatticeParams::new(0.246, 1000.0, 0.16, 80.0, 110.0, 1.0);
    let params_2deg = BilayerLatticeParams::new(0.246, 1000.0, 0.16, 80.0, 110.0, 2.0);
    let lm_1 = params_1deg.moire_period();
    let lm_2 = params_2deg.moire_period();

    let ratio = lm_1 / lm_2;
    assert!(
        (ratio - 2.0).abs() < 0.02,
        "Period ratio for 1.0 deg vs 2.0 deg should be ~2.0: got {}",
        ratio
    );

    // Test scaling order: L_M(0.8 deg) > L_M(1.08 deg) > L_M(2.0 deg)
    let params_small = BilayerLatticeParams::new(0.246, 1000.0, 0.16, 80.0, 110.0, 0.8);
    let lm_small = params_small.moire_period();
    assert!(lm_small > lm_magic);
    assert!(lm_magic > lm_2);

    // Verify reciprocal lattice vectors G_1, G_2
    let (g1, g2) = params_magic.reciprocal_lattice_vectors();
    let g_mag = (4.0 * PI) / (3.0_f64.sqrt() * lm_magic);
    let g1_mag = (g1[0] * g1[0] + g1[1] * g1[1]).sqrt();
    let g2_mag = (g2[0] * g2[0] + g2[1] * g2[1]).sqrt();
    assert!((g1_mag - g_mag).abs() < 1e-6);
    assert!((g2_mag - g_mag).abs() < 1e-6);
}

#[test]
fn test_atomic_relaxation_shrinkage() {
    let params = BilayerLatticeParams::default();
    let field = AtomicRelaxationField::new(params);

    // 1. Unrelaxed AA domain fraction must be ~33.3%
    let unrelaxed_frac = field.aa_domain_fraction(false);
    assert!(
        (unrelaxed_frac - 1.0 / 3.0).abs() < 0.02,
        "Unrelaxed AA domain fraction should be ~33.3%: got {}",
        unrelaxed_frac
    );

    // 2. Relaxed AA domain fraction must shrink to < 18.0%
    let relaxed_frac = field.aa_domain_fraction(true);
    assert!(
        relaxed_frac < 0.18,
        "Relaxed AA domain fraction must shrink to < 18.0%: got {}",
        relaxed_frac
    );
    assert!(
        relaxed_frac < unrelaxed_frac,
        "Relaxed AA area fraction must be strictly smaller than unrelaxed"
    );

    // 3. Displacement at AA core (origin) should be zero by symmetry
    let u_origin = field.displacement_at(0.0, 0.0);
    assert!(
        u_origin[0].abs() < 1e-12 && u_origin[1].abs() < 1e-12,
        "Displacement at AA center should vanish by C3 symmetry"
    );

    // 4. Stacking classification test
    assert_eq!(
        field.stacking_at(0.0, 0.0, true),
        StackingClassification::AA,
        "Origin must be classified as AA"
    );

    let lm = params.moire_period();
    // Far from origin, region should not be AA
    let far_stacking = field.stacking_at(0.4 * lm, 0.0, true);
    assert_ne!(
        far_stacking,
        StackingClassification::AA,
        "Point at 0.4*L_M should not be AA stacking"
    );

    // 5. Dynamic strain tensor evaluation
    let strain_origin = field.strain_at(0.0, 0.0);
    assert!(
        strain_origin.trace().abs() > 0.0,
        "Hydrostatic strain trace at AA node should be non-zero"
    );
    assert!(
        strain_origin.von_mises_equivalent() >= 0.0,
        "von Mises strain must be non-negative"
    );
}

#[test]
fn test_magic_angle_flat_band_formation() {
    // 1. Test at magic angle theta = 1.08 deg
    let params_magic = BilayerLatticeParams::default();
    let band_magic = MoireBandStructure::compute(params_magic, true, 80);

    // Bandwidth must be < 1.0 meV
    assert!(
        band_magic.flat_bandwidth_mev < 1.0,
        "Flat bandwidth at magic angle must be < 1.0 meV: got {} meV",
        band_magic.flat_bandwidth_mev
    );

    // Normalized group velocity quenching v_g / v_0 <= 0.05
    assert!(
        band_magic.normalized_group_velocity <= 0.05,
        "Normalized group velocity at magic angle must be <= 0.05: got {}",
        band_magic.normalized_group_velocity
    );

    // Energy gap to excited bands should be well defined (> 5.0 meV)
    assert!(
        band_magic.isolation_gap_mev > 5.0,
        "Flat band should be isolated from dispersive branches by a gap > 5 meV: got {}",
        band_magic.isolation_gap_mev
    );

    // 2. Test away from magic angle (theta = 1.80 deg)
    let params_off = BilayerLatticeParams::new(0.246, 1000.0, 0.16, 80.0, 110.0, 1.80);
    let band_off = MoireBandStructure::compute(params_off, true, 80);

    // Bandwidth off-resonance must be much wider (> 5.0 meV)
    assert!(
        band_off.flat_bandwidth_mev > 5.0,
        "Bandwidth off-resonance at 1.80 deg must be > 5.0 meV: got {} meV",
        band_off.flat_bandwidth_mev
    );

    // Group velocity off-resonance must NOT be quenched (> 0.20)
    assert!(
        band_off.normalized_group_velocity > 0.20,
        "Group velocity off-resonance should not be quenched: got {}",
        band_off.normalized_group_velocity
    );
}

#[test]
fn test_van_hove_singularity_dos_peak_enhancement() {
    // 1. Density of states at magic angle
    let params_magic = BilayerLatticeParams::default();
    let band_magic = MoireBandStructure::compute(params_magic, true, 80);
    let dos_magic = DensityOfStates::compute(&band_magic, 20.0, 100);

    // 2. Density of states off-magic angle (theta = 2.0 deg)
    let params_off = BilayerLatticeParams::new(0.246, 1000.0, 0.16, 80.0, 110.0, 2.0);
    let band_off = MoireBandStructure::compute(params_off, true, 80);
    let dos_off = DensityOfStates::compute(&band_off, 20.0, 100);

    // Magic angle DOS should display giant Van Hove singularity peak
    assert!(
        dos_magic.peak_dos > 10.0,
        "Peak DOS at magic angle should be > 10 states/meV: got {}",
        dos_magic.peak_dos
    );

    // Enhancement ratio relative to off-magic angle should exceed 5.0
    let peak_enhancement = dos_magic.peak_dos / dos_off.peak_dos;
    assert!(
        peak_enhancement > 5.0,
        "Van Hove singularity DOS peak enhancement at magic angle should be > 5.0: got {}",
        peak_enhancement
    );

    // Peak ratio relative to background should also be substantial
    assert!(
        dos_magic.peak_ratio > 10.0,
        "Magic angle DOS peak-to-background ratio should exceed 10.0: got {}",
        dos_magic.peak_ratio
    );
}

#[test]
fn test_localized_acoustic_soliton_confinement() {
    let params = BilayerLatticeParams::default();
    let p_0 = 5.0; // 5.0 MPa peak pressure
    let soliton = LocalizedAcousticSoliton::solve(&params, p_0, 60);

    let lm = params.moire_period();

    // 1. Localization length must be < 0.5 * L_M and < 0.4 * L_M
    assert!(
        soliton.xi_nm < 0.5 * lm,
        "Soliton localization length xi must be < 0.5 * L_M: got {} nm vs 0.5 * L_M = {} nm",
        soliton.xi_nm,
        0.5 * lm
    );
    assert!(
        soliton.xi_nm < 0.4 * lm,
        "Soliton localization length xi must be < 0.4 * L_M: got {} nm vs 0.4 * L_M = {} nm",
        soliton.xi_nm,
        0.4 * lm
    );
    assert!(
        soliton.confinement_ratio < 0.4,
        "Confinement ratio xi / L_M must be < 0.4: got {}",
        soliton.confinement_ratio
    );

    // 2. Pressure profile shape test
    let p_center = soliton.pressure_at(0.0, 0.0);
    assert!(
        (p_center - p_0).abs() < 1e-6,
        "Pressure at origin should equal peak pressure p_0: got {}",
        p_center
    );

    let p_at_xi = soliton.pressure_at(soliton.xi_nm, 0.0);
    let sech_1 = 1.0 / 1.0_f64.cosh();
    assert!(
        (p_at_xi - p_0 * sech_1).abs() < 1e-4,
        "Pressure at r = xi should match p_0 * sech(1): got {} vs {}",
        p_at_xi,
        p_0 * sech_1
    );

    // Monotonic decay with distance
    let p_at_2xi = soliton.pressure_at(2.0 * soliton.xi_nm, 0.0);
    assert!(p_center > p_at_xi);
    assert!(p_at_xi > p_at_2xi);

    // Cross section must not be empty
    assert!(!soliton.cross_section.is_empty());
}
