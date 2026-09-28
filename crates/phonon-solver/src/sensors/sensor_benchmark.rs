//! 10,000-Tick Parallel Rayon Co-Simulation Benchmark Runner
//!
//! Benchmarks multi-world co-simulation throughput (ticks/sec), IMU Allan variance
//! tracking error (RMSE), specific force gravity compensation, and piezoresistive/capacitive
//! tactile force telemetry across parallel Rayon worker threads.

use crate::sensors::aerovex_bridge::{
    AerovexCoSimPacket, AerovexPhononBridge, AerovexRigidBodyState,
};
use phonon_models::em::{ChannelRng, Vector3D};
use phonon_models::sensors::{CollisionContactInput, ImuConfig, Quaternion, STANDARD_GRAVITY_M_S2};
use rayon::prelude::*;
use std::time::Instant;

/// Performance and validation report for multi-physics sensor co-simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorBenchmarkReport {
    /// Total number of physics co-simulation ticks processed across all worlds.
    pub total_ticks: usize,
    /// Total elapsed wall-clock execution time in milliseconds.
    pub elapsed_ms: f64,
    /// Co-simulation stepping throughput in ticks per second.
    pub ticks_per_second: f64,
    /// Accelerometer specific force measurement RMSE against ground truth (m/s^2).
    pub accel_rmse: f64,
    /// Gyroscope angular rate measurement RMSE against ground truth (rad/s).
    pub gyro_rmse: f64,
    /// Ratio of transduced tactile force fidelity against applied normal force.
    pub tactile_tracking_fidelity: f64,
    /// Total number of contact impact frames detected.
    pub contact_impacts_detected: usize,
    /// Verification flag confirming stationary accelerometer gravity compensation.
    pub gravity_compensation_verified: bool,
}

/// Parallel Rayon Sensor Benchmark Runner.
pub struct SensorBenchmarkRunner;

impl SensorBenchmarkRunner {
    /// Executes the multi-threaded co-simulation benchmark across `num_ticks` total ticks.
    pub fn run_benchmark(num_ticks: usize) -> SensorBenchmarkReport {
        let start = Instant::now();

        let num_worlds = 16.min(num_ticks.max(1));
        let ticks_per_world = num_ticks.div_ceil(num_worlds);
        let dt = 0.002; // 500 Hz physics tick

        // Run co-simulation across parallel Rayon threads:
        let world_results: Vec<(f64, f64, usize, f64, usize, bool)> = (0..num_worlds)
            .into_par_iter()
            .map(|world_id| {
                let mut rng = ChannelRng::new(0xDEADBEEF ^ (world_id as u64 + 1));
                let mut bridge = AerovexPhononBridge::new(
                    ImuConfig::new_industrial_9dof(),
                    4,
                    4,
                    0.005, // 5 mm pitch
                );

                let mut accel_err_sq_sum = 0.0;
                let mut gyro_err_sq_sum = 0.0;
                let mut contact_frames = 0;
                let mut tactile_fidelity_sum = 0.0;
                let mut gravity_comp_ok = true;

                for k in 0..ticks_per_world {
                    let sim_time_s = k as f64 * dt;

                    // Kinematics trajectory (dynamic maneuvers):
                    let pitch = 0.08 * (std::f64::consts::TAU * 0.5 * sim_time_s).sin();
                    let roll = 0.05 * (std::f64::consts::TAU * 0.7 * sim_time_s).cos();
                    let yaw = 0.02 * sim_time_s;
                    let orientation = Quaternion::from_euler_rpy(roll, pitch, yaw);

                    // Angular velocity [p, q, r] in body frame:
                    let p = -0.05
                        * (std::f64::consts::TAU * 0.7)
                        * (std::f64::consts::TAU * 0.7 * sim_time_s).sin();
                    let q = 0.08
                        * (std::f64::consts::TAU * 0.5)
                        * (std::f64::consts::TAU * 0.5 * sim_time_s).cos();
                    let r = 0.02;
                    let true_gyro = Vector3D::new(p, q, r);

                    let true_accel_body = Vector3D::new(
                        0.15 * (3.0 * sim_time_s).sin(),
                        0.10 * (2.0 * sim_time_s).cos(),
                        0.05 * sim_time_s.sin(),
                    );

                    // Ground truth specific force: f = a_body - g_body
                    let g_world = Vector3D::new(0.0, 0.0, -STANDARD_GRAVITY_M_S2);
                    let g_body = orientation.rotate_vector_world_to_body(g_world);
                    let true_specific_force = true_accel_body - g_body;

                    let vehicle_pos = Vector3D::new(0.0, 0.0, 1.0);

                    // Collision contact manifold during landing phase (last 25% of ticks):
                    let is_touchdown = k >= (ticks_per_world * 3 / 4);
                    let contacts = if is_touchdown {
                        let f_n = 14.71 + 2.0 * (40.0 * sim_time_s).sin();
                        vec![CollisionContactInput::new(
                            vehicle_pos + bridge.sensor_center_body,
                            Vector3D::new(0.0, 0.0, 1.0),
                            f_n,
                            Vector3D::new(0.1 * f_n, 0.0, 0.0),
                            0.001,
                        )]
                    } else {
                        Vec::new()
                    };

                    let packet = AerovexCoSimPacket {
                        sim_time_s,
                        tick_index: k as u64,
                        world_id: world_id as u32,
                        rigid_body: AerovexRigidBodyState {
                            position_world: vehicle_pos,
                            linear_velocity_world: Vector3D::new(0.0, 0.0, 0.0),
                            linear_accel_body: true_accel_body,
                            angular_velocity_body: true_gyro,
                            orientation,
                            mass_kg: 1.5,
                        },
                        collision_contacts: contacts.clone(),
                        ambient_temp_kelvin: 298.15,
                    };

                    let output = bridge.step(&packet, &mut rng);

                    // Check gravity compensation on tick 0:
                    if k == 0 {
                        let diff_g =
                            (output.imu_telemetry.accel_m_s2.z - STANDARD_GRAVITY_M_S2).abs();
                        if diff_g > 0.5 {
                            gravity_comp_ok = false;
                        }
                    }

                    // Accumulate errors:
                    let a_err =
                        (output.imu_telemetry.accel_m_s2 - true_specific_force).norm_squared();
                    let g_err = (output.imu_telemetry.gyro_rad_s - true_gyro).norm_squared();
                    accel_err_sq_sum += a_err;
                    gyro_err_sq_sum += g_err;

                    if !contacts.is_empty() {
                        contact_frames += 1;
                        let applied_fn = contacts[0].normal_force_n;
                        if applied_fn > 1e-3 {
                            let ratio = output.total_contact_force_n / applied_fn;
                            tactile_fidelity_sum += ratio;
                        }
                    }
                }

                (
                    accel_err_sq_sum,
                    gyro_err_sq_sum,
                    ticks_per_world,
                    tactile_fidelity_sum,
                    contact_frames,
                    gravity_comp_ok,
                )
            })
            .collect();

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;

        let mut total_accel_err_sq = 0.0;
        let mut total_gyro_err_sq = 0.0;
        let mut total_ticks_done = 0;
        let mut total_tactile_fidelity_sum = 0.0;
        let mut total_contact_frames = 0;
        let mut all_gravity_ok = true;

        for (a_err, g_err, ticks, tact_fid, contacts, grav_ok) in world_results {
            total_accel_err_sq += a_err;
            total_gyro_err_sq += g_err;
            total_ticks_done += ticks;
            total_tactile_fidelity_sum += tact_fid;
            total_contact_frames += contacts;
            if !grav_ok {
                all_gravity_ok = false;
            }
        }

        let total_ticks_f = total_ticks_done.max(1) as f64;
        let accel_rmse = (total_accel_err_sq / total_ticks_f).sqrt();
        let gyro_rmse = (total_gyro_err_sq / total_ticks_f).sqrt();
        let ticks_per_second = (total_ticks_done as f64) / (elapsed_ms / 1000.0).max(1e-6);

        let tactile_fidelity = if total_contact_frames > 0 {
            total_tactile_fidelity_sum / (total_contact_frames as f64)
        } else {
            1.0
        };

        SensorBenchmarkReport {
            total_ticks: total_ticks_done,
            elapsed_ms,
            ticks_per_second,
            accel_rmse,
            gyro_rmse,
            tactile_tracking_fidelity: tactile_fidelity,
            contact_impacts_detected: total_contact_frames,
            gravity_compensation_verified: all_gravity_ok,
        }
    }
}
