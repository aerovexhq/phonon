#![deny(unsafe_code)]

//! Physical test suite for Phase 444:
//! Topological Acoustic Boundary-Mode Valley-Hall Quantum Router & Entanglement Concentrator.

use phonon_solver::valley_quantum_router::{
    EntanglementConcentratorParams, EntanglementConcentratorSolver, RouterChannelTarget,
    TopologicalValleyQuantumProcessor, ValleyHallLatticeParams, ValleyHallLatticeSolver,
    ValleyQuantumRouterParams, ValleyQuantumRouterSolver,
};

#[test]
fn test_valley_hall_lattice_dispersion_and_topological_gap() {
    let params = ValleyHallLatticeParams::default();
    let solver = ValleyHallLatticeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.valley_bulk_gap_mhz >= 3.0,
        "Valley bulk gap {:.2} MHz is less than 3.0 MHz threshold",
        metrics.valley_bulk_gap_mhz
    );
    assert_eq!(metrics.valley_chern_difference, 1.0);
    assert!(
        metrics.edge_group_velocity_ms >= 1200.0,
        "Group velocity {:.1} m/s is less than 1200 m/s threshold",
        metrics.edge_group_velocity_ms
    );
    assert!(
        metrics.modal_confinement_percent >= 88.0,
        "Modal confinement {:.1}% is below 88%",
        metrics.modal_confinement_percent
    );
    assert!(
        metrics.corner_transmission_ratio >= 0.95,
        "Corner transmission ratio {:.3} is below 0.95",
        metrics.corner_transmission_ratio
    );
    assert!(
        metrics.defect_immunity_ratio >= 0.95,
        "Defect immunity ratio {:.3} is below 0.95",
        metrics.defect_immunity_ratio
    );

    let dispersion = solver.compute_dispersion(32);
    assert_eq!(dispersion.len(), 32);
    for p in &dispersion {
        assert!(p.bulk_upper_mhz > p.bulk_lower_mhz);
    }

    let spatial = solver.compute_spatial_profile(32);
    assert_eq!(spatial.len(), 32);
    let peak = spatial.iter().map(|p| p.pressure_amplitude).fold(0.0f64, f64::max);
    assert!((peak - 1.0).abs() < 0.05);
}

#[test]
fn test_valley_quantum_router_switching_and_crosstalk() {
    let mut params = ValleyQuantumRouterParams::default();
    params.target_channel = RouterChannelTarget::Port2Deflected60Kp;

    let solver = ValleyQuantumRouterSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.target_transmission_percent >= 90.0,
        "Target transmission {:.1}% is below 90%",
        metrics.target_transmission_percent
    );
    assert!(
        metrics.insertion_loss_db <= 0.50,
        "Insertion loss {:.2} dB exceeds 0.50 dB",
        metrics.insertion_loss_db
    );
    assert!(
        metrics.crosstalk_isolation_db >= 35.0,
        "Crosstalk isolation {:.1} dB is below 35 dB",
        metrics.crosstalk_isolation_db
    );
    assert!(
        metrics.switching_latency_ns <= 3.0,
        "Switching latency {:.2} ns exceeds 3.0 ns threshold",
        metrics.switching_latency_ns
    );
    assert!(metrics.routing_bandwidth_mhz >= 1.5);

    let spectrum = solver.compute_spectral_response(32);
    assert_eq!(spectrum.len(), 32);

    let dynamic = solver.compute_dynamic_trace(32);
    assert_eq!(dynamic.len(), 32);
    let last = dynamic.last().unwrap();
    assert!(last.port2_power_norm > last.port1_power_norm);
}

#[test]
fn test_entanglement_concentrator_distillation_and_readout() {
    let params = EntanglementConcentratorParams::default();
    let solver = EntanglementConcentratorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.concentrated_concurrence >= 0.96,
        "Distilled concurrence {:.3} is below 0.96",
        metrics.concentrated_concurrence
    );
    assert!(
        metrics.bell_state_fidelity >= 0.995,
        "Bell fidelity {:.4} is below 0.995",
        metrics.bell_state_fidelity
    );
    assert!(
        metrics.success_probability_percent >= 25.0,
        "Success probability {:.1}% is below 25%",
        metrics.success_probability_percent
    );
    assert!(
        metrics.dispersive_readout_snr_db >= 18.0,
        "Readout SNR {:.1} dB is below 18.0 dB",
        metrics.dispersive_readout_snr_db
    );
    assert!(metrics.qnd_fidelity >= 0.998);

    let rho = solver.compute_density_matrix();
    let tr = rho.rho_00 + rho.rho_01 + rho.rho_10 + rho.rho_11;
    assert!(
        (tr - 1.0).abs() < 1e-3,
        "Trace of density matrix {:.4} deviates from 1.0",
        tr
    );
    assert!(rho.re_rho_01_10 > 0.45);

    let readout = solver.compute_readout_spectrum(32);
    assert_eq!(readout.len(), 32);

    let yield_curve = solver.compute_distillation_yield(32);
    assert_eq!(yield_curve.len(), 32);
    let end_yield = yield_curve.last().unwrap();
    assert!(end_yield.concurrence >= 0.96);
}

#[test]
fn test_topological_valley_quantum_system_10_point_audit() {
    let processor = TopologicalValleyQuantumProcessor::default();
    let audit = processor.audit_system();

    assert!(audit.valley_bulk_gap_pass, "Valley bulk gap failed");
    assert!(audit.valley_chern_difference_pass, "Chern difference failed");
    assert!(audit.edge_group_velocity_pass, "Edge group velocity failed");
    assert!(audit.corner_transmission_pass, "Corner transmission failed");
    assert!(audit.crosstalk_isolation_pass, "Crosstalk isolation failed");
    assert!(audit.target_transmission_pass, "Target transmission failed");
    assert!(audit.switching_latency_pass, "Switching latency failed");
    assert!(audit.concentrated_concurrence_pass, "Concentrated concurrence failed");
    assert!(audit.bell_fidelity_pass, "Bell fidelity failed");
    assert!(audit.dispersive_readout_snr_pass, "Dispersive readout SNR failed");

    assert_eq!(audit.total_score, 10);
    assert!(audit.all_passed);
}
