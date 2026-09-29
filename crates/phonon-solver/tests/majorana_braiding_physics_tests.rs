//! Automated unit and physical validation tests for topological non-Abelian
//! Majorana braiding in phononic Josephson metamaterials.

use phonon_models::topological_majorana_braiding::MajoranaBraidingParams;
use phonon_solver::topological_majorana_braiding::TopologicalMajoranaBraidingSolver;

#[test]
fn test_non_abelian_braiding_gate_fidelity() {
    let params = MajoranaBraidingParams::default();
    let solver = TopologicalMajoranaBraidingSolver::new(params);
    let fid = solver.compute_braiding_gate_fidelity();

    // Target braiding fidelity >= 99.90%
    assert!(
        fid >= 0.9990,
        "Braiding gate fidelity must be >= 99.90%, got {:.4}%",
        fid * 100.0
    );
    assert!(
        fid <= 1.0,
        "Braiding gate fidelity cannot exceed unity, got {:.6}",
        fid
    );
}

#[test]
fn test_non_abelian_geometric_phase_error() {
    let params = MajoranaBraidingParams::default();
    let solver = TopologicalMajoranaBraidingSolver::new(params);
    let error = solver.compute_non_abelian_phase_error_rad();

    // Target non-Abelian phase error <= 1.0e-4 rad
    assert!(
        error <= 1.0e-4,
        "Non-Abelian phase error must be <= 1.0e-4 rad, got {:.3e} rad",
        error
    );
    assert!(
        error > 0.0,
        "Phase error must be positive, got {:.3e}",
        error
    );
}

#[test]
fn test_fermion_parity_readout_contrast() {
    let params = MajoranaBraidingParams::default();
    let solver = TopologicalMajoranaBraidingSolver::new(params);
    let contrast = solver.compute_parity_readout_contrast();

    // Target parity readout contrast >= 95.0%
    assert!(
        contrast >= 0.950,
        "Parity readout contrast must be >= 95.0%, got {:.2}%",
        contrast * 100.0
    );
    assert!(
        contrast <= 1.0,
        "Readout contrast cannot exceed unity, got {:.4}",
        contrast
    );
}

#[test]
fn test_braiding_cycle_period_ns() {
    let params = MajoranaBraidingParams::default();
    let solver = TopologicalMajoranaBraidingSolver::new(params);
    let tau = solver.compute_braiding_cycle_period_ns();

    // Target braiding cycle duration <= 50.0 ns
    assert!(
        tau <= 50.0,
        "Braiding cycle period must be <= 50.0 ns, got {:.2} ns",
        tau
    );
    assert!(
        tau > 0.0,
        "Braiding cycle period must be positive, got {:.2} ns",
        tau
    );
}

#[test]
fn test_topological_gap_protection_ratio() {
    let params = MajoranaBraidingParams::default();
    let solver = TopologicalMajoranaBraidingSolver::new(params);
    let ratio = solver.compute_topological_gap_protection_ratio();

    // Target gap protection ratio >= 20.0
    assert!(
        ratio >= 20.0,
        "Topological gap protection ratio Delta / (k_B T) must be >= 20.0, got {:.2}",
        ratio
    );
}

#[test]
fn test_full_majorana_braiding_physical_compliance() {
    let params = MajoranaBraidingParams::default();
    let solver = TopologicalMajoranaBraidingSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default Majorana braiding system must satisfy all physical compliance thresholds: {:?}",
        metrics
    );
    assert!(metrics.braiding_gate_fidelity >= 0.9990);
    assert!(metrics.non_abelian_phase_error_rad <= 1.0e-4);
    assert!(metrics.parity_readout_contrast >= 0.950);
    assert!(metrics.braiding_cycle_period_ns <= 50.0);
    assert!(metrics.topological_gap_protection_ratio >= 20.0);
}
