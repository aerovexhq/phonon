//! Automated unit and physical validation tests for topological phononic Floquet Weyl semimetals.

use phonon_models::topological_weyl_acoustics::WeylAcousticParams;
use phonon_solver::topological_weyl_acoustics::TopologicalWeylAcousticSolver;

#[test]
fn test_weyl_point_separation_norm() {
    let params = WeylAcousticParams::default();
    let solver = TopologicalWeylAcousticSolver::new(params);
    let sep = solver.compute_weyl_point_separation_norm();

    // Target separation >= 0.350 in units of pi/a
    assert!(
        sep >= 0.350,
        "Weyl point separation must be >= 0.350, got {:.4}",
        sep
    );
    assert!(
        sep <= 1.0,
        "Weyl point separation cannot exceed BZ boundary 1.0, got {:.4}",
        sep
    );
}

#[test]
fn test_fermi_arc_acoustic_transmission() {
    let params = WeylAcousticParams::default();
    let solver = TopologicalWeylAcousticSolver::new(params);
    let t_arc = solver.compute_fermi_arc_transmission();

    // Target Fermi arc transmission >= 94.0%
    assert!(
        t_arc >= 0.940,
        "Surface Fermi arc transmission must be >= 94.0%, got {:.3}%",
        t_arc * 100.0
    );
    assert!(
        t_arc <= 1.0,
        "Transmission cannot exceed unity, got {:.4}",
        t_arc
    );
}

#[test]
fn test_screw_dislocation_mode_purity() {
    let params = WeylAcousticParams::default();
    let solver = TopologicalWeylAcousticSolver::new(params);
    let purity = solver.compute_dislocation_mode_purity();

    // Target dislocation mode purity >= 96.0%
    assert!(
        purity >= 0.960,
        "Topological screw dislocation mode purity must be >= 96.0%, got {:.3}%",
        purity * 100.0
    );
    assert!(
        purity <= 1.0,
        "Mode purity cannot exceed unity, got {:.4}",
        purity
    );
}

#[test]
fn test_bulk_bandgap_isolation() {
    let params = WeylAcousticParams::default();
    let solver = TopologicalWeylAcousticSolver::new(params);
    let isolation = solver.compute_bulk_bandgap_isolation_db();

    // Target bulk bandgap isolation >= 30.0 dB
    assert!(
        isolation >= 30.0,
        "Bulk bandgap isolation must be >= 30.0 dB, got {:.2} dB",
        isolation
    );
}

#[test]
fn test_chiral_monopole_charge_quantization() {
    let params = WeylAcousticParams::default();
    let solver = TopologicalWeylAcousticSolver::new(params);
    let charge = solver.compute_chiral_monopole_charge();

    // Quantized Chern monopole charge |C_w| = 1.0
    assert!(
        (charge - 1.0).abs() < 1.0e-9,
        "Chiral monopole charge must be exactly 1.0, got {:.4}",
        charge
    );
}

#[test]
fn test_full_weyl_acoustic_metrics_compliance() {
    let params = WeylAcousticParams::default();
    let solver = TopologicalWeylAcousticSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(
        metrics.is_physically_compliant,
        "Default Weyl acoustic system must satisfy all physical compliance thresholds: {:?}",
        metrics
    );
    assert!(metrics.weyl_point_separation_norm >= 0.350);
    assert!(metrics.fermi_arc_transmission >= 0.940);
    assert!(metrics.dislocation_mode_purity >= 0.960);
    assert!(metrics.bulk_bandgap_isolation_db >= 30.0);
    assert!((metrics.chiral_monopole_charge - 1.0).abs() < 1.0e-9);
}
