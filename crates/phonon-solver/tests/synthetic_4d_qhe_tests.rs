#![deny(unsafe_code)]

use phonon_solver::synthetic_4d_qhe::{
    FourDimLatticeSolver, Synthetic4dParams, SyntheticHallEngine, SyntheticHallParams,
};

#[test]
fn test_4d_topological_phase_and_second_chern_number() {
    // Topological phase: 2 < m < 4 -> C2 = -1
    let topo_params = Synthetic4dParams {
        mass_m: 3.0,
        ..Default::default()
    };
    assert_eq!(topo_params.second_chern_number(), -1);
    assert!(topo_params.is_topological());
    assert!(topo_params.theoretical_bulk_gap_khz() > 0.0);

    // High-Chern topological phase: 0 < m < 2 -> C2 = +3
    let high_chern = Synthetic4dParams {
        mass_m: 1.0,
        ..Default::default()
    };
    assert_eq!(high_chern.second_chern_number(), 3);
    assert!(high_chern.is_topological());

    // Trivial phase: m >= 4.0 -> C2 = 0
    let trivial_params = Synthetic4dParams {
        mass_m: 5.0,
        ..Default::default()
    };
    assert_eq!(trivial_params.second_chern_number(), 0);
    assert!(!trivial_params.is_topological());
}

#[test]
fn test_4d_bulk_band_structure_and_gap() {
    let params = Synthetic4dParams {
        mass_m: 3.0,
        hopping_t_khz: 2.5,
        ..Default::default()
    };
    let solver = FourDimLatticeSolver::new(params);

    assert!(!solver.bulk_dispersion.is_empty());
    assert!(solver.calculated_bulk_gap_khz > 0.0);

    for pt in &solver.bulk_dispersion {
        // Doubly degenerate lower and upper Dirac bands
        assert!((pt.eigenvalues_khz[0] - pt.eigenvalues_khz[1]).abs() < 1e-6);
        assert!((pt.eigenvalues_khz[2] - pt.eigenvalues_khz[3]).abs() < 1e-6);
        assert!(pt.eigenvalues_khz[0] <= 0.0);
        assert!(pt.eigenvalues_khz[2] >= 0.0);
    }
}

#[test]
fn test_3d_boundary_chiral_hypersurface_modes() {
    let topo_params = Synthetic4dParams {
        mass_m: 3.0,
        hopping_t_khz: 2.5,
        nx_layers: 10,
        disorder_w: 0.0,
        ..Default::default()
    };
    let topo_solver = FourDimLatticeSolver::new(topo_params);

    assert!(!topo_solver.boundary_modes.is_empty());
    assert!(
        topo_solver.boundary_confinement_ratio >= 0.85,
        "Boundary state confinement must be >= 85%, got {:.3}",
        topo_solver.boundary_confinement_ratio
    );

    // Verify chiral crossing near k=0
    let zero_crossing = topo_solver
        .boundary_modes
        .iter()
        .min_by(|a, b| a.k_parallel.abs().partial_cmp(&b.k_parallel.abs()).unwrap())
        .unwrap();
    assert!(
        zero_crossing.energy_khz.abs() < 0.5,
        "Chiral branch should cross mid-gap near k=0"
    );
    assert!(zero_crossing.group_velocity_km_s.abs() > 0.0);

    // Trivial phase
    let trivial_params = Synthetic4dParams {
        mass_m: 5.0,
        nx_layers: 10,
        ..Default::default()
    };
    let trivial_solver = FourDimLatticeSolver::new(trivial_params);
    assert!(
        trivial_solver.boundary_confinement_ratio < 0.35,
        "Trivial phase should not have localized boundary states, got {:.3}",
        trivial_solver.boundary_confinement_ratio
    );
}

#[test]
fn test_nonlinear_4d_hall_response_scaling() {
    let topo_params = SyntheticHallParams {
        lattice: Synthetic4dParams {
            mass_m: 3.0,
            ..Default::default()
        },
        drive_field_ey: 1.5,
        synthetic_field_bzw: 2.0,
        ..Default::default()
    };
    let topo_engine = SyntheticHallEngine::new(topo_params);

    assert!(topo_engine.metrics.non_linear_hall_conductance < 0.0); // C2 = -1
    assert!(topo_engine.metrics.non_linear_hall_current < 0.0);
    assert_eq!(
        (topo_engine.metrics.non_linear_hall_current
            / (topo_engine.metrics.non_linear_hall_conductance * 1.5 * 2.0)
            - 1.0)
            .abs()
            < 1e-6,
        true
    );

    // Test high-Chern phase with C2 = +3
    let high_params = SyntheticHallParams {
        lattice: Synthetic4dParams {
            mass_m: 1.0,
            ..Default::default()
        },
        drive_field_ey: 1.0,
        synthetic_field_bzw: 1.0,
        ..Default::default()
    };
    let high_engine = SyntheticHallEngine::new(high_params);
    assert!(high_engine.metrics.non_linear_hall_conductance > 0.0);
    assert!(high_engine.metrics.non_linear_hall_current > 0.0);

    // Test trivial phase with C2 = 0
    let trivial_params = SyntheticHallParams {
        lattice: Synthetic4dParams {
            mass_m: 5.0,
            ..Default::default()
        },
        drive_field_ey: 1.0,
        synthetic_field_bzw: 1.0,
        ..Default::default()
    };
    let trivial_engine = SyntheticHallEngine::new(trivial_params);
    assert_eq!(trivial_engine.metrics.non_linear_hall_conductance, 0.0);
    assert_eq!(trivial_engine.metrics.non_linear_hall_current, 0.0);
}

#[test]
fn test_topological_defect_immunity_and_synthetic_harmonics() {
    let disordered_params = SyntheticHallParams {
        lattice: Synthetic4dParams {
            mass_m: 3.0,
            disorder_w: 0.25,
            ..Default::default()
        },
        drive_field_ey: 1.0,
        synthetic_field_bzw: 1.0,
        harmonic_span: 3,
        ..Default::default()
    };
    let engine = SyntheticHallEngine::new(disordered_params);

    assert!(
        engine.metrics.disorder_retention_ratio >= 0.90,
        "Disorder retention ratio must be >= 90%, got {:.3}",
        engine.metrics.disorder_retention_ratio
    );
    assert!(
        engine.metrics.chiral_directivity_db >= 20.0,
        "Chiral directivity must be >= 20 dB, got {:.2} dB",
        engine.metrics.chiral_directivity_db
    );

    // Check synthetic frequency ladder harmonics
    assert_eq!(engine.metrics.synthetic_harmonics.len(), 7); // -3..=3
    let total_power: f64 = engine
        .metrics
        .synthetic_harmonics
        .iter()
        .map(|h| h.power_fraction)
        .sum();
    assert!(
        (total_power - 1.0).abs() < 1e-4,
        "Synthetic frequency harmonic powers must sum to 1.0, got {:.6}",
        total_power
    );
}
