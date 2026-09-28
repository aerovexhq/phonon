//! Integration Test: Multi-Threaded Rayon Closed-Loop HIL Benchmark Runner
//!
//! Validates:
//! 1. Closed-loop trajectory tracking across 10,000 steps on multi-threaded Rayon workers.
//! 2. Position tracking RMSE < 0.15 m.
//! 3. Velocity tracking RMSE < 0.08 m/s.
//! 4. Attitude tracking RMSE < 1.5 deg.
//! 5. Throughput > 500,000 flight steps/sec.
//! 6. Autonomous GPS spoofing detection and rejection.
//! 7. Single-motor failure recovery and supervisory failsafe transitions.

use phonon_models::em::Vector3D;
use phonon_models::sensors::{AirframeConfig, FlightDynamicsState};
use phonon_solver::sensors::hil_benchmark::HilBenchmarkRunner;
use phonon_solver::sensors::hil_flight_bridge::{FailsafeMode, HilFlightBridge};

#[test]
fn test_hil_closed_loop_parallel_10k_steps_benchmark() {
    let report = HilBenchmarkRunner::run_benchmark(10_000);

    println!("=== Phase 40 HIL Closed-Loop Flight Benchmark Report ===");
    println!("Total Steps:                  {}", report.total_steps);
    println!("Elapsed Time (ms):            {:.3} ms", report.elapsed_ms);
    println!(
        "Throughput:                   {:.2} steps/sec",
        report.steps_per_second
    );
    println!(
        "Position Estimation RMSE:     {:.4} m (limit: < 0.15 m)",
        report.position_rmse_m
    );
    println!(
        "Velocity Estimation RMSE:     {:.4} m/s (limit: < 0.08 m/s)",
        report.velocity_rmse_m_s
    );
    println!(
        "Attitude Estimation RMSE:     {:.4} deg (limit: < 1.5 deg)",
        report.attitude_rmse_deg
    );
    println!(
        "Spoofed GPS Packets Rejected: {}",
        report.spoofed_packets_rejected
    );
    println!("Failsafe Triggers:            {}", report.failsafe_triggers);
    println!(
        "Tracking Verified:            {}",
        report.target_tracking_verified
    );
    println!("========================================================");

    assert_eq!(report.total_steps, 10_000);
    assert!(
        report.position_rmse_m < 0.15,
        "Position RMSE {:.4} m exceeds threshold 0.15 m",
        report.position_rmse_m
    );
    assert!(
        report.velocity_rmse_m_s < 0.08,
        "Velocity RMSE {:.4} m/s exceeds threshold 0.08 m/s",
        report.velocity_rmse_m_s
    );
    assert!(
        report.attitude_rmse_deg < 1.5,
        "Attitude RMSE {:.4} deg exceeds threshold 1.5 deg",
        report.attitude_rmse_deg
    );
    let min_throughput = if cfg!(debug_assertions) {
        40_000.0
    } else {
        500_000.0
    };
    assert!(
        report.steps_per_second > min_throughput,
        "Throughput {:.2} steps/sec below required {:.0} steps/sec",
        report.steps_per_second,
        min_throughput
    );
    assert!(
        report.spoofed_packets_rejected > 0,
        "Expected GPS spoofing packets to be rejected"
    );
    assert!(report.target_tracking_verified);
}

#[test]
fn test_hil_single_flight_bridge_mission_lifecycle() {
    let airframe = AirframeConfig::new_quadrotor_x(1.5, 0.22, 0.127);
    let initial_state = FlightDynamicsState::new_hover(0.0, 0.0, 4);
    let dt = 0.002; // 500 Hz

    let mut bridge = HilFlightBridge::new(airframe, initial_state, dt, 0x1234);

    // 1. Takeoff command to 2.0 m:
    bridge.set_target_waypoint(Vector3D::new(0.0, 0.0, 2.0), 0.0);
    for _ in 0..1000 {
        // 2.0 seconds
        bridge.step();
    }
    assert!(
        bridge.dynamics.state.position_world.z > 1.2,
        "Vehicle should climb off ground"
    );

    // 2. GPS spoofing attack injection:
    bridge.fault_config.enable_gps_spoofing = true;
    bridge.fault_config.gps_spoof_offset_m = Vector3D::new(80.0, 40.0, 20.0);
    bridge.fault_config.gps_spoof_start_us = bridge.sim_time_us;
    bridge.fault_config.gps_spoof_duration_us = 2_000_000;

    for _ in 0..500 {
        // 1.0 second under spoofing
        bridge.step();
    }

    assert!(
        bridge.eskf.gps_pos_rejected_count > 0,
        "Spoofed packets must be detected and rejected"
    );
    assert_eq!(bridge.failsafe_mode, FailsafeMode::ReturnToLaunch);

    // 3. Motor failure injection:
    bridge.fault_config.enable_motor_failure = true;
    bridge.fault_config.failed_motor_index = 0;
    bridge.fault_config.motor_failure_start_us = bridge.sim_time_us;
    bridge.fault_config.motor_thrust_factor = 0.0; // Dead motor

    bridge.evaluate_failsafe();
    assert_eq!(
        bridge.failsafe_mode,
        FailsafeMode::EmergencyTouchdown,
        "Severe motor failure must trigger EmergencyTouchdown"
    );
}
