//! Integration tests for acoustic PT-symmetry breaking, exceptional points, and eigenvector coalescence.

use phonon_models::non_hermitian_pt_symmetry::PtSymmetryParams;
use phonon_solver::non_hermitian_pt_symmetry::PtSymmetrySolver;

#[test]
fn test_default_pt_symmetry_and_coalescence() {
    let params = PtSymmetryParams::default();
    let solver = PtSymmetrySolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.pt_phase_rigidity <= 0.20,
        "Phase rigidity {} must be <= 0.20 near EP",
        metrics.pt_phase_rigidity
    );
    assert!(
        metrics.coalescence_fidelity_pct >= 95.0,
        "Coalescence fidelity {}% must be >= 95.0%",
        metrics.coalescence_fidelity_pct
    );
    assert!(
        metrics.directional_absorption_pct >= 90.0,
        "Directional absorption {}% must be >= 90.0%",
        metrics.directional_absorption_pct
    );
}

#[test]
fn test_gain_loss_ratio_tuning() {
    let ratios = [0.95, 0.98, 1.00, 1.02, 1.05];
    for &r in &ratios {
        let coupling = 30.0;
        let params = PtSymmetryParams {
            intercavity_coupling_mhz: coupling,
            gain_loss_rate_mhz: coupling * r,
            ..Default::default()
        };
        let solver = PtSymmetrySolver::new(params);
        let rigidity = solver.compute_pt_phase_rigidity();
        let fid = solver.compute_coalescence_fidelity_pct();
        let abs = solver.compute_directional_absorption_pct();

        assert!(rigidity <= 0.20, "Rigidity at ratio {} was {}", r, rigidity);
        assert!(fid >= 95.0, "Fidelity at ratio {} was {}", r, fid);
        assert!(abs >= 90.0, "Absorption at ratio {} was {}", r, abs);
    }
}
