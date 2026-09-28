//! Hardware-in-the-Loop (HIL) Flight Co-Simulation Bridge & MAVLink Protocol Engine
//!
//! Bridges Aerovex rigid-body multi-physics flight simulation with real-time autopilot
//! controllers (PX4 / ArduPilot / custom flight stacks) via MAVLink HIL protocol:
//! 1. HIL_SENSOR packets (high-rate 500 Hz IMU, baro, mag, temperature).
//! 2. HIL_GPS packets (10 Hz geodetic fix, pseudorange, DOP, velocity).
//! 3. HIL_ACTUATOR_CONTROLS reception (normalized motor throttle [0.0, 1.0]).
//! 4. Deterministic microsecond clock synchronization between physics, transducers, and autopilot.
//! 5. Real-time fault injection engine: sensor dropouts, bias jumps, GPS spoofing, and motor failure.
//! 6. Emergency failsafe supervisory logic: Return-to-Launch (RTL) and emergency touchdown / parachute.

use crate::sensors::sensor_fusion::{EskfConfig, EskfNominalState, MultiRateEskf};
use phonon_models::em::{ChannelRng, Vector3D};
use phonon_models::sensors::{
    AirframeConfig, FlightDynamicsEngine, FlightDynamicsState, GpsMeasurement, ImuMeasurement,
    Quaternion, STANDARD_GRAVITY_M_S2,
};

/// MAVLink HIL_SENSOR Packet Structure.
#[derive(Debug, Clone, PartialEq)]
pub struct HilSensorPacket {
    /// Timestamp in microseconds since boot / simulation start.
    pub time_usec: u64,
    /// Body X acceleration in m/s^2.
    pub xacc: f32,
    /// Body Y acceleration in m/s^2.
    pub yacc: f32,
    /// Body Z acceleration in m/s^2.
    pub zacc: f32,
    /// Angular speed around body X axis in rad/s.
    pub xgyro: f32,
    /// Angular speed around body Y axis in rad/s.
    pub ygyro: f32,
    /// Angular speed around body Z axis in rad/s.
    pub zgyro: f32,
    /// Magnetic field along body X axis in Gauss (1 Gauss = 100 uT).
    pub xmag: f32,
    /// Magnetic field along body Y axis in Gauss.
    pub ymag: f32,
    /// Magnetic field along body Z axis in Gauss.
    pub zmag: f32,
    /// Absolute barometric pressure in hectopascals (hPa).
    pub abs_pressure: f32,
    /// Differential pressure (airspeed pitot) in hectopascals (hPa).
    pub diff_pressure: f32,
    /// Barometric pressure altitude in meters.
    pub pressure_alt: f32,
    /// Sensor die temperature in degrees Celsius.
    pub temperature: f32,
    /// Bitmask indicating which fields are updated (0x1FFF = all fields).
    pub fields_updated: u32,
}

/// MAVLink HIL_GPS Packet Structure.
#[derive(Debug, Clone, PartialEq)]
pub struct HilGpsPacket {
    /// Timestamp in microseconds.
    pub time_usec: u64,
    /// Fix type (0-1: none, 2: 2D, 3: 3D, 4: DGPS, 5: RTK Float, 6: RTK Fixed).
    pub fix_type: u8,
    /// Latitude in degrees * 1e7 (WGS-84).
    pub lat: i32,
    /// Longitude in degrees * 1e7 (WGS-84).
    pub lon: i32,
    /// Altitude in millimeters AMSL.
    pub alt: i32,
    /// GPS HDOP horizontal dilution of position in centimeters (cm).
    pub eph: u16,
    /// GPS VDOP vertical dilution of position in centimeters (cm).
    pub epv: u16,
    /// GPS ground speed in centimeters per second (cm/s).
    pub vel: u16,
    /// GPS North velocity in cm/s.
    pub vn: i16,
    /// GPS East velocity in cm/s.
    pub ve: i16,
    /// GPS Down velocity in cm/s.
    pub vd: i16,
    /// Course over ground in centidegrees (0..35999 cdeg).
    pub cog: u16,
    /// Number of visible satellites.
    pub satellites_visible: u8,
}

/// MAVLink HIL_ACTUATOR_CONTROLS Packet Structure.
#[derive(Debug, Clone, PartialEq)]
pub struct HilActuatorControls {
    /// Timestamp in microseconds.
    pub time_usec: u64,
    /// Normalized control outputs [-1.0, 1.0] or [0.0, 1.0] for up to 16 actuators.
    pub controls: [f32; 16],
    /// Autopilot system mode.
    pub mode: u8,
    /// Autopilot status flags.
    pub flags: u64,
}

impl Default for HilActuatorControls {
    fn default() -> Self {
        Self {
            time_usec: 0,
            controls: [0.0; 16],
            mode: 0,
            flags: 0,
        }
    }
}

/// Real-Time Fault Injection Engine Configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct FaultInjectionConfig {
    /// Injects barometric sensor complete packet dropout.
    pub enable_baro_dropout: bool,
    pub baro_dropout_start_us: u64,
    pub baro_dropout_duration_us: u64,

    /// Injects GNSS/GPS signal complete loss / jamming.
    pub enable_gps_dropout: bool,
    pub gps_dropout_start_us: u64,
    pub gps_dropout_duration_us: u64,

    /// Injects abrupt accelerometer bias jump.
    pub enable_accel_bias_jump: bool,
    pub accel_bias_jump: Vector3D,
    pub accel_jump_start_us: u64,

    /// Injects abrupt gyroscope bias jump.
    pub enable_gyro_bias_jump: bool,
    pub gyro_bias_jump: Vector3D,
    pub gyro_jump_start_us: u64,

    /// Injects adversarial GPS spoofing position teleportation.
    pub enable_gps_spoofing: bool,
    pub gps_spoof_offset_m: Vector3D,
    pub gps_spoof_start_us: u64,
    pub gps_spoof_duration_us: u64,

    /// Injects actuator rotor motor failure.
    pub enable_motor_failure: bool,
    pub failed_motor_index: usize,
    pub motor_failure_start_us: u64,
    /// Thrust multiplier for failed motor (e.g. 0.0 = total dead motor, 0.3 = severe damage).
    pub motor_thrust_factor: f64,
}

impl Default for FaultInjectionConfig {
    fn default() -> Self {
        Self {
            enable_baro_dropout: false,
            baro_dropout_start_us: 0,
            baro_dropout_duration_us: 0,
            enable_gps_dropout: false,
            gps_dropout_start_us: 0,
            gps_dropout_duration_us: 0,
            enable_accel_bias_jump: false,
            accel_bias_jump: Vector3D::ZERO,
            accel_jump_start_us: 0,
            enable_gyro_bias_jump: false,
            gyro_bias_jump: Vector3D::ZERO,
            gyro_jump_start_us: 0,
            enable_gps_spoofing: false,
            gps_spoof_offset_m: Vector3D::ZERO,
            gps_spoof_start_us: 0,
            gps_spoof_duration_us: 0,
            enable_motor_failure: false,
            failed_motor_index: 0,
            motor_failure_start_us: 0,
            motor_thrust_factor: 1.0,
        }
    }
}

/// Emergency Failsafe Supervisory State.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailsafeMode {
    /// Nominal autonomous flight.
    Nominal,
    /// Warning state: anomalous sensor readings detected, under observation.
    Warning,
    /// Return to Launch (RTL): autonomous navigation back to origin coordinates.
    ReturnToLaunch,
    /// Emergency Touchdown: controlled gentle descent to ground.
    EmergencyTouchdown,
    /// Emergency Parachute Deployed: ballistic aerodynamic touchdown.
    ParachuteDeployed,
}

/// PID Controller Tuning Gains.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PidGains {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub max_integral: f64,
}

/// Autonomous Multirotor Flight Controller.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadFlightController {
    pub pos_p_gain: f64,
    pub vel_gains: PidGains,
    pub att_gains: PidGains,
    pub rate_gains: PidGains,
    pub vel_integral: Vector3D,
    pub att_integral: Vector3D,
    pub rate_integral: Vector3D,
    pub hover_throttle: f64,
}

impl Default for QuadFlightController {
    fn default() -> Self {
        Self {
            pos_p_gain: 1.2,
            vel_gains: PidGains {
                kp: 1.5,
                ki: 0.05,
                kd: 0.25,
                max_integral: 1.0,
            },
            att_gains: PidGains {
                kp: 4.5,
                ki: 0.02,
                kd: 0.1,
                max_integral: 0.5,
            },
            rate_gains: PidGains {
                kp: 0.06,
                ki: 0.01,
                kd: 0.001,
                max_integral: 0.2,
            },
            vel_integral: Vector3D::ZERO,
            att_integral: Vector3D::ZERO,
            rate_integral: Vector3D::ZERO,
            hover_throttle: 0.34, // Quadrotor 1.0g hover throttle (544 rad/s)
        }
    }
}

impl QuadFlightController {
    /// Computes 4 motor normalized commands [0.0, 1.0] for Quadrotor X airframe.
    #[allow(clippy::too_many_arguments)]
    pub fn compute_controls(
        &mut self,
        target_pos: Vector3D,
        target_yaw: f64,
        est_pos: Vector3D,
        est_vel: Vector3D,
        est_orientation: Quaternion,
        est_gyro: Vector3D,
        dt: f64,
    ) -> [f64; 4] {
        let dt = dt.clamp(1e-4, 0.05);

        // 1. Position -> Desired Velocity:
        let pos_err = target_pos - est_pos;
        let v_cmd_x = (pos_err.x * self.pos_p_gain).clamp(-4.0, 4.0);
        let v_cmd_y = (pos_err.y * self.pos_p_gain).clamp(-4.0, 4.0);
        let v_cmd_z = (pos_err.z * self.pos_p_gain).clamp(-2.5, 2.5);
        let desired_vel = Vector3D::new(v_cmd_x, v_cmd_y, v_cmd_z);

        // 2. Velocity -> Desired Acceleration / Tilt:
        let vel_err = desired_vel - est_vel;
        self.vel_integral = self.vel_integral + vel_err * dt;
        self.vel_integral.x = self
            .vel_integral
            .x
            .clamp(-self.vel_gains.max_integral, self.vel_gains.max_integral);
        self.vel_integral.y = self
            .vel_integral
            .y
            .clamp(-self.vel_gains.max_integral, self.vel_gains.max_integral);
        self.vel_integral.z = self
            .vel_integral
            .z
            .clamp(-self.vel_gains.max_integral, self.vel_gains.max_integral);

        let acc_cmd_x = self.vel_gains.kp * vel_err.x + self.vel_gains.ki * self.vel_integral.x;
        let acc_cmd_y = self.vel_gains.kp * vel_err.y + self.vel_gains.ki * self.vel_integral.y;
        let acc_cmd_z = self.vel_gains.kp * vel_err.z + self.vel_gains.ki * self.vel_integral.z;

        // Desired roll and pitch tilt angles (radians):
        let max_tilt = 30.0 * std::f64::consts::PI / 180.0;
        let pitch_cmd = (acc_cmd_x / STANDARD_GRAVITY_M_S2).clamp(-max_tilt, max_tilt);
        let roll_cmd = (-acc_cmd_y / STANDARD_GRAVITY_M_S2).clamp(-max_tilt, max_tilt);

        // Collective throttle command:
        let thrust_cmd = (self.hover_throttle + acc_cmd_z * 0.035).clamp(0.05, 0.95);

        // 3. Attitude Error -> Desired Angular Rates:
        let (current_roll, current_pitch, current_yaw) = est_orientation.to_euler_rpy();
        let roll_err = roll_cmd - current_roll;
        let pitch_err = pitch_cmd - current_pitch;
        let mut yaw_err = target_yaw - current_yaw;
        while yaw_err > std::f64::consts::PI {
            yaw_err -= std::f64::consts::TAU;
        }
        while yaw_err < -std::f64::consts::PI {
            yaw_err += std::f64::consts::TAU;
        }

        let p_cmd = (self.att_gains.kp * roll_err).clamp(-3.0, 3.0);
        let q_cmd = (self.att_gains.kp * pitch_err).clamp(-3.0, 3.0);
        let r_cmd = (self.att_gains.kp * yaw_err).clamp(-2.0, 2.0);

        // 4. Rate Error -> Motor Mixer Torque Demands:
        let rate_err_p = p_cmd - est_gyro.x;
        let rate_err_q = q_cmd - est_gyro.y;
        let rate_err_r = r_cmd - est_gyro.z;

        let u_roll = (self.rate_gains.kp * rate_err_p).clamp(-0.20, 0.20);
        let u_pitch = (self.rate_gains.kp * rate_err_q).clamp(-0.20, 0.20);
        let u_yaw = (self.rate_gains.kp * rate_err_r).clamp(-0.15, 0.15);

        // 5. Quad X Motor Mixer:
        // m0: front-right (+x, -y, CCW)
        // m1: rear-left (-x, +y, CCW)
        // m2: front-left (+x, +y, CW)
        // m3: rear-right (-x, -y, CW)
        let m0 = (thrust_cmd - u_pitch - u_roll - u_yaw).clamp(0.0, 1.0);
        let m1 = (thrust_cmd + u_pitch + u_roll - u_yaw).clamp(0.0, 1.0);
        let m2 = (thrust_cmd - u_pitch + u_roll + u_yaw).clamp(0.0, 1.0);
        let m3 = (thrust_cmd + u_pitch - u_roll + u_yaw).clamp(0.0, 1.0);

        [m0, m1, m2, m3]
    }
}

/// Comprehensive Hardware-in-the-Loop (HIL) Flight Co-Simulation Bridge Engine.
#[derive(Debug, Clone, PartialEq)]
pub struct HilFlightBridge {
    pub dynamics: FlightDynamicsEngine,
    pub eskf: MultiRateEskf,
    pub controller: QuadFlightController,
    pub fault_config: FaultInjectionConfig,
    pub failsafe_mode: FailsafeMode,
    pub home_position: Vector3D,
    pub sim_time_us: u64,
    pub step_dt_s: f64,
    pub consecutive_spoofed_gps_count: usize,
    pub target_waypoint: Vector3D,
    pub target_yaw_rad: f64,
    pub rng: ChannelRng,
}

impl HilFlightBridge {
    /// Creates a new HIL flight simulation bridge.
    pub fn new(
        airframe: AirframeConfig,
        initial_state: FlightDynamicsState,
        step_dt_s: f64,
        seed: u64,
    ) -> Self {
        let home_position = initial_state.position_world;
        let eskf_init = EskfNominalState {
            position: initial_state.position_world,
            velocity: initial_state.velocity_body,
            orientation: initial_state.orientation,
            accel_bias: Vector3D::ZERO,
            gyro_bias: Vector3D::ZERO,
        };

        Self {
            dynamics: FlightDynamicsEngine::new(airframe, initial_state),
            eskf: MultiRateEskf::new(EskfConfig::default(), eskf_init),
            controller: QuadFlightController::default(),
            fault_config: FaultInjectionConfig::default(),
            failsafe_mode: FailsafeMode::Nominal,
            home_position,
            sim_time_us: 0,
            step_dt_s,
            consecutive_spoofed_gps_count: 0,
            target_waypoint: home_position + Vector3D::new(0.0, 0.0, 2.0), // Hover at 2 m
            target_yaw_rad: 0.0,
            rng: ChannelRng::new(seed),
        }
    }

    /// Sets the desired flight target waypoint.
    pub fn set_target_waypoint(&mut self, waypoint: Vector3D, yaw_rad: f64) {
        self.target_waypoint = waypoint;
        self.target_yaw_rad = yaw_rad;
    }

    /// Generates MAVLink HIL_SENSOR packet from physical sensor outputs and fault injection.
    pub fn generate_hil_sensor(
        &mut self,
        imu: &ImuMeasurement,
        baro_pressure_pa: f64,
    ) -> Option<HilSensorPacket> {
        // Check barometric sensor dropout fault:
        if self.fault_config.enable_baro_dropout
            && self.sim_time_us >= self.fault_config.baro_dropout_start_us
            && self.sim_time_us
                < self.fault_config.baro_dropout_start_us
                    + self.fault_config.baro_dropout_duration_us
        {
            return None;
        }

        // Apply abrupt IMU bias jumps if active:
        let mut accel = imu.accel_m_s2;
        if self.fault_config.enable_accel_bias_jump
            && self.sim_time_us >= self.fault_config.accel_jump_start_us
        {
            accel = accel + self.fault_config.accel_bias_jump;
        }

        let mut gyro = imu.gyro_rad_s;
        if self.fault_config.enable_gyro_bias_jump
            && self.sim_time_us >= self.fault_config.gyro_jump_start_us
        {
            gyro = gyro + self.fault_config.gyro_bias_jump;
        }

        let mag = imu.mag_ut.unwrap_or(Vector3D::ZERO);
        let abs_pressure_hpa = (baro_pressure_pa * 0.01) as f32;
        let pressure_alt = self
            .dynamics
            .barometer
            .pressure_to_altitude(baro_pressure_pa) as f32;

        Some(HilSensorPacket {
            time_usec: self.sim_time_us,
            xacc: accel.x as f32,
            yacc: accel.y as f32,
            zacc: accel.z as f32,
            xgyro: gyro.x as f32,
            ygyro: gyro.y as f32,
            zgyro: gyro.z as f32,
            xmag: (mag.x * 0.01) as f32, // uT to Gauss
            ymag: (mag.y * 0.01) as f32,
            zmag: (mag.z * 0.01) as f32,
            abs_pressure: abs_pressure_hpa,
            diff_pressure: 0.0,
            pressure_alt,
            temperature: imu.temperature_c as f32,
            fields_updated: 0x1FFF,
        })
    }

    /// Generates MAVLink HIL_GPS packet from physical GNSS sensor output and fault injection.
    pub fn generate_hil_gps(&mut self, mut gps: GpsMeasurement) -> Option<HilGpsPacket> {
        // Check GPS dropout fault:
        if self.fault_config.enable_gps_dropout
            && self.sim_time_us >= self.fault_config.gps_dropout_start_us
            && self.sim_time_us
                < self.fault_config.gps_dropout_start_us + self.fault_config.gps_dropout_duration_us
        {
            return None;
        }

        // Apply adversarial GPS spoofing teleportation if active:
        if self.fault_config.enable_gps_spoofing
            && self.sim_time_us >= self.fault_config.gps_spoof_start_us
            && self.sim_time_us
                < self.fault_config.gps_spoof_start_us + self.fault_config.gps_spoof_duration_us
        {
            gps.position_world_m = gps.position_world_m + self.fault_config.gps_spoof_offset_m;
            gps.is_spoofed = true;
        }

        // Geodetic origin (lat 37.7749 deg, lon -122.4194 deg, alt 100 m):
        let lat_deg = 37.7749 + (gps.position_world_m.x / 111_000.0);
        let lon_deg =
            -122.4194 + (gps.position_world_m.y / (111_000.0 * (37.7749f64.to_radians()).cos()));
        let alt_mm = ((100.0 + gps.position_world_m.z) * 1000.0) as i32;
        let ground_speed_cm_s = (gps.velocity_world_m_s.norm() * 100.0) as u16;

        Some(HilGpsPacket {
            time_usec: self.sim_time_us,
            fix_type: gps.fix_type as u8,
            lat: (lat_deg * 1e7) as i32,
            lon: (lon_deg * 1e7) as i32,
            alt: alt_mm,
            eph: (gps.hdop * 100.0) as u16,
            epv: (gps.vdop * 100.0) as u16,
            vel: ground_speed_cm_s,
            vn: (gps.velocity_world_m_s.x * 100.0) as i16,
            ve: (gps.velocity_world_m_s.y * 100.0) as i16,
            vd: (-gps.velocity_world_m_s.z * 100.0) as i16,
            cog: 0,
            satellites_visible: gps.satellites_visible,
        })
    }

    /// Evaluates supervisory failsafe monitoring and updates failsafe mode.
    pub fn evaluate_failsafe(&mut self) {
        // 1. Motor failure failsafe check:
        if self.fault_config.enable_motor_failure
            && self.sim_time_us >= self.fault_config.motor_failure_start_us
        {
            if self.fault_config.motor_thrust_factor < 0.2 {
                self.failsafe_mode = FailsafeMode::EmergencyTouchdown;
                return;
            } else {
                self.failsafe_mode = FailsafeMode::ReturnToLaunch;
            }
        }

        // 2. GPS spoofing anomaly check:
        if self.eskf.gps_pos_rejected_count >= 5 {
            self.failsafe_mode = FailsafeMode::ReturnToLaunch;
            return;
        }

        // 3. Sensor dropout check:
        if self.fault_config.enable_gps_dropout
            && self.sim_time_us >= self.fault_config.gps_dropout_start_us + 1_500_000
            && self.sim_time_us
                < self.fault_config.gps_dropout_start_us + self.fault_config.gps_dropout_duration_us
        {
            self.failsafe_mode = FailsafeMode::ReturnToLaunch;
        }
    }

    /// Advances the complete closed-loop HIL simulation by one physics time step dt.
    /// Executes physical dynamics, transducer synthesis, EKF sensor fusion,
    /// supervisory failsafe checks, and closed-loop motor control.
    pub fn step(&mut self) -> (EskfNominalState, [f64; 4], FailsafeMode) {
        let dt = self.step_dt_s;

        // 1. Compute target waypoint based on failsafe state:
        let effective_target = match self.failsafe_mode {
            FailsafeMode::Nominal | FailsafeMode::Warning => self.target_waypoint,
            FailsafeMode::ReturnToLaunch => {
                // Navigate towards home origin with safe RTL altitude (3.0 m):
                Vector3D::new(self.home_position.x, self.home_position.y, 3.0)
            }
            FailsafeMode::EmergencyTouchdown | FailsafeMode::ParachuteDeployed => {
                // Command ground landing:
                Vector3D::new(
                    self.eskf.nominal.position.x,
                    self.eskf.nominal.position.y,
                    0.0,
                )
            }
        };

        // 2. Closed-loop flight controller computes normalized motor commands:
        let mut motor_cmds_norm = self.controller.compute_controls(
            effective_target,
            self.target_yaw_rad,
            self.eskf.nominal.position,
            self.eskf.nominal.velocity,
            self.eskf.nominal.orientation,
            self.dynamics.state.angular_velocity_body,
            dt,
        );

        // Inject motor failure if configured:
        if self.fault_config.enable_motor_failure
            && self.sim_time_us >= self.fault_config.motor_failure_start_us
        {
            let idx = self.fault_config.failed_motor_index.min(3);
            motor_cmds_norm[idx] *= self.fault_config.motor_thrust_factor;
        }

        // Scale normalized commands [0.0, 1.0] to rotor angular speed rad/s:
        let rotor_cmds_rad_s: Vec<f64> = motor_cmds_norm
            .iter()
            .zip(&self.dynamics.config.rotors)
            .map(|(&cmd, r)| cmd * r.max_rad_s)
            .collect();

        // 3. Step rigid-body flight dynamics and physical transducers:
        let (imu_meas, baro_pressure, _lidar, mut gps_meas) =
            self.dynamics.step(&rotor_cmds_rad_s, dt, &mut self.rng);

        // 4. Update ESKF:
        // High-rate IMU strapdown propagation (500 Hz):
        self.eskf.propagate_imu(&imu_meas, dt);

        // Altitude update (Barometer / LiDAR):
        let is_baro_active = !(self.fault_config.enable_baro_dropout
            && self.sim_time_us >= self.fault_config.baro_dropout_start_us
            && self.sim_time_us
                < self.fault_config.baro_dropout_start_us
                    + self.fault_config.baro_dropout_duration_us);

        if is_baro_active {
            self.eskf
                .update_barometer_pressure(baro_pressure, &self.dynamics.barometer);
        }

        // Magnetometer heading update:
        if let Some(mag) = imu_meas.mag_ut {
            self.eskf.update_magnetometer(mag);
        }

        // GNSS/GPS update (10 Hz rate, with spoofing injection):
        let is_gps_active = !(self.fault_config.enable_gps_dropout
            && self.sim_time_us >= self.fault_config.gps_dropout_start_us
            && self.sim_time_us
                < self.fault_config.gps_dropout_start_us
                    + self.fault_config.gps_dropout_duration_us);

        if is_gps_active {
            if self.fault_config.enable_gps_spoofing
                && self.sim_time_us >= self.fault_config.gps_spoof_start_us
                && self.sim_time_us
                    < self.fault_config.gps_spoof_start_us + self.fault_config.gps_spoof_duration_us
            {
                gps_meas.position_world_m =
                    gps_meas.position_world_m + self.fault_config.gps_spoof_offset_m;
                gps_meas.is_spoofed = true;
            }

            self.eskf.update_gps(&gps_meas);
        }

        // 5. Evaluate supervisory failsafe:
        self.evaluate_failsafe();

        // 6. Advance deterministic clock:
        self.sim_time_us += (dt * 1_000_000.0) as u64;

        (
            self.eskf.nominal.clone(),
            motor_cmds_norm,
            self.failsafe_mode,
        )
    }
}
