//! Automated unit and multi-physics validation tests for quantum acoustic
//! topological Chern insulators and chiral phonon diode circulators.

use phonon_models::topological_chern_circulator::TopologicalChernCirculatorParams;
use phonon_solver::topological_chern_circulator::TopologicalChernCirculatorSolver;

#[test]
fn test_forward_transmission_bounds() {
    let params = TopologicalChernCirculatorParams::default();
    let solver = TopologicalChernCirculatorSolver::new(params);
    let t_fwd = solver.compute_forward_transmission();

    // Target forward acoustic transmission >= 95.0%
    assert!(
        t_fwd >= 0.950,
        "Forward acoustic transmission must be >= 0.950, got {:.4}",
        t_fwd
    );
    assert!(
        t_fwd <= 1.0,
        "Forward transmission cannot exceed unity, got {:.4}",
        t_fwd
    );
}

#[test]
fn test_non_reciprocal_isolation_bounds() {
    let params = TopologicalChernCirculatorParams::default();
    let solver = TopologicalChernCirculatorSolver::new(params);
    let isolation_db = solver.compute_non_reciprocal_isolation_db();

    // Target non-reciprocal isolation >= 35.0 dB
    assert!(
        isolation_db >= 35.0,
        "Non-reciprocal isolation must be >= 35.0 dB, got {:.2} dB",
        isolation_db
    );
    assert!(
        isolation_db <= 60.0,
        "Non-reciprocal isolation unphysical upper bound, got {:.2} dB",
        isolation_db
    );
}

#[test]
fn test_topological_bandgap_ratio() {
    let params = TopologicalChernCirculatorParams::default();
    let solver = TopologicalChernCirculatorSolver::new(params);
    let gap_ratio = solver.compute_topological_bandgap_ratio();

    // Target topological bandgap ratio Delta omega / omega_0 >= 12.0%
    assert!(
        gap_ratio >= 0.120,
        "Topological bandgap ratio must be >= 0.120, got {:.4}",
        gap_ratio
    );
    assert!(
        gap_ratio <= 0.500,
        "Topological bandgap ratio unphysical upper bound, got {:.4}",
        gap_ratio
    );
}

#[test]
fn test_backscattering_reflection_immunity() {
    let params = TopologicalChernCirculatorParams::default();
    let solver = TopologicalChernCirculatorSolver::new(params);
    let refl_db = solver.compute_backscattering_reflection_db();

    // Target backscattering reflection <= -40.0 dB
    assert!(
        refl_db <= -40.0,
        "Backscattering reflection must be <= -40.0 dB, got {:.2} dB",
        refl_db
    );
}

#[test]
fn test_insertion_loss_bounds() {
    let params = TopologicalChernCirculatorParams::default();
    let solver = TopologicalChernCirculatorSolver::new(params);
    let loss_db = solver.compute_insertion_loss_db();

    // Target insertion loss <= 0.80 dB
    assert!(
        loss_db <= 0.80,
        "Waveguide insertion loss must be <= 0.80 dB, got {:.3} dB",
        loss_db
    );
    assert!(
        loss_db >= 0.0,
        "Insertion loss must be non-negative, got {:.3} dB",
        loss_db
    );
}

#[test]
fn test_topological_chern_number_quantization() {
    let params = TopologicalChernCirculatorParams::default();
    let solver = TopologicalChernCirculatorSolver::new(params);
    let chern = solver.compute_topological_chern_number();

    // Quantized first Chern invariant |C| = 1
    assert_eq!(
        chern, 1,
        "Topological Chern number must be quantized to 1, got {}",
        chern
    );
}

#[test]
fn test_full_metrics_physical_compliance() {
    let params = TopologicalChernCirculatorParams::default();
    let solver = TopologicalChernCirculatorSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default Chern circulator configuration must satisfy all physical compliance thresholds: {:?}",
        metrics
    );
    assert!(metrics.forward_transmission >= 0.950);
    assert!(metrics.non_reciprocal_isolation_db >= 35.0);
    assert!(metrics.topological_bandgap_ratio >= 0.120);
    assert!(metrics.backscattering_reflection_db <= -40.0);
    assert!(metrics.insertion_loss_db <= 0.80);
    assert_eq!(metrics.topological_chern_number, 1);
}

#[test]
fn test_parameter_clamping() {
    let params = TopologicalChernCirculatorParams::new(
        0.01,    // clamped to 0.5
        100.0,   // clamped to 15.0
        1.0,     // clamped to 10.0
        500.0,   // clamped to 150.0
        1.5,     // clamped to 0.30
        0.001,   // clamped to 1.0
        100,     // clamped to 8
        1.0e10,  // clamped to 1.0e7
    );

    assert_eq!(params.lattice_constant_um, 0.5);
    assert_eq!(params.center_frequency_ghz, 15.0);
    assert_eq!(params.synthetic_angular_momentum_mhz, 10.0);
    assert_eq!(params.inter_site_coupling_mhz, 150.0);
    assert_eq!(params.defect_disorder_fraction, 0.30);
    assert_eq!(params.operating_temp_m_k, 1.0);
    assert_eq!(params.circulator_ports_count, 8);
    assert_eq!(params.acoustic_intrinsic_q, 1.0e7);
}
