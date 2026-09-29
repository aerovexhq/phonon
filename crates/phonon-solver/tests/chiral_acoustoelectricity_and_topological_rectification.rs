//! Integration tests for Chiral Acoustoelectricity, non-reciprocal rectification,
//! and multi-terminal corner acoustic sensors.

use phonon_models::non_hermitian_chiral_hoti::NonHermitianHotiParams;
use phonon_solver::non_hermitian_chiral_hoti::NonHermitianHotiSolver;

#[test]
fn test_acoustoelectric_rectification_bound() {
    let params = NonHermitianHotiParams::default();
    let solver = NonHermitianHotiSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.acoustoelectric_rectification_db >= 20.0,
        "Acoustoelectric rectification {} dB must be >= 20.0 dB",
        metrics.acoustoelectric_rectification_db
    );
    assert!(
        metrics.acoustoelectric_current_density_a_m2 > 0.0,
        "Acoustoelectric current density {} must be positive",
        metrics.acoustoelectric_current_density_a_m2
    );
    assert!(
        metrics.corner_sensor_snr_db >= 20.0,
        "Corner sensor SNR {} dB must be >= 20.0 dB",
        metrics.corner_sensor_snr_db
    );
}

#[test]
fn test_power_and_mobility_current_scaling() {
    let p_base = NonHermitianHotiParams::default();
    let p_high_power = NonHermitianHotiParams {
        saw_power_flux_mw_mm: p_base.saw_power_flux_mw_mm * 2.0,
        ..p_base
    };
    let p_high_mobility = NonHermitianHotiParams {
        carrier_mobility_cm2_v_s: p_base.carrier_mobility_cm2_v_s * 2.0,
        ..p_base
    };

    let solver_base = NonHermitianHotiSolver::new(p_base);
    let solver_power = NonHermitianHotiSolver::new(p_high_power);
    let solver_mobility = NonHermitianHotiSolver::new(p_high_mobility);

    let j_base = solver_base.compute_acoustoelectric_current_density_a_m2();
    let j_power = solver_power.compute_acoustoelectric_current_density_a_m2();
    let j_mobility = solver_mobility.compute_acoustoelectric_current_density_a_m2();

    assert!(
        (j_power - 2.0 * j_base).abs() < 1e-6,
        "Acoustoelectric current must scale linearly with SAW power flux"
    );
    assert!(
        (j_mobility - 2.0 * j_base).abs() < 1e-6,
        "Acoustoelectric current must scale linearly with electronic carrier mobility"
    );
}

#[test]
fn test_rectification_ratio_non_reciprocal_asymmetry() {
    let p_low_eta = NonHermitianHotiParams {
        non_reciprocal_asymmetry_eta: 0.30,
        ..Default::default()
    };
    let p_high_eta = NonHermitianHotiParams {
        non_reciprocal_asymmetry_eta: 0.70,
        ..Default::default()
    };

    let solver_low = NonHermitianHotiSolver::new(p_low_eta);
    let solver_high = NonHermitianHotiSolver::new(p_high_eta);

    let rect_low = solver_low.compute_acoustoelectric_rectification_db();
    let rect_high = solver_high.compute_acoustoelectric_rectification_db();

    assert!(
        rect_high > rect_low,
        "Higher non-reciprocal asymmetry must enhance rectification contrast"
    );
    assert!(rect_low >= 20.0);
    assert!(rect_high >= 20.0);
}
