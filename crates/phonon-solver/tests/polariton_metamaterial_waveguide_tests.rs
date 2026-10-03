#![deny(unsafe_code)]

//! Test suite for Phase 331: Multi-Mode Polariton Waveguide & Chiral Edge State Solver.
//!
//! Verifies exciton-photon anti-crossing Rabi splitting, Hopfield fractions normalization,
//! group velocity and effective mass, chiral edge mode non-reciprocal transmission,
//! topological backscattering defect immunity, and 2D mode profile perimeter concentration.

use phonon_solver::polariton_waveguide::{
    ChiralEdgeModeSolver, ChiralLatticeDefect, MultiModePolaritonDispersionSolver,
    PolaritonWaveguideParams,
};

#[test]
fn test_multimode_dispersion_solver_and_hopfield_normalization() {
    let params = PolaritonWaveguideParams::default();
    let solver = MultiModePolaritonDispersionSolver::new(params.clone());

    // Sweep transverse modes m = 0, 1, 2 across k in [-4.0, 4.0] um^-1
    let modes = solver.sweep_multi_mode(3, -4.0, 4.0, 41);
    assert_eq!(modes.len(), 3);

    for (m_idx, mode_pts) in modes.iter().enumerate() {
        assert_eq!(mode_pts.len(), 41);

        for pt in mode_pts {
            assert_eq!(pt.mode_index_m, m_idx);

            // Hopfield fractions normalization: |X|^2 + |C|^2 == 1.0
            let sum_frac = pt.exciton_fraction + pt.photon_fraction;
            assert!(
                (sum_frac - 1.0).abs() < 1e-12,
                "Hopfield sum must equal 1.0, got {}",
                sum_frac
            );
            assert!(pt.exciton_fraction >= 0.0 && pt.exciton_fraction <= 1.0);
            assert!(pt.photon_fraction >= 0.0 && pt.photon_fraction <= 1.0);

            // UP branch must always lie above LP branch
            assert!(
                pt.energy_up_mev > pt.energy_lp_mev,
                "UP branch must exceed LP branch"
            );

            // Splitting must be at least the vacuum Rabi splitting 2g
            assert!(
                pt.splitting_mev >= 2.0 * params.rabi_coupling_g_mev - 1e-9,
                "Splitting {} must be >= 2g",
                pt.splitting_mev
            );

            // Damping linewidth must be positive and bounded between gamma_x and kappa_c
            let min_damp = params.exciton_decay_rate_gamma_x.min(params.cavity_decay_rate_kappa_c);
            let max_damp = params.exciton_decay_rate_gamma_x.max(params.cavity_decay_rate_kappa_c);
            assert!(pt.damping_linewidth >= min_damp - 1e-9);
            assert!(pt.damping_linewidth <= max_damp + 1e-9);
        }
    }
}

#[test]
fn test_anticrossing_vacuum_rabi_splitting() {
    let mut params = PolaritonWaveguideParams::default();
    params.bare_exciton_energy_mev = 1500.0;
    params.cavity_cutoff_energy_mev = 1500.0;
    params.rabi_coupling_g_mev = 15.0; // Rabi splitting 2g = 30 meV

    let solver = MultiModePolaritonDispersionSolver::new(params.clone());

    // At k = 0, m = 0, detuning delta = 0
    let pt = solver.solve_point(0.0, 0);

    // Bare cavity energy at cutoff must equal E_{c0}
    assert!((pt.energy_cavity_mev - 1500.0).abs() < 1e-9);
    assert!((pt.energy_exciton_mev - 1500.0).abs() < 1e-9);

    // Minimum anti-crossing splitting Delta = 2g = 30.0 meV
    let rabi_split = 2.0 * params.rabi_coupling_g_mev;
    assert!(
        (pt.splitting_mev - rabi_split).abs() < 1e-9,
        "Splitting {} must equal 2g = {}",
        pt.splitting_mev,
        rabi_split
    );
    assert!((solver.minimum_energy_gap() - rabi_split).abs() < 1e-9);

    // Symmetrical lower and upper polariton branches: 1485.0 meV and 1515.0 meV
    assert!((pt.energy_lp_mev - 1485.0).abs() < 1e-9);
    assert!((pt.energy_up_mev - 1515.0).abs() < 1e-9);

    // Equal exciton and photon fractions at zero detuning (|X|^2 = 0.5, |C|^2 = 0.5)
    assert!((pt.exciton_fraction - 0.5).abs() < 1e-9);
    assert!((pt.photon_fraction - 0.5).abs() < 1e-9);
}

#[test]
fn test_group_velocity_and_effective_mass_calculation() {
    let params = PolaritonWaveguideParams::default();
    let solver = MultiModePolaritonDispersionSolver::new(params);

    // At k = 0, group velocity must be zero
    let pt_k0 = solver.solve_point(0.0, 0);
    assert!(
        pt_k0.group_velocity.abs() < 1e-9,
        "Group velocity at k=0 must be zero"
    );
    assert!(pt_k0.group_velocity_c.abs() < 1e-9);

    // At k > 0, group velocity must be positive and sub-luminal
    let pt_k_pos = solver.solve_point(2.0, 0);
    assert!(
        pt_k_pos.group_velocity > 0.0,
        "Group velocity for forward propagation must be positive"
    );
    assert!(
        pt_k_pos.group_velocity_c > 0.0 && pt_k_pos.group_velocity_c < 1.0,
        "Group velocity must be sub-luminal"
    );

    // Effective mass of cavity polaritons must be orders of magnitude lighter than m_e (~1e-5 to 1e-4 m_e)
    assert!(
        pt_k0.effective_mass > 1e-6 && pt_k0.effective_mass < 1e-3,
        "Polariton effective mass must be extremely light (~1e-5 to 1e-4 m_e), got {}",
        pt_k0.effective_mass
    );
    assert!(pt_k_pos.effective_mass > 0.0);
}

#[test]
fn test_chiral_edge_state_unidirectional_non_reciprocal_transmission() {
    let params = PolaritonWaveguideParams {
        chern_number: 1, // Non-trivial Chern insulator C = +1
        ..Default::default()
    };
    let solver = ChiralEdgeModeSolver::new(params.clone());

    // Evaluate transmission in the topological gap at exciton resonance (1500 meV)
    let pt = solver.evaluate_transmission_at_energy(params.bare_exciton_energy_mev);

    // Forward transmission S_21 must be near unity (insertion loss < 0.5 dB)
    assert!(
        pt.s21_linear > 0.95,
        "Forward transmission |S_21| {} must be near unity",
        pt.s21_linear
    );
    assert!(
        pt.insertion_loss_db < 0.5,
        "Forward insertion loss {} dB must be < 0.5 dB",
        pt.insertion_loss_db
    );

    // Backward transmission S_12 must be deeply suppressed (isolation > 30 dB)
    assert!(
        pt.s12_linear < 0.05,
        "Backward transmission |S_12| {} must be strongly attenuated",
        pt.s12_linear
    );
    assert!(
        pt.isolation_db > 30.0,
        "Non-reciprocal isolation {} dB must exceed 30 dB",
        pt.isolation_db
    );
    assert!(
        pt.s21_linear > 10.0 * pt.s12_linear,
        "Forward transmission must vastly exceed backward transmission"
    );
}

#[test]
fn test_topological_backscattering_immunity_with_defects() {
    // 1. Topological regime: C = +1
    let params_topo = PolaritonWaveguideParams {
        chern_number: 1,
        ..Default::default()
    };
    let solver_topo = ChiralEdgeModeSolver::new(params_topo);

    let (t_clean, _t_defect, ratio) = solver_topo.compute_transmission_with_defect();
    assert!(t_clean > 0.90, "Clean transmission {} must be high", t_clean);
    assert!(
        ratio >= 0.90,
        "Topological defect transmission ratio {} must be >= 0.90",
        ratio
    );

    // Test with explicit corner obstacle
    let solver_corner = solver_topo.clone().with_defect(ChiralLatticeDefect::CornerObstacle {
        x: 10,
        y: 19,
        size: 2,
    });
    let pt_corner = solver_corner.evaluate_transmission_at_energy(1500.0);
    let ratio_corner = (pt_corner.s21_linear * pt_corner.s21_linear) / t_clean;
    assert!(
        ratio_corner >= 0.90,
        "Corner defect ratio {} must be >= 0.90",
        ratio_corner
    );

    // 2. Trivial regime: C = 0 (no topological protection)
    let params_trivial = PolaritonWaveguideParams {
        chern_number: 0,
        ..Default::default()
    };
    let solver_trivial = ChiralEdgeModeSolver::new(params_trivial);
    let (_t_triv_clean, _t_triv_defect, ratio_trivial) = solver_trivial.compute_transmission_with_defect();
    assert!(
        ratio_trivial < 0.60,
        "Trivial waveguide must suffer severe backscattering (ratio {} < 0.60)",
        ratio_trivial
    );
}

#[test]
fn test_2d_mode_profile_perimeter_concentration() {
    let params = PolaritonWaveguideParams {
        chern_number: 1,
        ..Default::default()
    };
    let solver = ChiralEdgeModeSolver::new(params).with_grid_size(16, 16);

    // Clean mode profile
    let wf_clean = solver.compute_wavefunction_2d();
    assert_eq!(wf_clean.nx, 16);
    assert_eq!(wf_clean.ny, 16);
    assert_eq!(wf_clean.intensity.len(), 256);
    assert!(!wf_clean.has_defect);

    // Perimeter concentration must be >= 0.70
    assert!(
        wf_clean.perimeter_confinement_ratio >= 0.70,
        "Perimeter concentration {} must be >= 0.70",
        wf_clean.perimeter_confinement_ratio
    );

    // Mode profile with defect obstacle
    let solver_defect = solver.clone().with_defect(ChiralLatticeDefect::CornerObstacle {
        x: 8,
        y: 15,
        size: 2,
    });
    let wf_defect = solver_defect.compute_wavefunction_2d();
    assert!(wf_defect.has_defect);
    assert!(
        wf_defect.perimeter_confinement_ratio >= 0.70,
        "Defect mode perimeter concentration {} must be >= 0.70",
        wf_defect.perimeter_confinement_ratio
    );

    // Defect site intensity must be zero
    assert_eq!(wf_defect.intensity_at(8, 15), 0.0);

    // Circulation arrows must be generated
    let arrows = solver.circulation_arrows();
    assert!(!arrows.is_empty(), "Circulation arrows must be generated");
}
