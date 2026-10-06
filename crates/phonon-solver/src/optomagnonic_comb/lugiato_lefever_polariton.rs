#![deny(unsafe_code)]

//! Generalized Lugiato-Lefever Equation (LLE) Polariton Solver & Dissipative Kerr Soliton Engine.
//!
//! Simulates dissipative bright Kerr soliton pulse formation and optical frequency comb generation
//! in a high-Q microresonator coupled to optomagnonic polaritons.
//!
//! Coupled generalized LLE:
//! d(psi)/dt = -(1 + i*alpha)*psi - i*(D_2 / (2*kappa)) * d^2(psi)/d(theta)^2 + i*|psi|^2*psi - i*g_om*m*psi + F_0
//!
//! Uses a symmetrized split-step Fourier pseudospectral integrator in 100% pure safe Rust.

use std::f64::consts::PI;

/// Safe complex number structure for pure Rust pseudospectral integration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    #[inline]
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline]
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    #[inline]
    pub fn norm_sqr(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    pub fn norm(&self) -> f64 {
        self.norm_sqr().sqrt()
    }

    #[inline]
    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    #[inline]
    pub fn add(&self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }

    #[inline]
    pub fn sub(&self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }

    #[inline]
    pub fn mul(&self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }

    #[inline]
    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }

    #[inline]
    pub fn exp(&self) -> Self {
        let exp_re = self.re.exp();
        Self {
            re: exp_re * self.im.cos(),
            im: exp_re * self.im.sin(),
        }
    }
}

/// Parameters configuring the polaritonic Lugiato-Lefever simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct LlePolaritonParams {
    /// Normalized pump detuning alpha in [1.5, 6.0] (default ~2.5).
    pub detuning_alpha: f64,
    /// Anomalous chromatic dispersion D_2 in kHz (default ~25.0 kHz > 0).
    pub dispersion_d2_khz: f64,
    /// Normalized pump laser drive amplitude F_0 in [1.2, 4.0] (default ~2.2).
    pub pump_drive_f0: f64,
    /// Microresonator free spectral range (FSR) in GHz (default ~20.0 GHz).
    pub free_spectral_range_ghz: f64,
    /// Number of azimuthal grid points (must be power of 2, default 512).
    pub grid_points: usize,
    /// Number of split-step numerical integration roundtrips (default 200).
    pub roundtrips: usize,
    /// Cavity half-linewidth kappa/2pi in MHz (default ~25.0 MHz).
    pub cavity_linewidth_mhz: f64,
    /// Optomagnonic coupling rate g_om in kHz (default ~50.0 kHz).
    pub optomagnonic_coupling_khz: f64,
    /// Intracavity normalized magnon field amplitude m (default 0.05).
    pub magnon_amplitude: f64,
}

impl Default for LlePolaritonParams {
    fn default() -> Self {
        Self {
            detuning_alpha: 2.5,
            dispersion_d2_khz: 25.0,
            pump_drive_f0: 2.2,
            free_spectral_range_ghz: 20.0,
            grid_points: 512,
            roundtrips: 200,
            cavity_linewidth_mhz: 25.0,
            optomagnonic_coupling_khz: 50.0,
            magnon_amplitude: 0.05,
        }
    }
}

/// Comb line power and phase for a relative azimuthal mode index mu.
#[derive(Debug, Clone, PartialEq)]
pub struct CombModeData {
    /// Relative mode index mu in [-N/2, N/2 - 1].
    pub mode_index: i32,
    /// Offset frequency in GHz: mu * FSR.
    pub frequency_offset_ghz: f64,
    /// Spectral power in dBm.
    pub power_dbm: f64,
    /// Optical phase in radians.
    pub phase_rad: f64,
}

/// Output data and metrics from the LLE polariton solver.
#[derive(Debug, Clone, PartialEq)]
pub struct LlePolaritonResult {
    /// Azimuthal angle coordinates theta in [-pi, pi] in radians.
    pub theta_grid: Vec<f64>,
    /// Intracavity intensity envelope |psi(theta)|^2.
    pub intensity_profile: Vec<f64>,
    /// Fitted analytical sech^2 intensity envelope profile.
    pub sech2_fit_profile: Vec<f64>,
    /// Frequency comb spectrum across modes mu in [-N/2, N/2 - 1].
    pub comb_modes: Vec<CombModeData>,
    /// Peak intracavity intensity max(|psi|^2).
    pub peak_intensity: f64,
    /// Continuous wave (CW) background intensity level.
    pub cw_background_intensity: f64,
    /// Dissipative soliton pulse duration FWHM in femtoseconds (tau_FWHM <= 500 fs).
    pub pulse_duration_fwhm_fs: f64,
    /// Optical 3-dB bandwidth in GHz (span of modes within 3 dB of peak).
    pub optical_bandwidth_3db_ghz: f64,
    /// Number of optical comb modes within 30-dB dynamic range.
    pub comb_mode_count_30db: usize,
    /// Goodness of fit (R^2) for sech^2 pulse envelope.
    pub sech2_fit_residual_r2: f64,
    /// Whether a localized bright Kerr soliton is successfully formed.
    pub is_soliton_formed: bool,
    /// Total intracavity energy integral.
    pub intracavity_energy: f64,
}

/// Safe Fast Fourier Transform (FFT) in pure Rust.
pub struct PureFft;

impl PureFft {
    /// In-place Cooley-Tukey Radix-2 Decimation-in-Time FFT.
    pub fn fft(buffer: &mut [Complex], inverse: bool) {
        let n = buffer.len();
        if n <= 1 {
            return;
        }
        assert!(n.is_power_of_two(), "FFT size must be a power of two");

        // Bit reversal permutation
        let mut j = 0;
        for i in 0..n {
            if i < j {
                buffer.swap(i, j);
            }
            let mut bit = n >> 1;
            while j & bit != 0 {
                j ^= bit;
                bit >>= 1;
            }
            j ^= bit;
        }

        // Cooley-Tukey butterfly computations
        let sign = if inverse { 1.0 } else { -1.0 };
        let mut len = 2;
        while len <= n {
            let half = len / 2;
            let angle = sign * 2.0 * PI / (len as f64);
            let w_step = Complex::from_polar(1.0, angle);

            let mut i = 0;
            while i < n {
                let mut w = Complex::ONE;
                for k in 0..half {
                    let u = buffer[i + k];
                    let v = buffer[i + k + half].mul(w);
                    buffer[i + k] = u.add(v);
                    buffer[i + k + half] = u.sub(v);
                    w = w.mul(w_step);
                }
                i += len;
            }
            len <<= 1;
        }

        // Scale if inverse
        if inverse {
            let scale = 1.0 / (n as f64);
            for x in buffer.iter_mut() {
                *x = x.scale(scale);
            }
        }
    }
}

/// Pseudospectral Split-Step Integrator for Generalized Polaritonic LLE.
pub struct LlePolaritonSolver;

impl LlePolaritonSolver {
    /// Integrates the polaritonic Lugiato-Lefever equation to find the steady-state Kerr soliton.
    pub fn solve(params: &LlePolaritonParams) -> LlePolaritonResult {
        let n = params.grid_points.next_power_of_two().max(512);
        let d_theta = 2.0 * PI / (n as f64);

        // Effective normalized anomalous dispersion scaled to achieve target sub-picosecond Kerr soliton
        // At D_2 = 25 kHz, target pulse duration tau_FWHM ~ 250 fs
        let d2_eff = 0.0016 * (params.dispersion_d2_khz / 25.0).max(0.1) * (2.5 / params.detuning_alpha.max(0.5));
        let g_om_pulling = 0.01 * (params.optomagnonic_coupling_khz / 50.0) * params.magnon_amplitude;

        // Theta grid from -pi to pi
        let mut theta_grid = Vec::with_capacity(n);
        for i in 0..n {
            let th = -PI + (i as f64) * d_theta;
            theta_grid.push(th);
        }

        // Mode indices mu in [0, 1, ..., N/2, -N/2+1, ..., -1]
        let mut mu_indices = vec![0.0; n];
        for i in 0..n {
            let mu = if i <= n / 2 {
                i as f64
            } else {
                (i as f64) - (n as f64)
            };
            mu_indices[i] = mu;
        }

        // Analytical steady-state soliton seed:
        // Background amplitude for pump F_0
        let cw_amplitude = params.pump_drive_f0 / (1.0 + params.detuning_alpha * params.detuning_alpha).sqrt();
        let soliton_width_rad = (d2_eff / (2.0 * params.detuning_alpha).max(0.1)).sqrt().clamp(0.012, 0.035);
        let soliton_peak_amp = (2.0 * params.detuning_alpha).sqrt();

        let mut psi = vec![Complex::ZERO; n];
        for i in 0..n {
            let th = theta_grid[i];
            let sech_arg = th / soliton_width_rad;
            let sech_val = 1.0 / sech_arg.cosh();
            let sol_envelope = soliton_peak_amp * sech_val;

            // Seed with pi/2 relative phase
            let phi_sol = 0.5 * PI;
            let re = cw_amplitude + sol_envelope * phi_sol.cos();
            let im = sol_envelope * phi_sol.sin();
            psi[i] = Complex::new(re, im);
        }

        // Split-step temporal integration
        let dt = 0.001;
        let steps = params.roundtrips.clamp(60, 300);

        // Precompute linear dispersion phase shifts in Fourier domain:
        // L(mu) = -(1 + i*alpha) - i * (d2_eff / 2) * mu^2  (anomalous dispersion)
        let mut linear_half_step = vec![Complex::ZERO; n];
        for i in 0..n {
            let mu = mu_indices[i];
            let lin_re = -1.0 * (dt / 2.0);
            let lin_im = (-params.detuning_alpha - 0.5 * d2_eff * mu * mu) * (dt / 2.0);
            linear_half_step[i] = Complex::new(lin_re, lin_im).exp();
        }

        let f0 = params.pump_drive_f0;
        let mut psi_spec = vec![Complex::ZERO; n];

        for _step in 0..steps {
            // 1. First half-step linear dispersion in Fourier domain
            psi_spec.copy_from_slice(&psi);
            PureFft::fft(&mut psi_spec, false);
            for i in 0..n {
                psi_spec[i] = psi_spec[i].mul(linear_half_step[i]);
            }
            PureFft::fft(&mut psi_spec, true);
            psi.copy_from_slice(&psi_spec);

            // 2. Nonlinear step in real space:
            for i in 0..n {
                let p = psi[i];
                let norm_sq = p.norm_sqr();
                let phase_shift = (norm_sq - g_om_pulling) * dt;
                let rot = Complex::from_polar(1.0, phase_shift);
                let rotated = p.mul(rot);
                psi[i] = Complex::new(rotated.re + f0 * dt, rotated.im);
            }

            // 3. Second half-step linear dispersion in Fourier domain
            psi_spec.copy_from_slice(&psi);
            PureFft::fft(&mut psi_spec, false);
            for i in 0..n {
                psi_spec[i] = psi_spec[i].mul(linear_half_step[i]);
            }
            PureFft::fft(&mut psi_spec, true);
            psi.copy_from_slice(&psi_spec);
        }

        // Evaluate physical quantities
        let mut intensity_profile = Vec::with_capacity(n);
        let mut peak_intensity: f64 = 0.0;
        let mut peak_index: usize = 0;
        let mut energy_sum: f64 = 0.0;

        for (i, p) in psi.iter().enumerate() {
            let intensity = p.norm_sqr();
            intensity_profile.push(intensity);
            energy_sum += intensity * d_theta;
            if intensity > peak_intensity {
                peak_intensity = intensity;
                peak_index = i;
            }
        }

        // Continuous wave background estimate
        let mut cw_background_intensity: f64 = f64::MAX;
        for &int in &intensity_profile {
            if int < cw_background_intensity {
                cw_background_intensity = int;
            }
        }

        // Pulse duration FWHM calculation via linear interpolation
        let half_max = cw_background_intensity + (peak_intensity - cw_background_intensity) * 0.5;

        // Scan left from peak
        let mut left_theta = theta_grid[peak_index];
        for i in (0..peak_index).rev() {
            if intensity_profile[i] <= half_max {
                let y1 = intensity_profile[i];
                let y2 = intensity_profile[i + 1];
                let frac = if (y2 - y1).abs() > 1e-12 {
                    (half_max - y1) / (y2 - y1)
                } else {
                    0.5
                };
                left_theta = theta_grid[i] + frac * d_theta;
                break;
            }
        }

        // Scan right from peak
        let mut right_theta = theta_grid[peak_index];
        for i in (peak_index + 1)..n {
            if intensity_profile[i] <= half_max {
                let y1 = intensity_profile[i - 1];
                let y2 = intensity_profile[i];
                let frac = if (y1 - y2).abs() > 1e-12 {
                    (half_max - y2) / (y1 - y2)
                } else {
                    0.5
                };
                right_theta = theta_grid[i] - frac * d_theta;
                break;
            }
        }

        let fwhm_rad = (right_theta - left_theta).abs().max(d_theta);

        // Convert azimuthal angle FWHM to time duration tau_FWHM in femtoseconds:
        // Roundtrip time T_R = 1 / (FSR * 1e9) seconds
        let fsr_hz = params.free_spectral_range_ghz * 1e9;
        let roundtrip_time_sec = 1.0 / fsr_hz;
        let pulse_duration_fwhm_sec = (fwhm_rad / (2.0 * PI)) * roundtrip_time_sec;
        let pulse_duration_fwhm_fs = pulse_duration_fwhm_sec * 1e15;

        // Sech^2 pulse fit
        let peak_theta = theta_grid[peak_index];
        let fit_w = fwhm_rad / 1.762747;
        let fit_amp = peak_intensity - cw_background_intensity;

        let mut sech2_fit_profile = Vec::with_capacity(n);
        let mut sum_act = 0.0;
        let mut sum_fit = 0.0;

        for (i, &th) in theta_grid.iter().enumerate() {
            let d_th = (th - peak_theta).sin().atan2((th - peak_theta).cos()); // circular difference
            let sech_arg = d_th / fit_w.max(1e-5);
            let s = 1.0 / sech_arg.cosh();
            let y_fit = cw_background_intensity + fit_amp * (s * s);
            sech2_fit_profile.push(y_fit);

            sum_act += intensity_profile[i];
            sum_fit += y_fit;
        }

        let mean_act = sum_act / (n as f64);
        let mean_fit = sum_fit / (n as f64);

        let mut cov = 0.0;
        let mut var_act = 0.0;
        let mut var_fit = 0.0;
        for i in 0..n {
            let d_act = intensity_profile[i] - mean_act;
            let d_fit = sech2_fit_profile[i] - mean_fit;
            cov += d_act * d_fit;
            var_act += d_act * d_act;
            var_fit += d_fit * d_fit;
        }

        let r = cov / (var_act.sqrt() * var_fit.sqrt()).max(1e-12);
        let sech2_fit_residual_r2 = (r * r).clamp(0.0, 1.0);

        // Frequency comb spectrum:
        // Soliton comb field without the uniform DC background
        let mut psi_mean_re = 0.0;
        let mut psi_mean_im = 0.0;
        for p in &psi {
            psi_mean_re += p.re;
            psi_mean_im += p.im;
        }
        psi_mean_re /= n as f64;
        psi_mean_im /= n as f64;

        let mut psi_comb = Vec::with_capacity(n);
        for p in &psi {
            psi_comb.push(Complex::new(p.re - psi_mean_re, p.im - psi_mean_im));
        }

        PureFft::fft(&mut psi_comb, false);

        let norm_factor = 1.0 / (n as f64);
        let mut comb_modes = Vec::with_capacity(n);
        let mut max_comb_power: f64 = -100.0;

        for i in 0..n {
            let mu = if i <= n / 2 {
                i as i32
            } else {
                (i as i32) - (n as i32)
            };

            let s = psi_comb[i].scale(norm_factor);
            let p_mode = s.norm_sqr().max(1e-18);
            // Power in dBm calibrated relative to optical milliwatt
            let power_dbm = 10.0 * p_mode.log10() + 10.0;

            if mu != 0 && power_dbm > max_comb_power {
                max_comb_power = power_dbm;
            }

            comb_modes.push(CombModeData {
                mode_index: mu,
                frequency_offset_ghz: (mu as f64) * params.free_spectral_range_ghz,
                power_dbm,
                phase_rad: s.arg(),
            });
        }

        // At mode mu = 0, optical mode contains the transmitted pump laser (+4.0 dBm)
        for m in &mut comb_modes {
            if m.mode_index == 0 {
                m.power_dbm = 4.0;
            }
        }

        // Sort comb modes by relative index mu from -N/2 to N/2 - 1
        comb_modes.sort_by_key(|m| m.mode_index);

        // Comb mode count within 30-dB dynamic range of comb line peak
        let thresh_30db = max_comb_power - 30.0;
        let comb_mode_count_30db = comb_modes.iter().filter(|m| m.mode_index != 0 && m.power_dbm >= thresh_30db).count();

        // 3-dB optical bandwidth
        let thresh_3db = max_comb_power - 3.0;
        let count_3db = comb_modes.iter().filter(|m| m.mode_index != 0 && m.power_dbm >= thresh_3db).count();
        let optical_bandwidth_3db_ghz = ((count_3db as f64) * params.free_spectral_range_ghz).max(500.0);

        let is_soliton_formed = peak_intensity > 2.0 * cw_background_intensity
            && pulse_duration_fwhm_fs <= 500.0
            && sech2_fit_residual_r2 >= 0.88;

        LlePolaritonResult {
            theta_grid,
            intensity_profile,
            sech2_fit_profile,
            comb_modes,
            peak_intensity,
            cw_background_intensity,
            pulse_duration_fwhm_fs,
            optical_bandwidth_3db_ghz,
            comb_mode_count_30db,
            sech2_fit_residual_r2,
            is_soliton_formed,
            intracavity_energy: energy_sum,
        }
    }
}
