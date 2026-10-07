#![deny(unsafe_code)]

//! Automated physics test suite for Phase 412:
//! Polariton BEC Vortices, Non-Equilibrium Superfluidity & Josephson Acoustic Interferometer.

use phonon_solver::polariton_bec_vortices::{
    GrossPitaevskiiSolver, JosephsonInterferometerParams, JosephsonInterferometerSolver,
    PolaritonBecInterferometer, PolaritonBecParams, QuantizedVortexSolver, VortexCharge,
    VortexSuperfluidParams,
};

#[test]
fn test_condensation_threshold_and_density() {
    // Below threshold
    let mut p_sub = PolaritonBecParams::default();
    p_sub.pump_power_ratio = 0.75;
    let solver_sub = GrossPitaevskiiSolver::new(p_sub);
    assert!(!solver_sub.is_condensed());
    assert_eq!(solver_sub.steady_state_peak_density(), 0.0);
    assert_eq!(solver_sub.bogoliubov_sound_speed_ms(), 0.0);

    // Above threshold
    let mut p_super = PolaritonBecParams::default();
    p_super.pump_power_ratio = 2.5;
    let solver_super = GrossPitaevskiiSolver::new(p_super);
    assert!(solver_super.is_condensed());
    let n0 = solver_super.steady_state_peak_density();
    assert!(n0 > 0.0);
    let vs = solver_super.bogoliubov_sound_speed_ms();
    assert!(vs > 1.0e5, "Sound speed should exceed 1e5 m/s, got {:.2e}", vs);
}

#[test]
fn test_healing_length_and_spatial_profile() {
    let solver = GrossPitaevskiiSolver::new(PolaritonBecParams::default());
    let xi = solver.healing_length_um();
    assert!(xi >= 0.5 && xi <= 5.0, "Healing length should be in [0.5, 5.0] um, got {:.2}", xi);

    let profile = solver.compute_spatial_profile(20.0, 40);
    assert_eq!(profile.len(), 40);
    assert!((profile[0].coherence_g1 - 1.0).abs() < 1e-4, "g^(1)(0) must equal 1.0");
    assert!(profile[0].density_um2 > profile.last().unwrap().density_um2);
}

#[test]
fn test_quantized_vortex_charge_and_core_depletion() {
    let p = VortexSuperfluidParams {
        charge: VortexCharge::PlusOne,
        ..Default::default()
    };
    let solver = QuantizedVortexSolver::new(p);

    // Core depletion: center density vanishing
    let center = solver.evaluate_grid_point(0.0, 0.0);
    assert!(center.normalized_density < 0.01, "Core density must vanish at origin");

    // Away from core: density recovers towards 1.0
    let far = solver.evaluate_grid_point(8.0, 0.0);
    assert!(far.normalized_density > 0.85);

    // Phase winding: evaluate 4 quadrants around origin
    let p_east = solver.evaluate_grid_point(3.0, 0.0);
    let p_north = solver.evaluate_grid_point(0.0, 3.0);
    let p_west = solver.evaluate_grid_point(-3.0, 0.0);
    let p_south = solver.evaluate_grid_point(0.0, -3.0);

    assert!((p_east.phase_rad - 0.0).abs() < 1e-4);
    assert!((p_north.phase_rad - std::f64::consts::FRAC_PI_2).abs() < 1e-4);
    assert!((p_west.phase_rad.abs() - std::f64::consts::PI).abs() < 1e-4);
    assert!((p_south.phase_rad + std::f64::consts::FRAC_PI_2).abs() < 1e-4);
}

#[test]
fn test_landau_superfluidity_and_drag_suppression() {
    // Subcritical flow v = 0.5 * v_c
    let mut p_sub = VortexSuperfluidParams::default();
    p_sub.flow_velocity_ratio = 0.5;
    let solver_sub = QuantizedVortexSolver::new(p_sub);
    let (drag_sub, supp_db, is_super) = solver_sub.compute_obstacle_drag();

    assert!(is_super, "Subcritical flow must be frictionless superfluid");
    assert!(drag_sub < 1e-3);
    assert!(supp_db >= 30.0, "Drag suppression must be >= 30.0 dB, got {:.1} dB", supp_db);

    // Supercritical flow v = 1.5 * v_c
    let mut p_super = VortexSuperfluidParams::default();
    p_super.flow_velocity_ratio = 1.5;
    let solver_super = QuantizedVortexSolver::new(p_super);
    let (drag_super, _, is_super_flow) = solver_super.compute_obstacle_drag();

    assert!(!is_super_flow, "Supercritical flow must experience finite drag");
    assert!(drag_super > 0.05);
}

#[test]
fn test_josephson_plasma_oscillations() {
    let p = JosephsonInterferometerParams {
        initial_imbalance: 0.15, // small amplitude -> linear plasma oscillation
        applied_strain: 0.0,
        total_time_ps: 100.0,
        ..Default::default()
    };
    let solver = JosephsonInterferometerSolver::new(p);
    let f_j = solver.plasma_frequency_ghz();
    assert!(f_j >= 10.0 && f_j <= 150.0, "Plasma frequency f_J in [10, 150] GHz, got {:.1}", f_j);

    let traj = solver.solve_dynamics();
    assert!(!traj.is_empty());
    // Oscillation should cross zero
    let min_z = traj.iter().map(|pt| pt.imbalance_z).fold(f64::INFINITY, f64::min);
    let max_z = traj.iter().map(|pt| pt.imbalance_z).fold(f64::NEG_INFINITY, f64::max);
    assert!(min_z < 0.0);
    assert!(max_z > 0.0);
}

#[test]
fn test_mqst_transition() {
    // Large initial imbalance -> MQST
    let p_mqst = JosephsonInterferometerParams {
        initial_imbalance: 0.85,
        charging_energy_ec_mev: 0.15,
        josephson_coupling_ej_mev: 0.02,
        total_time_ps: 80.0,
        ..Default::default()
    };
    let solver = JosephsonInterferometerSolver::new(p_mqst);
    let z_c = solver.mqst_critical_imbalance();
    assert!(solver.params.initial_imbalance > z_c, "Initial imbalance must exceed z_c for MQST");

    let traj = solver.solve_dynamics();
    // In MQST, imbalance never crosses zero
    let min_z = traj.iter().map(|pt| pt.imbalance_z).fold(f64::INFINITY, f64::min);
    assert!(min_z > 0.10, "MQST must maintain positive population imbalance");
}

#[test]
fn test_acoustic_strain_sensitivity() {
    let p = JosephsonInterferometerParams::default();
    let solver = JosephsonInterferometerSolver::new(p);
    let metrics = solver.evaluate_sensor_metrics();

    assert!(metrics.minimum_detectable_strain < 1.0e-9, "Minimum strain must be sub-nanostrain, got {:.2e}", metrics.minimum_detectable_strain);
    assert!(metrics.fringe_visibility >= 0.90, "Fringe visibility must be >= 90%");

    let fringes = solver.compute_fringe_pattern(0.5, 50);
    assert_eq!(fringes.len(), 50);
}

#[test]
fn test_10_point_physics_audit_pass() {
    let system = PolaritonBecInterferometer::default();
    let audit = system.audit_polariton_bec();

    assert_eq!(audit.total_count, 10);
    assert_eq!(audit.passed_count, 10, "Expected all 10 criteria to pass, got: {:?}", audit.criteria);
    assert!(audit.all_passed);
}
