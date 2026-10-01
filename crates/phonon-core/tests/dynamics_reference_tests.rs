#![deny(unsafe_code)]

//! Analytical unit tests and high-speed benchmark validation for the Phonon
//! Pure Safe Rust Reference RK4 6-DOF Dynamics Engine.

use phonon_core::{
    ActuatorInputs, DynamicsError, DynamicsTelemetry, PhysicsDynamicsBackend,
    ReferenceDynamicsBackend, ReferenceDynamicsParams,
};

#[test]
fn test_default_state_initialization() {
    let backend = ReferenceDynamicsBackend::default();
    let telem = backend.current_telemetry();
    assert_eq!(telem.position_m, [0.0, 0.0, 0.0]);
    assert_eq!(telem.velocity_m_per_s, [0.0, 0.0, 0.0]);
    assert_eq!(telem.acceleration_m_per_s2, [0.0, 0.0, 0.0]);
    assert_eq!(telem.orientation_quat, [1.0, 0.0, 0.0, 0.0]);
    assert_eq!(telem.angular_velocity_rad_per_s, [0.0, 0.0, 0.0]);
    assert_eq!(telem.angular_acceleration_rad_per_s2, [0.0, 0.0, 0.0]);
    assert_eq!(telem.sim_time_s, 0.0);
    assert_eq!(telem.step_count, 0);
    assert_eq!(telem.altitude_m(), 0.0);
    assert_eq!(telem.speed_m_per_s(), 0.0);
    assert!(backend.is_healthy());
}

#[test]
fn test_gravity_freefall() {
    let params = ReferenceDynamicsParams::default().with_drag_coefficient(0.0);
    let mut backend = ReferenceDynamicsBackend::new(params);
    let inputs = ActuatorInputs::default();
    let dt = 0.01;
    let telem = backend.step(dt, &inputs).expect("step should succeed");

    // In NED frame, z is pointing down, so downward gravitational acceleration is +g = 9.80665 m/s^2
    assert!((telem.acceleration_m_per_s2[0]).abs() < 1e-12);
    assert!((telem.acceleration_m_per_s2[1]).abs() < 1e-12);
    assert!((telem.acceleration_m_per_s2[2] - 9.80665).abs() < 1e-12);

    // Velocity after dt: v_z = g * dt
    assert!((telem.velocity_m_per_s[2] - 9.80665 * dt).abs() < 1e-12);
    // Position after dt: z = 0.5 * g * dt^2
    assert!((telem.position_m[2] - 0.5 * 9.80665 * dt * dt).abs() < 1e-12);
    // Altitude = -z
    assert!((telem.altitude_m() - (-0.5 * 9.80665 * dt * dt)).abs() < 1e-12);
}

#[test]
fn test_hover_equilibrium() {
    let mut backend = ReferenceDynamicsBackend::default();
    let m = backend.params().mass_kg;
    let g = backend.params().gravity_m_per_s2;
    let hover_thrust_per_rotor = m * g / 4.0;
    let inputs = ActuatorInputs::hover(hover_thrust_per_rotor);

    // Step multiple times (100 steps of 0.01s = 1.0s of flight)
    for _ in 0..100 {
        let telem = backend.step(0.01, &inputs).expect("step should succeed");
        assert!(telem.acceleration_m_per_s2[0].abs() < 1e-6);
        assert!(telem.acceleration_m_per_s2[1].abs() < 1e-6);
        assert!(telem.acceleration_m_per_s2[2].abs() < 1e-6);
        assert!(telem.velocity_m_per_s[2].abs() < 1e-6);
        assert!(telem.position_m[2].abs() < 1e-6);
        assert!(telem.altitude_m().abs() < 1e-6);
    }
}

#[test]
fn test_symmetric_rotor_thrust_balance() {
    let params = ReferenceDynamicsParams::default().with_drag_coefficient(0.0);
    let mut backend = ReferenceDynamicsBackend::new(params);
    let thrust = 5.0; // 5 N per rotor > hover
    let inputs = ActuatorInputs::new([thrust; 4], [0.0; 4], 0.0);

    let telem = backend.step(0.01, &inputs).expect("step should succeed");

    // Angular acceleration in roll, pitch, yaw must be strictly 0
    assert!(telem.angular_acceleration_rad_per_s2[0].abs() < 1e-12);
    assert!(telem.angular_acceleration_rad_per_s2[1].abs() < 1e-12);
    assert!(telem.angular_acceleration_rad_per_s2[2].abs() < 1e-12);

    // Angular velocity must remain zero
    assert!(telem.angular_velocity_rad_per_s[0].abs() < 1e-12);
    assert!(telem.angular_velocity_rad_per_s[1].abs() < 1e-12);
    assert!(telem.angular_velocity_rad_per_s[2].abs() < 1e-12);

    // Linear horizontal acceleration must be strictly zero (pure vertical force)
    assert!(telem.acceleration_m_per_s2[0].abs() < 1e-12);
    assert!(telem.acceleration_m_per_s2[1].abs() < 1e-12);

    // Upward vertical acceleration in NED: a_z = g - (4 * 5.0) / 1.5 < 0
    let expected_az = 9.80665 - (20.0 / 1.5);
    assert!((telem.acceleration_m_per_s2[2] - expected_az).abs() < 1e-12);
}

#[test]
fn test_roll_pitch_moment_generation() {
    let mut backend = ReferenceDynamicsBackend::default();
    let arm = backend.params().arm_length_m;
    let ixx = backend.params().inertia_diag_kg_m2[0];
    let iyy = backend.params().inertia_diag_kg_m2[1];

    // Differential roll: T3 = 4.0, T1 = 2.0, T0 = T2 = 3.0
    let inputs_roll = ActuatorInputs::new([3.0, 2.0, 3.0, 4.0], [0.0; 4], 0.0);
    let telem_roll = backend.step(0.001, &inputs_roll).expect("step should succeed");
    let expected_tau_x = arm * (4.0 - 2.0);
    let expected_p_dot = expected_tau_x / ixx;
    assert!((telem_roll.angular_acceleration_rad_per_s2[0] - expected_p_dot).abs() < 1e-6);
    assert!(telem_roll.angular_acceleration_rad_per_s2[1].abs() < 1e-6);

    // Reset and test pitch: T2 = 4.5, T0 = 1.5, T1 = T3 = 3.0
    backend.reset(None).unwrap();
    let inputs_pitch = ActuatorInputs::new([1.5, 3.0, 4.5, 3.0], [0.0; 4], 0.0);
    let telem_pitch = backend.step(0.001, &inputs_pitch).expect("step should succeed");
    let expected_tau_y = arm * (4.5 - 1.5);
    let expected_q_dot = expected_tau_y / iyy;
    assert!(telem_pitch.angular_acceleration_rad_per_s2[0].abs() < 1e-6);
    assert!((telem_pitch.angular_acceleration_rad_per_s2[1] - expected_q_dot).abs() < 1e-6);
}

#[test]
fn test_yaw_torque_generation() {
    let mut backend = ReferenceDynamicsBackend::default();
    let k_tau = backend.params().torque_coefficient;
    let izz = backend.params().inertia_diag_kg_m2[2];

    // Counter-rotating pair (0, 2) vs (1, 3)
    let inputs_yaw = ActuatorInputs::new([4.0, 2.0, 4.0, 2.0], [0.0; 4], 0.0);
    let telem = backend.step(0.001, &inputs_yaw).expect("step should succeed");
    let expected_tau_z = k_tau * (4.0 - 2.0 + 4.0 - 2.0);
    let expected_r_dot = expected_tau_z / izz;

    assert!(telem.angular_acceleration_rad_per_s2[0].abs() < 1e-6);
    assert!(telem.angular_acceleration_rad_per_s2[1].abs() < 1e-6);
    assert!((telem.angular_acceleration_rad_per_s2[2] - expected_r_dot).abs() < 1e-6);
}

#[test]
fn test_quaternion_norm_preservation() {
    let mut backend = ReferenceDynamicsBackend::default();
    let inputs = ActuatorInputs::new([3.8, 3.2, 4.1, 3.5], [0.0; 4], 0.0);
    let dt = 0.001;

    for _ in 0..10_000 {
        let telem = backend.step(dt, &inputs).expect("step should succeed");
        let q = telem.orientation_quat;
        let norm = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        let diff = (norm - 1.0).abs();
        assert!(
            diff < 1e-12,
            "Quaternion norm deviated by {} (> 1e-12) at step {}",
            diff,
            telem.step_count
        );
    }
}

#[test]
fn test_aerodynamic_drag_terminal_velocity() {
    let mut backend = ReferenceDynamicsBackend::default();
    let m = backend.params().mass_kg;
    let g = backend.params().gravity_m_per_s2;
    let cd = backend.params().drag_coefficient;
    let v_term_expected = (m * g / cd).sqrt();

    let inputs = ActuatorInputs::default(); // 0 thrust
    let dt = 0.001;
    // 20 seconds of sustained freefall
    for _ in 0..20_000 {
        backend.step(dt, &inputs).expect("step should succeed");
    }

    let vz = backend.current_telemetry().velocity_m_per_s[2];
    let diff = (vz - v_term_expected).abs();
    assert!(
        diff < 1e-4,
        "Terminal velocity divergence: got {}, expected {}, diff {}",
        vz,
        v_term_expected,
        diff
    );
}

#[test]
fn test_rk4_convergence_order() {
    let run_simulation = |step_h: f64| -> f64 {
        let params = ReferenceDynamicsParams::default();
        let mut backend = ReferenceDynamicsBackend::new(params);
        let inputs = ActuatorInputs::default();
        let total_time = 1.0;
        let num_steps = (total_time / step_h).round() as usize;
        let actual_h = total_time / num_steps as f64;
        for _ in 0..num_steps {
            backend.step(actual_h, &inputs).expect("step should succeed");
        }
        backend.current_telemetry().velocity_m_per_s[2]
    };

    let ref_val = run_simulation(0.0001);
    let h1 = 0.1;
    let h2 = 0.05;
    let h3 = 0.025;

    let val1 = run_simulation(h1);
    let val2 = run_simulation(h2);
    let val3 = run_simulation(h3);

    let err1 = (val1 - ref_val).abs();
    let err2 = (val2 - ref_val).abs();
    let err3 = (val3 - ref_val).abs();

    let ratio_1_2 = err1 / err2;
    let ratio_2_3 = err2 / err3;
    let order_1_2 = ratio_1_2.log2();
    let order_2_3 = ratio_2_3.log2();

    // 4th-order Runge-Kutta convergence exhibits error ratio ~ 16 and order ~ 4.0
    assert!(
        (order_1_2 - 4.0).abs() < 0.25,
        "Order 1/2 was {}, expected ~4.0",
        order_1_2
    );
    assert!(
        (order_2_3 - 4.0).abs() < 0.25,
        "Order 2/3 was {}, expected ~4.0",
        order_2_3
    );
}

#[test]
fn test_reset_functionality() {
    let mut backend = ReferenceDynamicsBackend::default();
    let inputs = ActuatorInputs::hover(4.0);
    backend.step(0.05, &inputs).unwrap();
    assert!(backend.current_telemetry().step_count > 0);

    // Reset with None restores default zero/identity state
    backend.reset(None).unwrap();
    let telem_reset = backend.current_telemetry();
    assert_eq!(telem_reset.step_count, 0);
    assert_eq!(telem_reset.sim_time_s, 0.0);
    assert_eq!(telem_reset.position_m, [0.0, 0.0, 0.0]);
    assert_eq!(telem_reset.velocity_m_per_s, [0.0, 0.0, 0.0]);
    assert_eq!(telem_reset.orientation_quat, [1.0, 0.0, 0.0, 0.0]);

    // Reset with custom state restores custom state
    let custom_state = DynamicsTelemetry::new(
        [10.0, 20.0, -50.0],
        [1.0, 2.0, -3.0],
        [0.1, 0.2, -0.3],
        [1.0, 0.0, 0.0, 0.0],
        [0.05, -0.05, 0.01],
        [0.01, -0.01, 0.001],
        12.5,
        250,
    );
    backend.reset(Some(&custom_state)).unwrap();
    let telem_custom = backend.current_telemetry();
    assert_eq!(telem_custom.position_m, [10.0, 20.0, -50.0]);
    assert_eq!(telem_custom.velocity_m_per_s, [1.0, 2.0, -3.0]);
    assert_eq!(telem_custom.altitude_m(), 50.0);
    assert_eq!(telem_custom.sim_time_s, 12.5);
    assert_eq!(telem_custom.step_count, 250);
}

#[test]
fn test_high_speed_benchmark() {
    let mut backend = ReferenceDynamicsBackend::default();
    let inputs = ActuatorInputs::new([3.6, 3.7, 3.6, 3.7], [0.0; 4], 0.0);
    let dt = 0.001;

    let start = std::time::Instant::now();
    for _ in 0..100_000 {
        backend.step(dt, &inputs).expect("step should succeed");
    }
    let elapsed = start.elapsed();
    let ticks_per_sec = 100_000.0 / elapsed.as_secs_f64();
    println!("High-speed benchmark: 100,000 RK4 steps in {:?}, throughput: {:.2} ticks/sec", elapsed, ticks_per_sec);

    assert!(
        ticks_per_sec > 1_000_000.0,
        "Throughput was {:.2} ticks/sec, which is below 1,000,000 ticks/sec",
        ticks_per_sec
    );
}

#[test]
fn test_backend_info_and_health() {
    let backend = ReferenceDynamicsBackend::default();
    let info = backend.info();
    assert_eq!(info.name, "Phonon Pure Safe Rust Reference RK4 Dynamics Engine");
    assert_eq!(info.version, "0.1.0");
    assert!(!info.is_hardware_accelerated);
    assert!(info.max_tick_rate_hz >= 10_000_000.0);
    assert!(backend.is_healthy());
}

#[test]
fn test_invalid_inputs_and_error_handling() {
    let mut backend = ReferenceDynamicsBackend::default();

    // Invalid negative or zero dt
    let inputs = ActuatorInputs::default();
    let err_dt = backend.step(-0.01, &inputs);
    assert!(matches!(err_dt, Err(DynamicsError::StepFailed(_))));

    let err_zero = backend.step(0.0, &inputs);
    assert!(matches!(err_zero, Err(DynamicsError::StepFailed(_))));

    // Non-finite actuator input
    let nan_inputs = ActuatorInputs {
        rotor_thrust_n: [f64::NAN, 1.0, 1.0, 1.0],
        control_surfaces_rad: [0.0; 4],
        timestamp_s: 0.0,
    };
    let err_nan = backend.step(0.01, &nan_inputs);
    assert!(matches!(err_nan, Err(DynamicsError::InvalidActuatorInput(_))));

    // Non-finite control surface
    let inf_inputs = ActuatorInputs {
        rotor_thrust_n: [1.0; 4],
        control_surfaces_rad: [f64::INFINITY, 0.0, 0.0, 0.0],
        timestamp_s: 0.0,
    };
    let err_inf = backend.step(0.01, &inf_inputs);
    assert!(matches!(err_inf, Err(DynamicsError::InvalidActuatorInput(_))));
}
