#![deny(unsafe_code)]

use phonon_models::superconducting_spintronics::{
    CooperPairSymmetry, SuperconductingSpintronicJunction,
};
use std::f64::consts::PI;

#[test]
fn test_spin_triplet_generation_and_penetration() {
    // Collinear alignment (alpha = 0): no long-range triplet generated
    let junc_collinear = SuperconductingSpintronicJunction::new(10.0e-6, 0.0, 0.0, 0.1, 10.0);
    assert_eq!(junc_collinear.triplet_fraction, 0.0);

    // Non-collinear alignment (alpha = pi/2): maximal triplet generation
    let junc_orthogonal = SuperconductingSpintronicJunction::new(10.0e-6, PI / 2.0, 0.0, 0.1, 10.0);
    assert!(
        junc_orthogonal.triplet_fraction > 0.6,
        "Triplet fraction should peak at perpendicular alignment, got {}",
        junc_orthogonal.triplet_fraction
    );

    // Triplet penetration depth at 4K vs 100mK with diffusion constant D = 0.001 m^2/s
    let d_diff = 0.001;
    let xi_4k = junc_orthogonal.triplet_penetration_depth_nm(d_diff, 4.0);
    let xi_100mk = junc_orthogonal.triplet_penetration_depth_nm(d_diff, 0.1);

    // Low temperature increases normal/triplet coherence length
    assert!(xi_100mk > xi_4k);
    assert!(xi_100mk > 50.0, "Long range triplet penetrates > 50 nm");
}

#[test]
fn test_phi0_junction_anomalous_ground_state_and_supercurrent() {
    let phi0 = PI / 3.0; // 60 degrees anomalous shift
    let junc = SuperconductingSpintronicJunction::new(
        20.0e-6, // 20 uA
        PI / 4.0,
        phi0,
        0.25,
        15.0,
    );

    // In a phi_0 junction, the ground state phase is shifted from 0
    let ground_phase = junc.ground_state_phase();
    assert!(ground_phase.abs() > 0.05);

    // Anomalous supercurrent at zero phase difference I_s(phi=0) != 0
    let is_zero = junc.supercurrent_at_phase(0.0);
    assert!(
        is_zero.abs() > 1.0e-7,
        "Anomalous Josephson supercurrent should be non-zero at phi=0, got {}",
        is_zero
    );

    // Supercurrent at phase = pi/2
    let is_half_pi = junc.supercurrent_at_phase(PI / 2.0);
    assert!(is_half_pi.abs() > 0.0);
}

#[test]
fn test_supercurrent_spin_orbit_torque() {
    let junc = SuperconductingSpintronicJunction::new(50.0e-6, PI / 4.0, PI / 6.0, 0.4, 5.0);

    // Spin torque at phi = pi/2 (near maximum supercurrent)
    let torque_max = junc.supercurrent_spin_torque(PI / 2.0);
    // Spin torque at phi = 0
    let torque_zero = junc.supercurrent_spin_torque(0.0);

    assert!(torque_max > 0.0);
    assert!(torque_zero > 0.0);
    assert!(
        torque_max > torque_zero,
        "Torque should be larger near critical current than at zero phase"
    );
}

#[test]
fn test_cooper_pair_symmetry_variants() {
    let singlet = CooperPairSymmetry::Singlet;
    let triplet_lr = CooperPairSymmetry::LongRangeTriplet;
    let triplet_sr = CooperPairSymmetry::ShortRangeTriplet;

    assert_ne!(singlet, triplet_lr);
    assert_ne!(triplet_lr, triplet_sr);
}
