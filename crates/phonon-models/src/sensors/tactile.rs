//! Physics-Coupled Tactile & Contact Force Transducer Models
//!
//! Formulates first-principles electromechanical models for tactile sensors:
//! 1. Piezoresistive conductive elastomer sensors with Wheatstone bridge readout.
//! 2. Capacitive elastomeric dielectric sensors responding to contact normal force.
//! 3. Multi-taxel spatial tactile matrix arrays resolving pressure distributions
//!    and Center of Pressure (CoP) directly from rigid-body collision contact manifolds.

use crate::em::Vector3D;
use phonon_core::constants::EPSILON_0;

/// Physical collision contact point input from Aerovex rigid-body dynamics engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionContactInput {
    /// World Cartesian position of contact point in meters.
    pub position_world: Vector3D,
    /// Contact surface normal unit vector (pointing from obstacle to sensor body).
    pub normal: Vector3D,
    /// Magnitude of contact normal reaction force in Newtons (F_N >= 0).
    pub normal_force_n: f64,
    /// Tangential friction force vector in Newtons.
    pub tangential_force_n: Vector3D,
    /// Interpenetration depth in meters.
    pub penetration_depth_m: f64,
}

impl CollisionContactInput {
    pub fn new(
        position_world: Vector3D,
        normal: Vector3D,
        normal_force_n: f64,
        tangential_force_n: Vector3D,
        penetration_depth_m: f64,
    ) -> Self {
        Self {
            position_world,
            normal: normal.normalize(),
            normal_force_n: normal_force_n.max(0.0),
            tangential_force_n,
            penetration_depth_m: penetration_depth_m.max(0.0),
        }
    }
}

/// Piezoresistive force sensor configuration (e.g. Velostat / carbon-doped conductive polymer).
#[derive(Debug, Clone, PartialEq)]
pub struct PiezoresistiveSensorConfig {
    /// Unloaded rest resistance in Ohms (R_0, e.g. 10,000 Ohms).
    pub rest_resistance_ohms: f64,
    /// Characteristic saturation force in Newtons (F_char, e.g. 5.0 N).
    pub characteristic_force_n: f64,
    /// Non-linear piezoresistive power-law exponent (gamma, e.g. 0.85).
    pub piezoresistive_exponent: f64,
    /// Wheatstone bridge DC excitation voltage in Volts (e.g. 3.3 V or 5.0 V).
    pub bridge_excitation_voltage_v: f64,
}

impl Default for PiezoresistiveSensorConfig {
    fn default() -> Self {
        Self {
            rest_resistance_ohms: 10_000.0,
            characteristic_force_n: 5.0,
            piezoresistive_exponent: 0.85,
            bridge_excitation_voltage_v: 3.3,
        }
    }
}

impl PiezoresistiveSensorConfig {
    /// Evaluates sensor resistance in Ohms as a function of contact normal force F_N:
    ///
    /// R(F_N) = R_0 * (1 + F_N / F_char)^(-gamma)
    pub fn resistance_ohms(&self, normal_force_n: f64) -> f64 {
        let f = normal_force_n.max(0.0);
        let ratio = 1.0 + f / self.characteristic_force_n.max(1e-4);
        self.rest_resistance_ohms * ratio.powf(-self.piezoresistive_exponent)
    }

    /// Evaluates differential output voltage Delta_V in Volts from a half-bridge Wheatstone circuit:
    ///
    /// Delta_V = V_ex * [R(F_N) / (R_0 + R(F_N)) - 0.5]
    pub fn wheatstone_differential_voltage_v(&self, normal_force_n: f64) -> f64 {
        let r_f = self.resistance_ohms(normal_force_n);
        let r_0 = self.rest_resistance_ohms;
        let v_ex = self.bridge_excitation_voltage_v;

        v_ex * (r_f / (r_0 + r_f) - 0.5)
    }
}

/// Capacitive elastomeric tactile force sensor configuration.
#[derive(Debug, Clone, PartialEq)]
pub struct CapacitiveSensorConfig {
    /// Initial unloaded dielectric spacer thickness in meters (e.g. 0.5 mm = 5e-4 m).
    pub dielectric_thickness_m: f64,
    /// Parallel plate sensing area in m^2 (e.g. 1.0 cm^2 = 1e-4 m^2).
    pub plate_area_m2: f64,
    /// Relative dielectric permittivity of elastomeric foam/silicone (e.g. 3.2).
    pub relative_permittivity: f64,
    /// Mechanical spring stiffness of elastomeric dielectric layer in N/m (e.g. 20,000 N/m).
    pub elastomer_stiffness_n_per_m: f64,
}

impl Default for CapacitiveSensorConfig {
    fn default() -> Self {
        Self {
            dielectric_thickness_m: 5.0e-4, // 0.5 mm
            plate_area_m2: 1.0e-4,          // 1 cm^2
            relative_permittivity: 3.2,
            elastomer_stiffness_n_per_m: 20_000.0, // 20 kN/m
        }
    }
}

impl CapacitiveSensorConfig {
    /// Evaluates mechanical thickness deflection Delta_d in meters for a given normal force F_N:
    ///
    /// Delta_d = min(F_N / k_elastomer, 0.85 * d_0)
    pub fn deflection_m(&self, normal_force_n: f64) -> f64 {
        let f = normal_force_n.max(0.0);
        let raw_deflection = f / self.elastomer_stiffness_n_per_m.max(1.0);
        let max_deflection = self.dielectric_thickness_m * 0.85; // Prevent physical contact singularity
        raw_deflection.min(max_deflection)
    }

    /// Evaluates dynamic capacitance in Farads under contact normal compression:
    ///
    /// C(F_N) = (epsilon_r * epsilon_0 * A) / (d_0 - Delta_d(F_N))
    pub fn capacitance_f(&self, normal_force_n: f64) -> f64 {
        let delta_d = self.deflection_m(normal_force_n);
        let effective_d = (self.dielectric_thickness_m - delta_d).max(1e-7);
        (self.relative_permittivity * EPSILON_0 * self.plate_area_m2) / effective_d
    }

    /// Unloaded rest capacitance C_0 in Farads.
    pub fn rest_capacitance_f(&self) -> f64 {
        self.capacitance_f(0.0)
    }
}

/// 2D Multi-Taxel Spatial Tactile Matrix Array.
#[derive(Debug, Clone, PartialEq)]
pub struct TactileMatrixArray {
    /// Number of rows of sensing taxels (e.g. 4 or 8).
    pub rows: usize,
    /// Number of columns of sensing taxels (e.g. 4 or 8).
    pub cols: usize,
    /// Physical taxel pitch in meters (e.g. 5 mm = 5e-3 m).
    pub pitch_m: f64,
    /// Underlying piezoresistive sensor configuration per taxel.
    pub taxel_config: PiezoresistiveSensorConfig,
    /// Instantaneous normal force distribution across taxels in Newtons (rows * cols).
    pub taxel_forces_n: Vec<f64>,
}

impl TactileMatrixArray {
    /// Creates a new M x N taxel tactile array with uniform pitch.
    pub fn new(
        rows: usize,
        cols: usize,
        pitch_m: f64,
        taxel_config: PiezoresistiveSensorConfig,
    ) -> Self {
        let total_taxels = rows * cols;
        Self {
            rows,
            cols,
            pitch_m,
            taxel_config,
            taxel_forces_n: vec![0.0; total_taxels],
        }
    }

    /// Updates taxel force distribution from a rigid body contact manifold.
    ///
    /// Projects contact point onto the local tactile plane and distributes normal force
    /// to nearest neighbor taxels via bilinear spatial interpolation.
    pub fn update_from_contact(
        &mut self,
        contact: &CollisionContactInput,
        sensor_center_world: Vector3D,
        sensor_normal_world: Vector3D,
        sensor_up_world: Vector3D,
    ) {
        // Zero all taxel forces:
        self.taxel_forces_n.fill(0.0);

        if contact.normal_force_n <= 1e-4 {
            return;
        }

        // Project contact position into local sensor plane:
        let norm = sensor_normal_world.normalize();
        let up = sensor_up_world.normalize();
        let right = norm.cross(&up).normalize();

        let diff = contact.position_world - sensor_center_world;
        let x_local = diff.dot(&right);
        let y_local = diff.dot(&up);

        let half_w = ((self.cols as f64) * self.pitch_m) / 2.0;
        let half_h = ((self.rows as f64) * self.pitch_m) / 2.0;

        if x_local < -half_w || x_local > half_w || y_local < -half_h || y_local > half_h {
            // Contact outside sensor active region:
            return;
        }

        // Taxel center coordinates [0.0, cols - 1]:
        let u = ((x_local + half_w) / self.pitch_m - 0.5).clamp(0.0, (self.cols - 1) as f64);
        let v = ((y_local + half_h) / self.pitch_m - 0.5).clamp(0.0, (self.rows - 1) as f64);

        let c0 = (u.floor() as usize).min(self.cols - 1);
        let r0 = (v.floor() as usize).min(self.rows - 1);
        let c1 = (c0 + 1).min(self.cols - 1);
        let r1 = (r0 + 1).min(self.rows - 1);

        let du = u - c0 as f64;
        let dv = v - r0 as f64;

        let w00 = (1.0 - du) * (1.0 - dv);
        let w10 = du * (1.0 - dv);
        let w01 = (1.0 - du) * dv;
        let w11 = du * dv;

        let f = contact.normal_force_n;
        self.taxel_forces_n[r0 * self.cols + c0] += f * w00;
        self.taxel_forces_n[r0 * self.cols + c1] += f * w10;
        self.taxel_forces_n[r1 * self.cols + c0] += f * w01;
        self.taxel_forces_n[r1 * self.cols + c1] += f * w11;
    }

    /// Evaluates total integrated normal force across all taxels in Newtons.
    pub fn total_normal_force_n(&self) -> f64 {
        self.taxel_forces_n.iter().sum()
    }

    /// Evaluates the 2D Center of Pressure (CoP) coordinates (x, y) in meters on the sensor plane.
    pub fn center_of_pressure_m(&self) -> (f64, f64) {
        let f_tot = self.total_normal_force_n();
        if f_tot <= 1e-6 {
            return (0.0, 0.0);
        }

        let half_w = ((self.cols as f64) * self.pitch_m) / 2.0;
        let half_h = ((self.rows as f64) * self.pitch_m) / 2.0;

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;

        for r in 0..self.rows {
            for c in 0..self.cols {
                let f = self.taxel_forces_n[r * self.cols + c];
                let x = (c as f64 + 0.5) * self.pitch_m - half_w;
                let y = (r as f64 + 0.5) * self.pitch_m - half_h;
                sum_x += x * f;
                sum_y += y * f;
            }
        }

        (sum_x / f_tot, sum_y / f_tot)
    }

    /// Evaluates all taxel Wheatstone bridge differential voltages in Volts.
    pub fn taxel_voltages_v(&self) -> Vec<f64> {
        self.taxel_forces_n
            .iter()
            .map(|&f| self.taxel_config.wheatstone_differential_voltage_v(f))
            .collect()
    }
}
