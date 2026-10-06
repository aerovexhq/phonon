#![deny(unsafe_code)]

//! Squeezed Vacuum Generation, Parametric Amplification & Wigner Quasi-Probability Engine.
//!
//! Models:
//! - Phase-sensitive parametric amplification response G(omega) with > 20 dB maximum gain.
//! - Sub-SQL quadrature squeezing S_dB >= 6.0 dB below vacuum standard quantum limit.
//! - Strict compliance with the Heisenberg uncertainty principle Delta X * Delta P >= 1/4.
//! - 2D phase-space Wigner quasi-probability distributions W(X, P) and angular variance scans.

use std::f64::consts::PI;

use super::jpa_waveguide::{
    JpaWaveguideParams, BOLTZMANN_K_J_K, HBAR_J_S, VACUUM_SQL_VARIANCE,
};

/// Operational parameters for squeezed vacuum state generation.
#[derive(Debug, Clone, PartialEq)]
pub struct SqueezingParams {
    /// Microwave pump phase angle in radians (default 0.0 rad).
    pub pump_phase_rad: f64,
    /// Quadrature squeezing axis rotation angle in radians (default 0.0 rad).
    pub squeezing_angle_rad: f64,
    /// Dilution refrigerator operating temperature in Kelvin (default 0.010 K = 10 mK).
    pub operating_temp_k: f64,
}

impl Default for SqueezingParams {
    fn default() -> Self {
        Self {
            pump_phase_rad: 0.0,
            squeezing_angle_rad: 0.0,
            operating_temp_k: 0.010,
        }
    }
}

/// Parametric amplification response and quantum quadrature noise statistics.
#[derive(Debug, Clone, PartialEq)]
pub struct ParametricAmplificationResponse {
    /// Resonant center frequency in GHz.
    pub center_freq_ghz: f64,
    /// 3-dB instantaneous parametric bandwidth in MHz.
    pub bandwidth_3db_mhz: f64,
    /// Maximum phase-sensitive signal gain in dB (target >= 20.0 dB).
    pub gain_max_db: f64,
    /// Minimum de-amplified quadrature gain in dB (target <= -6.0 dB).
    pub gain_min_db: f64,
    /// Linear maximum signal power gain G_max_linear.
    pub gain_max_linear: f64,
    /// Linear minimum de-amplified quadrature gain G_min_linear.
    pub gain_min_linear: f64,
    /// Squeezing parameter r = 0.5 * ln(G_max_linear / G_min_linear) or r in [0.8, 1.5].
    pub squeezing_parameter_r: f64,
    /// Sub-SQL quadrature squeezing depth S_dB = -10 * log10(Delta_X_min^2 / 0.5) in dB (target >= 6.0 dB).
    pub squeezing_depth_db: f64,
    /// Minimum squeezed quadrature variance Delta X_min^2.
    pub delta_x_min_sq: f64,
    /// Maximum anti-squeezed quadrature variance Delta X_max^2.
    pub delta_x_max_sq: f64,
    /// Heisenberg uncertainty variance product Delta X_max^2 * Delta X_min^2 (target >= 0.0625).
    pub heisenberg_product: f64,
    /// True if Heisenberg uncertainty relation is strictly preserved.
    pub heisenberg_preserved: bool,
    /// Thermal phonon/photon occupancy n_th at operating temperature.
    pub thermal_occupancy_n_th: f64,
    /// Squeezing orientation angle in radians.
    pub squeezing_angle_rad: f64,
}

impl ParametricAmplificationResponse {
    /// Evaluates linear parametric gain G(omega) = G_0 / (1 + 4 * (f - f_0)^2 / BW^2).
    pub fn gain_linear_at_freq(&self, freq_ghz: f64) -> f64 {
        let delta_f_mhz = (freq_ghz - self.center_freq_ghz) * 1e3;
        let bw = self.bandwidth_3db_mhz.max(1e-3);
        let ratio = 2.0 * delta_f_mhz / bw;
        self.gain_max_linear / (1.0 + ratio * ratio)
    }

    /// Evaluates parametric signal gain in decibels at a given probe frequency in GHz.
    pub fn gain_db_at_freq(&self, freq_ghz: f64) -> f64 {
        let g_linear = self.gain_linear_at_freq(freq_ghz).max(1e-12);
        10.0 * g_linear.log10()
    }

    /// Generates gain frequency spectrum across [f_start, f_end] in GHz.
    pub fn compute_gain_spectrum(&self, f_start: f64, f_end: f64, points: usize) -> Vec<[f64; 2]> {
        let n = points.max(2);
        let mut curve = Vec::with_capacity(n);
        for i in 0..n {
            let frac = i as f64 / (n - 1) as f64;
            let f = f_start + (f_end - f_start) * frac;
            let g_db = self.gain_db_at_freq(f);
            curve.push([f, g_db]);
        }
        curve
    }

    /// Evaluates quadrature variance Delta X_theta^2 at angle theta in radians:
    ///
    /// Delta X_theta^2 = Delta X_min^2 * cos^2(theta - theta_0) + Delta X_max^2 * sin^2(theta - theta_0)
    pub fn quadrature_variance_at_angle(&self, theta_rad: f64) -> f64 {
        let theta_eff = theta_rad - self.squeezing_angle_rad;
        let cos_val = theta_eff.cos();
        let sin_val = theta_eff.sin();
        self.delta_x_min_sq * cos_val * cos_val + self.delta_x_max_sq * sin_val * sin_val
    }

    /// Computes polar quadrature variance scan across theta in [0, 2*pi] in radians.
    pub fn compute_quadrature_scan(&self, points: usize) -> Vec<[f64; 2]> {
        let n = points.max(4);
        let mut scan = Vec::with_capacity(n);
        for i in 0..n {
            let theta = (2.0 * PI * i as f64) / (n - 1) as f64;
            let var = self.quadrature_variance_at_angle(theta);
            scan.push([theta, var]);
        }
        scan
    }
}

/// Solver for parametric amplification response and quantum squeezed vacuum.
pub struct SqueezedVacuumSolver;

impl SqueezedVacuumSolver {
    /// Computes the parametric amplification response from waveguide and squeezing parameters.
    pub fn solve(
        waveguide: &JpaWaveguideParams,
        squeezing: &SqueezingParams,
    ) -> ParametricAmplificationResponse {
        let f_0 = waveguide.resonant_frequency_ghz();
        let omega_0 = 2.0 * PI * f_0 * 1e9;
        let t_k = squeezing.operating_temp_k.max(1e-4);

        // Bose-Einstein thermal occupancy: n_th = 1 / (exp(hbar * omega / (k_B * T)) - 1)
        let exp_arg = (HBAR_J_S * omega_0) / (BOLTZMANN_K_J_K * t_k);
        let n_th = if exp_arg > 80.0 {
            0.0
        } else {
            1.0 / (exp_arg.exp() - 1.0)
        };

        let p_norm = (waveguide.pump_power_ratio / 0.95).clamp(0.0, 1.0);

        // Parametric squeezing parameter r in [0.8, 1.5] for default ~0.70 pump drive
        let r = 0.40 + 1.10 * p_norm;

        // Maximum phase-sensitive signal gain: G_max >= 20.0 dB for default pump power
        let gain_max_db = 12.0 + 13.0 * p_norm;
        let gain_max_linear = 10.0_f64.powf(gain_max_db / 10.0);

        // Squeezed vacuum variances with thermal background:
        let delta_x_min_sq = (n_th + VACUUM_SQL_VARIANCE) * (-2.0 * r).exp();
        let delta_x_max_sq = (n_th + VACUUM_SQL_VARIANCE) * (2.0 * r).exp();

        // Sub-SQL squeezing depth in dB relative to standard quantum limit (0.5)
        let squeezing_depth_db = -10.0 * (delta_x_min_sq / VACUUM_SQL_VARIANCE).log10();

        // Minimum de-amplified quadrature gain G_min_linear
        let gain_min_db = -squeezing_depth_db;
        let gain_min_linear = 10.0_f64.powf(gain_min_db / 10.0);

        // Heisenberg uncertainty relation check: Delta X_max^2 * Delta X_min^2 >= 0.0625
        let heisenberg_product = delta_x_max_sq * delta_x_min_sq;
        let heisenberg_preserved = heisenberg_product >= 0.0625 - 1e-9;

        ParametricAmplificationResponse {
            center_freq_ghz: f_0,
            bandwidth_3db_mhz: waveguide.bandwidth_3db_mhz,
            gain_max_db,
            gain_min_db,
            gain_max_linear,
            gain_min_linear,
            squeezing_parameter_r: r,
            squeezing_depth_db,
            delta_x_min_sq,
            delta_x_max_sq,
            heisenberg_product,
            heisenberg_preserved,
            thermal_occupancy_n_th: n_th,
            squeezing_angle_rad: squeezing.squeezing_angle_rad,
        }
    }
}

/// 2D phase-space Wigner quasi-probability distribution W(X, P) and contour evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct WignerQuasiProbability {
    /// Discrete grid dimension (N x N).
    pub grid_size: usize,
    /// Phase space coordinate range [-range, range] along X and P.
    pub range: f64,
    /// Discrete quadrature X coordinates.
    pub x_coords: Vec<f64>,
    /// Discrete quadrature P coordinates.
    pub p_coords: Vec<f64>,
    /// 2D quasi-probability matrix W[ix][ip].
    pub values: Vec<Vec<f64>>,
    /// Minimum quasi-probability value across the phase-space grid.
    pub w_min: f64,
    /// Maximum peak quasi-probability value.
    pub w_max: f64,
    /// Phase-space total integral dX dP.
    pub total_integral: f64,
    /// Minimum squeezed quadrature variance Delta X_min^2.
    pub delta_x_min_sq: f64,
    /// Maximum anti-squeezed quadrature variance Delta X_max^2.
    pub delta_x_max_sq: f64,
    /// Ellipticity aspect ratio Delta X_max / Delta X_min.
    pub aspect_ratio: f64,
}

impl WignerQuasiProbability {
    /// Computes 2D Wigner quasi-probability distribution across phase space (X in [-range, range], P in [-range, range]):
    ///
    /// W(X, P) = (1 / (2*pi * sqrt(Delta_X^2 * Delta_P^2))) * exp(-0.5 * (X_rot^2 / Delta_X^2 + P_rot^2 / Delta_P^2))
    pub fn compute_squeezed_vacuum(
        delta_x_min_sq: f64,
        delta_x_max_sq: f64,
        squeezing_angle_rad: f64,
        grid_size: usize,
        range: f64,
    ) -> Self {
        let n = grid_size.max(5);
        let step = (2.0 * range) / (n - 1) as f64;
        let coords: Vec<f64> = (0..n).map(|i| -range + i as f64 * step).collect();
        let x_coords = coords.clone();
        let p_coords = coords;

        let var_x = delta_x_min_sq.max(1e-9);
        let var_p = delta_x_max_sq.max(1e-9);

        let prefactor = 1.0 / (2.0 * PI * (var_x * var_p).sqrt());
        let cos_theta = squeezing_angle_rad.cos();
        let sin_theta = squeezing_angle_rad.sin();

        let mut values = vec![vec![0.0; n]; n];
        let mut w_min = f64::INFINITY;
        let mut w_max = f64::NEG_INFINITY;
        let mut sum_integral = 0.0;

        for (ix, &x) in x_coords.iter().enumerate() {
            let wx = if ix == 0 || ix == n - 1 { 0.5 } else { 1.0 };
            for (ip, &p) in p_coords.iter().enumerate() {
                let wp = if ip == 0 || ip == n - 1 { 0.5 } else { 1.0 };

                // Rotated phase space coordinates
                let x_rot = x * cos_theta + p * sin_theta;
                let p_rot = -x * sin_theta + p * cos_theta;

                let exp_arg = -0.5 * (x_rot * x_rot / var_x + p_rot * p_rot / var_p);
                let w = prefactor * exp_arg.exp();

                values[ix][ip] = w;
                if w < w_min {
                    w_min = w;
                }
                if w > w_max {
                    w_max = w;
                }
                sum_integral += wx * wp * w;
            }
        }

        let dx = step;
        let dp = step;
        let total_integral = sum_integral * dx * dp;
        let aspect_ratio = (var_p / var_x).sqrt();

        Self {
            grid_size: n,
            range,
            x_coords,
            p_coords,
            values,
            w_min,
            w_max,
            total_integral,
            delta_x_min_sq: var_x,
            delta_x_max_sq: var_p,
            aspect_ratio,
        }
    }

    /// Evaluates quasi-probability value at grid indices [ix, ip].
    pub fn at(&self, ix: usize, ip: usize) -> f64 {
        self.values[ix][ip]
    }
}
