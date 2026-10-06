#![deny(unsafe_code)]

//! Automated Test Suite for Phase 375:
//! Non-Hermitian Higher-Order Topological Corner Laser & Spectral Singularity Engine.

use phonon_solver::non_hermitian_corner_laser::{
    CornerLaserLatticeKind, CornerLaserLatticeParams, HotLaserModeKind,
    NonHermitianCornerLaserEngine, NonHermitianHotLattice,
};
use std::time::Instant;

#[test]
fn test_hot_laser_lattice_dimensions_and_bandgap() {
    let params = CornerLaserLatticeParams {
        lattice_size_x: 4,
        lattice_size_y: 4,
        intracell_coupling_mhz: 2.5,
        intercell_coupling_mhz: 10.0,
        ..Default::default()
    };
    let lattice = NonHermitianHotLattice::new(params, CornerLaserLatticeKind::QuadrupoleCornerLaser);

    assert_eq!(lattice.total_sites(), 4 * 4 * 4); // 4 sublattices per unit cell
    assert_eq!(lattice.topological_quadrupole_moment(), 0.5);
    assert!((lattice.bulk_bandgap_mhz() - 15.0).abs() < 1e-10); // 2 * (10 - 2.5) = 15.0 MHz
}

#[test]
fn test_topological_phase_transition_quadrupole_moment() {
    // Topological regime: gamma < lambda
    let topo_params = CornerLaserLatticeParams {
        intracell_coupling_mhz: 2.0,
        intercell_coupling_mhz: 8.0,
        ..Default::default()
    };
    let topo_lattice = NonHermitianHotLattice::new(topo_params, CornerLaserLatticeKind::QuadrupoleCornerLaser);
    assert_eq!(topo_lattice.topological_quadrupole_moment(), 0.5);

    // Trivial regime: gamma > lambda
    let trivial_params = CornerLaserLatticeParams {
        intracell_coupling_mhz: 8.0,
        intercell_coupling_mhz: 2.0,
        ..Default::default()
    };
    let trivial_lattice = NonHermitianHotLattice::new(trivial_params, CornerLaserLatticeKind::QuadrupoleCornerLaser);
    assert_eq!(trivial_lattice.topological_quadrupole_moment(), 0.0);
}

#[test]
fn test_selective_corner_mode_gain_and_bulk_damping() {
    let params = CornerLaserLatticeParams {
        intracell_coupling_mhz: 2.5,
        intercell_coupling_mhz: 10.0,
        corner_gain_mhz: 1.8,
        bulk_loss_mhz: 1.2,
        ..Default::default()
    };
    let engine = NonHermitianCornerLaserEngine::new(params, CornerLaserLatticeKind::QuadrupoleCornerLaser);

    assert!(engine.metrics.active_lasing_mode_count >= 1);
    assert!(engine.metrics.side_mode_suppression_ratio_db >= 30.0);

    // Check that corner modes have positive imaginary gain
    let corner_modes: Vec<_> = engine
        .eigenmodes
        .iter()
        .filter(|m| m.mode_kind == HotLaserModeKind::LasingCorner)
        .collect();
    assert!(!corner_modes.is_empty());
    for cm in &corner_modes {
        assert!(cm.modal_net_gain_mhz > 0.0, "Corner mode must experience positive gain");
        assert!(
            cm.corner_confinement_percent >= 85.0,
            "Corner confinement must be >= 85%, got {}",
            cm.corner_confinement_percent
        );
    }

    // Check that bulk/edge modes are damped (Im(E) < 0)
    let non_corner_modes: Vec<_> = engine
        .eigenmodes
        .iter()
        .filter(|m| m.mode_kind != HotLaserModeKind::LasingCorner)
        .collect();
    assert!(!non_corner_modes.is_empty());
    for ncm in &non_corner_modes {
        assert!(ncm.modal_net_gain_mhz < 0.0, "Bulk/edge mode must be damped");
    }
}

#[test]
fn test_light_current_curve_and_slope_efficiency() {
    let params = CornerLaserLatticeParams::default();
    let engine = NonHermitianCornerLaserEngine::new(params, CornerLaserLatticeKind::QuadrupoleCornerLaser);

    let li_curve = engine.compute_light_current_curve(20);
    assert_eq!(li_curve.len(), 20);

    // Below threshold, power should be minimal
    let p_low = li_curve[1].1;
    // Above threshold, power should increase linearly
    let p_high = li_curve[18].1;
    assert!(p_high > p_low * 10.0, "Stimulated lasing output must dramatically exceed spontaneous level");

    let smsr_spectrum = engine.compute_smsr_spectrum();
    assert!(!smsr_spectrum.is_empty());
}

#[test]
fn test_exceptional_point_sweep_dynamics() {
    let params = CornerLaserLatticeParams::default();
    let engine = NonHermitianCornerLaserEngine::new(params, CornerLaserLatticeKind::QuadrupoleCornerLaser);

    let sweep = engine.compute_gain_loss_sweep(16);
    assert_eq!(sweep.len(), 16);

    // Initial real splitting should be positive, then decrease towards EP
    let re_split_start = sweep[0].1;
    let re_split_near_ep = sweep[8].1;
    assert!(re_split_start >= re_split_near_ep);
}

#[test]
fn test_fast_initialization_cold_boot_latency() {
    let start = Instant::now();
    let params = CornerLaserLatticeParams::default();
    let engine = NonHermitianCornerLaserEngine::new_fast(params);
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "NonHermitianCornerLaserEngine::new_fast must execute in < 5ms, took {:?}",
        elapsed
    );
    assert_eq!(engine.metrics.quantized_quadrupole_moment, 0.50);
    assert!(engine.metrics.side_mode_suppression_ratio_db >= 32.0);
    assert!(engine.metrics.corner_confinement_ratio_percent >= 85.0);
}
