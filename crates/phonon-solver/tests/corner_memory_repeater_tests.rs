#![deny(unsafe_code)]

use phonon_solver::corner_memory_repeater::{
    CornerMemoryRepeaterProcessor, CvQuantumRepeaterParams, CvQuantumRepeaterSolver,
    HigherOrderCornerMemoryParams, HigherOrderCornerMemorySolver,
    SyntheticGaugeTransductionParams, SyntheticGaugeTransductionSolver,
};

#[test]
fn test_corner_state_memory_quadrupole_and_confinement() {
    let params = HigherOrderCornerMemoryParams::default();
    let solver = HigherOrderCornerMemorySolver::new(params);

    let metrics = solver.compute_metrics();
    assert_eq!(
        metrics.quadrupole_moment_qxy, 0.5,
        "Quadrupole moment must be quantized to 0.5 in SOTI phase"
    );
    assert!(
        metrics.bulk_bandgap_mhz >= 15.0,
        "Bulk bandgap must be >= 15.0 MHz, got {} MHz",
        metrics.bulk_bandgap_mhz
    );
    assert!(
        metrics.corner_confinement_ratio >= 0.90,
        "Corner confinement ratio must be >= 0.90, got {}",
        metrics.corner_confinement_ratio
    );
    assert!(
        metrics.storage_lifetime_ms >= 2.0,
        "Storage lifetime must be >= 2.0 ms, got {} ms",
        metrics.storage_lifetime_ms
    );
    assert!(
        metrics.retrieval_efficiency >= 0.92,
        "Retrieval efficiency must be >= 0.92, got {}",
        metrics.retrieval_efficiency
    );
    assert!(
        metrics.thermal_phonon_occupancy <= 1.0e-4,
        "Thermal phonon occupancy at 15 mK must be <= 1.0e-4, got {}",
        metrics.thermal_phonon_occupancy
    );

    // Trivial phase check (gamma > lambda)
    let trivial_params = HigherOrderCornerMemoryParams {
        intracell_gamma_mhz: 12.0,
        intercell_lambda_mhz: 3.0,
        ..HigherOrderCornerMemoryParams::default()
    };
    let trivial_solver = HigherOrderCornerMemorySolver::new(trivial_params);
    assert_eq!(trivial_solver.calculate_quadrupole_moment(), 0.0);
    assert!(trivial_solver.calculate_corner_confinement() < 0.20);
}

#[test]
fn test_synthetic_gauge_transduction_chiral_isolation() {
    let params = SyntheticGaugeTransductionParams::default();
    let solver = SyntheticGaugeTransductionSolver::new(params);

    let metrics = solver.compute_metrics();
    assert!(
        metrics.forward_insertion_loss_db <= 0.35,
        "Forward insertion loss must be <= 0.35 dB, got {} dB",
        metrics.forward_insertion_loss_db
    );
    assert!(
        metrics.reverse_chiral_isolation_db >= 40.0,
        "Reverse chiral isolation must be >= 40.0 dB, got {} dB",
        metrics.reverse_chiral_isolation_db
    );
    assert!(
        metrics.directivity_db >= 40.0,
        "Directivity must be >= 40.0 dB, got {} dB",
        metrics.directivity_db
    );
    assert!(
        metrics.state_transfer_fidelity >= 0.995,
        "Coherent state transfer fidelity must be >= 0.995, got {}",
        metrics.state_transfer_fidelity
    );

    let spectrum = solver.generate_transmission_spectrum(30);
    assert_eq!(spectrum.len(), 30);
    for pt in &spectrum {
        assert!(pt.chiral_isolation_db >= 0.0);
    }
}

#[test]
fn test_cv_quantum_repeater_squeezing_and_swapping() {
    let params = CvQuantumRepeaterParams::default();
    let solver = CvQuantumRepeaterSolver::new(params);

    let metrics = solver.compute_metrics();
    assert!(
        metrics.squeezing_depth_db >= 6.5,
        "Squeezing depth must be >= 6.5 dB, got {} dB",
        metrics.squeezing_depth_db
    );
    assert!(
        metrics.duan_simon_nullifier <= 0.40,
        "Duan-Simon EPR nullifier must be <= 0.40, got {}",
        metrics.duan_simon_nullifier
    );
    assert!(
        metrics.duan_simon_nullifier < 1.0,
        "EPR inseparability requires nullifier < 1.0"
    );
    assert!(
        metrics.entanglement_swapping_fidelity >= 0.990,
        "Entanglement swapping fidelity must be >= 0.990, got {}",
        metrics.entanglement_swapping_fidelity
    );
    assert!(
        metrics.total_cryo_power_mw <= 1.5,
        "Total Cryo-CMOS repeater power must be <= 1.5 mW, got {} mW",
        metrics.total_cryo_power_mw
    );

    let nodes = solver.generate_repeater_nodes();
    assert_eq!(nodes.len(), 4);
    for (i, node) in nodes.iter().enumerate() {
        assert_eq!(node.node_id, i + 1);
        assert!(node.local_squeezing_db >= 6.5);
    }
}

#[test]
fn test_corner_memory_repeater_10_point_physics_audit() {
    let processor = CornerMemoryRepeaterProcessor::default();
    let audit = processor.audit();

    assert!(audit.quadrupole_bulk_moment_pass, "Check 1 failed");
    assert!(audit.corner_state_confinement_pass, "Check 2 failed");
    assert!(audit.corner_storage_lifetime_pass, "Check 3 failed");
    assert!(audit.thermal_occupancy_pass, "Check 4 failed");
    assert!(audit.chiral_isolation_pass, "Check 5 failed");
    assert!(audit.transduction_loss_pass, "Check 6 failed");
    assert!(audit.state_transfer_fidelity_pass, "Check 7 failed");
    assert!(audit.quadrature_squeezing_pass, "Check 8 failed");
    assert!(audit.entanglement_swapping_pass, "Check 9 failed");
    assert!(audit.duan_simon_nullifier_pass, "Check 10 failed");

    assert_eq!(audit.passed_count, 10);
    assert_eq!(audit.total_count, 10);
    assert!(audit.is_all_pass(), "10-point audit must pass completely");
}

#[test]
fn test_spatial_distribution_and_polar_profile() {
    let memory_solver = HigherOrderCornerMemorySolver::new(HigherOrderCornerMemoryParams::default());
    let spatial_points = memory_solver.generate_spatial_distribution();
    assert_eq!(spatial_points.len(), 6 * 6 * 4);

    let corner_count = spatial_points.iter().filter(|p| p.is_corner).count();
    assert_eq!(corner_count, 4, "Must identify exactly 4 outer corner sites");

    let repeater_solver = CvQuantumRepeaterSolver::new(CvQuantumRepeaterParams::default());
    let polar_profile = repeater_solver.generate_squeezing_polar_profile(36);
    assert_eq!(polar_profile.len(), 36);

    let min_var = polar_profile
        .iter()
        .map(|p| p.variance)
        .fold(f64::INFINITY, f64::min);
    assert!(
        min_var < 0.5,
        "Minimum variance must be below SQL reference 0.5, got {}",
        min_var
    );
}
