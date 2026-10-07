#![deny(unsafe_code)]

//! Automated physics test suite for Phase 415: Phonon Studio Topological Acoustic Moiré
//! Flat-Band Polariton Soliton & Higher-Order Corner Comb Generator.

use phonon_solver::moire_polariton_comb::{
    CornerMicrocombParams, CornerMicrocombSolver, MoireFlatBandParams, MoireFlatBandSolver,
    MoirePolaritonComb, PolaritonSolitonParams, PolaritonSolitonSolver,
};

#[test]
fn test_moire_flat_band_metrics_and_magic_angle() {
    let params = MoireFlatBandParams {
        twist_angle_deg: 1.08,
        ..Default::default()
    };
    let solver = MoireFlatBandSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_magic_angle);
    // Bandwidth of the flat band must be quenched <= 0.50 MHz
    assert!(
        metrics.flat_bandwidth_mhz <= 0.50,
        "Flat bandwidth {:.3} MHz must be <= 0.50 MHz",
        metrics.flat_bandwidth_mhz
    );
    // Bulk bandgap must be >= 1.50 MHz
    assert!(
        metrics.bulk_bandgap_mhz >= 1.50,
        "Bulk bandgap {:.2} MHz must be >= 1.50 MHz",
        metrics.bulk_bandgap_mhz
    );
    // Flatness ratio F = W_flat / Delta_gap <= 0.20
    assert!(
        metrics.flatness_ratio <= 0.20,
        "Flatness ratio {:.3} must be <= 0.20",
        metrics.flatness_ratio
    );
    assert!(metrics.moire_period_lm_mm > 50.0);
}

#[test]
fn test_valley_chern_number_and_mass_divergence() {
    let params = MoireFlatBandParams::default();
    let solver = MoireFlatBandSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Quantized valley Chern number
    assert_eq!(metrics.valley_chern_number, 1.0);
    // Effective mass divergence in flat bands
    assert!(
        metrics.effective_mass_ratio > 5.0,
        "Effective mass ratio {:.1} must be > 5.0",
        metrics.effective_mass_ratio
    );
}

#[test]
fn test_moire_dispersion_path_calculation() {
    let params = MoireFlatBandParams::default();
    let solver = MoireFlatBandSolver::new(params.clone());
    let dispersion = solver.compute_band_dispersion(12);

    assert!(dispersion.len() >= 36);
    // Flat band must remain tightly bounded around center frequency
    for pt in &dispersion {
        let diff = (pt.flat_band_mhz - params.center_freq_mhz).abs();
        assert!(
            diff <= 0.35,
            "Flat band frequency deviation {:.3} MHz exceeds bound",
            diff
        );
        assert!(pt.lower_band_mhz < pt.flat_band_mhz);
        assert!(pt.upper_band_mhz > pt.flat_band_mhz);
    }
}

#[test]
fn test_moire_spatial_interference_pattern() {
    let params = MoireFlatBandParams::default();
    let solver = MoireFlatBandSolver::new(params);
    let profile = solver.generate_spatial_profile(14);

    assert_eq!(profile.len(), 14 * 14);
    // Verify existence of AA, AB, and BA stacking regions
    let has_aa = profile.iter().any(|pt| pt.stacking_type == 0);
    let has_ab = profile.iter().any(|pt| pt.stacking_type == 1);
    let has_ba = profile.iter().any(|pt| pt.stacking_type == 2);
    assert!(has_aa && has_ab && has_ba);
}

#[test]
fn test_polariton_soliton_envelope_and_self_trapping() {
    let params = PolaritonSolitonParams::default();
    let solver = PolaritonSolitonSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Soliton width <= 15.0 um
    assert!(
        metrics.soliton_width_um <= 15.0,
        "Soliton width {:.2} um must be <= 15.0 um",
        metrics.soliton_width_um
    );
    // Self-trapping ratio >= 85.0%
    assert!(
        metrics.self_trapping_ratio >= 0.85,
        "Self-trapping ratio {:.2}% must be >= 85.0%",
        metrics.self_trapping_ratio * 100.0
    );
    assert!(metrics.is_soliton_stable);
    assert!(metrics.nonlinear_phase_shift_rad > 0.0);
}

#[test]
fn test_soliton_dispersion_suppression() {
    let params = PolaritonSolitonParams::default();
    let solver = PolaritonSolitonSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Dispersion suppression factor > 1.0 (soliton avoids linear spreading)
    assert!(metrics.dispersion_suppression_ratio > 1.0);

    let profile = solver.compute_spatial_profile(30, 80.0);
    assert_eq!(profile.len(), 31);
    // Peak intensity at center (x = 0)
    let center_pt = profile.iter().min_by(|a, b| a.pos_x_um.abs().partial_cmp(&b.pos_x_um.abs()).unwrap()).unwrap();
    assert!(center_pt.soliton_intensity >= params.peak_power_mw * 0.95);
    assert!(center_pt.soliton_intensity > center_pt.linear_dispersive_intensity);
}

#[test]
fn test_higher_order_corner_microcomb_generation() {
    let params = CornerMicrocombParams {
        pump_power_mw: 18.0,
        target_comb_lines: 42,
        ..Default::default()
    };
    let solver = CornerMicrocombSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_comb_active);
    // Comb line count >= 40
    assert!(
        metrics.comb_line_count >= 40,
        "Comb line count {} must be >= 40",
        metrics.comb_line_count
    );
    // Conversion efficiency >= 25.0%
    assert!(
        metrics.conversion_efficiency >= 0.25,
        "Conversion efficiency {:.2}% must be >= 25.0%",
        metrics.conversion_efficiency * 100.0
    );
    // Corner confinement ratio >= 85.0%
    assert!(
        metrics.corner_confinement_ratio >= 0.85,
        "Corner confinement {:.2}% must be >= 85.0%",
        metrics.corner_confinement_ratio * 100.0
    );
    // Phase noise <= -110 dBc/Hz
    assert!(metrics.phase_noise_10khz_dbc <= -110.0);

    // Spectrum verification
    let spectrum = solver.compute_comb_spectrum();
    assert_eq!(spectrum.len(), metrics.comb_line_count);
    let pump_line = spectrum.iter().find(|l| l.is_pump_line).expect("Pump line exists");
    assert_eq!(pump_line.line_index, 0);

    // Corner spatial profile
    let corner_profile = solver.generate_corner_mode_profile(14);
    assert_eq!(corner_profile.len(), 14 * 14);
    let corner_pts: Vec<_> = corner_profile.iter().filter(|p| p.is_corner_site).collect();
    assert!(!corner_pts.is_empty());
}

#[test]
fn test_system_orchestrator_10_point_audit() {
    let system = MoirePolaritonComb::default();
    let audit = system.audit_moire_polariton_comb();

    assert_eq!(audit.total_count, 10);
    assert_eq!(
        audit.passed_count, 10,
        "Audit failed criteria: {:?}",
        audit
            .criteria
            .iter()
            .filter(|c| !c.passed)
            .collect::<Vec<_>>()
    );
    assert!(audit.all_passed);
}
