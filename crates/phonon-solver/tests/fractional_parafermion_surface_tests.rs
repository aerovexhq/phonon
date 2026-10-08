#![deny(unsafe_code)]

//! Physical test suite for Phase 445:
//! Quantum Metamaterial Non-Abelian Fractional Parafermion Surface-Code Lattice & Anyonic Braid Repeater.

use phonon_solver::fractional_parafermion_surface::{
    FractionalParafermionLatticeParams, FractionalParafermionLatticeSolver,
    FractionalParafermionOrder, FractionalParafermionProcessor, QuditDistillationParams,
    QuditDistillationSolver, SurfaceCodeRepeaterParams, SurfaceCodeRepeaterSolver,
};

#[test]
fn test_parafermion_lattice_topological_gap_and_braiding() {
    let params = FractionalParafermionLatticeParams::default();
    let solver = FractionalParafermionLatticeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.topological_gap_mhz >= 1.8,
        "Topological gap {:.2} MHz is below 1.8 MHz",
        metrics.topological_gap_mhz
    );
    assert!(
        metrics.braid_fidelity >= 0.999,
        "Braid fidelity {:.5} is below 0.999",
        metrics.braid_fidelity
    );
    assert!(
        metrics.diabatic_leakage_error < 1.0e-4,
        "Diabatic leakage error {:.2e} exceeds 1e-4",
        metrics.diabatic_leakage_error
    );
    assert!(
        metrics.localization_length_nm <= 55.0,
        "Localization length {:.1} nm exceeds 55 nm",
        metrics.localization_length_nm
    );
    assert!(
        metrics.poisoning_lifetime_us >= 80.0,
        "Poisoning lifetime {:.1} us is below 80 us",
        metrics.poisoning_lifetime_us
    );

    let packets = solver.compute_wavepackets(32);
    assert_eq!(packets.len(), 32);

    let traj = solver.compute_braid_trajectory(32);
    assert_eq!(traj.len(), 32);
    let end_pt = traj.last().unwrap();
    assert!(end_pt.manifold_fidelity >= 0.999);

    // Test Z4 order
    let mut z4_params = FractionalParafermionLatticeParams::default();
    z4_params.order = FractionalParafermionOrder::Z4Clock;
    let z4_solver = FractionalParafermionLatticeSolver::new(z4_params);
    let z4_m = z4_solver.evaluate_metrics();
    assert!(z4_m.topological_gap_mhz >= 1.8);
}

#[test]
fn test_surface_code_threshold_and_braid_repeater() {
    let params = SurfaceCodeRepeaterParams::default();
    let solver = SurfaceCodeRepeaterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.threshold_error_percent >= 1.5,
        "Threshold error {:.1}% is below 1.5%",
        metrics.threshold_error_percent
    );
    assert!(
        metrics.logical_error_rate < 1.0e-4,
        "Logical error rate {:.2e} exceeds 1e-4",
        metrics.logical_error_rate
    );
    assert!(
        metrics.repeater_fidelity >= 0.992,
        "Repeater fidelity {:.4} is below 0.992",
        metrics.repeater_fidelity
    );
    assert!(
        metrics.distribution_rate_khz >= 120.0,
        "Distribution rate {:.1} kHz is below 120 kHz",
        metrics.distribution_rate_khz
    );
    assert!(metrics.error_suppression_factor >= 50.0);

    let nodes = solver.generate_lattice_nodes();
    assert_eq!(nodes.len(), 25); // 5x5 grid

    let scaling = solver.compute_threshold_scaling(32);
    assert_eq!(scaling.len(), 32);
    let first = scaling.first().unwrap();
    assert!(first.logical_error_d5 < first.logical_error_d3);
}

#[test]
fn test_qudit_distillation_and_dispersive_readout() {
    let params = QuditDistillationParams::default();
    let solver = QuditDistillationSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.distilled_magic_fidelity >= 0.999,
        "Distilled magic fidelity {:.5} is below 0.999",
        metrics.distilled_magic_fidelity
    );
    assert!(
        metrics.output_infidelity < 1.0e-3,
        "Output infidelity {:.2e} exceeds 1e-3",
        metrics.output_infidelity
    );
    assert!(
        metrics.acceptance_probability_percent >= 18.0,
        "Acceptance probability {:.1}% is below 18%",
        metrics.acceptance_probability_percent
    );
    assert!(
        metrics.dispersive_readout_snr_db >= 18.5,
        "Readout SNR {:.1} dB is below 18.5 dB",
        metrics.dispersive_readout_snr_db
    );
    assert!(metrics.qudit_discrimination_fidelity >= 0.996);
    assert!(metrics.factory_node_footprint <= 24);

    let rounds = solver.compute_distillation_rounds();
    assert_eq!(rounds.len(), 3);
    assert!(rounds[2].magic_state_fidelity >= 0.999);

    let spec = solver.compute_cavity_spectrum(32);
    assert_eq!(spec.len(), 32);
}

#[test]
fn test_fractional_parafermion_system_10_point_audit() {
    let processor = FractionalParafermionProcessor::default();
    let audit = processor.audit_system();

    assert!(audit.topological_gap_pass, "Topological gap failed");
    assert!(audit.braid_fidelity_pass, "Braid fidelity failed");
    assert!(audit.diabatic_leakage_pass, "Diabatic leakage failed");
    assert!(audit.poisoning_lifetime_pass, "Poisoning lifetime failed");
    assert!(audit.threshold_error_pass, "Threshold error failed");
    assert!(audit.logical_error_rate_pass, "Logical error rate failed");
    assert!(audit.repeater_fidelity_pass, "Repeater fidelity failed");
    assert!(audit.magic_distillation_fidelity_pass, "Magic distillation fidelity failed");
    assert!(audit.acceptance_probability_pass, "Acceptance probability failed");
    assert!(audit.dispersive_readout_snr_pass, "Dispersive readout SNR failed");

    assert_eq!(audit.total_score, 10);
    assert!(audit.all_passed);
}
