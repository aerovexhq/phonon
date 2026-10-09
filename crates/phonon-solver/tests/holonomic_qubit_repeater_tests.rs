#![deny(unsafe_code)]

use phonon_solver::holonomic_qubit_repeater::{
    FractionalValleyParams, FractionalValleySolver, HolonomicBraidingParams,
    HolonomicBraidingSolver, HolonomicCryoParams, HolonomicCryoSolver, HolonomicQubitGate,
    HolonomicQubitRepeaterParams, HolonomicQubitRepeaterProcessor,
};

#[test]
fn test_holonomic_braiding_corner_confinement_and_non_abelian_commutator() {
    let mut params = HolonomicBraidingParams::default();
    params.intracell_gamma_mhz = 2.5;
    params.intercell_lambda_mhz = 12.0;
    params.braid_duration_ns = 90.0;

    let (metrics, spatial_points, loop_points) = HolonomicBraidingSolver::solve(&params);

    assert_eq!(metrics.quadrupole_moment, 0.50);
    assert!(metrics.bulk_bandgap_mhz >= 15.0);
    assert!(metrics.corner_confinement_ratio >= 0.90);
    assert!(metrics.non_abelian_commutator_norm >= 0.70);
    assert!(metrics.gate_process_fidelity >= 0.998);
    assert!(metrics.diabatic_leakage_rate <= 1.0e-4);

    assert_eq!(spatial_points.len(), params.grid_size * params.grid_size);
    assert_eq!(loop_points.len(), 60);

    // Test across universal gate set
    let gates = [
        HolonomicQubitGate::Hadamard,
        HolonomicQubitGate::PhaseS,
        HolonomicQubitGate::PauliX,
        HolonomicQubitGate::PauliZ,
        HolonomicQubitGate::TGate,
        HolonomicQubitGate::ControlledPhase,
    ];

    for gate in gates {
        params.target_gate = gate;
        let (m, _, _) = HolonomicBraidingSolver::solve(&params);
        assert!(m.gate_process_fidelity >= 0.998);
        assert!(m.geometric_phase_rad > 0.0);
    }
}

#[test]
fn test_fractional_valley_chern_and_charge() {
    let mut params = FractionalValleyParams::default();
    params.valley_staggering_mhz = 18.0;
    params.waveguide_length_um = 120.0;

    let (metrics, spectrum, waveguide_points) = FractionalValleySolver::solve(&params);

    assert_eq!(metrics.valley_chern_contrast, 2);
    assert!((metrics.fractional_valley_charge - 1.0 / 3.0).abs() <= 0.01);
    assert!(metrics.forward_insertion_loss_db <= 0.35);
    assert!(metrics.reverse_chiral_isolation_db >= 42.0);
    assert!(metrics.directivity_db >= 40.0);
    assert!(metrics.defect_immunity_ratio >= 0.95);
    assert!(metrics.chiral_group_velocity_ms > 2500.0);

    assert!(!spectrum.is_empty());
    assert!(!waveguide_points.is_empty());
}

#[test]
fn test_cryogenic_coprocessor_at_15mk() {
    let params = HolonomicCryoParams::default();
    let (metrics, spectrum, nodes) = HolonomicCryoSolver::solve(&params);

    assert!(metrics.thermal_phonon_occupancy <= 1.0e-4);
    assert!(metrics.quadrature_squeezing_db >= 7.0);
    assert!(metrics.duan_simon_nullifier <= 0.35);
    assert!(metrics.duan_simon_nullifier < 1.0);
    assert!(metrics.entanglement_swap_fidelity >= 0.995);
    assert!(metrics.dispersive_doublet_splitting_mhz >= 5.0);
    assert!(metrics.readout_snr_db >= 17.0);
    assert!(metrics.single_shot_readout_fidelity >= 0.998);
    assert!(metrics.cryo_power_dissipation_mw <= 0.80);

    assert_eq!(nodes.len(), params.num_repeater_nodes);
    assert!(!spectrum.is_empty());
}

#[test]
fn test_spatial_points_and_trajectories() {
    let params = HolonomicQubitRepeaterParams::default();
    let solution = HolonomicQubitRepeaterProcessor::solve(&params);

    assert!(!solution.spatial_points.is_empty());
    let corners: Vec<_> = solution.spatial_points.iter().filter(|p| p.is_corner).collect();
    assert_eq!(corners.len(), 4);
    for c in corners {
        assert!(c.intensity >= 0.85);
    }

    assert_eq!(solution.loop_points.len(), 60);
    for pt in solution.loop_points {
        assert!(pt.fidelity_instant >= 0.990);
    }

    assert_eq!(solution.repeater_nodes.len(), 4);
    for node in solution.repeater_nodes {
        assert!(node.local_squeezing_db >= 7.0);
        assert!(node.fidelity >= 0.990);
    }
}

#[test]
fn test_10_point_rigorous_physics_audit() {
    let params = HolonomicQubitRepeaterParams::default();
    let audit = HolonomicQubitRepeaterProcessor::audit_coprocessor(&params);

    assert_eq!(audit.total_count, 10);
    assert_eq!(audit.passed_count, 10);
    assert!(audit.is_all_pass());

    assert!(audit.corner_energy_confinement_pass);
    assert!(audit.non_abelian_commutator_pass);
    assert!(audit.holonomic_gate_fidelity_pass);
    assert!(audit.adiabatic_leakage_suppression_pass);
    assert!(audit.valley_chern_contrast_pass);
    assert!(audit.fractional_valley_charge_pass);
    assert!(audit.reverse_chiral_isolation_pass);
    assert!(audit.thermal_phonon_occupancy_pass);
    assert!(audit.duan_simon_nullifier_pass);
    assert!(audit.dispersive_readout_snr_pass);
}
