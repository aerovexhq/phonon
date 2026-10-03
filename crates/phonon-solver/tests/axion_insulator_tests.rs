#![deny(unsafe_code)]

//! Integration test suite for Phase 349: Higher-Order Axion Insulator Simulator in `phonon-solver`.
//!
//! Verifies:
//! - 3D bulk bandgap Delta_bulk and Dirac cone linear dispersion.
//! - Quantized magnetoelectric polarizability P_3 = 0.5 (theta = PI) vs 0.0 (theta = 0.0).
//! - Half-quantized surface Hall conductance sigma_xy^surf = pm 0.5 (e^2/h).
//! - Existence of 1D chiral gapless hinge modes traversing the surface gap.
//! - Hinge spatial energy confinement >= 80% along prism rod corners.
//! - Unidirectional propagation directivity >= 25.0 dB and non-magnetic disorder immunity.

use phonon_solver::axion_insulator::{
    AxionHamiltonian, AxionParams, AxionRodLattice, HingeId,
};

#[test]
fn test_3d_bulk_bandgap_and_dirac_dispersion() {
    let params = AxionParams::topological();
    let hamiltonian = AxionHamiltonian::new(params);

    // Default topological parameters: t_hop = 5.0, M_0 = 10.0
    // Bulk bandgap at Gamma: 2 * |M_0 - 3*t_hop| = 2 * |10.0 - 15.0| = 10.0 MHz
    let bulk_gap = hamiltonian.bulk_bandgap();
    assert!(
        (bulk_gap - 10.0).abs() < 1e-6,
        "Bulk gap should be exactly 10.0 MHz, got {bulk_gap}"
    );

    // Eigenvalues at Gamma (0,0,0)
    let evals_gamma = hamiltonian.eigenvalues(0.0, 0.0, 0.0);
    assert_eq!(evals_gamma.len(), 4);
    assert!((evals_gamma[0] - (-5.0)).abs() < 1e-6);
    assert!((evals_gamma[1] - (-5.0)).abs() < 1e-6);
    assert!((evals_gamma[2] - 5.0).abs() < 1e-6);
    assert!((evals_gamma[3] - 5.0).abs() < 1e-6);

    // Dirac velocity dispersion around Gamma along kx
    let dk = 0.05;
    let evals_dk = hamiltonian.eigenvalues(dk, 0.0, 0.0);
    let v_eff = params.effective_velocity(); // 15.0 / 5.0 = 3.0 MHz
    let expected_e = ((10.0 - 5.0 * (dk.cos() + 2.0)).powi(2) + v_eff.powi(2) * dk.sin().powi(2)).sqrt();
    assert!(
        (evals_dk[2] - expected_e).abs() < 1e-6,
        "Dispersion must match analytical Clifford eigenvalues"
    );

    // Verify 4-band dispersion computation along high-symmetry path
    let dispersion = hamiltonian.compute_bulk_dispersion(10);
    assert!(!dispersion.is_empty());
    assert!(dispersion.len() >= 50);
}

#[test]
fn test_quantized_magnetoelectric_polarizability() {
    // Topological preset (theta = PI)
    let topo_params = AxionParams::topological();
    assert!(topo_params.is_topological());
    assert_eq!(
        topo_params.polarizability_p3(),
        0.5,
        "P_3 must be half-quantized (0.5) for theta = PI"
    );

    let topo_h = AxionHamiltonian::new(topo_params);
    assert_eq!(topo_h.polarizability_p3(), 0.5);

    // Trivial preset (theta = 0)
    let trivial_params = AxionParams::trivial();
    assert!(!trivial_params.is_topological());
    assert_eq!(
        trivial_params.polarizability_p3(),
        0.0,
        "P_3 must be zero (0.0) for trivial phase"
    );

    let trivial_h = AxionHamiltonian::new(trivial_params);
    assert_eq!(trivial_h.polarizability_p3(), 0.0);
}

#[test]
fn test_half_quantized_surface_hall_conductance() {
    let topo_params = AxionParams::topological();
    let (sigma_top, sigma_bottom) = topo_params.surface_hall_conductance();
    assert_eq!(sigma_top, 0.5, "Top surface Hall conductance must be +0.5");
    assert_eq!(sigma_bottom, -0.5, "Bottom surface Hall conductance must be -0.5");

    let trivial_params = AxionParams::trivial();
    let (sig_triv_top, sig_triv_bottom) = trivial_params.surface_hall_conductance();
    assert_eq!(sig_triv_top, 0.0);
    assert_eq!(sig_triv_bottom, 0.0);
}

#[test]
fn test_1d_chiral_hinge_modes_crossing_surface_gap() {
    let params = AxionParams::topological();
    let lattice = AxionRodLattice::new(5, 5, params);
    let result = lattice.solve();

    assert!(result.is_topological);
    assert_eq!(result.hinge_modes.len(), 4, "Must identify exactly 4 hinge modes");

    // Check hinge identities and chiral velocities
    let h1 = &result.hinge_modes[0];
    let h2 = &result.hinge_modes[1];
    let h3 = &result.hinge_modes[2];
    let h4 = &result.hinge_modes[3];

    assert_eq!(h1.hinge_id, HingeId::Hinge1);
    assert_eq!(h1.chiral_velocity_direction, 1, "Hinge 1 must be forward propagating (+z)");

    assert_eq!(h2.hinge_id, HingeId::Hinge2);
    assert_eq!(h2.chiral_velocity_direction, -1, "Hinge 2 must be backward propagating (-z)");

    assert_eq!(h3.hinge_id, HingeId::Hinge3);
    assert_eq!(h3.chiral_velocity_direction, 1, "Hinge 3 must be forward propagating (+z)");

    assert_eq!(h4.hinge_id, HingeId::Hinge4);
    assert_eq!(h4.chiral_velocity_direction, -1, "Hinge 4 must be backward propagating (-z)");

    // Total net chirality must be zero across the cross section
    let total_chirality: i8 = result.hinge_modes.iter().map(|m| m.chiral_velocity_direction).sum();
    assert_eq!(total_chirality, 0, "Net chiral charge on rod must cancel to 0");

    // In-gap energies: hinge modes must reside inside the surface gap
    let surface_gap = result.surface_bandgap_mhz;
    assert!(surface_gap > 0.0);
    for m in &result.hinge_modes {
        assert!(
            m.energy_mhz.abs() < surface_gap,
            "Hinge mode energy {:.3} MHz must reside inside surface gap {:.3} MHz",
            m.energy_mhz,
            surface_gap
        );
    }
}

#[test]
fn test_hinge_spatial_energy_confinement_ratio() {
    let params = AxionParams::topological();
    let lattice = AxionRodLattice::new(5, 5, params);
    let result = lattice.solve();

    // Confinement ratio for each of the 4 hinge modes must be >= 80%
    for mode in &result.hinge_modes {
        assert!(
            mode.confinement_ratio >= 0.80,
            "Mode {} confinement ratio {:.3} must be >= 80%",
            mode.hinge_id.label(),
            mode.confinement_ratio
        );
    }

    assert!(
        result.mean_hinge_confinement >= 0.80,
        "Mean hinge confinement {:.3} must be >= 80%",
        result.mean_hinge_confinement
    );
}

#[test]
fn test_unidirectional_directivity_and_disorder_immunity() {
    let params = AxionParams::topological();
    let lattice = AxionRodLattice::new(4, 4, params);

    // 1. Directivity metric
    let result = lattice.solve();
    assert!(
        result.directivity_db >= 25.0,
        "Topological hinge directivity {:.1} dB must be >= 25.0 dB",
        result.directivity_db
    );

    // 2. S-parameter spectrum
    let s_params = lattice.compute_s_parameters(30);
    assert_eq!(s_params.freq_ghz.len(), 30);
    assert!(
        s_params.peak_directivity_db >= 25.0,
        "Peak S-parameter directivity must be >= 25.0 dB"
    );

    // 3. Disorder resilience against symmetric on-site perturbation W = 0.5 MHz
    let disorder_result = lattice.evaluate_disorder_robustness(0.5, 3, 12345);
    assert!(
        disorder_result.is_robust,
        "Chiral hinge modes must remain robust under disorder"
    );
    assert!(
        disorder_result.mean_confinement_ratio >= 0.75,
        "Disordered confinement ratio must remain >= 75%"
    );
    assert!(
        disorder_result.mean_energy_shift_mhz < 1.0,
        "Disorder energy shift must be small"
    );
}
