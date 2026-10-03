#![deny(unsafe_code)]

//! Comprehensive test suite for Topological Acoustic Flat-Band Lieb-Lattice & Aharonov-Bohm Caging.

use std::f64::consts::PI;
use phonon_solver::lieb_lattice::{
    AbCagingSimulator, LiebHamiltonian, LiebLattice, LiebParams,
};

#[test]
fn test_exact_flatness_of_mid_band() {
    let params = LiebParams::default();
    let hamiltonian = LiebHamiltonian::new(params);

    // Sample 50x50 = 2500 points across the full Brillouin Zone [-pi, pi] x [-pi, pi]
    let (max_flatness_error, max_vg) = hamiltonian.sample_bz_flatness(50);

    assert!(
        max_flatness_error < 1e-10,
        "Flat band energy error must be < 1e-10, got {max_flatness_error}"
    );
    assert_eq!(
        max_vg, 0.0,
        "Flat band group velocity must be strictly 0.0 m/s"
    );

    // Also check along high-symmetry path
    let path_points = hamiltonian.dispersion_along_path(20);
    assert!(!path_points.is_empty());
    for pt in &path_points {
        assert!(
            pt.flatness_error < 1e-10,
            "Flatness error along path at k=({:.3}, {:.3}) is {:.3e}",
            pt.kx,
            pt.ky,
            pt.flatness_error
        );
        assert_eq!(pt.group_velocity_flat, 0.0);
    }
}

#[test]
fn test_dirac_cone_touching_at_m_point() {
    let params = LiebParams::default();
    let hamiltonian = LiebHamiltonian::new(params);

    let (kx_m, ky_m) = LiebHamiltonian::dirac_point();
    assert!((kx_m - PI).abs() < 1e-12);
    assert!((ky_m - PI).abs() < 1e-12);

    let (e_lower, e_flat, e_upper) = hamiltonian.eigenvalues(kx_m, ky_m);

    assert!(
        e_lower.abs() < 1e-12,
        "Lower band must touch E = 0 at M point, got {e_lower}"
    );
    assert!(
        e_flat.abs() < 1e-12,
        "Flat band must touch E = 0 at M point, got {e_flat}"
    );
    assert!(
        e_upper.abs() < 1e-12,
        "Upper band must touch E = 0 at M point, got {e_upper}"
    );

    // All three bands meet at Dirac frequency
    assert_eq!(hamiltonian.dirac_frequency_ghz(), 1.0);
}

#[test]
fn test_zero_group_velocity_for_flat_band() {
    let params = LiebParams::default();
    let hamiltonian = LiebHamiltonian::new(params);

    // Sample points including Gamma, X, M, and generic non-symmetry k-points
    let test_points = [
        (0.0, 0.0),
        (PI, 0.0),
        (PI, PI),
        (0.0, PI),
        (0.732 * PI, 0.284 * PI),
        (-0.419 * PI, 0.812 * PI),
    ];

    for (kx, ky) in test_points {
        let (vgx, vgy) = hamiltonian.flat_band_group_velocity(kx, ky);
        assert_eq!(vgx, 0.0);
        assert_eq!(vgy, 0.0);
    }
}

#[test]
fn test_analytical_cls_destructive_interference() {
    let lattice = LiebLattice::new(4, 4, LiebParams::default());
    let (h_re, _) = lattice.assemble_hamiltonian();
    let dim = lattice.total_sites();

    // Generate analytical CLS at central plaquette (1, 1)
    let cls = lattice
        .generate_analytical_cls(1, 1)
        .expect("Plaquette (1, 1) must exist on a 4x4 lattice");

    assert_eq!(cls.cell_x, 1);
    assert_eq!(cls.cell_y, 1);
    assert_eq!(cls.energy, 0.0);
    assert_eq!(cls.confinement_ratio, 1.0);

    // Verify destructive interference: residual |(H * psi)_i| must vanish everywhere (< 1e-12)
    let max_residual = cls.verify_destructive_interference(&h_re, dim);
    assert!(
        max_residual < 1e-12,
        "Destructive interference residual must be < 1e-12, got {max_residual}"
    );

    // Verify that corner site A(1, 1) receives net zero coupling
    let s_a = lattice.site_index(1, 1, 0);
    let mut row_sum_a = 0.0_f64;
    for k in 0..4 {
        row_sum_a += h_re[s_a * dim + cls.site_indices[k]] * cls.amplitudes[k];
    }
    assert!(
        row_sum_a.abs() < 1e-12,
        "Corner site A must receive exactly zero net coupling, got {row_sum_a}"
    );
}

#[test]
fn test_100_percent_spatial_energy_confinement_of_single_plaquette_cls() {
    let lattice = LiebLattice::new(5, 5, LiebParams::default());
    let cls = lattice
        .generate_analytical_cls(2, 2)
        .expect("Central plaquette (2, 2) must exist on a 5x5 lattice");

    // Confinement within the 4 edge sites is 100%
    let mut energy_in_plaquette = 0.0_f64;
    for &idx in &cls.site_indices {
        energy_in_plaquette += cls.spatial_intensity[idx];
    }
    assert!(
        (energy_in_plaquette - 1.0).abs() < 1e-12,
        "Energy inside plaquette must be 1.0 (100%), got {energy_in_plaquette}"
    );

    // All sites outside the 4 edge sites must have exactly zero energy
    let dim = lattice.total_sites();
    for i in 0..dim {
        if !cls.site_indices.contains(&i) {
            assert_eq!(
                cls.spatial_intensity[i], 0.0,
                "Site {i} outside plaquette must have zero energy"
            );
        }
    }
}

#[test]
fn test_aharonov_bohm_caging_at_flux_pi() {
    let mut params_caged = LiebParams::default();
    params_caged.phi = PI; // Synthetic gauge flux Phi = PI for caging

    let lattice_caged = LiebLattice::new(4, 4, params_caged);
    let sim_caged = AbCagingSimulator::new(lattice_caged);

    let mut params_free = LiebParams::default();
    params_free.phi = 0.0; // Standard Lieb lattice without gauge flux

    let lattice_free = LiebLattice::new(4, 4, params_free);
    let sim_free = AbCagingSimulator::new(lattice_free);

    // Launch single-site wavepacket at center corner site A
    let psi_0_caged = sim_caged.initial_center_site_wavepacket();
    let psi_0_free = sim_free.initial_center_site_wavepacket();

    let t_eval = 200.0;
    let psi_caged = sim_caged.evolve_wave_packet(&psi_0_caged, t_eval);
    let psi_free = sim_free.evolve_wave_packet(&psi_0_free, t_eval);
    let ipr_caged = AbCagingSimulator::inverse_participation_ratio(&psi_caged);
    let ipr_free = AbCagingSimulator::inverse_participation_ratio(&psi_free);

    assert!(
        ipr_caged > ipr_free,
        "Caged IPR ({ipr_caged:.4}) at t = 200 ns must exceed dispersed free IPR ({ipr_free:.4})"
    );
    assert!(
        ipr_caged > 0.05,
        "Caged state must remain localized with IPR > 0.05, got {ipr_caged}"
    );

    // Verify caging curve vs flux has a peak at Phi = pi
    let caging_curve = sim_caged.caging_curve_vs_flux(9, 20.0);
    assert_eq!(caging_curve.len(), 9);
    // Find point closest to Phi = pi (phi / pi = 1.0)
    let mid_point = caging_curve.iter().find(|(phi_pi, _)| (phi_pi - 1.0).abs() < 1e-4);
    assert!(mid_point.is_some());
}

#[test]
fn test_disorder_robustness_of_cls() {
    let lattice = LiebLattice::new(4, 4, LiebParams::default());
    let sim = AbCagingSimulator::new(lattice);

    // Introduce moderate on-site disorder W = 0.5 MHz (10% of hopping J = 5.0 MHz)
    let disorder_result = sim.evaluate_disorder_resilience(0.5, 8, 42);

    assert_eq!(disorder_result.disorder_w, 0.5);
    assert!(
        disorder_result.mean_energy_shift_mhz < 0.5,
        "Energy shift must be bounded by W, got {:.3}",
        disorder_result.mean_energy_shift_mhz
    );
    assert!(
        disorder_result.mean_confinement_ratio > 0.85,
        "Mean confinement ratio must remain > 0.85 under disorder, got {:.3}",
        disorder_result.mean_confinement_ratio
    );
    assert!(
        disorder_result.is_robust,
        "Compact localized state must be marked robust"
    );
}
