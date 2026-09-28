#![allow(clippy::needless_range_loop)]
//! Integration tests for Phase 45: Spacecraft GNC, Reaction Wheel Cluster & Benchmark Engine.

use phonon_models::em::Vector3D;
use phonon_models::sensors::imu::Quaternion;
use phonon_models::space::attitude::{MagneticTorquerSystem, ReactionWheelCluster};
use phonon_models::space::star_tracker::{StarTrackerCamera, StarTrackerSystem};
use phonon_solver::space::{
    GncBenchmarkRunner, GncConfig, PointingMode, SpacecraftGncSolver, SpacecraftState,
};

#[test]
fn test_closed_loop_nadir_pointing_control_convergence() {
    let config = GncConfig {
        kp_attitude: 10.0,
        kd_attitude: 22.0,
        pointing_mode: PointingMode::NadirPointing,
        ..Default::default()
    };
    let camera = StarTrackerCamera::default();
    let tracker = StarTrackerSystem::new(camera);

    let mut state = SpacecraftState::new_leo(500_000.0, 0.0);
    // Introduce initial attitude perturbation: 5 degrees tilt from nominal nadir
    let q_nadir = state.target_quaternion(PointingMode::NadirPointing);
    let q_perturb = Quaternion::from_euler_rpy(0.087, -0.05, 0.0);
    state.attitude_q = q_nadir.multiply(&q_perturb);
    state.attitude_q_estimated = state.attitude_q;

    let dt = 0.05; // 50 ms time step
    for _ in 0..500 {
        SpacecraftGncSolver::step(&mut state, &config, &tracker, dt);
    }

    let final_pointing_error = state.pointing_error_deg(PointingMode::NadirPointing);
    assert!(
        final_pointing_error < 0.005,
        "Closed-loop nadir pointing error must converge to < 0.005 deg, got {final_pointing_error} deg"
    );
}

#[test]
fn test_reaction_wheel_cluster_momentum_and_desaturation() {
    let mut cluster = ReactionWheelCluster::pyramid_cluster(0.002, 600.0, 0.05);
    let tau_cmd = Vector3D::new(0.01, -0.015, 0.005);

    let actual_torque = cluster.step(tau_cmd, 0.1);
    assert!(
        actual_torque.norm() > 0.0,
        "Reaction wheel cluster must exert reaction torque"
    );

    let h_tot = cluster.total_momentum();
    assert!(
        h_tot.norm() > 0.0,
        "Wheel cluster must accumulate angular momentum"
    );

    // Magnetic desaturation:
    let torquers = MagneticTorquerSystem::new(15.0);
    let b_field = Vector3D::new(1.0e-5, -2.0e-5, 3.0e-5); // 30 microTesla
    let m_dipole = torquers.evaluate_desaturation_dipole(h_tot, b_field);
    let tau_desat = torquers.evaluate_torque(m_dipole, b_field);

    assert!(
        m_dipole.norm() > 0.0,
        "Desaturation dipole must be non-zero"
    );
    assert!(tau_desat.norm() > 0.0, "Magnetic torque must be generated");
}

#[test]
fn test_mekf_attitude_and_gyro_bias_estimation() {
    let config = GncConfig::default();
    let camera = StarTrackerCamera::default();
    let tracker = StarTrackerSystem::new(camera);

    let mut state = SpacecraftState::new_leo(500_000.0, 51.6 * std::f64::consts::PI / 180.0);
    let true_bias = state.gyro_bias_true;

    let dt = 0.1;
    for _ in 0..100 {
        SpacecraftGncSolver::step(&mut state, &config, &tracker, dt);
    }

    let bias_err = Vector3D::new(
        true_bias.x - state.gyro_bias_estimated.x,
        true_bias.y - state.gyro_bias_estimated.y,
        true_bias.z - state.gyro_bias_estimated.z,
    );

    assert!(
        bias_err.norm() < true_bias.norm(),
        "MEKF should reduce gyroscope bias error: initial {:e}, final {:e}",
        true_bias.norm(),
        bias_err.norm()
    );
}

#[test]
fn test_multi_threaded_rayon_gnc_benchmark_suite() {
    let report = GncBenchmarkRunner::run_benchmark(500, 0.05);

    assert!(
        report.benchmarks_passed,
        "All Phase 45 GNC benchmark criteria must pass: {:#?}",
        report
    );
    assert!(
        report.nadir_pointing_error_deg < 0.005,
        "Nadir pointing error must be < 0.005 deg, got {}",
        report.nadir_pointing_error_deg
    );
    assert!(
        report.max_wheel_momentum_utilization < 0.85,
        "Wheel momentum utilization must be < 85%, got {}",
        report.max_wheel_momentum_utilization
    );
    assert!(
        report.orbital_energy_conservation_fraction < 1.0e-3,
        "Orbital energy conservation must be < 1e-3, got {:e}",
        report.orbital_energy_conservation_fraction
    );
    assert!(
        report.throughput_steps_per_sec > 10_000.0,
        "Multi-threaded Rayon throughput must be > 10,000 steps/sec, got {}",
        report.throughput_steps_per_sec
    );
}
