//! Integration tests for acoustic spin-orbit coupling and chiral transducer efficiency.

use phonon_models::chiral_phonon_spin_mechanics::ChiralPhononSpinParams;
use phonon_solver::chiral_phonon_spin_mechanics::ChiralPhononSpinSolver;

#[test]
fn test_default_chiral_transduction_and_purity() {
    let params = ChiralPhononSpinParams::default();
    let solver = ChiralPhononSpinSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.transduction_efficiency_pct >= 70.0,
        "Transduction efficiency {} must be >= 70.0%",
        metrics.transduction_efficiency_pct
    );
    assert!(
        metrics.insertion_loss_db <= 2.0,
        "Insertion loss {} must be <= 2.0 dB",
        metrics.insertion_loss_db
    );
    assert!(
        metrics.spin_orbit_purity_pct >= 90.0,
        "Spin-orbit conversion purity {} must be >= 90.0%",
        metrics.spin_orbit_purity_pct
    );
}

#[test]
fn test_frequency_and_coupling_scaling() {
    let freqs = [1.5, 3.5, 6.0, 9.0];
    for &f in &freqs {
        let params = ChiralPhononSpinParams {
            acoustic_frequency_ghz: f,
            electromechanical_coupling_k2: 0.065,
            transducer_finger_pairs: 60,
            ..Default::default()
        };
        let solver = ChiralPhononSpinSolver::new(params);
        let eff = solver.compute_transduction_efficiency_pct();
        let il = solver.compute_insertion_loss_db();
        let purity = solver.compute_spin_orbit_purity_pct();

        assert!(eff >= 70.0, "Efficiency at {} GHz should be >= 70.0%", f);
        assert!(il <= 2.0, "Insertion loss at {} GHz should be <= 2.0 dB", f);
        assert!(purity >= 90.0, "Purity at {} GHz should be >= 90.0%", f);
    }
}

#[test]
fn test_finger_pairs_scaling() {
    let pair_counts = [25, 50, 80, 110];
    for &pairs in &pair_counts {
        let params = ChiralPhononSpinParams {
            transducer_finger_pairs: pairs,
            ..Default::default()
        };
        let solver = ChiralPhononSpinSolver::new(params);
        let eff = solver.compute_transduction_efficiency_pct();
        let purity = solver.compute_spin_orbit_purity_pct();

        assert!(eff >= 70.0);
        assert!(purity >= 90.0);
    }
}
