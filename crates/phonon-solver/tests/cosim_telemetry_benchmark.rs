#![deny(unsafe_code)]

//! Integration Tests: Co-Simulation Telemetry & Parallel Benchmark
//!
//! Validates:
//! 1. Flight co-simulation bridge stepping across continuous quadrotor maneuvers.
//! 2. Touchdown contact impact telemetry with piezoresistive and capacitive transducers.
//! 3. 10,000-tick parallel Rayon co-simulation benchmark with Allan variance RMSE validation.

use phonon_models::em::{ChannelRng, Vector3D};
use phonon_models::sensors::{CollisionContactInput, ImuConfig, Quaternion};
use phonon_solver::sensors::{
    CoSimPacket, PhononCoSimBridge, RigidBodyCoSimState, SensorBenchmarkRunner,
};

#[test]
fn test_phonon_cosim_bridge_stepping_and_touchdown() {
    let mut bridge = PhononCoSimBridge::new(ImuConfig::new_industrial_9dof(), 4, 4, 0.005);
    let mut rng = ChannelRng::new(54321);

    // 1. Step in free flight:
    let flight_packet = CoSimPacket {
        sim_time_s: 0.002,
        tick_index: 1,
        world_id: 0,
        rigid_body: RigidBodyCoSimState {
            position_world: Vector3D::new(0.0, 0.0, 5.0),
            linear_velocity_world: Vector3D::new(1.0, 0.0, 0.0),
            linear_accel_body: Vector3D::new(0.0, 0.0, 0.0),
            angular_velocity_body: Vector3D::new(0.0, 0.0, 0.0),
            orientation: Quaternion::default(),
            mass_kg: 1.5,
        },
        collision_contacts: Vec::new(),
        ambient_temp_kelvin: 298.15,
    };

    let out_flight = bridge.step(&flight_packet, &mut rng);
    assert_eq!(out_flight.total_contact_force_n, 0.0);
    assert_eq!(out_flight.sim_time_s, 0.002);
    assert_eq!(out_flight.tick_index, 1);
    // Unloaded capacitance:
    assert!((out_flight.capacitive_contact_f - 5.6667e-12).abs() < 1e-15);

    // 2. Step with landing touchdown on pad:
    let touchdown_force = 18.5; // Newtons
    let touchdown_contact = CollisionContactInput::new(
        Vector3D::new(0.0, 0.0, 0.0) + bridge.sensor_center_body,
        Vector3D::new(0.0, 0.0, 1.0),
        touchdown_force,
        Vector3D::new(1.0, 0.0, 0.0),
        0.001,
    );

    let landing_packet = CoSimPacket {
        sim_time_s: 0.004,
        tick_index: 2,
        world_id: 0,
        rigid_body: RigidBodyCoSimState {
            position_world: Vector3D::new(0.0, 0.0, 0.0),
            linear_velocity_world: Vector3D::ZERO,
            linear_accel_body: Vector3D::ZERO,
            angular_velocity_body: Vector3D::ZERO,
            orientation: Quaternion::default(),
            mass_kg: 1.5,
        },
        collision_contacts: vec![touchdown_contact],
        ambient_temp_kelvin: 298.15,
    };

    let out_landing = bridge.step(&landing_packet, &mut rng);
    assert!((out_landing.total_contact_force_n - touchdown_force).abs() < 1e-6);
    // Loaded capacitance increases:
    assert!(out_landing.capacitive_contact_f > out_flight.capacitive_contact_f);
    // Center of pressure is near (0, 0):
    assert!(out_landing.center_of_pressure_m.0.abs() < 1e-3);
    assert!(out_landing.center_of_pressure_m.1.abs() < 1e-3);
}

#[test]
fn test_parallel_rayon_sensor_benchmark_10k_ticks() {
    let report = SensorBenchmarkRunner::run_benchmark(10_000);

    println!("\n=== Multi-Physics Sensor Co-Simulation Benchmark Report ===");
    println!("Total Ticks Processed: {}", report.total_ticks);
    println!("Elapsed Wall-Clock:    {:.2} ms", report.elapsed_ms);
    println!(
        "Throughput:            {:.0} ticks/sec",
        report.ticks_per_second
    );
    println!("Accel RMSE:            {:.4} m/s^2", report.accel_rmse);
    println!("Gyro RMSE:             {:.5} rad/s", report.gyro_rmse);
    println!(
        "Tactile Fidelity:      {:.4}",
        report.tactile_tracking_fidelity
    );
    println!("Contact Impacts:       {}", report.contact_impacts_detected);
    println!(
        "Gravity Comp Verified: {}",
        report.gravity_compensation_verified
    );

    assert!(report.total_ticks == 10_000);
    assert!(report.ticks_per_second > 50_000.0);
    assert!(report.accel_rmse < 0.5);
    assert!(report.gyro_rmse < 0.05);
    assert!(report.tactile_tracking_fidelity > 0.95);
    assert!(report.contact_impacts_detected > 0);
    assert!(report.gravity_compensation_verified);
}
