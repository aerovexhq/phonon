#![deny(unsafe_code)]

//! Integration Test Suite for Floquet Chiral Magnon-Phonon Polariton Router & Dissipative Quantum Memory (Phase 455).
//!
//! Validates:
//! 1. Floquet polariton hybridization gap (Delta >= 35.0 MHz) and wavenumber non-reciprocity (Delta_k >= 0.15 um^-1).
//! 2. 4-terminal cyclic circulator crossbar with cross-isolation >= 40.0 dB and bandwidth >= 120.0 MHz.
//! 3. Dynamic synthetic gauge field non-Abelian braiding with Artin error <= 1.0e-5 and fidelity >= 99.9%.
//! 4. Dissipative cryogenic quantum memory state retention lifetime tau_ret >= 60.0 us and thermal occupancy <= 0.05.
//! 5. 10-point rigorous physics audit scoring 10/10 PASS.

use phonon_solver::floquet_magnon_memory::{
    DissipativeMemoryParams, DissipativeMemorySolver, FloquetMagnonMemoryProcessor,
    FloquetPolaritonDispersionParams, FloquetPolaritonDispersionSolver,
    FourTerminalCirculatorParams, FourTerminalCirculatorSolver, SyntheticGaugeBraidParams,
    SyntheticGaugeBraidSolver,
};

#[test]
fn test_floquet_polariton_dispersion_and_non_reciprocity() {
    let params = FloquetPolaritonDispersionParams::default();
    let solver = FloquetPolaritonDispersionSolver::new(params);

    let metrics = solver.evaluate_metrics();
    assert!(
        metrics.polariton_hybridization_gap_mhz >= 35.0,
        "Hybridization gap must be >= 35.0 MHz, got {} MHz",
        metrics.polariton_hybridization_gap_mhz
    );
    assert!(
        metrics.wavenumber_non_reciprocity_um_inv >= 0.15,
        "Wavenumber non-reciprocity must be >= 0.15 um^-1, got {} um^-1",
        metrics.wavenumber_non_reciprocity_um_inv
    );
    assert!(
        metrics.forward_insertion_loss_db <= 0.40,
        "Forward insertion loss must be <= 0.40 dB, got {} dB",
        metrics.forward_insertion_loss_db
    );
    assert!(
        metrics.backward_isolation_db >= 36.0,
        "Backward isolation must be >= 36.0 dB, got {} dB",
        metrics.backward_isolation_db
    );

    let dispersion = solver.compute_dispersion(40);
    assert_eq!(dispersion.len(), 40);
}

#[test]
fn test_four_terminal_circulator_cyclic_matrix() {
    let params = FourTerminalCirculatorParams::default();
    let solver = FourTerminalCirculatorSolver::new(params);

    let metrics = solver.evaluate_metrics();
    assert!(metrics.forward_insertion_loss_db <= 0.40);
    assert!(
        metrics.cross_terminal_isolation_db >= 40.0,
        "Cross isolation must be >= 40.0 dB, got {} dB",
        metrics.cross_terminal_isolation_db
    );
    assert!(
        metrics.backward_isolation_db >= 36.0,
        "Backward isolation must be >= 36.0 dB, got {} dB",
        metrics.backward_isolation_db
    );
    assert!(
        metrics.port_return_loss_db >= 22.0,
        "Return loss must be >= 22.0 dB, got {} dB",
        metrics.port_return_loss_db
    );
    assert!(
        metrics.circulation_bandwidth_3db_mhz >= 120.0,
        "Circulation 3-dB bandwidth must be >= 120.0 MHz, got {} MHz",
        metrics.circulation_bandwidth_3db_mhz
    );
    assert!(
        metrics.corner_defect_transmission_pct >= 95.0,
        "Corner defect transmission must be >= 95.0%, got {}%",
        metrics.corner_defect_transmission_pct
    );

    let s_params = solver.compute_s_parameters(40);
    assert_eq!(s_params.len(), 40);
}

#[test]
fn test_synthetic_gauge_field_braiding_artin() {
    let params = SyntheticGaugeBraidParams::default();
    let solver = SyntheticGaugeBraidSolver::new(params);

    let metrics = solver.evaluate_metrics();
    assert!(
        metrics.artin_relation_error <= 1.0e-5,
        "Artin relation error must be <= 1.0e-5, got {}",
        metrics.artin_relation_error
    );
    assert!(
        metrics.far_commutation_error <= 1.0e-5,
        "Far commutation error must be <= 1.0e-5, got {}",
        metrics.far_commutation_error
    );
    assert!(
        metrics.dynamic_braiding_fidelity_pct >= 99.9,
        "Dynamic braiding fidelity must be >= 99.9%, got {}%",
        metrics.dynamic_braiding_fidelity_pct
    );
    assert!(
        metrics.diabatic_leakage_prob <= 1.0e-4,
        "Diabatic leakage probability must be <= 1.0e-4, got {}",
        metrics.diabatic_leakage_prob
    );

    let traj = solver.compute_synthetic_trajectory(50);
    assert_eq!(traj.len(), 50);
    for pt in traj {
        assert!(pt.protection_gap_mhz >= 10.0);
    }
}

#[test]
fn test_dissipative_quantum_memory_retention() {
    let params = DissipativeMemoryParams::default();
    let solver = DissipativeMemorySolver::new(params);

    let metrics = solver.evaluate_metrics();
    assert!(
        metrics.memory_retention_time_us >= 60.0,
        "Memory retention time must be >= 60.0 us, got {} us",
        metrics.memory_retention_time_us
    );
    assert!(
        metrics.cryogenic_thermal_occupancy <= 0.05,
        "Thermal occupancy must be <= 0.05 quanta, got {}",
        metrics.cryogenic_thermal_occupancy
    );
    assert!(
        metrics.parity_readout_contrast_pct >= 88.0,
        "Parity readout contrast must be >= 88.0%, got {}%",
        metrics.parity_readout_contrast_pct
    );
    assert!(metrics.dissipative_enhancement_factor >= 3.0);

    let curve = solver.compute_retention_curve(40);
    assert_eq!(curve.len(), 40);
}

#[test]
fn test_floquet_magnon_memory_10_point_audit() {
    let processor = FloquetMagnonMemoryProcessor::new(
        FloquetPolaritonDispersionParams::default(),
        FourTerminalCirculatorParams::default(),
        SyntheticGaugeBraidParams::default(),
        DissipativeMemoryParams::default(),
    );

    let audit = processor.evaluate_audit();
    assert_eq!(
        audit.score(),
        (10, 10),
        "Audit report must pass 10/10 criteria, got {:?}",
        audit
    );
    assert!(audit.is_pass());
}
