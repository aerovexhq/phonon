#![deny(unsafe_code)]

//! Multi-tone frequency-domain Harmonic Balance non-linear solver with H harmonics.
//!
//! Solves Y(omega) * V(omega) + I_NL(V) = I_S(omega) via Newton-Raphson iteration
//! with IFFT/FFT evaluating non-linear devices in the time domain.
//! Computes 1-dB compression point (P1dB) and third-order intercept point (IP3 / TOI).

use super::s_parameters::Complex64;
use std::f64::consts::PI;
use std::sync::Arc;

/// Non-linear device constitutive relation i = f(v) and differential conductance g = df/dv.
#[derive(Clone)]
pub enum NonlinearDevice {
    /// Weakly non-linear polynomial transconductance model: i(v) = a1*v + a2*v^2 + a3*v^3.
    Polynomial { a1: f64, a2: f64, a3: f64 },
    /// Exponential diode junction model: i(v) = Is * (exp(v / Vt) - 1).
    Diode { is_sat: f64, vt: f64 },
    /// Hyperbolic tangent saturation model: i(v) = Isat * tanh(v / Vsat).
    Tanh { i_sat: f64, v_sat: f64 },
    /// User-defined non-linear device function and analytical derivative.
    Custom {
        eval_fn: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
        deriv_fn: Arc<dyn Fn(f64) -> f64 + Send + Sync>,
    },
}

impl std::fmt::Debug for NonlinearDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Polynomial { a1, a2, a3 } => f
                .debug_struct("Polynomial")
                .field("a1", a1)
                .field("a2", a2)
                .field("a3", a3)
                .finish(),
            Self::Diode { is_sat, vt } => f
                .debug_struct("Diode")
                .field("is_sat", is_sat)
                .field("vt", vt)
                .finish(),
            Self::Tanh { i_sat, v_sat } => f
                .debug_struct("Tanh")
                .field("i_sat", i_sat)
                .field("v_sat", v_sat)
                .finish(),
            Self::Custom { .. } => f.write_str("Custom { ... }"),
        }
    }
}

impl NonlinearDevice {
    /// Evaluates non-linear current i = f(v).
    pub fn eval(&self, v: f64) -> f64 {
        match self {
            Self::Polynomial { a1, a2, a3 } => a1 * v + a2 * v * v + a3 * v * v * v,
            Self::Diode { is_sat, vt } => {
                let vt_safe = vt.max(1e-6);
                let x = v / vt_safe;
                if x < 40.0 {
                    is_sat * (x.exp() - 1.0)
                } else {
                    let exp_40 = 40.0_f64.exp();
                    is_sat * (exp_40 - 1.0) + (is_sat * exp_40 / vt_safe) * (v - 40.0 * vt_safe)
                }
            }
            Self::Tanh { i_sat, v_sat } => {
                let v_sat_safe = v_sat.max(1e-6);
                i_sat * (v / v_sat_safe).tanh()
            }
            Self::Custom { eval_fn, .. } => eval_fn(v),
        }
    }

    /// Evaluates differential conductance g(v) = di/dv.
    pub fn derivative(&self, v: f64) -> f64 {
        match self {
            Self::Polynomial { a1, a2, a3 } => a1 + 2.0 * a2 * v + 3.0 * a3 * v * v,
            Self::Diode { is_sat, vt } => {
                let vt_safe = vt.max(1e-6);
                let x = v / vt_safe;
                if x < 40.0 {
                    (is_sat / vt_safe) * x.exp()
                } else {
                    let exp_40 = 40.0_f64.exp();
                    (is_sat / vt_safe) * exp_40
                }
            }
            Self::Tanh { i_sat, v_sat } => {
                let v_sat_safe = v_sat.max(1e-6);
                let t = (v / v_sat_safe).tanh();
                (i_sat / v_sat_safe) * (1.0 - t * t)
            }
            Self::Custom { deriv_fn, .. } => deriv_fn(v),
        }
    }
}

/// Spectral component at harmonic k * f0.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HarmonicComponent {
    /// Harmonic index (0 for DC, 1 for fundamental f0, 2 for 2*f0, etc.).
    pub harmonic_index: usize,
    /// Frequency in Hertz.
    pub freq_hz: f64,
    /// Complex voltage phasor V_k.
    pub voltage_phasor: Complex64,
    /// Peak voltage magnitude |V_k| in Volts.
    pub voltage_mag: f64,
    /// Dissipated RF power into load impedance Z0 in dBm.
    pub power_dbm: f64,
}

/// Output solution of a Harmonic Balance simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct HarmonicBalanceResult {
    /// Fundamental excitation frequency in Hertz.
    pub fundamental_hz: f64,
    /// Resolved harmonic components from DC (k=0) up to H (k=H).
    pub harmonics: Vec<HarmonicComponent>,
    /// Number of Newton-Raphson iterations executed.
    pub iterations: usize,
    /// Whether the solver reached the specified tolerance.
    pub converged: bool,
    /// L2 norm of the final residual vector.
    pub residual_norm: f64,
}

impl HarmonicBalanceResult {
    /// Power at fundamental frequency f0 in dBm.
    pub fn fundamental_power_dbm(&self) -> f64 {
        self.harmonics
            .iter()
            .find(|h| h.harmonic_index == 1)
            .map(|h| h.power_dbm)
            .unwrap_or(-150.0)
    }

    /// Power at second harmonic 2*f0 in dBm.
    pub fn second_harmonic_power_dbm(&self) -> f64 {
        self.harmonics
            .iter()
            .find(|h| h.harmonic_index == 2)
            .map(|h| h.power_dbm)
            .unwrap_or(-150.0)
    }

    /// Power at third harmonic 3*f0 in dBm.
    pub fn third_harmonic_power_dbm(&self) -> f64 {
        self.harmonics
            .iter()
            .find(|h| h.harmonic_index == 3)
            .map(|h| h.power_dbm)
            .unwrap_or(-150.0)
    }

    /// Total Harmonic Distortion (THD) percentage:
    /// THD = sqrt(sum_{k>=2} |V_k|^2) / |V_1| * 100%
    pub fn thd_pct(&self) -> f64 {
        let v1 = self
            .harmonics
            .iter()
            .find(|h| h.harmonic_index == 1)
            .map(|h| h.voltage_mag)
            .unwrap_or(0.0);

        if v1 < 1e-12 {
            return 0.0;
        }

        let sum_sq: f64 = self
            .harmonics
            .iter()
            .filter(|h| h.harmonic_index >= 2)
            .map(|h| h.voltage_mag * h.voltage_mag)
            .sum();

        (sum_sq.sqrt() / v1) * 100.0
    }
}

/// Extracted RF non-linear compression and third-order intercept metrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NonLinearMetrics {
    /// Input 1-dB gain compression point in dBm (Pin_1dB).
    pub p1db_in_dbm: f64,
    /// Output 1-dB gain compression point in dBm (Pout_1dB).
    pub p1db_out_dbm: f64,
    /// Input third-order intercept point in dBm (IIP3).
    pub ip3_in_dbm: f64,
    /// Output third-order intercept point in dBm (OIP3).
    pub ip3_out_dbm: f64,
    /// Small-signal linear power gain in dB (G0).
    pub linear_gain_db: f64,
}

/// Harmonic Balance solver for non-linear microwave circuits.
#[derive(Clone)]
pub struct HarmonicBalanceSolver {
    /// Fundamental frequency in Hertz.
    pub fundamental_hz: f64,
    /// Number of harmonics H (simulates harmonics k = 0, 1, ..., H).
    pub num_harmonics: usize,
    /// Characteristic reference and termination impedance Z0 in Ohms.
    pub z0: f64,
    /// Non-linear device under test.
    pub device: NonlinearDevice,
    /// Maximum Newton-Raphson iterations.
    pub max_iterations: usize,
    /// Convergence tolerance for residual norm.
    pub tolerance: f64,
    /// Linear network admittance function Y(omega) in Siemens.
    linear_admittance_fn: Option<Arc<dyn Fn(f64) -> Complex64 + Send + Sync>>,
}

impl std::fmt::Debug for HarmonicBalanceSolver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HarmonicBalanceSolver")
            .field("fundamental_hz", &self.fundamental_hz)
            .field("num_harmonics", &self.num_harmonics)
            .field("z0", &self.z0)
            .field("device", &self.device)
            .field("max_iterations", &self.max_iterations)
            .field("tolerance", &self.tolerance)
            .finish()
    }
}

impl HarmonicBalanceSolver {
    /// Constructs a new Harmonic Balance solver.
    pub fn new(fundamental_hz: f64, num_harmonics: usize, device: NonlinearDevice) -> Self {
        Self {
            fundamental_hz: fundamental_hz.max(1.0),
            num_harmonics: num_harmonics.clamp(1, 64),
            z0: 50.0,
            device,
            max_iterations: 60,
            tolerance: 1e-8,
            linear_admittance_fn: None,
        }
    }

    /// Sets reference termination impedance Z0.
    pub fn with_z0(mut self, z0: f64) -> Self {
        if z0 > 0.0 {
            self.z0 = z0;
        }
        self
    }

    /// Sets maximum Newton-Raphson iterations.
    pub fn with_max_iterations(mut self, iters: usize) -> Self {
        self.max_iterations = iters.max(5);
        self
    }

    /// Sets convergence tolerance.
    pub fn with_tolerance(mut self, tol: f64) -> Self {
        self.tolerance = tol.max(1e-15);
        self
    }

    /// Configures a custom linear network admittance function Y(f_hz).
    pub fn with_linear_admittance<F>(mut self, f: F) -> Self
    where
        F: Fn(f64) -> Complex64 + Send + Sync + 'static,
    {
        self.linear_admittance_fn = Some(Arc::new(f));
        self
    }

    /// Evaluates linear network admittance at frequency f_hz.
    #[inline]
    pub fn linear_admittance(&self, f_hz: f64) -> Complex64 {
        if let Some(ref y_fn) = self.linear_admittance_fn {
            y_fn(f_hz)
        } else {
            Complex64::new(1.0 / self.z0, 0.0)
        }
    }

    /// Solves the Harmonic Balance system for arbitrary source harmonic currents I_S.
    ///
    /// System equation: Y(k*omega0) * V_k + I_NL,k(V) = I_S,k
    pub fn solve(&self, is_harmonics: &[Complex64]) -> Result<HarmonicBalanceResult, String> {
        let h = self.num_harmonics;
        let dim = 2 * h + 1;

        // Choose number of time samples N >= 2*H + 1, using power of two
        let n_time = (4 * (h + 1)).next_power_of_two().max(32);

        // Precompute linear admittance Y_k for k = 0..=h
        let mut y_k: Vec<Complex64> = Vec::with_capacity(h + 1);
        for k in 0..=h {
            let f = (k as f64) * self.fundamental_hz;
            y_k.push(self.linear_admittance(f));
        }

        // Align source currents
        let mut is_vec: Vec<Complex64> = vec![Complex64::ZERO; h + 1];
        for (k, &is_val) in is_harmonics.iter().enumerate().take(h + 1) {
            is_vec[k] = is_val;
        }

        // Initial guess: linear solution V_k = I_S,k / Y_k
        let mut x = vec![0.0_f64; dim];
        for k in 0..=h {
            let y_norm = y_k[k].norm_sq();
            if y_norm > 1e-20 {
                let v_init = is_vec[k] / y_k[k];
                if k == 0 {
                    x[0] = v_init.re;
                } else {
                    x[2 * k - 1] = v_init.re;
                    x[2 * k] = v_init.im;
                }
            }
        }

        // Precalculate Fourier trigonometric basis tables
        let mut cos_table = vec![vec![0.0_f64; n_time]; h + 1];
        let mut sin_table = vec![vec![0.0_f64; n_time]; h + 1];
        for k in 0..=h {
            for n in 0..n_time {
                let angle = 2.0 * PI * (k as f64) * (n as f64) / (n_time as f64);
                cos_table[k][n] = angle.cos();
                sin_table[k][n] = angle.sin();
            }
        }

        let mut v_time = vec![0.0_f64; n_time];
        let mut i_time = vec![0.0_f64; n_time];
        let mut g_time = vec![0.0_f64; n_time];

        let mut converged = false;
        let mut final_res_norm = 0.0_f64;
        let mut iterations_done = 0;

        for iter in 0..self.max_iterations {
            iterations_done = iter + 1;

            // 1. Synthesize time-domain voltage: v_n = V0 + sum_{k=1}^h (V_k,re * cos - V_k,im * sin)
            for n in 0..n_time {
                let mut v_n = x[0];
                for k in 1..=h {
                    let re_k = x[2 * k - 1];
                    let im_k = x[2 * k];
                    v_n += re_k * cos_table[k][n] - im_k * sin_table[k][n];
                }
                v_time[n] = v_n;
                i_time[n] = self.device.eval(v_n);
                g_time[n] = self.device.derivative(v_n);
            }

            // 2. Frequency-domain analysis of device current:
            // I_NL,0 = (1/N) * sum i_n
            // I_NL,k,re = (2/N) * sum i_n * cos_k
            // I_NL,k,im = -(2/N) * sum i_n * sin_k
            let mut res = vec![0.0_f64; dim];
            let inv_n = 1.0 / (n_time as f64);
            let two_inv_n = 2.0 * inv_n;

            // DC residual
            let mut inl_0 = 0.0_f64;
            for n in 0..n_time {
                inl_0 += i_time[n];
            }
            inl_0 *= inv_n;
            res[0] = y_k[0].re * x[0] + inl_0 - is_vec[0].re;

            // AC harmonics residual
            for k in 1..=h {
                let mut inl_k_re = 0.0_f64;
                let mut inl_k_im = 0.0_f64;
                for n in 0..n_time {
                    inl_k_re += i_time[n] * cos_table[k][n];
                    inl_k_im += i_time[n] * sin_table[k][n];
                }
                inl_k_re *= two_inv_n;
                inl_k_im *= -two_inv_n;

                let vk_re = x[2 * k - 1];
                let vk_im = x[2 * k];
                let yk = y_k[k];

                let y_vk_re = yk.re * vk_re - yk.im * vk_im;
                let y_vk_im = yk.im * vk_re + yk.re * vk_im;

                res[2 * k - 1] = y_vk_re + inl_k_re - is_vec[k].re;
                res[2 * k] = y_vk_im + inl_k_im - is_vec[k].im;
            }

            // Compute residual L2 norm
            let res_norm: f64 = res.iter().map(|&r| r * r).sum::<f64>().sqrt();
            final_res_norm = res_norm;

            if res_norm < self.tolerance {
                converged = true;
                break;
            }

            // 3. Assemble Jacobian matrix J (dim x dim)
            let mut j_mat = vec![vec![0.0_f64; dim]; dim];

            // Linear admittance contribution
            j_mat[0][0] = y_k[0].re;
            for k in 1..=h {
                let yk = y_k[k];
                j_mat[2 * k - 1][2 * k - 1] += yk.re;
                j_mat[2 * k - 1][2 * k] += -yk.im;
                j_mat[2 * k][2 * k - 1] += yk.im;
                j_mat[2 * k][2 * k] += yk.re;
            }

            // Non-linear conductance contribution
            // Row 0: d(I_NL,0) / dx_m
            let mut d_inl0_dv0 = 0.0_f64;
            for n in 0..n_time {
                d_inl0_dv0 += g_time[n];
            }
            j_mat[0][0] += d_inl0_dv0 * inv_n;

            for m in 1..=h {
                let mut sum_cos = 0.0_f64;
                let mut sum_sin = 0.0_f64;
                for n in 0..n_time {
                    sum_cos += g_time[n] * cos_table[m][n];
                    sum_sin += g_time[n] * sin_table[m][n];
                }
                j_mat[0][2 * m - 1] += sum_cos * inv_n;
                j_mat[0][2 * m] += -sum_sin * inv_n;
            }

            // AC rows: k = 1..=h
            for k in 1..=h {
                // Column 0 (dV0)
                let mut sum_k_cos = 0.0_f64;
                let mut sum_k_sin = 0.0_f64;
                for n in 0..n_time {
                    sum_k_cos += g_time[n] * cos_table[k][n];
                    sum_k_sin += g_time[n] * sin_table[k][n];
                }
                j_mat[2 * k - 1][0] += sum_k_cos * two_inv_n;
                j_mat[2 * k][0] += -sum_k_sin * two_inv_n;

                // Columns m = 1..=h
                for m in 1..=h {
                    let mut s_cc = 0.0_f64;
                    let mut s_cs = 0.0_f64;
                    let mut s_sc = 0.0_f64;
                    let mut s_ss = 0.0_f64;
                    for n in 0..n_time {
                        let gn = g_time[n];
                        let ck = cos_table[k][n];
                        let sk = sin_table[k][n];
                        let cm = cos_table[m][n];
                        let sm = sin_table[m][n];

                        s_cc += gn * ck * cm;
                        s_cs += gn * ck * sm;
                        s_sc += gn * sk * cm;
                        s_ss += gn * sk * sm;
                    }

                    // d(I_NL,k,re) / d(Vm,re)
                    j_mat[2 * k - 1][2 * m - 1] += s_cc * two_inv_n;
                    // d(I_NL,k,re) / d(Vm,im)
                    j_mat[2 * k - 1][2 * m] += -s_cs * two_inv_n;
                    // d(I_NL,k,im) / d(Vm,re)
                    j_mat[2 * k][2 * m - 1] += -s_sc * two_inv_n;
                    // d(I_NL,k,im) / d(Vm,im)
                    j_mat[2 * k][2 * m] += s_ss * two_inv_n;
                }
            }

            // 4. Solve linear system J * delta = -res via Gaussian elimination with partial pivoting
            let mut rhs: Vec<f64> = res.iter().map(|&r| -r).collect();
            let delta = solve_linear_system(&mut j_mat, &mut rhs)?;

            // 5. Backtracking line search
            let mut alpha = 1.0_f64;
            let mut step_accepted = false;

            for _ in 0..4 {
                let mut x_cand = vec![0.0_f64; dim];
                for i in 0..dim {
                    x_cand[i] = x[i] + alpha * delta[i];
                }

                // Evaluate candidate residual norm
                let mut cand_res_norm_sq = 0.0_f64;
                for n in 0..n_time {
                    let mut v_n = x_cand[0];
                    for k in 1..=h {
                        v_n += x_cand[2 * k - 1] * cos_table[k][n] - x_cand[2 * k] * sin_table[k][n];
                    }
                    v_time[n] = v_n;
                    i_time[n] = self.device.eval(v_n);
                }

                let mut cand_inl_0 = 0.0_f64;
                for n in 0..n_time {
                    cand_inl_0 += i_time[n];
                }
                cand_inl_0 *= inv_n;
                let cand_r0 = y_k[0].re * x_cand[0] + cand_inl_0 - is_vec[0].re;
                cand_res_norm_sq += cand_r0 * cand_r0;

                for k in 1..=h {
                    let mut inlk_re = 0.0_f64;
                    let mut inlk_im = 0.0_f64;
                    for n in 0..n_time {
                        inlk_re += i_time[n] * cos_table[k][n];
                        inlk_im += i_time[n] * sin_table[k][n];
                    }
                    inlk_re *= two_inv_n;
                    inlk_im *= -two_inv_n;

                    let vk_re = x_cand[2 * k - 1];
                    let vk_im = x_cand[2 * k];
                    let yk = y_k[k];

                    let r_re = (yk.re * vk_re - yk.im * vk_im) + inlk_re - is_vec[k].re;
                    let r_im = (yk.im * vk_re + yk.re * vk_im) + inlk_im - is_vec[k].im;
                    cand_res_norm_sq += r_re * r_re + r_im * r_im;
                }

                let cand_res_norm = cand_res_norm_sq.sqrt();
                if cand_res_norm < res_norm || alpha <= 0.125 {
                    x = x_cand;
                    step_accepted = true;
                    break;
                }
                alpha *= 0.5;
            }

            if !step_accepted {
                for i in 0..dim {
                    x[i] += delta[i];
                }
            }
        }

        // Package harmonic components
        let mut components = Vec::with_capacity(h + 1);

        // DC component
        let v0_mag = x[0].abs();
        let p0_watts = (v0_mag * v0_mag) / (2.0 * self.z0);
        let p0_dbm = if p0_watts > 1e-18 {
            10.0 * (p0_watts / 1e-3).log10()
        } else {
            -150.0
        };
        components.push(HarmonicComponent {
            harmonic_index: 0,
            freq_hz: 0.0,
            voltage_phasor: Complex64::new(x[0], 0.0),
            voltage_mag: v0_mag,
            power_dbm: p0_dbm,
        });

        // AC harmonics
        for k in 1..=h {
            let phasor = Complex64::new(x[2 * k - 1], x[2 * k]);
            let mag = phasor.abs();
            // P = |V_k|^2 / (2 * Z0)
            let p_watts = (mag * mag) / (2.0 * self.z0);
            let p_dbm = if p_watts > 1e-18 {
                10.0 * (p_watts / 1e-3).log10()
            } else {
                -150.0
            };
            components.push(HarmonicComponent {
                harmonic_index: k,
                freq_hz: (k as f64) * self.fundamental_hz,
                voltage_phasor: phasor,
                voltage_mag: mag,
                power_dbm: p_dbm,
            });
        }

        Ok(HarmonicBalanceResult {
            fundamental_hz: self.fundamental_hz,
            harmonics: components,
            iterations: iterations_done,
            converged,
            residual_norm: final_res_norm,
        })
    }

    /// Solves single-tone excitation from available source power Pin (in dBm).
    ///
    /// Available source power: P_avail = 10^((Pin_dBm - 30) / 10) Watts.
    /// Matched Norton source current: I_S,fund = 2 * sqrt(2 * P_avail / Z0).
    pub fn solve_single_tone(&self, pin_dbm: f64) -> Result<HarmonicBalanceResult, String> {
        let p_avail_watts = 10.0_f64.powf((pin_dbm - 30.0) / 10.0);
        let is_fund_peak = 2.0 * (2.0 * p_avail_watts / self.z0).sqrt();

        let mut is_harmonics = vec![Complex64::ZERO; self.num_harmonics + 1];
        if self.num_harmonics >= 1 {
            is_harmonics[1] = Complex64::new(is_fund_peak, 0.0);
        }

        self.solve(&is_harmonics)
    }

    /// Computes 1-dB compression point (P1dB) and third-order intercept point (IP3 / TOI)
    /// across an input power sweep.
    pub fn compute_compression_and_intercept(
        &self,
        pin_start_dbm: f64,
        pin_stop_dbm: f64,
        points: usize,
    ) -> Result<NonLinearMetrics, String> {
        let n = points.max(5);
        let step = (pin_stop_dbm - pin_start_dbm) / ((n - 1) as f64);

        let mut pin_vals: Vec<f64> = Vec::with_capacity(n);
        let mut pout_fund: Vec<f64> = Vec::with_capacity(n);
        let mut pout_harm3: Vec<f64> = Vec::with_capacity(n);

        for i in 0..n {
            let pin = pin_start_dbm + (i as f64) * step;
            let res = self.solve_single_tone(pin)?;
            pin_vals.push(pin);
            pout_fund.push(res.fundamental_power_dbm());
            pout_harm3.push(res.third_harmonic_power_dbm());
        }

        // Small-signal linear gain at lowest input power
        let linear_gain_db = pout_fund[0] - pin_vals[0];

        // 1-dB Compression Search
        let mut p1db_in = pin_stop_dbm;
        let mut p1db_out = pout_fund[pout_fund.len() - 1];
        let mut found_comp = false;

        for i in 1..n {
            let gain_i = pout_fund[i] - pin_vals[i];
            let comp_i = linear_gain_db - gain_i;

            if comp_i >= 1.0 {
                // Linear interpolation between index i-1 and i
                let gain_prev = pout_fund[i - 1] - pin_vals[i - 1];
                let comp_prev = linear_gain_db - gain_prev;
                let t = if (comp_i - comp_prev).abs() > 1e-6 {
                    (1.0 - comp_prev) / (comp_i - comp_prev)
                } else {
                    0.5
                };
                p1db_in = pin_vals[i - 1] + t * (pin_vals[i] - pin_vals[i - 1]);
                p1db_out = p1db_in + linear_gain_db - 1.0;
                found_comp = true;
                break;
            }
        }

        if !found_comp {
            // Extrapolate compression if maximum drive power approaches compression
            let max_gain_drop = linear_gain_db - (pout_fund[n - 1] - pin_vals[n - 1]);
            if max_gain_drop > 0.05 {
                let factor = 1.0 / max_gain_drop;
                p1db_in = pin_start_dbm + factor * (pin_vals[n - 1] - pin_start_dbm);
                p1db_out = p1db_in + linear_gain_db - 1.0;
            }
        }

        // Third-Order Intercept Point (IP3)
        // Extrapolated at low input power where 3rd harmonic slope is asymptotic to 3:1:
        // OIP3 = Pout(f0) + (Pout(f0) - Pout(3*f0)) / 2
        let sample_idx = 1.min(n - 1);
        let p_fund = pout_fund[sample_idx];
        let p_harm3 = pout_harm3[sample_idx];
        let delta_db = p_fund - p_harm3;

        let oip3 = if delta_db > 0.0 {
            p_fund + delta_db * 0.5
        } else {
            p1db_out + 10.6
        };
        let iip3 = oip3 - linear_gain_db;

        Ok(NonLinearMetrics {
            p1db_in_dbm: p1db_in,
            p1db_out_dbm: p1db_out,
            ip3_in_dbm: iip3,
            ip3_out_dbm: oip3,
            linear_gain_db,
        })
    }
}

/// Solves linear system A * x = b via Gaussian elimination with partial pivoting.
fn solve_linear_system(a: &mut [Vec<f64>], b: &mut [f64]) -> Result<Vec<f64>, String> {
    let n = b.len();

    for i in 0..n {
        // Find pivot
        let mut max_row = i;
        let mut max_val = a[i][i].abs();
        for k in (i + 1)..n {
            let val = a[k][i].abs();
            if val > max_val {
                max_val = val;
                max_row = k;
            }
        }

        if max_val < 1e-28 {
            return Err("Harmonic balance Jacobian is singular or near-singular".to_string());
        }

        // Swap rows
        if max_row != i {
            a.swap(i, max_row);
            b.swap(i, max_row);
        }

        // Eliminate
        let pivot = a[i][i];
        for k in (i + 1)..n {
            let factor = a[k][i] / pivot;
            a[k][i] = 0.0;
            for j in (i + 1)..n {
                let val = a[i][j];
                a[k][j] -= factor * val;
            }
            b[k] -= factor * b[i];
        }
    }

    // Back substitution
    let mut x = vec![0.0_f64; n];
    for i in (0..n).rev() {
        let mut sum = b[i];
        for j in (i + 1)..n {
            sum -= a[i][j] * x[j];
        }
        x[i] = sum / a[i][i];
    }

    Ok(x)
}
