#![deny(unsafe_code)]

//! Unit tests for Phase 413: Synthetic Dimension Chern Insulator & Multiplexed Router.
//!
//! Verifies:
//! - Synthetic magnetic flux and quantized first Chern number |C_1| = 1.
//! - Chiral synthetic edge state confinement >= 85% and frequency ladder pumping.
//! - Harper-Hofstadter bulk bandgap Delta_bulk >= 1.5 MHz.
//! - 4D synthetic topology, second Chern number |C_2| = 1, and Fermi arc transport.
//! - Wavepacket trajectory tracking through physical and synthetic dimensions.
//! - Multiplexed router insertion loss <= 0.8 dB and inter-channel isolation >= 35 dB.
//! - Topological defect immunity T_defect / T_clean >= 95%.
//! - Comprehensive 10-point physics audit checklist with 10/10 PASS score.

use std::f64::consts::PI;
use phonon_solver::synthetic_dimension_router::{
    SyntheticDimensionRouter, SyntheticLatticeParams, SyntheticLatticeSolver,
    SyntheticRouterParams, SyntheticMultiplexedRouter, WeylSyntheticParams,
    WeylTransportSolver,
};

#[test]
fn test_synthetic_magnetic_flux_and_first_chern_number() {
    let mut params = SyntheticLatticeParams::default();
    params.phase_gradient_phi_rad = PI / 2.0;
    let solver = SyntheticLatticeSolver::new(params.clone());

    let metrics = solver.evaluate_metrics();
    assert!((metrics.flux_per_plaquette_ratio - 0.25).abs() < 1e-4);
    assert_eq!(metrics.first_chern_number, 1.0);

    // Test with preset high-Q Chern
    let high_q = SyntheticLatticeParams::preset_high_q_chern();
    let solver_hq = SyntheticLatticeSolver::new(high_q);
    let metrics_hq = solver_hq.evaluate_metrics();
    assert_eq!(metrics_hq.first_chern_number, 1.0);
    assert!(metrics_hq.bulk_bandgap_mhz >= 1.5);
}

#[test]
fn test_chiral_synthetic_edge_confinement_and_ladder_velocity() {
    let params = SyntheticLatticeParams::default();
    let solver = SyntheticLatticeSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.edge_confinement_ratio >= 0.85,
        "Edge confinement ratio must be >= 0.85, got {:.3}",
        metrics.edge_confinement_ratio
    );
    assert!(
        metrics.frequency_ladder_velocity_modes_us.abs() >= 5.0,
        "Ladder velocity magnitude must be >= 5.0 modes/us, got {:.2}",
        metrics.frequency_ladder_velocity_modes_us.abs()
    );
    assert!(
        metrics.synthetic_directivity_db >= 25.0,
        "Synthetic directivity must be >= 25.0 dB, got {:.1} dB",
        metrics.synthetic_directivity_db
    );

    // Spatial profile check
    let profile = solver.generate_spatial_profile();
    assert_eq!(profile.len(), solver.total_sites());
    let boundary_sites = profile.iter().filter(|p| p.is_boundary).count();
    assert!(boundary_sites > 0);
}

#[test]
fn test_harper_hofstadter_bulk_bandgap_and_dispersion() {
    let params = SyntheticLatticeParams::default();
    let solver = SyntheticLatticeSolver::new(params);

    let gap = solver.compute_bulk_bandgap_mhz();
    assert!(gap >= 1.5, "Bulk bandgap must be >= 1.5 MHz, got {:.2} MHz", gap);

    let dispersion = solver.compute_band_dispersion(30);
    assert!(!dispersion.is_empty());
    assert!(dispersion.iter().any(|b| b.band_index == 1)); // edge state present
}

#[test]
fn test_weyl_transport_second_chern_number_and_fermi_arcs() {
    let params = WeylSyntheticParams::default();
    let solver = WeylTransportSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert_eq!(metrics.second_chern_number, 1.0);
    assert!(
        metrics.non_local_transmission_ratio >= 0.80,
        "Non-local transmission must be >= 0.80, got {:.3}",
        metrics.non_local_transmission_ratio
    );
    assert!(metrics.fermi_arc_length_rad >= 0.5);

    let arcs = solver.compute_fermi_arc_dispersion(25);
    assert_eq!(arcs.len(), 26);
    assert!(arcs.iter().all(|a| a.surface_weight >= 0.80));
}

#[test]
fn test_weyl_wavepacket_trajectory_tracking() {
    let params = WeylSyntheticParams::default();
    let solver = WeylTransportSolver::new(params);

    let trajectory = solver.simulate_wavepacket_trajectory(20);
    assert_eq!(trajectory.len(), 21);
    assert!(trajectory.iter().all(|p| p.survival_probability >= 0.70));
    assert!(trajectory.iter().any(|p| p.physical_pos_x > 0.0));
}

#[test]
fn test_multiplexed_router_s_parameters_and_isolation() {
    let params = SyntheticRouterParams::default();
    let router = SyntheticMultiplexedRouter::new(params);
    let metrics = router.evaluate_metrics();

    assert!(
        metrics.insertion_loss_db <= 0.80,
        "Insertion loss must be <= 0.80 dB, got {:.2} dB",
        metrics.insertion_loss_db
    );
    assert!(
        metrics.target_transmission_ratio >= 0.832,
        "Target transmission ratio must be >= 0.832, got {:.3}",
        metrics.target_transmission_ratio
    );
    assert!(
        metrics.inter_channel_isolation_db >= 35.0,
        "Inter-channel isolation must be >= 35.0 dB, got {:.1} dB",
        metrics.inter_channel_isolation_db
    );
    assert!(
        metrics.return_loss_db <= -22.0,
        "Return loss must be <= -22.0 dB, got {:.1} dB",
        metrics.return_loss_db
    );
    assert!(metrics.channel_capacity_gbps >= 1.5);

    let matrix = router.generate_routing_matrix();
    assert_eq!(matrix.len(), 4 * 5); // 4 ports * 5 channels

    let spectrum = router.generate_transmission_spectrum(30);
    assert_eq!(spectrum.len(), 31);
}

#[test]
fn test_topological_defect_immunity() {
    let mut clean_params = SyntheticRouterParams::default();
    clean_params.defect_present = false;
    let clean_router = SyntheticMultiplexedRouter::new(clean_params);
    let clean_metrics = clean_router.evaluate_metrics();

    let mut defect_params = SyntheticRouterParams::default();
    defect_params.defect_present = true;
    defect_params.defect_detuning_ratio = 0.15;
    let defect_router = SyntheticMultiplexedRouter::new(defect_params);
    let defect_metrics = defect_router.evaluate_metrics();

    let ratio = defect_metrics.target_transmission_ratio / clean_metrics.target_transmission_ratio;
    assert!(
        ratio >= 0.95,
        "Defect transmission ratio must be >= 0.95, got {:.4}",
        ratio
    );
    assert!(defect_metrics.defect_immunity_ratio >= 0.95);
    assert!(defect_metrics.inter_channel_isolation_db >= 35.0);
}

#[test]
fn test_10_point_physics_audit_pass() {
    let router_system = SyntheticDimensionRouter::default();
    let audit = router_system.audit_synthetic_dimension_router();

    assert_eq!(audit.total_count, 10);
    assert_eq!(audit.passed_count, 10);
    assert!(audit.all_passed, "10-point physics audit must pass completely");
}
