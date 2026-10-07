#![deny(unsafe_code)]

//! Automated physics test suite for Phase 414: Phonon Studio Non-Hermitian Higher-Order
//! Topological Quadrupole Skin-Effect Laser & Chiral Edge Emitter.

use phonon_solver::non_hermitian_skin_laser::{
    ChiralEmitterParams, ChiralEmitterSolver, NonHermitianSkinLaser, QuadrupoleSkinParams,
    QuadrupoleSkinSolver, TopologicalLaserParams, TopologicalLaserSolver,
};

#[test]
fn test_quadrupole_skin_metrics_and_topological_moment() {
    let params = QuadrupoleSkinParams::default();
    let solver = QuadrupoleSkinSolver::new(params.clone());
    let metrics = solver.evaluate_metrics();

    // Verify topological quadrupole moment
    assert_eq!(metrics.quadrupole_moment_qxy, 0.5);
    // Verify bulk bandgap exceeds 1.5 MHz
    assert!(
        metrics.bulk_bandgap_mhz >= 1.5,
        "Bulk bandgap {:.2} MHz must be >= 1.5 MHz",
        metrics.bulk_bandgap_mhz
    );
    // Verify skin depth
    assert!(metrics.skin_depth_mm > 0.0);
    // Verify total site count for 6x6 grid with 4 sites/cell
    assert_eq!(metrics.total_sites, 6 * 6 * 4);
}

#[test]
fn test_generalized_brillouin_zone_and_winding_number() {
    let params = QuadrupoleSkinParams {
        skin_asymmetry_g: 0.85,
        ..Default::default()
    };
    let solver = QuadrupoleSkinSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // GBZ radius must be strictly less than 1.0 under asymmetric hopping
    assert!(
        metrics.gbz_radius < 1.0,
        "GBZ radius {:.4} must be < 1.0",
        metrics.gbz_radius
    );
    // Complex point-gap winding number must be 1
    assert_eq!(metrics.point_gap_winding, 1);

    // Verify GBZ trajectory sampling
    let traj = solver.compute_gbz_trajectory(20);
    assert_eq!(traj.len(), 20);
    for (re, im) in &traj {
        let r = (re.powi(2) + im.powi(2)).sqrt();
        assert!((r - metrics.gbz_radius).abs() < 1e-6);
    }
}

#[test]
fn test_corner_skin_mode_confinement() {
    let params = QuadrupoleSkinParams::default();
    let solver = QuadrupoleSkinSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Modal confinement in the corner must be >= 85.0%
    assert!(
        metrics.corner_confinement_ratio >= 0.85,
        "Corner confinement {:.2}% must be >= 85.0%",
        metrics.corner_confinement_ratio * 100.0
    );

    // Verify spatial intensity profile
    let profile = solver.generate_spatial_profile();
    assert_eq!(profile.len(), metrics.total_sites);

    // Peak intensity must occur on a corner site
    let max_point = profile
        .iter()
        .max_by(|a, b| a.intensity.partial_cmp(&b.intensity).unwrap())
        .expect("Non-empty profile");
    assert!(max_point.is_corner_site);
}

#[test]
fn test_complex_eigenfrequency_spectrum() {
    let params = QuadrupoleSkinParams::default();
    let solver = QuadrupoleSkinSolver::new(params.clone());
    let spectrum = solver.compute_complex_spectrum(30);

    assert!(spectrum.len() >= 30);
    // Corner modes are at center frequency
    let corner_modes: Vec<_> = spectrum.iter().filter(|m| m.is_corner_mode).collect();
    assert!(corner_modes.len() >= 2);
    for cm in corner_modes {
        assert!((cm.re_freq_mhz - params.center_freq_mhz).abs() < 1.0);
    }
}

#[test]
fn test_topological_laser_threshold_and_discrimination() {
    let params = TopologicalLaserParams::default();
    let solver = TopologicalLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    // Corner mode net gain must be positive (lasing)
    assert!(
        metrics.corner_gain_mhz > 0.0,
        "Corner mode net gain must be positive"
    );
    // Bulk mode net gain must be negative or severely suppressed
    assert!(metrics.bulk_gain_mhz < metrics.corner_gain_mhz);

    // Modal discrimination must be >= 15.0 dB
    assert!(
        metrics.modal_discrimination_db >= 15.0,
        "Modal discrimination {:.2} dB must be >= 15.0 dB",
        metrics.modal_discrimination_db
    );
}

#[test]
fn test_laser_curve_and_slope_efficiency() {
    let params = TopologicalLaserParams {
        pump_rate_mhz: 3.5,
        threshold_pump_mhz: 1.2,
        ..Default::default()
    };
    let solver = TopologicalLaserSolver::new(params);
    let metrics = solver.evaluate_metrics();

    assert!(metrics.is_lasing);
    // Slope efficiency >= 40.0%
    assert!(
        metrics.slope_efficiency >= 0.40,
        "Slope efficiency {:.2}% must be >= 40.0%",
        metrics.slope_efficiency * 100.0
    );
    assert!(metrics.output_power_mw > 0.0);

    // Linewidth narrowing: above-threshold linewidth is sub-kHz
    assert!(metrics.laser_linewidth_hz < 100.0);

    // Test L-I curve sampling
    let curve = solver.compute_laser_curve(25);
    assert_eq!(curve.len(), 26);
    let below = &curve[2];
    let above = &curve[20];
    assert!(above.output_power_mw > below.output_power_mw);
}

#[test]
fn test_chiral_emitter_directivity_and_hpbw() {
    let params = ChiralEmitterParams::default();
    let solver = ChiralEmitterSolver::new(params);
    let metrics = solver.evaluate_metrics(15.0);

    // Front-to-back directivity >= 25.0 dB
    assert!(
        metrics.front_to_back_directivity_db >= 25.0,
        "Directivity {:.2} dB must be >= 25.0 dB",
        metrics.front_to_back_directivity_db
    );
    // HPBW <= 25.0 deg
    assert!(
        metrics.hpbw_deg <= 25.0,
        "HPBW {:.2} deg must be <= 25.0 deg",
        metrics.hpbw_deg
    );
    assert!(metrics.forward_power_mw > metrics.backward_power_mw * 300.0);

    // Radiation pattern verification
    let pattern = solver.compute_radiation_pattern(36);
    assert_eq!(pattern.len(), 36);
    let peak = pattern
        .iter()
        .max_by(|a, b| a.power_db.partial_cmp(&b.power_db).unwrap())
        .unwrap();
    assert!(peak.angle_deg.abs() <= 15.0);
}

#[test]
fn test_defect_immunity_and_10_point_audit() {
    let mut emitter_params = ChiralEmitterParams::default();
    emitter_params.defect_present = true;
    let emitter = ChiralEmitterSolver::new(emitter_params);
    let emitter_m = emitter.evaluate_metrics(15.0);

    // Defect transmission ratio >= 95.0%
    assert!(
        emitter_m.defect_transmission_ratio >= 0.95,
        "Defect transmission ratio {:.2}% must be >= 95.0%",
        emitter_m.defect_transmission_ratio * 100.0
    );

    // Master system 10-point physics audit
    let system = NonHermitianSkinLaser::default();
    let audit = system.audit_non_hermitian_skin_laser();

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
