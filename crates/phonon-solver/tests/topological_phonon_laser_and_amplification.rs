//! Integration tests for topological acoustic laser arrays, chiral mode selection,
//! side-mode suppression ratio (>= 25.0 dB), and unidirectional amplification (>= 25.0 dB).

use phonon_models::non_hermitian_topo::NonHermitianSkinParams;
use phonon_solver::non_hermitian_topo::TopologicalPhononLaserSolver;

#[test]
fn test_topological_phonon_laser_and_amplification() {
    let params = NonHermitianSkinParams {
        lattice_sites_count: 35,
        forward_hopping_mhz: 3.0,
        backward_hopping_mhz: 0.40,
        onsite_gain_loss_mhz: 1.2,
        perturbation_epsilon: 1.0e-3,
        pump_power_mw: 8.0,
    };
    let solver = TopologicalPhononLaserSolver::new(params);
    let metrics = solver.solve_laser_metrics();

    // Lasing threshold power must be <= 5.0 mW
    assert!(
        metrics.laser_threshold_power_mw <= 5.0,
        "Laser threshold {:.2} mW must be <= 5.0 mW",
        metrics.laser_threshold_power_mw
    );

    // Side-mode suppression ratio (SMSR) must be >= 25.0 dB
    assert!(
        metrics.side_mode_suppression_ratio_db >= 25.0,
        "SMSR {:.2} dB must be >= 25.0 dB",
        metrics.side_mode_suppression_ratio_db
    );

    // Directional amplification gain contrast must be >= 25.0 dB
    assert!(
        metrics.directional_amplification_gain_db >= 25.0,
        "Directional gain {:.2} dB must be >= 25.0 dB",
        metrics.directional_amplification_gain_db
    );

    // Output power below threshold must vanish
    let p_below = solver.solve_laser_output_power_mw(metrics.laser_threshold_power_mw * 0.5);
    assert_eq!(p_below, 0.0);

    // Output power above threshold must be positive
    let p_above = solver.solve_laser_output_power_mw(metrics.laser_threshold_power_mw * 2.0);
    assert!(
        p_above > 0.0,
        "Output power above threshold should be positive"
    );
}
