#![deny(unsafe_code)]

//! Cavity Optomechanical Squeezing, Quantum Langevin Dynamics & Wigner Quasi-Probability Engine.
//!
//! Models:
//! - Linearized optomechanical Langevin equations under two-tone back-action evading (BAE) drive.
//! - Squeezing of mechanical quadrature fluctuations below the Standard Quantum Limit (SQL = 0.5).
//! - Angular and polar quadrature variance scanning Delta X_theta^2.
//! - 2D phase-space Wigner quasi-probability distributions W(X, P) and non-classicality negativity.

use std::f64::consts::PI;

/// Reduced Planck constant in J*s.
pub const HBAR: f64 = 1.054_571_817e-34;

/// Standard Quantum Limit (SQL) variance for dimensionless quadrature [X, P] = i.
pub const SQL_VARIANCE: f64 = 0.5;

/// Physical configuration parameters for cavity optomechanical squeezing engine.
#[derive(Debug, Clone, PartialEq)]
pub struct OptomechanicalSqueezingParams {
    /// Optical or microwave cavity resonance frequency in GHz (default ~10.0 GHz).
    pub cavity_freq_ghz: f64,
    /// Mechanical resonator frequency Omega_m in MHz (default ~15.0 MHz).
    pub mech_freq_mhz: f64,
    /// Single-phonon optomechanical coupling rate g_0 in kHz (default ~800.0 kHz).
    pub optomech_coupling_g0_khz: f64,
    /// Cavity decay rate / linewidth kappa in MHz (default ~2.0 MHz).
    pub cavity_decay_kappa_mhz: f64,
    /// Mechanical damping rate gamma_m in Hz (default ~150.0 Hz, Q_m ~ 1e5).
    pub mech_damping_gamma_hz: f64,
    /// Ambient thermal phonon occupancy n_th (default ~20.0).
    pub thermal_phonon_n_th: f64,
    /// Coherent drive laser power in mW (default ~2.5 mW).
    pub drive_power_laser_mw: f64,
    /// Squeezing parameter r (default ~0.80).
    pub squeezing_parameter_r: f64,
    /// Quadrature orientation angle theta in radians in [0, pi] (default 0.0).
    pub quadrature_angle_rad: f64,
}

impl Default for OptomechanicalSqueezingParams {
    fn default() -> Self {
        Self {
            cavity_freq_ghz: 10.0,
            mech_freq_mhz: 15.0,
            optomech_coupling_g0_khz: 800.0,
            cavity_decay_kappa_mhz: 2.0,
            mech_damping_gamma_hz: 150.0,
            thermal_phonon_n_th: 20.0,
            drive_power_laser_mw: 2.5,
            squeezing_parameter_r: 0.80,
            quadrature_angle_rad: 0.0,
        }
    }
}

impl OptomechanicalSqueezingParams {
    /// Preset: Quadrature Squeezed Vacuum state under two-tone BAE drive.
    pub fn preset_squeezed_vacuum() -> Self {
        Self {
            cavity_freq_ghz: 10.0,
            mech_freq_mhz: 15.0,
            optomech_coupling_g0_khz: 800.0,
            cavity_decay_kappa_mhz: 2.0,
            mech_damping_gamma_hz: 150.0,
            thermal_phonon_n_th: 20.0,
            drive_power_laser_mw: 2.5,
            squeezing_parameter_r: 0.80,
            quadrature_angle_rad: 0.0,
        }
    }

    /// Preset: Single Phonon Fock State |1> (non-classical quantum state).
    pub fn preset_single_phonon_fock() -> Self {
        Self {
            cavity_freq_ghz: 10.0,
            mech_freq_mhz: 15.0,
            optomech_coupling_g0_khz: 800.0,
            cavity_decay_kappa_mhz: 2.0,
            mech_damping_gamma_hz: 150.0,
            thermal_phonon_n_th: 0.0,
            drive_power_laser_mw: 0.0,
            squeezing_parameter_r: 0.0,
            quadrature_angle_rad: 0.0,
        }
    }

    /// Preset: Deep Sideband Ground-State Cooled (n_final < 0.10).
    pub fn preset_ground_state_cooled() -> Self {
        Self {
            cavity_freq_ghz: 10.0,
            mech_freq_mhz: 15.0,
            optomech_coupling_g0_khz: 800.0,
            cavity_decay_kappa_mhz: 2.0,
            mech_damping_gamma_hz: 150.0,
            thermal_phonon_n_th: 20.0,
            drive_power_laser_mw: 5.0,
            squeezing_parameter_r: 0.0,
            quadrature_angle_rad: 0.0,
        }
    }

    /// Preset: Thermal Phonon Bath (classical Gaussian state).
    pub fn preset_thermal_bath() -> Self {
        Self {
            cavity_freq_ghz: 10.0,
            mech_freq_mhz: 15.0,
            optomech_coupling_g0_khz: 800.0,
            cavity_decay_kappa_mhz: 2.0,
            mech_damping_gamma_hz: 150.0,
            thermal_phonon_n_th: 20.0,
            drive_power_laser_mw: 0.0,
            squeezing_parameter_r: 0.0,
            quadrature_angle_rad: 0.0,
        }
    }

    /// Computes intracavity photon number n_cav in the critical coupling limit.
    pub fn intracavity_photons(&self) -> f64 {
        if self.drive_power_laser_mw <= 0.0 || self.cavity_freq_ghz <= 0.0 {
            return 0.0;
        }
        let omega_c = 2.0 * PI * self.cavity_freq_ghz * 1e9;
        let kappa_rad = 2.0 * PI * self.cavity_decay_kappa_mhz * 1e6;
        let p_watt = self.drive_power_laser_mw * 1e-3;
        let photon_flux = p_watt / (HBAR * omega_c);
        // Critical coupling with external rate kappa_ext = kappa / 2
        (2.0 * photon_flux / kappa_rad).max(0.0)
    }

    /// Computes enhanced optomechanical coupling rate G = g_0 * sqrt(n_cav) in kHz.
    pub fn enhanced_coupling_g_khz(&self) -> f64 {
        let n_cav = self.intracavity_photons();
        self.optomech_coupling_g0_khz * n_cav.sqrt()
    }

    /// Computes optomechanical cooperativity C = 4 * G^2 / (kappa * gamma_m).
    pub fn optomechanical_cooperativity(&self) -> f64 {
        let kappa_hz = self.cavity_decay_kappa_mhz * 1e6;
        let gamma_hz = self.mech_damping_gamma_hz;
        if kappa_hz <= 0.0 || gamma_hz <= 0.0 {
            return 0.0;
        }
        let g_hz = self.enhanced_coupling_g_khz() * 1e3;
        (4.0 * g_hz * g_hz) / (kappa_hz * gamma_hz)
    }

    /// Computes optomechanical damping rate Gamma_opt = C * gamma_m in Hz.
    pub fn optomechanical_damping_rate_hz(&self) -> f64 {
        self.optomechanical_cooperativity() * self.mech_damping_gamma_hz
    }

    /// Computes effective thermal noise floor under two-tone BAE reservoir cooling.
    pub fn thermal_noise_floor(&self) -> f64 {
        let c = self.optomechanical_cooperativity();
        if c <= 0.0 {
            return self.thermal_phonon_n_th;
        }
        // In two-tone BAE / reservoir engineering, thermal occupation is cooled by (1 + C)
        self.thermal_phonon_n_th / (1.0 + c)
    }
}

/// Quadrature variance and squeezing metrics across phase space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadratureVariance {
    /// Variance along reference X quadrature Delta X^2.
    pub var_x: f64,
    /// Variance along conjugate P quadrature Delta P^2.
    pub var_p: f64,
    /// Cross-quadrature covariance cov(X, P).
    pub cov_xp: f64,
    /// Minimum quadrature variance Delta X_min^2 across all angles.
    pub var_min: f64,
    /// Maximum (anti-squeezed) quadrature variance Delta X_max^2.
    pub var_max: f64,
    /// Peak squeezing level in dB below Standard Quantum Limit (SQL = 0.5).
    pub squeezing_db: f64,
    /// Angle theta in radians corresponding to minimum variance.
    pub theta_min_rad: f64,
}

impl QuadratureVariance {
    /// Constructs a QuadratureVariance from components and computes principal axes.
    pub fn new(var_x: f64, var_p: f64, cov_xp: f64) -> Self {
        let mean = 0.5 * (var_x + var_p);
        let diff = 0.5 * (var_x - var_p);
        let radius = (diff * diff + cov_xp * cov_xp).sqrt();

        let var_min = (mean - radius).max(1e-12);
        let var_max = mean + radius;

        // Angle of minimum variance: Delta X_theta^2 = diff * cos(2*theta) + cov * sin(2*theta) + mean
        // Minimized when 2*theta is aligned opposite to (diff, cov)
        let theta_min_rad = 0.5 * (-cov_xp).atan2(-diff);
        let theta_min_norm = if theta_min_rad < 0.0 {
            theta_min_rad + PI
        } else {
            theta_min_rad
        };

        let squeezing_db = -10.0 * (var_min / SQL_VARIANCE).log10();

        Self {
            var_x,
            var_p,
            cov_xp,
            var_min,
            var_max,
            squeezing_db,
            theta_min_rad: theta_min_norm,
        }
    }

    /// Evaluates variance along arbitrary quadrature angle theta:
    /// Delta X_theta^2 = Delta X^2 * cos^2(theta) + Delta P^2 * sin^2(theta) + cov(X, P) * sin(2*theta).
    pub fn variance_at(&self, theta: f64) -> f64 {
        let c = theta.cos();
        let s = theta.sin();
        (self.var_x * c * c + self.var_p * s * s + self.cov_xp * (2.0 * theta).sin()).max(1e-12)
    }

    /// Computes squeezing level in dB below SQL at given angle theta:
    /// S_dB = -10.0 * log10(Delta X_theta^2 / 0.5).
    pub fn squeezing_db_at(&self, theta: f64) -> f64 {
        let var = self.variance_at(theta);
        -10.0 * (var / SQL_VARIANCE).log10()
    }

    /// Returns true if minimum variance is strictly below the Standard Quantum Limit.
    pub fn is_squeezed_below_sql(&self) -> bool {
        self.var_min < SQL_VARIANCE
    }
}

/// 2D Wigner quasi-probability distribution on a phase-space grid.
#[derive(Debug, Clone, PartialEq)]
pub struct WignerQuasiProbability {
    /// Number of grid points along each phase-space dimension.
    pub grid_size: usize,
    /// X quadrature range [min, max].
    pub x_range: (f64, f64),
    /// P quadrature range [min, max].
    pub p_range: (f64, f64),
    /// 1D coordinates along X quadrature.
    pub x_coords: Vec<f64>,
    /// 1D coordinates along P quadrature.
    pub p_coords: Vec<f64>,
    /// 2D Wigner values W(X_i, P_j) indexed by [i_x][i_p].
    pub values: Vec<Vec<f64>>,
    /// Minimum value W_min across the entire phase space grid.
    pub w_min: f64,
    /// Maximum value W_max across the entire phase space grid.
    pub w_max: f64,
    /// Numerical 2D phase-space integral: integral dX dP W(X, P).
    pub total_integral: f64,
    /// Integrated volume of negative Wigner regions: integral dX dP max(0, -W(X, P)).
    pub negative_volume: f64,
    /// Flag indicating non-classicality according to the strictly negative W_min < 0 criterion.
    pub is_nonclassical: bool,
}

impl WignerQuasiProbability {
    /// Computes 2D Wigner function for Single Phonon Fock State |1>:
    /// W_1(X, P) = (1 / pi) * exp(-(X^2 + P^2)) * (2 * (X^2 + P^2) - 1).
    pub fn compute_single_phonon_fock1(grid_size: usize, range: f64) -> Self {
        let (x_coords, p_coords) = Self::generate_coords(grid_size, range);
        let mut values = vec![vec![0.0; grid_size]; grid_size];

        for (ix, &x) in x_coords.iter().enumerate() {
            let x2 = x * x;
            for (ip, &p) in p_coords.iter().enumerate() {
                let r2 = x2 + p * p;
                let w = (1.0 / PI) * (-r2).exp() * (2.0 * r2 - 1.0);
                values[ix][ip] = w;
            }
        }

        Self::build(grid_size, range, x_coords, p_coords, values)
    }

    /// Computes 2D Wigner function for Ground State |0>:
    /// W_0(X, P) = (1 / pi) * exp(-(X^2 + P^2)).
    pub fn compute_ground_state(grid_size: usize, range: f64) -> Self {
        let (x_coords, p_coords) = Self::generate_coords(grid_size, range);
        let mut values = vec![vec![0.0; grid_size]; grid_size];

        for (ix, &x) in x_coords.iter().enumerate() {
            let x2 = x * x;
            for (ip, &p) in p_coords.iter().enumerate() {
                let r2 = x2 + p * p;
                let w = (1.0 / PI) * (-r2).exp();
                values[ix][ip] = w;
            }
        }

        Self::build(grid_size, range, x_coords, p_coords, values)
    }

    /// Computes 2D Wigner function for Squeezed Vacuum state with parameter r and orientation theta:
    /// W_sqz(X, P) = (1 / pi) * exp(-exp(2r) * (X')^2 - exp(-2r) * (P')^2).
    pub fn compute_squeezed_vacuum(r: f64, theta: f64, grid_size: usize, range: f64) -> Self {
        let (x_coords, p_coords) = Self::generate_coords(grid_size, range);
        let mut values = vec![vec![0.0; grid_size]; grid_size];

        let cos_t = theta.cos();
        let sin_t = theta.sin();
        let exp_2r = (2.0 * r).exp();
        let exp_neg_2r = (-2.0 * r).exp();

        for (ix, &x) in x_coords.iter().enumerate() {
            for (ip, &p) in p_coords.iter().enumerate() {
                let x_prime = x * cos_t + p * sin_t;
                let p_prime = -x * sin_t + p * cos_t;
                let exponent = exp_2r * x_prime * x_prime + exp_neg_2r * p_prime * p_prime;
                let w = (1.0 / PI) * (-exponent).exp();
                values[ix][ip] = w;
            }
        }

        Self::build(grid_size, range, x_coords, p_coords, values)
    }

    /// Computes 2D Wigner function for Thermal State with mean occupancy n_th:
    /// W_th(X, P) = (1 / (pi * (2 * n_th + 1))) * exp(-(X^2 + P^2) / (2 * n_th + 1)).
    pub fn compute_thermal_state(n_th: f64, grid_size: usize, range: f64) -> Self {
        let (x_coords, p_coords) = Self::generate_coords(grid_size, range);
        let mut values = vec![vec![0.0; grid_size]; grid_size];

        let denom = 2.0 * n_th + 1.0;
        let norm = 1.0 / (PI * denom);

        for (ix, &x) in x_coords.iter().enumerate() {
            let x2 = x * x;
            for (ip, &p) in p_coords.iter().enumerate() {
                let r2 = x2 + p * p;
                let w = norm * (-r2 / denom).exp();
                values[ix][ip] = w;
            }
        }

        Self::build(grid_size, range, x_coords, p_coords, values)
    }

    /// Computes 2D Wigner function for Schrodinger Cat State (|alpha> + |-alpha>) / N:
    /// Exhibiting interference fringes with deep negative ripples (W < 0).
    pub fn compute_schrodinger_cat(alpha: f64, grid_size: usize, range: f64) -> Self {
        let (x_coords, p_coords) = Self::generate_coords(grid_size, range);
        let mut values = vec![vec![0.0; grid_size]; grid_size];

        let alpha_x = (2.0_f64).sqrt() * alpha;
        let norm_factor = 1.0 / (2.0 * PI * (1.0 + (-2.0 * alpha * alpha).exp()));

        for (ix, &x) in x_coords.iter().enumerate() {
            let dx_plus = x - alpha_x;
            let dx_minus = x + alpha_x;
            for (ip, &p) in p_coords.iter().enumerate() {
                let p2 = p * p;
                let gaussian1 = (-(dx_plus * dx_plus + p2)).exp();
                let gaussian2 = (-(dx_minus * dx_minus + p2)).exp();
                let interference = 2.0 * (-(x * x + p2)).exp() * (2.0 * alpha_x * p).cos();
                let w = norm_factor * (gaussian1 + gaussian2 + interference);
                values[ix][ip] = w;
            }
        }

        Self::build(grid_size, range, x_coords, p_coords, values)
    }

    /// Internal coordinate generator across [-range, range].
    fn generate_coords(grid_size: usize, range: f64) -> (Vec<f64>, Vec<f64>) {
        let n = grid_size.max(3);
        let step = (2.0 * range) / (n - 1) as f64;
        let coords: Vec<f64> = (0..n).map(|i| -range + i as f64 * step).collect();
        (coords.clone(), coords)
    }

    /// Computes normalization integral, negative volume, and extremum statistics.
    fn build(
        grid_size: usize,
        range: f64,
        x_coords: Vec<f64>,
        p_coords: Vec<f64>,
        values: Vec<Vec<f64>>,
    ) -> Self {
        let n = grid_size;
        let dx = (2.0 * range) / (n - 1) as f64;
        let dp = (2.0 * range) / (n - 1) as f64;

        let mut w_min = f64::INFINITY;
        let mut w_max = f64::NEG_INFINITY;
        let mut total_sum = 0.0;
        let mut neg_vol_sum = 0.0;

        for (ix, row) in values.iter().enumerate() {
            let wx = if ix == 0 || ix == n - 1 { 0.5 } else { 1.0 };
            for (ip, &val) in row.iter().enumerate() {
                let wp = if ip == 0 || ip == n - 1 { 0.5 } else { 1.0 };
                let weight = wx * wp;

                if val < w_min {
                    w_min = val;
                }
                if val > w_max {
                    w_max = val;
                }

                total_sum += weight * val;
                if val < 0.0 {
                    neg_vol_sum += weight * (-val);
                }
            }
        }

        let total_integral = total_sum * dx * dp;
        let negative_volume = neg_vol_sum * dx * dp;
        let is_nonclassical = w_min < -1e-6 || negative_volume > 1e-6;

        Self {
            grid_size,
            x_range: (-range, range),
            p_range: (-range, range),
            x_coords,
            p_coords,
            values,
            w_min,
            w_max,
            total_integral,
            negative_volume,
            is_nonclassical,
        }
    }

    /// Evaluates quasi-probability value at discrete grid indices.
    pub fn at(&self, ix: usize, ip: usize) -> f64 {
        self.values[ix][ip]
    }
}

/// Linearized quantum Langevin equation solver for optomechanical quadrature squeezing.
#[derive(Debug, Clone, PartialEq)]
pub struct QuadratureSqueezingSolver {
    pub params: OptomechanicalSqueezingParams,
}

impl QuadratureSqueezingSolver {
    /// Creates a new solver with specified optomechanical parameters.
    pub fn new(params: OptomechanicalSqueezingParams) -> Self {
        Self { params }
    }

    /// Evaluates mechanical quadrature variance in the two-tone BAE regime:
    /// Delta X_squeezed^2 = 0.5 * exp(-2*r) + thermal_noise_floor.
    /// Delta P_anti^2 = 0.5 * exp(2*r) + thermal_noise_floor + backaction.
    pub fn solve_variance(&self) -> QuadratureVariance {
        let r = self.params.squeezing_parameter_r;
        let theta_0 = self.params.quadrature_angle_rad;
        let n_th_eff = self.params.thermal_noise_floor();

        // Linearized quantum Langevin steady-state variance in the two-tone BAE regime
        let var_squeezed = 0.5 * (-2.0 * r).exp() + n_th_eff;
        let var_anti = 0.5 * (2.0 * r).exp() + n_th_eff;

        // Rotate covariance matrix by quadrature orientation theta_0
        let cos_t = theta_0.cos();
        let sin_t = theta_0.sin();
        let cos2 = cos_t * cos_t;
        let sin2 = sin_t * sin_t;

        let var_x = var_squeezed * cos2 + var_anti * sin2;
        let var_p = var_squeezed * sin2 + var_anti * cos2;
        let cov_xp = (var_anti - var_squeezed) * sin_t * cos_t;

        QuadratureVariance::new(var_x, var_p, cov_xp)
    }

    /// Computes polar / angular quadrature profile Delta X_theta^2 across theta in [0, 2*pi].
    pub fn compute_quadrature_scan(&self, steps: usize) -> Vec<(f64, f64)> {
        let qv = self.solve_variance();
        let num_steps = steps.max(8);
        (0..=num_steps)
            .map(|i| {
                let theta = (i as f64 / num_steps as f64) * 2.0 * PI;
                let var = qv.variance_at(theta);
                (theta, var)
            })
            .collect()
    }

    /// Computes full polar quadrature profile across [0, pi] for compact angular display.
    pub fn compute_quadrature_scan_half(&self, steps: usize) -> Vec<(f64, f64)> {
        let qv = self.solve_variance();
        let num_steps = steps.max(8);
        (0..=num_steps)
            .map(|i| {
                let theta = (i as f64 / num_steps as f64) * PI;
                let var = qv.variance_at(theta);
                (theta, var)
            })
            .collect()
    }
}
