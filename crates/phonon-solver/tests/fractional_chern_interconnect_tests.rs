#![deny(unsafe_code)]

//! Unit and physics integration tests for Phase 440:
//! Quantum Metamaterial Fractional Chern Insulator & Non-Abelian Parafermion Braiding Interconnect.

use phonon_solver::fractional_chern_interconnect::{
    BraidingInterconnectParams, BraidingInterconnectSolver, FractionalChernInterconnectProcessor,
    FractionalChernLatticeParams, FractionalChernLatticeSolver, ParafermionDomainWallParams,
    ParafermionDomainWallSolver, QuditGateKind,
};

#[test]
fn test_fractional_chern_lattice_quantization_and_edge_transport() {
    let mut params = FractionalChernLatticeParams::default();
    params.intracell_hopping_mhz = 2.5;
    params.intercell_hopping_mhz = 8.5;
    params.has_edge_obstacle = false;

    let solver = FractionalChernLatticeSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // 1. Quantized Fractional Chern Number C = 1/3
    assert!(
        (metrics.fractional_chern_number - 1.0 / 3.0).abs() <= 0.005,
        "Fractional Chern number {:.4} deviated from 1/3",
        metrics.fractional_chern_number
    );

    // 2. Bulk topological acoustic bandgap >= 3.0 MHz
    assert!(
        metrics.bulk_bandgap_mhz >= 3.0,
        "Bulk bandgap {:.2} MHz is below 3.0 MHz threshold",
        metrics.bulk_bandgap_mhz
    );

    // 3. Fractional quasiparticle charge e^* = 1/3
    assert_eq!(metrics.quasiparticle_charge_e_star, 1.0 / 3.0);

    // 4. Boundary modal energy confinement >= 85.0%
    assert!(
        metrics.edge_confinement_ratio >= 0.85,
        "Edge confinement ratio {:.3} is below 85.0%",
        metrics.edge_confinement_ratio
    );

    // 5. Backscattering immunity across edge defect
    let mut defect_params = params.clone();
    defect_params.has_edge_obstacle = true;
    let defect_solver = FractionalChernLatticeSolver::new(defect_params);
    let defect_metrics = defect_solver.evaluate_metrics();
    assert!(
        defect_metrics.defect_transmission_ratio >= 0.95,
        "Defect transmission ratio {:.3} is below 0.95",
        defect_metrics.defect_transmission_ratio
    );

    // 6. Dispersion curve computation
    let dispersion = solver.compute_edge_dispersion(32);
    assert_eq!(dispersion.len(), 32);
    assert!(dispersion[0].group_velocity_m_s > 0.0);

    // 7. Real-space lattice field generation
    let field = solver.generate_lattice_field();
    assert!(!field.is_empty());
}

#[test]
fn test_parafermion_domain_wall_zero_modes_and_algebra() {
    let params = ParafermionDomainWallParams::default();
    let solver = ParafermionDomainWallSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // 1. Commutation algebra residual |alpha_j * alpha_k - omega * alpha_k * alpha_j| <= 1e-8
    assert!(
        metrics.algebra_commutation_residual <= 1.0e-8,
        "Algebra commutation residual {:.2e} exceeds 1e-8 threshold",
        metrics.algebra_commutation_residual
    );

    // 2. Spatial energy confinement >= 85.0%
    assert!(
        metrics.spatial_confinement_ratio >= 0.85,
        "Zero-mode confinement ratio {:.3} is below 85.0%",
        metrics.spatial_confinement_ratio
    );

    // 3. Ground-state topological degeneracy Z_3^2 = 9 for 4 zero modes
    assert_eq!(metrics.ground_state_degeneracy, 9);

    // 4. Localized zero modes array
    let modes = solver.get_zero_modes();
    assert_eq!(modes.len(), 4);
    assert!(modes[0].x_pos_um < modes[1].x_pos_um);
    assert!(modes[1].x_pos_um < modes[2].x_pos_um);

    // 5. Wavefunction profile computation
    let wave = solver.compute_wavefunction_profiles(40);
    assert_eq!(wave.len(), 40);
    assert!(wave[0].total_energy_density > 0.0);
}

#[test]
fn test_non_abelian_braiding_artin_relations_and_qudit_gates() {
    let mut params = BraidingInterconnectParams::default();
    params.braid_duration_ns = 35.0;

    let gates = [
        QuditGateKind::GeneralizedHadamard,
        QuditGateKind::PhaseS3,
        QuditGateKind::ShiftX3,
        QuditGateKind::ClockZ3,
        QuditGateKind::CSumTwoQutrit,
    ];

    for gate in gates {
        params.target_gate = gate;
        let solver = BraidingInterconnectSolver::new(params.clone());
        let metrics = solver.evaluate_metrics();

        // 1. Artin non-Abelian braid relation residual <= 1e-10
        assert!(
            metrics.artin_braid_residual <= 1.0e-10,
            "Artin braid residual {:.2e} exceeds 1e-10 threshold",
            metrics.artin_braid_residual
        );

        // 2. Compiled qudit gate process fidelity >= 0.999
        assert!(
            metrics.compiled_gate_fidelity >= 0.999,
            "Gate {:?} fidelity {:.4} is below 0.999",
            gate,
            metrics.compiled_gate_fidelity
        );

        // 3. Braiding latency <= 40.0 ns
        assert!(
            metrics.braiding_latency_ns <= 40.0,
            "Braiding latency {:.2} ns exceeds 40.0 ns threshold",
            metrics.braiding_latency_ns
        );

        // 4. Cryogenic dispersive readout SNR >= 18.0 dB
        assert!(
            metrics.readout_snr_db >= 18.0,
            "Readout SNR {:.2} dB is below 18.0 dB threshold",
            metrics.readout_snr_db
        );

        // 5. QND readout fidelity >= 0.998
        assert!(
            metrics.qnd_readout_fidelity >= 0.998,
            "QND readout fidelity {:.4} is below 0.998",
            metrics.qnd_readout_fidelity
        );
    }

    // Trajectory generation
    let solver = BraidingInterconnectSolver::new(params.clone());
    let trajs = solver.compute_braid_trajectories(25);
    assert_eq!(trajs.len(), 25);

    // Readout spectrum generation
    let spectrum = solver.compute_readout_spectrum(60);
    assert_eq!(spectrum.len(), 60);
}

#[test]
fn test_fractional_chern_interconnect_10_point_audit_all_passed() {
    let processor = FractionalChernInterconnectProcessor::default();
    let audit = processor.audit_interconnect();

    assert_eq!(
        audit.total_score, 10,
        "Audit score {} / 10 does not meet full pass requirement",
        audit.total_score
    );
    assert!(audit.all_passed);
    assert!(audit.chern_quantization_pass);
    assert!(audit.bulk_bandgap_pass);
    assert!(audit.quasiparticle_charge_pass);
    assert!(audit.backscattering_immunity_pass);
    assert!(audit.commutation_algebra_pass);
    assert!(audit.domain_wall_confinement_pass);
    assert!(audit.artin_braid_relation_pass);
    assert!(audit.gate_fidelity_pass);
    assert!(audit.cryogenic_readout_snr_pass);
    assert!(audit.braiding_latency_pass);
}
