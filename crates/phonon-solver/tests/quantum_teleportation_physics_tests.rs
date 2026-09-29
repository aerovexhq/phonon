//! Analytical and unit validation tests for superconducting optomechanical
//! quantum teleportation across phononic crystal waveguides (Phase 116).

use phonon_models::quantum_teleportation_waveguide::QuantumTeleportationParams;
use phonon_solver::quantum_teleportation_waveguide::QuantumTeleportationSolver;

#[test]
fn test_quantum_teleportation_fidelity_exceeds_classical_limit() {
    let params = QuantumTeleportationParams::default();
    let solver = QuantumTeleportationSolver::new(params);
    let fidelity = solver.compute_teleportation_fidelity();

    // Classical teleportation limit is 2/3 (~0.6667)
    // Target quantum teleportation fidelity >= 0.850 (85.0%)
    assert!(
        fidelity >= 0.850,
        "Teleportation fidelity must be >= 0.850, got {:.4}",
        fidelity
    );
    assert!(
        fidelity <= 0.995,
        "Teleportation fidelity cannot exceed clamp bound 0.995, got {:.4}",
        fidelity
    );
    assert!(
        fidelity > 2.0 / 3.0,
        "Teleportation fidelity must strictly surpass classical limit 2/3"
    );

    // Higher cooperativity should yield higher or equal fidelity
    let mut high_coop_params = params;
    high_coop_params.piezoelectric_cooperativity = 150.0;
    let high_coop_solver = QuantumTeleportationSolver::new(high_coop_params);
    assert!(
        high_coop_solver.compute_teleportation_fidelity() >= fidelity,
        "Higher piezoelectric cooperativity must increase or preserve fidelity"
    );

    // Lower waveguide loss should yield higher or equal fidelity
    let mut low_loss_params = params;
    low_loss_params.waveguide_loss_db_per_cm = 0.005;
    let low_loss_solver = QuantumTeleportationSolver::new(low_loss_params);
    assert!(
        low_loss_solver.compute_teleportation_fidelity() >= fidelity,
        "Lower acoustic loss must increase or preserve fidelity"
    );
}

#[test]
fn test_entanglement_distillation_purity() {
    let params = QuantumTeleportationParams::default();
    let solver = QuantumTeleportationSolver::new(params);
    let purity = solver.compute_entanglement_distillation_purity();

    // Target entanglement distillation purity >= 0.920 (92.0%)
    assert!(
        purity >= 0.920,
        "Distillation purity must be >= 0.920, got {:.4}",
        purity
    );
    assert!(
        purity <= 0.999,
        "Distillation purity cannot exceed clamp bound 0.999, got {:.4}",
        purity
    );

    // Distillation purity should improve with higher fidelity input
    let mut superior_params = params;
    superior_params.piezoelectric_cooperativity = 180.0;
    superior_params.bell_measurement_efficiency = 0.995;
    superior_params.waveguide_loss_db_per_cm = 0.002;
    let superior_solver = QuantumTeleportationSolver::new(superior_params);
    assert!(
        superior_solver.compute_entanglement_distillation_purity() >= purity,
        "Higher fidelity must yield higher or equal distillation purity"
    );
}

#[test]
fn test_waveguide_propagation_loss_compliance() {
    let params = QuantumTeleportationParams::default();
    let solver = QuantumTeleportationSolver::new(params);
    let loss_db = solver.compute_waveguide_propagation_loss_db_per_cm();

    // Target waveguide acoustic propagation loss <= 0.050 dB/cm
    assert!(
        loss_db <= 0.050,
        "Waveguide loss must be <= 0.050 dB/cm, got {:.4} dB/cm",
        loss_db
    );
    assert!(
        loss_db >= 0.001,
        "Waveguide loss must be >= 0.001 dB/cm, got {:.4} dB/cm",
        loss_db
    );
}

#[test]
fn test_quantum_memory_coherence_time() {
    let params = QuantumTeleportationParams::default();
    let solver = QuantumTeleportationSolver::new(params);
    let t2_ms = solver.compute_quantum_memory_coherence_time_ms();

    // Target quantum memory coherence time T2 >= 1.00 ms
    assert!(
        t2_ms >= 1.00,
        "Quantum memory T2 must be >= 1.00 ms, got {:.4} ms",
        t2_ms
    );
    assert!(
        t2_ms <= 50.0,
        "Quantum memory T2 cannot exceed clamp bound 50.0 ms, got {:.4} ms",
        t2_ms
    );
}

#[test]
fn test_bell_state_concurrence() {
    let params = QuantumTeleportationParams::default();
    let solver = QuantumTeleportationSolver::new(params);
    let concurrence = solver.compute_bell_state_concurrence();

    // Target Bell-state concurrence >= 0.800
    assert!(
        concurrence >= 0.800,
        "Bell-state concurrence must be >= 0.800, got {:.4}",
        concurrence
    );
    assert!(
        concurrence <= 0.995,
        "Bell-state concurrence cannot exceed clamp bound 0.995, got {:.4}",
        concurrence
    );
}

#[test]
fn test_full_physical_compliance_evaluation() {
    let params = QuantumTeleportationParams::default();
    let solver = QuantumTeleportationSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default quantum teleportation parameters must be physically compliant"
    );
    assert!(
        metrics.teleportation_fidelity >= 0.850,
        "Fidelity requirement violated: {:.4}",
        metrics.teleportation_fidelity
    );
    assert!(
        metrics.entanglement_distillation_purity >= 0.920,
        "Distillation purity requirement violated: {:.4}",
        metrics.entanglement_distillation_purity
    );
    assert!(
        metrics.waveguide_propagation_loss_db_per_cm <= 0.050,
        "Waveguide loss requirement violated: {:.4}",
        metrics.waveguide_propagation_loss_db_per_cm
    );
    assert!(
        metrics.quantum_memory_coherence_time_ms >= 1.00,
        "Quantum memory T2 requirement violated: {:.4}",
        metrics.quantum_memory_coherence_time_ms
    );
    assert!(
        metrics.bell_state_concurrence >= 0.800,
        "Bell concurrence requirement violated: {:.4}",
        metrics.bell_state_concurrence
    );
}

#[test]
fn test_parameter_clamping_and_bounds() {
    // Test that extreme parameter configurations are properly clamped
    let out_of_bounds = QuantumTeleportationParams::new(
        1.0,     // below min 3.0 -> clamped to 3.0
        100.0,   // above max 20.0 -> clamped to 20.0
        0.50,    // above max 0.10 -> clamped to 0.10
        1000.0,  // above max 200.0 -> clamped to 200.0
        5.0,     // above max 3.0 -> clamped to 3.0
        1.5,     // above max 0.999 -> clamped to 0.999
        200.0,   // above max 50.0 -> clamped to 50.0
        500.0,   // above max 100.0 -> clamped to 100.0
    );

    assert_eq!(out_of_bounds.qubit_frequency_ghz, 3.0);
    assert_eq!(out_of_bounds.waveguide_length_cm, 20.0);
    assert_eq!(out_of_bounds.waveguide_loss_db_per_cm, 0.10);
    assert_eq!(out_of_bounds.piezoelectric_cooperativity, 200.0);
    assert_eq!(out_of_bounds.two_mode_squeezing_param, 3.0);
    assert_eq!(out_of_bounds.bell_measurement_efficiency, 0.999);
    assert_eq!(out_of_bounds.quantum_memory_t2_ms, 50.0);
    assert_eq!(out_of_bounds.operating_temp_m_k, 100.0);
}
