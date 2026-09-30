#![deny(unsafe_code)]

//! Automated unit and multi-physics validation tests for quantum acoustic non-Hermitian
//! higher-order topological skin sensors and chiral octupole phonon lasers.

use phonon_models::non_hermitian_skin_octupole_laser::NonHermitianSkinOctupoleLaserParams;
use phonon_solver::non_hermitian_skin_octupole_laser::NonHermitianSkinOctupoleLaserSolver;

#[test]
fn test_parameter_boundary_clamping() {
    // Test values strictly below physical minimum bounds
    let underflow = NonHermitianSkinOctupoleLaserParams::new(
        0.50,  // below 1.05
        2.0,   // below 5.0 meV
        0.5,   // below 1.0 uW
        0.80,  // below 1.10
        0.5,   // below 1.0 GHz
        0.5,   // below 1.0 mK
        2.0,   // below 4.0
        5.0,   // below 10.0 nm
    );
    assert_eq!(underflow.non_hermitian_asymmetry_factor, 1.05);
    assert_eq!(underflow.octupole_hopping_coupling_mev, 5.0);
    assert_eq!(underflow.gain_saturation_intensity_uw, 1.0);
    assert_eq!(underflow.pump_rate_normalized, 1.10);
    assert_eq!(underflow.acoustic_octupole_frequency_ghz, 1.0);
    assert_eq!(underflow.cryogenic_temperature_mk, 1.0);
    assert_eq!(underflow.lattice_cell_count_3d, 4.0);
    assert_eq!(underflow.skin_localization_decay_length_nm, 10.0);

    // Test values strictly above physical maximum bounds
    let overflow = NonHermitianSkinOctupoleLaserParams::new(
        5.0,    // above 3.0
        60.0,   // above 45.0 meV
        80.0,   // above 50.0 uW
        10.0,   // above 5.0
        25.0,   // above 15.0 GHz
        80.0,   // above 50.0 mK
        35.0,   // above 24.0
        150.0,  // above 120.0 nm
    );
    assert_eq!(overflow.non_hermitian_asymmetry_factor, 3.0);
    assert_eq!(overflow.octupole_hopping_coupling_mev, 45.0);
    assert_eq!(overflow.gain_saturation_intensity_uw, 50.0);
    assert_eq!(overflow.pump_rate_normalized, 5.0);
    assert_eq!(overflow.acoustic_octupole_frequency_ghz, 15.0);
    assert_eq!(overflow.cryogenic_temperature_mk, 50.0);
    assert_eq!(overflow.lattice_cell_count_3d, 24.0);
    assert_eq!(overflow.skin_localization_decay_length_nm, 120.0);
}

#[test]
fn test_default_parameters_and_compliance() {
    let params = NonHermitianSkinOctupoleLaserParams::default();
    assert_eq!(params.non_hermitian_asymmetry_factor, 1.65);
    assert_eq!(params.octupole_hopping_coupling_mev, 24.0);
    assert_eq!(params.gain_saturation_intensity_uw, 15.0);
    assert_eq!(params.pump_rate_normalized, 2.2);
    assert_eq!(params.acoustic_octupole_frequency_ghz, 5.8);
    assert_eq!(params.cryogenic_temperature_mk, 10.0);
    assert_eq!(params.lattice_cell_count_3d, 10.0);
    assert_eq!(params.skin_localization_decay_length_nm, 35.0);

    let solver = NonHermitianSkinOctupoleLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Verify all 5 physical roadmap targets for default parameters
    assert!(
        metrics.corner_lasing_mode_purity >= 0.9980,
        "Corner lasing mode purity must be >= 0.9980, got {:.6}",
        metrics.corner_lasing_mode_purity
    );
    assert!(
        metrics.skin_sensitivity_factor >= 95.0,
        "Skin sensitivity factor must be >= 95.0, got {:.4}",
        metrics.skin_sensitivity_factor
    );
    assert!(
        metrics.higher_order_skin_topological_gap_mhz >= 48.0,
        "Higher-order skin topological gap must be >= 48.0 MHz, got {:.4} MHz",
        metrics.higher_order_skin_topological_gap_mhz
    );
    assert!(
        metrics.corner_to_bulk_crosstalk_isolation_db >= 55.0,
        "Corner-to-bulk crosstalk isolation must be >= 55.0 dB, got {:.4} dB",
        metrics.corner_to_bulk_crosstalk_isolation_db
    );
    assert!(
        metrics.topological_mode_dephasing_rate_hz <= 13.0,
        "Topological mode dephasing rate must be <= 13.0 Hz, got {:.4} Hz",
        metrics.topological_mode_dephasing_rate_hz
    );
    assert!(
        metrics.is_physically_compliant,
        "Default parameter set must be strictly physically compliant"
    );
}

#[test]
fn test_non_hermitian_asymmetry_factor_scaling() {
    let mut low_asym = NonHermitianSkinOctupoleLaserParams::default();
    low_asym.non_hermitian_asymmetry_factor = 1.10;
    let mut high_asym = NonHermitianSkinOctupoleLaserParams::default();
    high_asym.non_hermitian_asymmetry_factor = 2.80;

    let low_solver = NonHermitianSkinOctupoleLaserSolver::new(low_asym);
    let high_solver = NonHermitianSkinOctupoleLaserSolver::new(high_asym);

    assert!(
        high_solver.compute_skin_sensitivity_factor()
            > low_solver.compute_skin_sensitivity_factor(),
        "Higher non-Hermitian asymmetry must boost skin displacement sensitivity"
    );
    assert!(
        high_solver.compute_corner_lasing_mode_purity()
            > low_solver.compute_corner_lasing_mode_purity(),
        "Higher non-Hermitian asymmetry must enhance corner lasing mode purity"
    );
    assert!(
        high_solver.compute_higher_order_skin_topological_gap_mhz()
            > low_solver.compute_higher_order_skin_topological_gap_mhz(),
        "Higher non-Hermitian asymmetry must widen the topological protection gap"
    );
    assert!(
        high_solver.compute_corner_to_bulk_crosstalk_isolation_db()
            > low_solver.compute_corner_to_bulk_crosstalk_isolation_db(),
        "Higher non-Hermitian asymmetry must improve corner-to-bulk crosstalk isolation"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher non-Hermitian asymmetry must suppress topological dephasing rate"
    );
}

#[test]
fn test_octupole_hopping_coupling_scaling() {
    let mut low_oct = NonHermitianSkinOctupoleLaserParams::default();
    low_oct.octupole_hopping_coupling_mev = 8.0;
    let mut high_oct = NonHermitianSkinOctupoleLaserParams::default();
    high_oct.octupole_hopping_coupling_mev = 40.0;

    let low_solver = NonHermitianSkinOctupoleLaserSolver::new(low_oct);
    let high_solver = NonHermitianSkinOctupoleLaserSolver::new(high_oct);

    assert!(
        high_solver.compute_higher_order_skin_topological_gap_mhz()
            > low_solver.compute_higher_order_skin_topological_gap_mhz(),
        "Elevated octupole hopping coupling must widen the topological gap"
    );
    assert!(
        high_solver.compute_corner_lasing_mode_purity()
            > low_solver.compute_corner_lasing_mode_purity(),
        "Elevated octupole hopping coupling must improve corner lasing mode purity"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Elevated octupole hopping coupling must suppress dephasing rate"
    );
}

#[test]
fn test_gain_saturation_intensity_scaling() {
    let mut low_sat = NonHermitianSkinOctupoleLaserParams::default();
    low_sat.gain_saturation_intensity_uw = 5.0;
    let mut high_sat = NonHermitianSkinOctupoleLaserParams::default();
    high_sat.gain_saturation_intensity_uw = 45.0;

    let low_solver = NonHermitianSkinOctupoleLaserSolver::new(low_sat);
    let high_solver = NonHermitianSkinOctupoleLaserSolver::new(high_sat);

    assert!(
        high_solver.compute_corner_lasing_mode_purity()
            > low_solver.compute_corner_lasing_mode_purity(),
        "Higher saturation intensity ceiling must support higher corner lasing purity"
    );
    assert!(
        low_solver.compute_skin_sensitivity_factor()
            > high_solver.compute_skin_sensitivity_factor(),
        "Lower saturation threshold maintains higher small-signal skin sensitivity"
    );
}

#[test]
fn test_pump_rate_scaling() {
    let mut low_pump = NonHermitianSkinOctupoleLaserParams::default();
    low_pump.pump_rate_normalized = 1.20;
    let mut high_pump = NonHermitianSkinOctupoleLaserParams::default();
    high_pump.pump_rate_normalized = 4.50;

    let low_solver = NonHermitianSkinOctupoleLaserSolver::new(low_pump);
    let high_solver = NonHermitianSkinOctupoleLaserSolver::new(high_pump);

    assert!(
        high_solver.compute_corner_lasing_mode_purity()
            > low_solver.compute_corner_lasing_mode_purity(),
        "Higher pump rate above threshold must enhance stimulated corner lasing purity"
    );
    assert!(
        high_solver.compute_higher_order_skin_topological_gap_mhz()
            > low_solver.compute_higher_order_skin_topological_gap_mhz(),
        "Higher pump rate must widen the effective non-Hermitian topological gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher pump rate must stabilize mode coherence and suppress dephasing"
    );
}

#[test]
fn test_acoustic_octupole_frequency_scaling() {
    let mut low_f = NonHermitianSkinOctupoleLaserParams::default();
    low_f.acoustic_octupole_frequency_ghz = 1.5;
    let mut high_f = NonHermitianSkinOctupoleLaserParams::default();
    high_f.acoustic_octupole_frequency_ghz = 14.0;

    let low_solver = NonHermitianSkinOctupoleLaserSolver::new(low_f);
    let high_solver = NonHermitianSkinOctupoleLaserSolver::new(high_f);

    assert!(
        high_solver.compute_skin_sensitivity_factor()
            > low_solver.compute_skin_sensitivity_factor(),
        "Higher acoustic frequency must elevate skin displacement sensitivity"
    );
    assert!(
        high_solver.compute_higher_order_skin_topological_gap_mhz()
            > low_solver.compute_higher_order_skin_topological_gap_mhz(),
        "Higher acoustic frequency must expand topological protection gap"
    );
    assert!(
        high_solver.compute_topological_mode_dephasing_rate_hz()
            < low_solver.compute_topological_mode_dephasing_rate_hz(),
        "Higher acoustic frequency must suppress low-frequency dephasing"
    );
}

#[test]
fn test_cryogenic_temperature_scaling() {
    let mut low_temp = NonHermitianSkinOctupoleLaserParams::default();
    low_temp.cryogenic_temperature_mk = 2.0;
    let mut high_temp = NonHermitianSkinOctupoleLaserParams::default();
    high_temp.cryogenic_temperature_mk = 45.0;

    let low_solver = NonHermitianSkinOctupoleLaserSolver::new(low_temp);
    let high_solver = NonHermitianSkinOctupoleLaserSolver::new(high_temp);

    assert!(
        low_solver.compute_topological_mode_dephasing_rate_hz()
            < high_solver.compute_topological_mode_dephasing_rate_hz(),
        "Lower cryogenic temperature must suppress thermal dephasing"
    );
    assert!(
        low_solver.compute_corner_lasing_mode_purity()
            > high_solver.compute_corner_lasing_mode_purity(),
        "Lower cryogenic temperature must improve corner lasing purity"
    );
    assert!(
        low_solver.compute_corner_to_bulk_crosstalk_isolation_db()
            > high_solver.compute_corner_to_bulk_crosstalk_isolation_db(),
        "Lower cryogenic temperature must enhance crosstalk isolation"
    );
}

#[test]
fn test_lattice_cell_count_3d_scaling() {
    let mut small_lattice = NonHermitianSkinOctupoleLaserParams::default();
    small_lattice.lattice_cell_count_3d = 5.0;
    let mut large_lattice = NonHermitianSkinOctupoleLaserParams::default();
    large_lattice.lattice_cell_count_3d = 22.0;

    let small_solver = NonHermitianSkinOctupoleLaserSolver::new(small_lattice);
    let large_solver = NonHermitianSkinOctupoleLaserSolver::new(large_lattice);

    assert!(
        large_solver.compute_skin_sensitivity_factor()
            > small_solver.compute_skin_sensitivity_factor(),
        "Larger 3D lattice dimensions must exponentially amplify skin sensitivity"
    );
    assert!(
        large_solver.compute_corner_to_bulk_crosstalk_isolation_db()
            > small_solver.compute_corner_to_bulk_crosstalk_isolation_db(),
        "Larger 3D lattice dimensions must improve bulk-to-corner isolation"
    );
}

#[test]
fn test_skin_localization_decay_length_scaling() {
    let mut tight_localization = NonHermitianSkinOctupoleLaserParams::default();
    tight_localization.skin_localization_decay_length_nm = 15.0; // shorter = tighter
    let mut broad_localization = NonHermitianSkinOctupoleLaserParams::default();
    broad_localization.skin_localization_decay_length_nm = 110.0;

    let tight_solver = NonHermitianSkinOctupoleLaserSolver::new(tight_localization);
    let broad_solver = NonHermitianSkinOctupoleLaserSolver::new(broad_localization);

    assert!(
        tight_solver.compute_skin_sensitivity_factor()
            > broad_solver.compute_skin_sensitivity_factor(),
        "Tighter skin localization (shorter decay length) must enhance sensitivity"
    );
    assert!(
        tight_solver.compute_corner_lasing_mode_purity()
            > broad_solver.compute_corner_lasing_mode_purity(),
        "Tighter skin localization must increase corner lasing mode purity"
    );
    assert!(
        tight_solver.compute_corner_to_bulk_crosstalk_isolation_db()
            > broad_solver.compute_corner_to_bulk_crosstalk_isolation_db(),
        "Tighter skin localization must improve corner-to-bulk crosstalk isolation"
    );
    assert!(
        tight_solver.compute_topological_mode_dephasing_rate_hz()
            < broad_solver.compute_topological_mode_dephasing_rate_hz(),
        "Tighter skin localization must suppress topological mode dephasing"
    );
}
