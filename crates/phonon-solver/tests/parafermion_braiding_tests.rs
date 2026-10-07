#![deny(unsafe_code)]

//! Unit and Physics Test Suite for Phase 422:
//! Phonon Studio Quantum Acoustic Non-Abelian Parafermion Braiding
//! & Fractional Chern Number Interconnect.

use phonon_solver::parafermion_braiding::{
    FractionalChernNumber, FractionalReadoutParams, FractionalReadoutSolver,
    ParafermionBraidEngine, ParafermionBraidingParams, ParafermionGateKind,
    ParafermionLatticeParams, ParafermionLatticeSolver, ParafermionOrder, ParafermionParams,
    ParafermionProcessor,
};

#[test]
fn test_fractional_chern_lattice_and_gap() {
    let params = ParafermionLatticeParams::default();
    let solver = ParafermionLatticeSolver::new(params);

    assert_eq!(solver.params.order, ParafermionOrder::Z3);
    assert_eq!(solver.params.chern_number, FractionalChernNumber::OneThird);

    let gap = solver.protection_gap_mhz();
    assert!(
        gap >= 2.5,
        "Fractional topological protection gap {:?} MHz must be >= 2.5 MHz",
        gap
    );

    let xi = solver.localization_length_mm();
    assert!(xi > 0.5 && xi < 3.0);
}

#[test]
fn test_parafermion_zero_modes_and_confinement() {
    let solver = ParafermionLatticeSolver::new(ParafermionLatticeParams::default());
    let modes = solver.solve_parafermion_modes();

    assert_eq!(modes.len(), 6); // 6 parafermions for 2 logical qutrits
    for mode in &modes {
        assert!(
            mode.confinement_ratio >= 0.85,
            "Mode {} confinement {:?} must be >= 85%",
            mode.mode_id,
            mode.confinement_ratio
        );
        assert!(
            mode.energy_splitting_khz <= 5.0,
            "Mode {} hybridization splitting {:?} kHz must be <= 5.0 kHz",
            mode.mode_id,
            mode.energy_splitting_khz
        );
    }

    let grid = solver.generate_realspace_intensity();
    assert_eq!(grid.len(), 24);
    assert_eq!(grid[0].len(), 24);
}

#[test]
fn test_commutation_algebra_and_unitarity() {
    let solver_z3 = ParafermionLatticeSolver::new(ParafermionLatticeParams {
        order: ParafermionOrder::Z3,
        ..Default::default()
    });
    assert!(solver_z3.verify_commutation_algebra());

    let solver_z4 = ParafermionLatticeSolver::new(ParafermionLatticeParams {
        order: ParafermionOrder::Z4,
        chern_number: FractionalChernNumber::OneHalf,
        ..Default::default()
    });
    assert!(solver_z4.verify_commutation_algebra());
}

#[test]
fn test_artin_non_abelian_braid_relation() {
    let engine = ParafermionBraidEngine::new(ParafermionBraidingParams::default());
    let (pass, diff) = engine.verify_artin_braid_relation();

    assert!(pass, "Artin non-Abelian braid relation must be satisfied");
    assert!(diff < 1e-10, "Braid difference norm {:?} must be < 1e-10", diff);
}

#[test]
fn test_universal_gate_synthesis_and_high_fidelity() {
    let engine = ParafermionBraidEngine::new(ParafermionBraidingParams::default());

    let gates = [
        ParafermionGateKind::Hadamard,
        ParafermionGateKind::PhaseS,
        ParafermionGateKind::NonCliffordT,
        ParafermionGateKind::PauliX,
        ParafermionGateKind::PauliZ,
        ParafermionGateKind::ControlledSum,
    ];

    for gate_kind in gates {
        let compiled = engine.compile_gate(gate_kind);
        assert!(
            compiled.process_fidelity >= 0.999,
            "Gate {:?} fidelity {:?} must be >= 0.999",
            gate_kind,
            compiled.process_fidelity
        );
        assert!(
            compiled.diabatic_leakage_error < 1e-4,
            "Gate {:?} diabatic leakage {:?} must be < 1e-4",
            gate_kind,
            compiled.diabatic_leakage_error
        );
        assert!(!compiled.braid_word.is_empty());

        let preview = engine.generate_unitary_preview(gate_kind);
        assert_eq!(preview.len(), 3);
        assert_eq!(preview[0].len(), 3);
    }
}

#[test]
fn test_cryogenic_fractional_charge_readout_snr_and_fidelity() {
    let readout = FractionalReadoutSolver::new(FractionalReadoutParams::default());

    let snr = readout.calculate_snr_db();
    assert!(
        snr >= 18.0,
        "Fractional charge readout SNR {:?} dB must be >= 18.0 dB",
        snr
    );

    let fidelity = readout.calculate_readout_fidelity();
    assert!(
        fidelity >= 0.998,
        "Single-shot readout fidelity {:?} must be >= 0.998",
        fidelity
    );

    let spectrum = readout.generate_transmission_spectrum(41);
    assert_eq!(spectrum.len(), 41);
}

#[test]
fn test_master_processor_10_point_physics_audit() {
    let processor = ParafermionProcessor::new(ParafermionParams::default());
    let report = processor.audit_parafermion_processor();

    assert_eq!(
        report.total_pass_score, 10,
        "10-Point physics audit must achieve 10/10 PASS score"
    );
    assert!(report.all_passed);
    assert!(report.fractional_gap_pass);
    assert!(report.parafermion_confinement_pass);
    assert!(report.commutation_algebra_pass);
    assert!(report.artin_braid_relation_pass);
    assert!(report.non_clifford_t_fidelity_pass);
    assert!(report.hadamard_h3_fidelity_pass);
    assert!(report.diabatic_leakage_suppression_pass);
    assert!(report.dispersive_fractional_split_pass);
    assert!(report.fractional_readout_snr_pass);
    assert!(report.single_shot_fidelity_pass);
}
