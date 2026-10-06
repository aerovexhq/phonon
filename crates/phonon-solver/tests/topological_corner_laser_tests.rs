#![deny(unsafe_code)]

//! Automated test suite for Phase 405: Topological Higher-Order Acoustic Quadrupole
//! Corner-Pumped Polariton Laser and Parity-Time (PT) Symmetric Metamaterial Engine.

use phonon_solver::topological_corner_laser::{
    CornerLasingSolver, LasingParams, PtQuadrupoleLattice, PtQuadrupoleParams,
    TopologicalCornerLaserProcessor,
};

#[test]
fn test_bbh_quantized_quadrupole_moment_and_confinement() {
    // 1. Topological SOTI phase (lambda > gamma)
    let topo_params = PtQuadrupoleParams::new(6, 2.0, 10.0, 1.5, 3.0);
    let topo_lattice = PtQuadrupoleLattice::new(topo_params);

    assert!(topo_lattice.is_topological());
    assert_eq!(topo_lattice.quadrupole_moment(), 0.500);

    let confinement = topo_lattice.corner_confinement_ratio();
    assert!(
        confinement >= 0.80,
        "Expected confinement >= 80%, got {:.1}%",
        confinement * 100.0
    );
    assert!(
        confinement >= 0.85,
        "Measured confinement should exceed 85%, got {:.1}%",
        confinement * 100.0
    );

    // Verify 4 corner modes are extracted
    let corner_modes = topo_lattice.compute_corner_modes();
    assert_eq!(corner_modes.len(), 4);
    for m in &corner_modes {
        assert!(m.spatial_confinement_ratio >= 0.80);
    }

    // 2. Trivial phase (gamma > lambda)
    let trivial_params = PtQuadrupoleParams::trivial_preset();
    let trivial_lattice = PtQuadrupoleLattice::new(trivial_params);

    assert!(!trivial_lattice.is_topological());
    assert_eq!(trivial_lattice.quadrupole_moment(), 0.000);
    assert!(trivial_lattice.corner_confinement_ratio() < 0.25);
}

#[test]
fn test_pt_symmetry_breaking_and_exceptional_point() {
    let gamma = 2.0;
    let lambda = 10.0;
    let base_params = PtQuadrupoleParams::new(6, gamma, lambda, 0.5, 3.0);
    let lattice = PtQuadrupoleLattice::new(base_params);

    let ep_threshold = lattice.exceptional_point_threshold_mhz();
    assert!(ep_threshold > 0.0);

    // Below EP (Unbroken PT symmetry: gamma_gain < gamma_crit)
    let unbroken_params = PtQuadrupoleParams::new(6, gamma, lambda, 1.0, 3.0);
    let unbroken_lattice = PtQuadrupoleLattice::new(unbroken_params);
    assert!(unbroken_lattice.is_pt_unbroken());

    // Above EP (Broken PT symmetry: gamma_gain > gamma_crit)
    let broken_params = PtQuadrupoleParams::new(6, gamma, lambda, ep_threshold + 1.0, 3.0);
    let broken_lattice = PtQuadrupoleLattice::new(broken_params);
    assert!(!broken_lattice.is_pt_unbroken());

    // Check that broken PT eigenvalues contain complex conjugate pairs
    let broken_evals = broken_lattice.compute_eigenvalues();
    let has_imaginary = broken_evals.iter().any(|ev| ev.im.abs() > 1e-4);
    assert!(has_imaginary, "Broken PT must produce imaginary eigenvalue components");
}

#[test]
fn test_selective_corner_lasing_threshold_and_slope_efficiency() {
    let params = LasingParams::new(2.5, 0.05, 8.0, 2.0, 50.0, 20.0);
    let solver = CornerLasingSolver::new(params);

    let p_th_corner = solver.compute_threshold_power_mw();
    let p_th_bulk = solver.compute_bulk_threshold_power_mw();

    // Verify selective corner mode threshold inversion: P_th,corner < P_th,bulk
    assert!(
        p_th_corner < p_th_bulk,
        "Corner threshold ({:.2} mW) must be lower than bulk threshold ({:.2} mW)",
        p_th_corner, p_th_bulk
    );
    assert!(p_th_bulk / p_th_corner >= 2.0);

    // Verify slope efficiency eta_slope >= 35% (measured >= 40%)
    let slope = solver.compute_slope_efficiency();
    assert!(
        slope >= 0.35,
        "Expected slope efficiency >= 35%, got {:.1}%",
        slope * 100.0
    );
    assert!(
        slope >= 0.40,
        "Measured slope efficiency should exceed 40%, got {:.1}%",
        slope * 100.0
    );

    // Test L-I curve generation and threshold kink
    let li_curve = solver.compute_li_curve(20);
    assert_eq!(li_curve.len(), 20);

    let below_th = li_curve.iter().find(|p| p.pump_power_mw < p_th_corner).unwrap();
    let above_th = li_curve.iter().find(|p| p.pump_power_mw > p_th_corner * 1.5).unwrap();

    assert!(above_th.corner_output_power_mw > below_th.corner_output_power_mw * 5.0);
}

#[test]
fn test_side_mode_suppression_ratio_smsr() {
    // Operational pump power = 20.0 mW (well above threshold)
    let params = LasingParams::new(2.5, 0.05, 8.0, 2.0, 50.0, 20.0);
    let solver = CornerLasingSolver::new(params);
    let sol = solver.solve();

    assert!(sol.is_lasing);
    assert!(
        sol.smsr_db >= 30.0,
        "Expected SMSR >= 30.0 dB, got {:.1} dB",
        sol.smsr_db
    );
    assert!(
        sol.smsr_db >= 35.0,
        "Measured SMSR should exceed 35.0 dB, got {:.1} dB",
        sol.smsr_db
    );
}

#[test]
fn test_second_order_coherence_and_linewidth() {
    let processor = TopologicalCornerLaserProcessor::default();
    let sol = processor.lasing_solver.solve();
    let pump_mw = processor.lasing_solver.params.pump_rate_mw;
    let p_th_mw = sol.threshold_pump_power_mw;

    // Above threshold coherence
    let coh_above = processor.coherence_engine.evaluate_metrics(pump_mw, p_th_mw);
    assert!(
        coh_above.coherence_time_us >= 10.0,
        "Expected coherence time >= 10 us, got {:.2} us",
        coh_above.coherence_time_us
    );
    assert!(
        coh_above.schawlow_townes_linewidth_khz <= 50.0,
        "Expected linewidth <= 50 kHz, got {:.2} kHz",
        coh_above.schawlow_townes_linewidth_khz
    );
    assert!(
        coh_above.schawlow_townes_linewidth_khz < 30.0,
        "Measured linewidth should be < 30 kHz, got {:.2} kHz",
        coh_above.schawlow_townes_linewidth_khz
    );

    // Second-order coherence g^(2)(0) in [0.95, 1.05] (Poissonian coherent state)
    assert!(
        coh_above.zero_delay_second_order_coherence >= 0.95
            && coh_above.zero_delay_second_order_coherence <= 1.05,
        "Expected g^(2)(0) in [0.95, 1.05], got {:.3}",
        coh_above.zero_delay_second_order_coherence
    );
    assert!(coh_above.is_coherent_state);
    assert!(coh_above.emission_directivity_db >= 25.0);

    // Below threshold: thermal acoustic state g^(2)(0) approx 2.00
    let coh_below = processor.coherence_engine.evaluate_metrics(2.0, p_th_mw);
    assert!(
        (coh_below.zero_delay_second_order_coherence - 2.0).abs() < 0.15,
        "Expected thermal g^(2)(0) ~ 2.0 below threshold, got {:.3}",
        coh_below.zero_delay_second_order_coherence
    );
}

#[test]
fn test_ten_point_physics_audit_full_pass() {
    let processor = TopologicalCornerLaserProcessor::default();
    let audit = processor.audit_laser();

    assert_eq!(audit.total_tests, 10);
    assert_eq!(
        audit.pass_count, 10,
        "Failed tests: {:?}",
        audit.items.iter().filter(|i| !i.passed).map(|i| i.name).collect::<Vec<_>>()
    );
    assert!(audit.all_passed, "All 10 physics audit tests must pass");
}
