#![deny(unsafe_code)]

//! Automated test suite for Phase 439: Topological Acoustic Boundary-Mode
//! Soliton Logic Gate & Majority Voter.

use phonon_solver::topological_soliton_logic::{
    BoundarySolitonParams, BoundarySolitonSolver, CollisionalPhaseShiftSolver,
    MajorityVoterParams, MajorityVoterSolver, SolitonGateMode, TopologicalSolitonProcessor,
};

#[test]
fn test_boundary_soliton_topological_quantization_and_localization() {
    // 1. Topological regime (lambda > gamma)
    let solver_topo = BoundarySolitonSolver::new(BoundarySolitonParams {
        intra_cell_coupling_mhz: 2.0,
        inter_cell_coupling_mhz: 8.0,
        ..Default::default()
    });
    let (metrics_topo, profile) = solver_topo.solve_soliton(0.0);
    assert_eq!(metrics_topo.bulk_quadrupole_moment, 0.5);
    assert!(metrics_topo.edge_localization_ratio >= 0.85);
    assert!(metrics_topo.dispersion_balance_error <= 0.05);
    assert!(metrics_topo.fwhm_width_um > 0.0);
    assert!(!profile.is_empty());

    // 2. Trivial regime (lambda < gamma)
    let solver_triv = BoundarySolitonSolver::new(BoundarySolitonParams {
        intra_cell_coupling_mhz: 8.0,
        inter_cell_coupling_mhz: 2.0,
        ..Default::default()
    });
    assert_eq!(solver_triv.evaluate_quadrupole_moment(), 0.0);
    let (metrics_triv, _) = solver_triv.solve_soliton(0.0);
    assert!(metrics_triv.edge_localization_ratio < 0.50);
}

#[test]
fn test_collisional_phase_shift_and_shape_conservation() {
    let solver = CollisionalPhaseShiftSolver::default();
    let (metrics, trajectory) = solver.solve_collision();

    assert!(metrics.phase_shift_rad > 0.0);
    assert!(metrics.theoretical_phase_shift_rad > 0.0);
    assert!(metrics.phase_shift_residual_rad <= 0.08);
    assert!(metrics.shape_conservation_fidelity >= 0.950);
    assert!(metrics.peak_collision_energy_fj > 0.0);
    assert!(!trajectory.is_empty());
}

#[test]
fn test_majority_voter_truth_table_and_contrast() {
    // 1. Standard Majority mode
    let solver = MajorityVoterSolver::default();
    let (metrics, table, waveform) = solver.solve_gate();

    assert_eq!(table.len(), 8);
    for entry in &table {
        assert!(
            entry.state_pass,
            "Truth table state failed for A={}, B={}, C={}: expected {}, evaluated {}",
            entry.a, entry.b, entry.c, entry.expected_out, entry.evaluated_out
        );
    }
    assert!(metrics.truth_table_all_passed);
    assert!(metrics.contrast_ratio_db >= 20.0);
    assert!(metrics.propagation_delay_ns <= 30.0);
    assert!(metrics.switching_energy_fj <= 2.0);
    assert!(!waveform.is_empty());

    // 2. AND Gate synthesis (C = 0)
    let and_solver = MajorityVoterSolver::new(MajorityVoterParams {
        mode: SolitonGateMode::AndGate,
        ..Default::default()
    });
    let and_table = and_solver.evaluate_truth_table();
    assert!(and_table.iter().all(|e| e.state_pass));

    // 3. OR Gate synthesis (C = 1)
    let or_solver = MajorityVoterSolver::new(MajorityVoterParams {
        mode: SolitonGateMode::OrGate,
        ..Default::default()
    });
    let or_table = or_solver.evaluate_truth_table();
    assert!(or_table.iter().all(|e| e.state_pass));
}

#[test]
fn test_10_point_physics_audit_all_passed() {
    let processor = TopologicalSolitonProcessor::default();
    let report = processor.audit_system();

    assert!(report.bulk_quadrupole_quantization_pass, "Criterion 1 failed");
    assert!(report.edge_localization_pass, "Criterion 2 failed");
    assert!(report.dispersion_balance_pass, "Criterion 3 failed");
    assert!(report.collision_shape_fidelity_pass, "Criterion 4 failed");
    assert!(report.phase_shift_predictability_pass, "Criterion 5 failed");
    assert!(report.truth_table_completeness_pass, "Criterion 6 failed");
    assert!(report.logic_contrast_pass, "Criterion 7 failed");
    assert!(report.reconfigurable_modes_pass, "Criterion 8 failed");
    assert!(report.switching_energy_pass, "Criterion 9 failed");
    assert!(report.propagation_latency_pass, "Criterion 10 failed");

    assert_eq!(report.total_score, 10);
    assert!(report.all_passed);
}
