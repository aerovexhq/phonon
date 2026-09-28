use phonon_models::phononic_topological::HoneycombAcousticLattice;
use phonon_solver::phononic_topological::{ValleyEdgeProfile, ValleyEdgeSolver};
use std::f64::consts::PI;

#[test]
fn test_inversion_symmetry_and_valley_bandgap() {
    // Inversion-symmetric case (dA = dB)
    let symmetric = HoneycombAcousticLattice::symmetric(0.020, 0.008);
    assert_eq!(symmetric.asymmetry_parameter(), 0.0);
    assert_eq!(symmetric.valley_gap_rad_per_s(), 0.0);
    assert_eq!(symmetric.valley_chern_number(), 0);

    // Broken inversion symmetry with dA > dB
    let broken_pos = HoneycombAcousticLattice::new(0.020, 0.009, 0.007, 343.0, 1.225);
    let delta_a = broken_pos.asymmetry_parameter();
    assert!((delta_a - 0.10).abs() < 1e-6);
    assert_eq!(broken_pos.valley_chern_number(), 1);

    // Valley bandgap must exceed 5% of Dirac frequency
    let gap_ratio = broken_pos.valley_gap_ratio();
    assert!(
        gap_ratio > 0.05,
        "Valley gap ratio was {gap_ratio}, expected > 0.05"
    );
    assert!(
        gap_ratio > 0.15,
        "Valley gap ratio was {gap_ratio}, expected > 0.15"
    );

    // Inverted domain with dA < dB
    let broken_neg = HoneycombAcousticLattice::new(0.020, 0.007, 0.009, 343.0, 1.225);
    assert_eq!(broken_neg.valley_chern_number(), -1);
    assert!((broken_neg.asymmetry_parameter() + 0.10).abs() < 1e-6);
    assert!((broken_neg.valley_gap_rad_per_s() - broken_pos.valley_gap_rad_per_s()).abs() < 1e-9);

    // Gap scaling linearity with asymmetry
    let broken_half = HoneycombAcousticLattice::new(0.020, 0.0085, 0.0075, 343.0, 1.225);
    let ratio_half = broken_half.valley_gap_ratio();
    assert!((gap_ratio / ratio_half - 2.0).abs() < 1e-6);
}

#[test]
fn test_valley_berry_curvature() {
    let lattice = HoneycombAcousticLattice::new(0.020, 0.009, 0.007, 343.0, 1.225);
    // Valley K (tau = +1) vs Valley K' (tau = -1)
    let berry_k = lattice.berry_curvature(1, 0.0, 0.0);
    let berry_k_prime = lattice.berry_curvature(-1, 0.0, 0.0);

    assert!(berry_k.abs() > 0.0);
    assert!(
        (berry_k + berry_k_prime).abs() < 1e-12,
        "Berry curvature must have opposite sign between K and K'"
    );

    // Curvature falls off with momentum
    let berry_off = lattice.berry_curvature(1, 10.0, 10.0);
    assert!(berry_off.abs() < berry_k.abs());

    // Symmetric lattice has zero Berry curvature
    let sym = HoneycombAcousticLattice::symmetric(0.020, 0.008);
    assert_eq!(sym.berry_curvature(1, 0.0, 0.0), 0.0);
}

#[test]
fn test_domain_wall_chiral_edge_states() {
    let lattice = HoneycombAcousticLattice::new(0.020, 0.009, 0.007, 343.0, 1.225);
    let solver = ValleyEdgeSolver::new(lattice, 0.60, 101);

    let profile: ValleyEdgeProfile = solver.solve_edge_profile(0.0);
    assert!(profile.decay_length_m.is_finite() && profile.decay_length_m > 0.0);
    assert!(profile.group_velocity_m_per_s.abs() > 100.0);

    // Center pressure peak at y = 0
    let center_idx = 50; // midpoint of 101 points
    let center_p = profile.pressure_amplitude[center_idx];
    assert!((center_p - 1.0).abs() < 1e-6);

    // Pressure decay away from interface
    let edge_p = profile.pressure_amplitude[0];
    assert!(
        edge_p < 0.05,
        "Edge pressure {edge_p} should be decayed far from domain wall"
    );

    // Dispersion relation across valley bandgap
    let dispersion = solver.compute_dispersion(11);
    assert_eq!(dispersion.len(), 11);
    let (k_first, w_first) = dispersion[0];
    let (k_last, w_last) = dispersion[10];
    assert!(k_first < k_last);
    assert!(
        w_last > w_first,
        "Chiral edge state should have positive group velocity for Cv = +1"
    );
}

#[test]
fn test_topological_corner_backscattering_immunity() {
    let lattice = HoneycombAcousticLattice::new(0.020, 0.009, 0.007, 343.0, 1.225);
    let solver = ValleyEdgeSolver::new(lattice.clone(), 0.30, 101);

    // 60-degree bend
    let t_60 = lattice.corner_transmission(PI / 3.0);
    assert!(
        t_60 >= 0.90,
        "60-degree bend transmission was {t_60}, expected >= 0.90"
    );
    let il_60 = lattice.insertion_loss_db(t_60);
    assert!(
        il_60 < 0.50,
        "60-degree bend insertion loss was {il_60} dB, expected < 0.5 dB"
    );

    // 120-degree sharp bend
    let t_120 = lattice.corner_transmission(2.0 * PI / 3.0);
    assert!(
        t_120 >= 0.90,
        "120-degree bend transmission was {t_120}, expected >= 0.90"
    );

    // Z-waveguide with two consecutive 60-degree bends
    let t_z_bends = solver.evaluate_bend_transmission(&[60.0, -60.0]);
    assert!(
        t_z_bends >= 0.85,
        "Z-bend transmission was {t_z_bends}, expected >= 0.85"
    );
}
