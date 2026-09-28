#![allow(clippy::needless_range_loop)]
//! Spacecraft Attitude Dynamics, Reaction Wheel Clusters & Magnetic Torquers.
//!
//! Formulates:
//! - 3D coupled rigid-body Euler rotational equations of motion.
//! - 4D unit quaternion kinematics ($\dot{\mathbf{q}} = \frac{1}{2} \boldsymbol{\Omega}(\boldsymbol{\omega}) \mathbf{q}$).
//! - 4-wheel redundant reaction wheel pyramid cluster with motor back-EMF and torque limits.
//! - High-frequency wheel mass imbalance micro-vibrations (static $U_s$ and dynamic $U_d$).
//! - Triaxial magnetic torquer rods with geomagnetic dipole cross-product $\boldsymbol{\tau}_{mag} = \mathbf{m} \times \mathbf{B}$.
//! - Reaction wheel momentum desaturation management.

use crate::em::Vector3D;
use crate::sensors::imu::Quaternion;

/// 3x3 Spacecraft Moment of Inertia Tensor $\mathbf{I}_{sc}$ in $\text{kg}\cdot\text{m}^2$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InertiaTensor {
    pub ixx: f64,
    pub iyy: f64,
    pub izz: f64,
    pub ixy: f64,
    pub ixz: f64,
    pub iyz: f64,
}

impl InertiaTensor {
    /// Creates a principal diagonal inertia tensor.
    pub fn diagonal(ixx: f64, iyy: f64, izz: f64) -> Self {
        Self {
            ixx,
            iyy,
            izz,
            ixy: 0.0,
            ixz: 0.0,
            iyz: 0.0,
        }
    }

    /// Evaluates matrix-vector multiplication $\mathbf{I} \boldsymbol{\omega}$:
    pub fn multiply_vector(&self, w: Vector3D) -> Vector3D {
        Vector3D::new(
            self.ixx * w.x + self.ixy * w.y + self.ixz * w.z,
            self.ixy * w.x + self.iyy * w.y + self.iyz * w.z,
            self.ixz * w.x + self.iyz * w.y + self.izz * w.z,
        )
    }

    /// Evaluates inverse inertia applied to torque: $\mathbf{I}^{-1} \boldsymbol{\tau}$.
    pub fn inverse_multiply_vector(&self, tau: Vector3D) -> Vector3D {
        // For diagonal-dominant spacecraft:
        let det = self.ixx * (self.iyy * self.izz - self.iyz * self.iyz)
            - self.ixy * (self.ixy * self.izz - self.iyz * self.ixz)
            + self.ixz * (self.ixy * self.iyz - self.iyy * self.ixz);

        if det.abs() < 1e-12 {
            return Vector3D::new(
                tau.x / self.ixx.max(1e-6),
                tau.y / self.iyy.max(1e-6),
                tau.z / self.izz.max(1e-6),
            );
        }

        let inv_xx = (self.iyy * self.izz - self.iyz * self.iyz) / det;
        let inv_xy = (self.ixz * self.iyz - self.ixy * self.izz) / det;
        let inv_xz = (self.ixy * self.iyz - self.ixz * self.iyy) / det;
        let inv_yy = (self.ixx * self.izz - self.ixz * self.ixz) / det;
        let inv_yz = (self.ixy * self.ixz - self.ixx * self.iyz) / det;
        let inv_zz = (self.ixx * self.iyy - self.ixy * self.ixy) / det;

        Vector3D::new(
            inv_xx * tau.x + inv_xy * tau.y + inv_xz * tau.z,
            inv_xy * tau.x + inv_yy * tau.y + inv_yz * tau.z,
            inv_xz * tau.x + inv_yz * tau.y + inv_zz * tau.z,
        )
    }
}

/// Single Reaction Wheel actuator model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReactionWheel {
    /// Wheel spin-axis unit vector in spacecraft body frame $\hat{\mathbf{w}}$.
    pub spin_axis: Vector3D,
    /// Rotor moment of inertia $I_w$ in $\text{kg}\cdot\text{m}^2$.
    pub rotor_inertia_kg_m2: f64,
    /// Current spin velocity $\Omega_w$ in radians per second ($rad/s$).
    pub wheel_speed_rad_s: f64,
    /// Maximum wheel speed saturation $\Omega_{max}$ in $rad/s$.
    pub max_wheel_speed_rad_s: f64,
    /// Maximum motor torque output $\tau_{max}$ in Newton-meters ($N\cdot m$).
    pub max_torque_nm: f64,
    /// Motor back-EMF constant $K_e$ in $V / (rad/s)$.
    pub back_emf_constant: f64,
    /// Viscous friction damping coefficient $b_w$ in $N\cdot m / (rad/s)$.
    pub viscous_friction: f64,
    /// Static mass imbalance $U_s$ in $kg\cdot m$.
    pub static_imbalance_kg_m: f64,
    /// Dynamic mass imbalance $U_d$ in $kg\cdot m^2$.
    pub dynamic_imbalance_kg_m2: f64,
}

impl ReactionWheel {
    /// Creates a standard micro-satellite reaction wheel.
    pub fn new(spin_axis: Vector3D, rotor_inertia: f64, max_speed: f64, max_torque: f64) -> Self {
        Self {
            spin_axis: spin_axis.normalize(),
            rotor_inertia_kg_m2: rotor_inertia,
            wheel_speed_rad_s: 0.0,
            max_wheel_speed_rad_s: max_speed,
            max_torque_nm: max_torque,
            back_emf_constant: 0.015,
            viscous_friction: 1.0e-6,
            static_imbalance_kg_m: 1.0e-7,   // 0.1 g*mm
            dynamic_imbalance_kg_m2: 5.0e-8, // 0.05 g*mm^2
        }
    }

    /// Angular momentum of the wheel $h_w = I_w \Omega_w \hat{\mathbf{w}}$:
    pub fn momentum_vector(&self) -> Vector3D {
        let h = self.rotor_inertia_kg_m2 * self.wheel_speed_rad_s;
        Vector3D::new(
            h * self.spin_axis.x,
            h * self.spin_axis.y,
            h * self.spin_axis.z,
        )
    }

    /// Evaluates micro-vibration forces and torques generated by mass imbalances:
    pub fn evaluate_micro_vibrations(&self, phase_rad: f64) -> (Vector3D, Vector3D) {
        let omega2 = self.wheel_speed_rad_s * self.wheel_speed_rad_s;
        let f_vib_mag = self.static_imbalance_kg_m * omega2;
        let tau_vib_mag = self.dynamic_imbalance_kg_m2 * omega2;

        let cos_p = phase_rad.cos();
        let sin_p = phase_rad.sin();

        // Radial vibration forces perpendicular to spin axis:
        let f_vib = Vector3D::new(f_vib_mag * cos_p, f_vib_mag * sin_p, 0.0);
        let tau_vib = Vector3D::new(tau_vib_mag * cos_p, tau_vib_mag * sin_p, 0.0);

        (f_vib, tau_vib)
    }

    /// Steps wheel velocity forward under commanded motor torque $\tau_{cmd}$:
    pub fn step(&mut self, commanded_torque_nm: f64, dt_s: f64) -> f64 {
        let clamped_torque = commanded_torque_nm.clamp(-self.max_torque_nm, self.max_torque_nm);

        // Net torque on wheel rotor: tau_net = tau_motor - b * Omega
        let friction_torque = self.viscous_friction * self.wheel_speed_rad_s;
        let net_torque = clamped_torque - friction_torque;

        let alpha = net_torque / self.rotor_inertia_kg_m2;
        self.wheel_speed_rad_s += alpha * dt_s;
        self.wheel_speed_rad_s = self
            .wheel_speed_rad_s
            .clamp(-self.max_wheel_speed_rad_s, self.max_wheel_speed_rad_s);

        // Torque exerted on spacecraft body is opposite to rotor acceleration:
        -clamped_torque
    }
}

/// 4-Wheel Redundant Pyramid Reaction Wheel Cluster.
#[derive(Debug, Clone, PartialEq)]
pub struct ReactionWheelCluster {
    pub wheels: [ReactionWheel; 4],
}

impl ReactionWheelCluster {
    /// Creates a 4-wheel pyramid cluster with elevation angle $\beta = 35.26^\circ$:
    pub fn pyramid_cluster(rotor_inertia: f64, max_speed: f64, max_torque: f64) -> Self {
        let beta: f64 = 35.264_389_68 * std::f64::consts::PI / 180.0;
        let cos_b = beta.cos();
        let sin_b = beta.sin();

        let w1 = Vector3D::new(cos_b, 0.0, sin_b);
        let w2 = Vector3D::new(0.0, cos_b, sin_b);
        let w3 = Vector3D::new(-cos_b, 0.0, sin_b);
        let w4 = Vector3D::new(0.0, -cos_b, sin_b);

        Self {
            wheels: [
                ReactionWheel::new(w1, rotor_inertia, max_speed, max_torque),
                ReactionWheel::new(w2, rotor_inertia, max_speed, max_torque),
                ReactionWheel::new(w3, rotor_inertia, max_speed, max_torque),
                ReactionWheel::new(w4, rotor_inertia, max_speed, max_torque),
            ],
        }
    }

    /// Total wheel cluster angular momentum in body frame $\mathbf{h}_{rw}$:
    pub fn total_momentum(&self) -> Vector3D {
        let mut total = Vector3D::ZERO;
        for w in &self.wheels {
            let h = w.momentum_vector();
            total.x += h.x;
            total.y += h.y;
            total.z += h.z;
        }
        total
    }

    /// Allocates body 3-axis torque demand $\boldsymbol{\tau}_{cmd}$ across the 4 wheels using Moore-Penrose pseudo-inverse:
    pub fn allocate_torque(&self, tau_cmd: Vector3D) -> [f64; 4] {
        // 3x4 distribution matrix W = [w1, w2, w3, w4]
        // W * tau_wheels = tau_cmd
        // Minimum-norm solution: tau_wheels = W^T (W W^T)^-1 tau_cmd
        let w = [
            self.wheels[0].spin_axis,
            self.wheels[1].spin_axis,
            self.wheels[2].spin_axis,
            self.wheels[3].spin_axis,
        ];

        // Gram matrix G = W * W^T (3x3):
        let mut g = [[0.0; 3]; 3];
        for k in 0..4 {
            let ax = [w[k].x, w[k].y, w[k].z];
            for i in 0..3 {
                for j in 0..3 {
                    g[i][j] += ax[i] * ax[j];
                }
            }
        }

        // Invert 3x3 Gram matrix:
        let det = g[0][0] * (g[1][1] * g[2][2] - g[1][2] * g[2][1])
            - g[0][1] * (g[1][0] * g[2][2] - g[1][2] * g[2][0])
            + g[0][2] * (g[1][0] * g[2][1] - g[1][1] * g[2][0]);

        let inv_det = 1.0 / det.max(1e-12);
        let g_inv = [
            [
                (g[1][1] * g[2][2] - g[1][2] * g[2][1]) * inv_det,
                (g[0][2] * g[2][1] - g[0][1] * g[2][2]) * inv_det,
                (g[0][1] * g[1][2] - g[0][2] * g[1][1]) * inv_det,
            ],
            [
                (g[1][2] * g[2][0] - g[1][0] * g[2][2]) * inv_det,
                (g[0][0] * g[2][2] - g[0][2] * g[2][0]) * inv_det,
                (g[0][2] * g[1][0] - g[0][0] * g[1][2]) * inv_det,
            ],
            [
                (g[1][0] * g[2][1] - g[1][1] * g[2][0]) * inv_det,
                (g[0][1] * g[2][0] - g[0][0] * g[2][1]) * inv_det,
                (g[0][0] * g[1][1] - g[0][1] * g[1][0]) * inv_det,
            ],
        ];

        let tau_vec = [tau_cmd.x, tau_cmd.y, tau_cmd.z];
        let mut lambda = [0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                lambda[i] += g_inv[i][j] * tau_vec[j];
            }
        }

        let mut torques = [0.0; 4];
        for k in 0..4 {
            torques[k] = w[k].x * lambda[0] + w[k].y * lambda[1] + w[k].z * lambda[2];
        }

        torques
    }

    /// Steps all 4 wheels forward in time under body torque command, returning actual torque applied on body:
    pub fn step(&mut self, tau_cmd: Vector3D, dt_s: f64) -> Vector3D {
        let wheel_torques = self.allocate_torque(Vector3D::new(-tau_cmd.x, -tau_cmd.y, -tau_cmd.z));
        let mut actual_body_torque = Vector3D::ZERO;

        for (k, wheel) in self.wheels.iter_mut().enumerate() {
            let body_reaction = wheel.step(wheel_torques[k], dt_s);
            actual_body_torque.x += body_reaction * wheel.spin_axis.x;
            actual_body_torque.y += body_reaction * wheel.spin_axis.y;
            actual_body_torque.z += body_reaction * wheel.spin_axis.z;
        }

        actual_body_torque
    }
}

/// Triaxial Magnetic Torquer Rods for momentum dumping.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagneticTorquerSystem {
    /// Maximum magnetic dipole moment per rod $m_{max}$ in $\text{A}\cdot\text{m}^2$.
    pub max_dipole_a_m2: f64,
}

impl MagneticTorquerSystem {
    pub fn new(max_dipole: f64) -> Self {
        Self {
            max_dipole_a_m2: max_dipole,
        }
    }

    /// Evaluates control dipole moment $\mathbf{m}$ to desaturate excess wheel angular momentum $\mathbf{h}_{excess}$:
    /// $$\mathbf{m}_{cmd} = \frac{\mathbf{h}_{excess} \times \mathbf{B}}{\|\mathbf{B}\|^2}$$
    pub fn evaluate_desaturation_dipole(
        &self,
        h_excess: Vector3D,
        b_field_tesla: Vector3D,
    ) -> Vector3D {
        let b_mag2 = b_field_tesla.norm_squared();
        if b_mag2 < 1e-16 {
            return Vector3D::ZERO;
        }

        let m_unconstrained = h_excess.cross(&b_field_tesla);
        let scale = 1.0 / b_mag2;

        Vector3D::new(
            (m_unconstrained.x * scale).clamp(-self.max_dipole_a_m2, self.max_dipole_a_m2),
            (m_unconstrained.y * scale).clamp(-self.max_dipole_a_m2, self.max_dipole_a_m2),
            (m_unconstrained.z * scale).clamp(-self.max_dipole_a_m2, self.max_dipole_a_m2),
        )
    }

    /// Magnetic torque exerted on spacecraft body: $\boldsymbol{\tau}_{mag} = \mathbf{m} \times \mathbf{B}$:
    pub fn evaluate_torque(&self, m_dipole: Vector3D, b_field_tesla: Vector3D) -> Vector3D {
        m_dipole.cross(&b_field_tesla)
    }
}

/// Evaluates Earth's geomagnetic field in body frame using eccentric tilted magnetic dipole:
pub fn earth_geomagnetic_field(r_eci: Vector3D, q_body: Quaternion) -> Vector3D {
    let r_mag = r_eci.norm();
    if r_mag < 1e-3 {
        return Vector3D::ZERO;
    }

    // Earth magnetic dipole moment: ~ 7.94e22 A*m^2 pointing near south pole
    let b0 = 3.12e-5; // 31.2 microTesla at equator
    let re = 6_378_137.0;
    let factor = b0 * (re / r_mag).powi(3);

    // Dipole axis roughly aligned with -Z in ECI:
    let z_unit = Vector3D::new(0.0, 0.0, 1.0);
    let r_unit = r_eci.normalize();

    let b_eci = Vector3D::new(
        factor * (3.0 * r_unit.dot(&z_unit) * r_unit.x - z_unit.x),
        factor * (3.0 * r_unit.dot(&z_unit) * r_unit.y - z_unit.y),
        factor * (3.0 * r_unit.dot(&z_unit) * r_unit.z - z_unit.z),
    );

    q_body.rotate_vector_world_to_body(b_eci)
}
