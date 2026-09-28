//! Integration tests for FQH Fabry-P\u{00e9}rot interferometers, even-odd anyon visibility collapse, and regimes.

use phonon_models::fqh::{FabryPerotInterferometer, InterferometerRegime, LuttingerEdgeModel};
use phonon_solver::fqh::InterferometrySolver;

#[test]
fn test_ab_oscillation_periods() {
    let b_tesla = 3.0;
    let laughlin = LuttingerEdgeModel::laughlin_one_third(b_tesla);
    let moore_read = LuttingerEdgeModel::moore_read_five_halves(b_tesla);
    let fp = FabryPerotInterferometer::standard_ab_regime();

    // Area = 2.0 um^2 = 2.0e-12 m^2
    // \u{0394}B = \u{03a6}_0^* / A_cell
    let delta_b_laughlin = fp.ab_period_tesla(&laughlin);
    let delta_b_mr = fp.ab_period_tesla(&moore_read);

    // For Laughlin (e* = e/3), \u{03a6}_0^* = 3 * (h/e) approx 12.4e-15 Wb
    // \u{0394}B = 12.4e-15 / 2.0e-12 = 6.2 mT
    assert!(
        delta_b_laughlin > 0.005 && delta_b_laughlin < 0.008,
        "Laughlin \u{0394}B should be ~6.2 mT, got {}",
        delta_b_laughlin
    );

    // For Moore-Read (e* = e/4), \u{03a6}_0^* = 4 * (h/e) approx 16.5e-15 Wb
    // \u{0394}B = 16.5e-15 / 2.0e-12 = 8.27 mT
    assert!(
        delta_b_mr > 0.007 && delta_b_mr < 0.010,
        "Moore-Read \u{0394}B should be ~8.27 mT, got {}",
        delta_b_mr
    );
    assert!((delta_b_mr / delta_b_laughlin - 4.0 / 3.0).abs() < 1e-6);
}

#[test]
fn test_even_odd_non_abelian_visibility_extinction() {
    let b_tesla = 3.0;
    let moore_read = LuttingerEdgeModel::moore_read_five_halves(b_tesla);
    let fp = FabryPerotInterferometer::standard_ab_regime();
    let solver = InterferometrySolver::new(fp, moore_read);

    // Line scan with EVEN number of bulk anyons (N_\u{03c3} = 0)
    let scan_even_0 = solver.scan_magnetic_field(3.0, 3.02, 50, 0.0, 0, false);
    // Line scan with EVEN number of bulk anyons (N_\u{03c3} = 2)
    let scan_even_2 = solver.scan_magnetic_field(3.0, 3.02, 50, 0.0, 2, false);

    assert!(
        scan_even_0.visibility >= 0.80,
        "Even bulk anyons (N=0) must exhibit strong interference: got {}",
        scan_even_0.visibility
    );
    assert!(
        scan_even_2.visibility >= 0.80,
        "Even bulk anyons (N=2) must exhibit strong interference: got {}",
        scan_even_2.visibility
    );

    // Line scan with ODD number of bulk anyons (N_\u{03c3} = 1)
    let scan_odd_1 = solver.scan_magnetic_field(3.0, 3.02, 50, 0.0, 1, false);
    // Line scan with ODD number of bulk anyons (N_\u{03c3} = 3)
    let scan_odd_3 = solver.scan_magnetic_field(3.0, 3.02, 50, 0.0, 3, false);

    assert!(
        scan_odd_1.visibility < 1e-6,
        "Odd bulk anyons (N=1) must exhibit zero visibility collapse: got {}",
        scan_odd_1.visibility
    );
    assert!(
        scan_odd_3.visibility < 1e-6,
        "Odd bulk anyons (N=3) must exhibit zero visibility collapse: got {}",
        scan_odd_3.visibility
    );
}

#[test]
fn test_state_dependent_topological_phase_shift() {
    let b_tesla = 3.0;
    let moore_read = LuttingerEdgeModel::moore_read_five_halves(b_tesla);
    let fp = FabryPerotInterferometer::standard_ab_regime();

    // In Moore-Read \u{03bd} = 5/2, topological phase shift for |0\u{27e9} is -\u{03c0}/4, and for |1\u{27e9} is +3\u{03c0}/4
    let phase_0 = fp.topological_phase_shift_rad(&moore_read, false);
    let phase_1 = fp.topological_phase_shift_rad(&moore_read, true);

    let diff = (phase_1 - phase_0).abs();
    // Phase difference must be exactly \u{03c0} (180 degrees)!
    assert!(
        (diff - std::f64::consts::PI).abs() < 1e-12,
        "Phase difference between |0\u{27e9} and |1\u{27e9} must be \u{03c0}: got {}",
        diff
    );

    // Conductance peaks should be exactly anti-correlated at the same B-field point
    let g_0 = fp.tunneling_conductance_siemens(&moore_read, 3.0, 0.0, 0, false);
    let g_1 = fp.tunneling_conductance_siemens(&moore_read, 3.0, 0.0, 0, true);

    // When one is near maximum, the other is near minimum
    let avg = (g_0 + g_1) / 2.0;
    assert!(
        (g_0 - avg).abs() > 0.1 * avg,
        "Conductance should exhibit state-dependent modulation"
    );
}

#[test]
fn test_coulomb_vs_aharonov_bohm_regimes() {
    let fp_ab = FabryPerotInterferometer::standard_ab_regime();
    let fp_coulomb = FabryPerotInterferometer::standard_coulomb_regime();

    assert_eq!(fp_ab.operating_regime(), InterferometerRegime::AharonovBohm);
    assert!(fp_ab.coupling_ratio() < 0.5);

    assert_eq!(
        fp_coulomb.operating_regime(),
        InterferometerRegime::CoulombDominated
    );
    assert!(fp_coulomb.coupling_ratio() > 1.5);
}
