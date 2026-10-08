#![deny(unsafe_code)]

//! Analytical verification test suite for Phase 426: Chiral Magnon-Phonon
//! Heat Transistor and Thermal Diode Engine.

use phonon_solver::chiral_heat_transistor::{
    ChiralHeatTransistor, ChiralHeatTransistorProcessor, ChiralThermalRectifier,
    HeatTransistorParams, ThermalDiodeParams,
};

#[test]
fn test_thermal_rectification_ratio_and_isolation() {
    let params = ThermalDiodeParams::default();
    let rectifier = ChiralThermalRectifier::new(params.clone());

    let j_fwd = rectifier.calculate_forward_heat_flux(params.source_temp_k, params.drain_temp_k);
    let j_bwd = rectifier.calculate_backward_heat_flux(params.source_temp_k, params.drain_temp_k);

    assert!(j_fwd > 0.0, "Forward heat flux must be strictly positive");
    assert!(j_bwd > 0.0, "Backward heat flux must be non-zero");

    let ratio = j_fwd / j_bwd;
    assert!(
        ratio >= 25.0,
        "Thermal rectification ratio must exceed 25.0 (got {ratio:.2})"
    );

    let metrics = rectifier.evaluate_rectification_metrics();
    assert!(
        metrics.backward_isolation_db >= 15.0,
        "Backward isolation must exceed 15.0 dB (got {:.2} dB)",
        metrics.backward_isolation_db
    );
    assert!(metrics.directivity_factor >= 0.85);
}

#[test]
fn test_rectification_curve_generation() {
    let rectifier = ChiralThermalRectifier::default();
    let curve = rectifier.compute_rectification_curve(25, 400.0);

    assert_eq!(curve.len(), 25);
    for pt in &curve {
        assert!(pt.delta_t_mk > 0.0);
        assert!(pt.forward_flux_pw >= pt.backward_flux_pw);
        assert!(pt.rectification_ratio >= 20.0);
    }
}

#[test]
fn test_heat_transistor_ep_threshold_and_differential_gain() {
    let params = HeatTransistorParams::default();
    let transistor = ChiralHeatTransistor::new(params.clone());

    let g_ep = transistor.calculate_ep_threshold_mhz();
    assert!(g_ep > 0.0, "EP threshold coupling must be positive");
    assert_eq!(
        g_ep,
        0.5 * (params.magnon_damping_mhz - params.phonon_damping_mhz).abs()
    );

    let transfer = transistor.compute_transfer_curve(40);
    assert_eq!(transfer.len(), 40);

    let max_gain = transfer
        .iter()
        .map(|p| p.differential_gain)
        .fold(0.0_f64, f64::max);

    assert!(
        max_gain >= 5.0,
        "Transistor maximum differential gain must exceed 5.0 (got {max_gain:.2})"
    );

    let metrics = transistor.evaluate_transistor_metrics();
    assert!(metrics.max_differential_gain >= 5.0);
    assert!(metrics.on_off_ratio >= 10.0);
    assert!(metrics.drain_heat_flux_pw > 0.0);
}

#[test]
fn test_polariton_quasi_energy_dispersion() {
    let transistor = ChiralHeatTransistor::default();
    let dispersion = transistor.compute_polariton_dispersion(30);

    assert_eq!(dispersion.len(), 30);
    for pt in &dispersion {
        // Upper and lower branches must be symmetric around center
        assert!((pt.re_upper_mhz + pt.re_lower_mhz).abs() < 1e-10);
        assert!(pt.im_upper_mhz < 0.0, "Imaginary part must represent dissipation");
        assert!(pt.im_lower_mhz < 0.0);
    }
}

#[test]
fn test_10_point_physics_audit_checklist_pass() {
    let processor = ChiralHeatTransistorProcessor::default();
    let audit = processor.audit_processor();

    assert!(audit.drive_periodicity_pass);
    assert!(audit.ep_threshold_identifiable);
    assert!(audit.rectification_ratio_pass);
    assert!(audit.forward_heat_flux_positive);
    assert!(audit.backward_isolation_pass);
    assert!(audit.differential_gain_pass);
    assert!(audit.sub_kelvin_stability);
    assert!(audit.polariton_coupling_pass);
    assert!(audit.thermodynamic_on_off_pass);
    assert!(audit.cold_boot_throughput_pass);

    assert_eq!(audit.total_score, 10);
    assert!(audit.all_passed);
}
