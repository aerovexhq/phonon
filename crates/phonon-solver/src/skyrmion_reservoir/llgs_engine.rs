#![deny(unsafe_code)]

//! Landau-Lifshitz-Gilbert-Slonczewski (LLGS) Magnetization Dynamics Engine.
//!
//! Models unit magnetization vector dynamics under effective magnetic fields
//! (external, uniaxial anisotropy, demagnetization, exchange, and DMI) and
//! Slonczewski spin-transfer torques (damping-like and field-like).
//! Provides a Spin-Torque Oscillator (STTO) integrator with RK4 numerical stepping.

pub const GYROMAGNETIC_RATIO: f64 = 1.760_859_644e11; // rad / (s * T)
pub const HBAR: f64 = 1.054_571_817e-34; // J * s
pub const ELEMENTARY_CHARGE: f64 = 1.602_176_634e-19; // C
pub const MU_0: f64 = 1.256_637_061_4e-6; // T * m / A (H / m)

/// 3D vector with standard vector algebra in Euclidean space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const EX: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const EY: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const EZ: Self = Self { x: 0.0, y: 0.0, z: 1.0 };

    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }

    pub fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }

    pub fn scale(self, s: f64) -> Self {
        Self {
            x: self.x * s,
            y: self.y * s,
            z: self.z * s,
        }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn norm_sq(self) -> f64 {
        self.dot(self)
    }

    pub fn norm(self) -> f64 {
        self.norm_sq().sqrt()
    }

    pub fn normalize(self) -> Self {
        let n = self.norm();
        if n > 1e-18 {
            self.scale(1.0 / n)
        } else {
            Self::EZ
        }
    }
}

/// Physical parameters for LLGS magnetization simulation.
#[derive(Debug, Clone)]
pub struct LlgsParams {
    /// Gyromagnetic ratio gamma in rad / (s * T). Default ~ 1.76e11.
    pub gamma: f64,
    /// Dimensionless Gilbert damping parameter alpha (typically 0.005 - 0.05).
    pub alpha: f64,
    /// Saturation magnetization M_s in A / m (e.g. 8.0e5 A/m for Permalloy/CoFeB).
    pub ms: f64,
    /// Thin film thickness d_z in meters (e.g. 1.0 nm = 1.0e-9 m).
    pub thickness: f64,
    /// Spin polarization efficiency eta (0.0 to 1.0, typically 0.35 - 0.6).
    pub spin_polarization: f64,
    /// Field-like torque ratio beta_field (typically 0.0 to 0.2).
    pub beta_field: f64,
    /// Uniaxial perpendicular magnetic anisotropy constant K_u in J / m^3.
    pub ku: f64,
    /// Demagnetization tensor diagonal components (Nx, Ny, Nz) summing to 1.0.
    pub demag_factors: (f64, f64, f64),
}

impl Default for LlgsParams {
    fn default() -> Self {
        Self {
            gamma: GYROMAGNETIC_RATIO,
            alpha: 0.02,
            ms: 8.0e5, // A / m
            thickness: 1.0e-9, // 1 nm
            spin_polarization: 0.45,
            beta_field: 0.05,
            ku: 6.0e5, // J / m^3
            demag_factors: (0.0, 0.0, 1.0), // Thin film out-of-plane demagnetization
        }
    }
}

/// Representation of effective magnetic fields acting on the magnetization.
#[derive(Debug, Clone)]
pub struct EffectiveField {
    /// External bias magnetic field in Tesla (T).
    pub h_ext: Vector3,
    /// Additional effective fields (e.g. Oersted, exchange, DMI) in Tesla (T).
    pub h_extra: Vector3,
}

impl EffectiveField {
    pub fn new(h_ext: Vector3) -> Self {
        Self {
            h_ext,
            h_extra: Vector3::ZERO,
        }
    }

    /// Evaluates total effective magnetic field in Tesla at unit magnetization m.
    pub fn compute_total_field(&self, m: Vector3, params: &LlgsParams) -> Vector3 {
        // Anisotropy field in Tesla: B_K = (2 * K_u / M_s) * m_z * e_z
        let b_k_z = (2.0 * params.ku / params.ms) * m.z;
        let b_ani = Vector3::new(0.0, 0.0, b_k_z);

        // Demagnetization field: H_demag = -M_s * (N_x m_x e_x + N_y m_y e_y + N_z m_z e_z) in A/m
        // In Tesla: B_demag = mu_0 * H_demag
        let (nx, ny, nz) = params.demag_factors;
        let b_demag = Vector3::new(
            -MU_0 * params.ms * nx * m.x,
            -MU_0 * params.ms * ny * m.y,
            -MU_0 * params.ms * nz * m.z,
        );

        self.h_ext.add(self.h_extra).add(b_ani).add(b_demag)
    }
}

/// Spin-Torque Oscillator (STTO) co-processor with LLGS dynamics.
#[derive(Debug, Clone)]
pub struct SpinTorqueOscillator {
    pub params: LlgsParams,
    /// Current unit magnetization state vector m (norm == 1.0).
    pub m: Vector3,
    /// Spin polarizer fixed orientation vector m_p (norm == 1.0).
    pub m_p: Vector3,
    /// Drive current density J_e in A / m^2.
    pub current_density: f64,
    /// Effective field generator.
    pub field: EffectiveField,
    /// Accumulated time in seconds.
    pub time_s: f64,
}

impl SpinTorqueOscillator {
    pub fn new(params: LlgsParams, initial_m: Vector3, m_p: Vector3, field: EffectiveField) -> Self {
        Self {
            params,
            m: initial_m.normalize(),
            m_p: m_p.normalize(),
            current_density: 0.0,
            field,
            time_s: 0.0,
        }
    }

    /// Computes dm/dt according to the full LLGS equation.
    pub fn evaluate_derivative(&self, m: Vector3, current_j: f64) -> Vector3 {
        let b_eff = self.field.compute_total_field(m, &self.params);
        let alpha = self.params.alpha;
        let gamma_prime = self.params.gamma / (1.0 + alpha * alpha);

        // Standard LLG precession: -gamma' * (m x B_eff)
        let m_cross_b = m.cross(b_eff);
        let precession = m_cross_b.scale(-gamma_prime);

        // LLG Gilbert damping: -alpha * gamma' * [m x (m x B_eff)]
        let m_cross_m_cross_b = m.cross(m_cross_b);
        let damping = m_cross_m_cross_b.scale(-alpha * gamma_prime);

        // Slonczewski spin-transfer torque coefficient:
        // a_J = (hbar * eta * J_e) / (2 * e * M_s * d_z)
        let a_j = (HBAR * self.params.spin_polarization * current_j)
            / (2.0 * ELEMENTARY_CHARGE * self.params.ms * self.params.thickness);
        let b_j = self.params.beta_field * a_j;

        // Damping-like torque: (a_J / (1 + alpha^2)) * [m x (m x m_p) - alpha * (m x m_p)]
        // Field-like torque:   -(b_J / (1 + alpha^2)) * [m x m_p + alpha * (m x (m x m_p))]
        let m_cross_p = m.cross(self.m_p);
        let m_cross_m_cross_p = m.cross(m_cross_p);

        let dl_term = m_cross_m_cross_p.sub(m_cross_p.scale(alpha)).scale(a_j / (1.0 + alpha * alpha));
        let fl_term = m_cross_p.add(m_cross_m_cross_p.scale(alpha)).scale(-b_j / (1.0 + alpha * alpha));

        precession.add(damping).add(dl_term).add(fl_term)
    }

    /// Advances the magnetization vector by time step dt using 4th-order Runge-Kutta (RK4).
    pub fn step_rk4(&mut self, dt: f64) {
        let j = self.current_density;
        let m0 = self.m;

        let k1 = self.evaluate_derivative(m0, j);
        let m1 = m0.add(k1.scale(0.5 * dt)).normalize();

        let k2 = self.evaluate_derivative(m1, j);
        let m2 = m0.add(k2.scale(0.5 * dt)).normalize();

        let k3 = self.evaluate_derivative(m2, j);
        let m3 = m0.add(k3.scale(dt)).normalize();

        let k4 = self.evaluate_derivative(m3, j);

        let dm = k1.add(k2.scale(2.0)).add(k3.scale(2.0)).add(k4).scale(dt / 6.0);
        self.m = m0.add(dm).normalize();
        self.time_s += dt;
    }

    /// Simulates steady-state trajectory for `num_steps` with step size `dt`.
    /// Returns time array and (mx, my, mz) traces.
    pub fn simulate_trajectory(&mut self, dt: f64, num_steps: usize) -> (Vec<f64>, Vec<Vector3>) {
        let mut times = Vec::with_capacity(num_steps);
        let mut traj = Vec::with_capacity(num_steps);

        for _ in 0..num_steps {
            self.step_rk4(dt);
            times.push(self.time_s);
            traj.push(self.m);
        }

        (times, traj)
    }

    /// Evaluates the fundamental precession frequency (in GHz) via zero-crossing or FFT peak.
    pub fn estimate_precession_frequency(&mut self, dt: f64, sample_count: usize) -> f64 {
        let (_, traj) = self.simulate_trajectory(dt, sample_count);
        if traj.len() < 4 {
            return 0.0;
        }

        // Count zero-crossings of m_x (from negative to positive)
        let mut zero_crossings = Vec::new();
        for i in 1..traj.len() {
            if traj[i - 1].x < 0.0 && traj[i].x >= 0.0 {
                let frac = -traj[i - 1].x / (traj[i].x - traj[i - 1].x + 1e-20);
                let t_cross = (i as f64 - 1.0 + frac) * dt;
                zero_crossings.push(t_cross);
            }
        }

        if zero_crossings.len() < 2 {
            // Alternatively compute gyroscopic Larmor analytical estimate: f = gamma * B_eff / (2 * pi)
            let b_eff = self.field.compute_total_field(self.m, &self.params).norm();
            return (self.params.gamma * b_eff) / (2.0 * std::f64::consts::PI * 1e9);
        }

        let mut total_period = 0.0;
        for i in 1..zero_crossings.len() {
            total_period += zero_crossings[i] - zero_crossings[i - 1];
        }
        let avg_period = total_period / (zero_crossings.len() - 1) as f64;
        if avg_period > 1e-15 {
            (1.0 / avg_period) / 1e9 // in GHz
        } else {
            0.0
        }
    }
}
