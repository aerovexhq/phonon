#![deny(unsafe_code)]

//! High-frequency 2-port RF network analysis, S-parameter conversions, frequency sweeps,
//! Touchstone S2P export, and RF stability and gain metrics.

use std::f64::consts::PI;

/// Complex number helper for RF matrix mathematics.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Complex64 {
    pub re: f64,
    pub im: f64,
}

impl Complex64 {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    #[inline]
    pub fn from_real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    #[inline]
    pub fn from_polar(r: f64, theta_rad: f64) -> Self {
        Self {
            re: r * theta_rad.cos(),
            im: r * theta_rad.sin(),
        }
    }

    #[inline]
    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    #[inline]
    pub fn norm(&self) -> f64 {
        self.abs()
    }

    #[inline]
    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    pub fn abs(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    #[inline]
    pub fn arg(&self) -> f64 {
        self.im.atan2(self.re)
    }

    #[inline]
    pub fn exp(&self) -> Self {
        let r = self.re.exp();
        Self {
            re: r * self.im.cos(),
            im: r * self.im.sin(),
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
    pub fn inv(&self) -> Self {
        Self::ONE.div(*self)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn add(self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    #[inline]
    #[allow(clippy::should_implement_trait)]
    pub fn div(self, other: Self) -> Self {
        let den = other.norm_sq().max(1e-30);
        Self::new(
            (self.re * other.re + self.im * other.im) / den,
            (self.im * other.re - self.re * other.im) / den,
        )
    }
}

impl std::ops::Add for Complex64 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl std::ops::Sub for Complex64 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.sub(rhs)
    }
}

impl std::ops::Mul for Complex64 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.mul(rhs)
    }
}

impl std::ops::Div for Complex64 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: Self) -> Self::Output {
        self.div(rhs)
    }
}

impl std::ops::Neg for Complex64 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        Self::new(-self.re, -self.im)
    }
}

impl std::ops::Mul<f64> for Complex64 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        self.scale(rhs)
    }
}

impl std::ops::Div<f64> for Complex64 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        let inv = 1.0 / rhs;
        self.scale(inv)
    }
}

/// Frequency sweep type: Linear or Logarithmic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SweepType {
    Linear,
    Logarithmic,
}

/// Frequency sweep generator supporting linear and logarithmic spans from 100 kHz to 100 GHz.
#[derive(Debug, Clone, PartialEq)]
pub struct FrequencySweep {
    pub start_hz: f64,
    pub stop_hz: f64,
    pub points: usize,
    pub sweep_type: SweepType,
}

impl Default for FrequencySweep {
    fn default() -> Self {
        Self {
            start_hz: 100_000.0,          // 100 kHz
            stop_hz: 100_000_000_000.0,   // 100 GHz
            points: 101,
            sweep_type: SweepType::Logarithmic,
        }
    }
}

impl FrequencySweep {
    /// Creates a new frequency sweep configuration.
    pub fn new(start_hz: f64, stop_hz: f64, points: usize, sweep_type: SweepType) -> Self {
        Self {
            start_hz: start_hz.max(1.0),
            stop_hz: stop_hz.max(start_hz),
            points: points.max(1),
            sweep_type,
        }
    }

    /// Constructs a linear frequency sweep.
    pub fn linear(start_hz: f64, stop_hz: f64, points: usize) -> Self {
        Self::new(start_hz, stop_hz, points, SweepType::Linear)
    }

    /// Constructs a logarithmic frequency sweep.
    pub fn logarithmic(start_hz: f64, stop_hz: f64, points: usize) -> Self {
        Self::new(start_hz, stop_hz, points, SweepType::Logarithmic)
    }

    /// Generates frequency points in Hertz.
    pub fn points(&self) -> Vec<f64> {
        let n = self.points.max(1);
        if n == 1 {
            return vec![self.start_hz];
        }

        match self.sweep_type {
            SweepType::Linear => {
                let step = (self.stop_hz - self.start_hz) / ((n - 1) as f64);
                (0..n).map(|i| self.start_hz + (i as f64) * step).collect()
            }
            SweepType::Logarithmic => {
                let f_start = self.start_hz.max(1e-6);
                let f_stop = self.stop_hz.max(f_start);
                let log_start = f_start.log10();
                let log_stop = f_stop.log10();
                let step = (log_stop - log_start) / ((n - 1) as f64);
                (0..n)
                    .map(|i| 10.0_f64.powf(log_start + (i as f64) * step))
                    .collect()
            }
        }
    }

    /// Alias for `points()`.
    pub fn frequencies(&self) -> Vec<f64> {
        self.points()
    }
}

/// 2-Port Scattering Parameters (S-parameters) evaluated at frequency f.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoPortSParameters {
    /// Analysis frequency in Hertz (f).
    pub freq_hz: f64,
    /// Reference port characteristic impedance in Ohms (Z0, standard 50 Ohm).
    pub z0_ref: f64,
    /// Input reflection coefficient S11.
    pub s11: Complex64,
    /// Reverse transmission coefficient S12.
    pub s12: Complex64,
    /// Forward transmission coefficient S21.
    pub s21: Complex64,
    /// Output reflection coefficient S22.
    pub s22: Complex64,
}

impl TwoPortSParameters {
    pub fn new(
        freq_hz: f64,
        z0_ref: f64,
        s11: Complex64,
        s12: Complex64,
        s21: Complex64,
        s22: Complex64,
    ) -> Self {
        Self {
            freq_hz,
            z0_ref,
            s11,
            s12,
            s21,
            s22,
        }
    }

    /// Input reflection magnitude |S11|.
    #[inline]
    pub fn s11_mag(&self) -> f64 {
        self.s11.abs()
    }

    /// Forward transmission magnitude |S21|.
    #[inline]
    pub fn s21_mag(&self) -> f64 {
        self.s21.abs()
    }

    /// Reverse transmission magnitude |S12|.
    #[inline]
    pub fn s12_mag(&self) -> f64 {
        self.s12.abs()
    }

    /// Output reflection magnitude |S22|.
    #[inline]
    pub fn s22_mag(&self) -> f64 {
        self.s22.abs()
    }

    /// Input reflection phase in degrees.
    #[inline]
    pub fn s11_phase_deg(&self) -> f64 {
        self.s11.arg().to_degrees()
    }

    /// Forward transmission phase in degrees.
    #[inline]
    pub fn s21_phase_deg(&self) -> f64 {
        self.s21.arg().to_degrees()
    }

    /// Reverse transmission phase in degrees.
    #[inline]
    pub fn s12_phase_deg(&self) -> f64 {
        self.s12.arg().to_degrees()
    }

    /// Output reflection phase in degrees.
    #[inline]
    pub fn s22_phase_deg(&self) -> f64 {
        self.s22.arg().to_degrees()
    }

    /// Input Return Loss in decibels (dB):
    /// RL_dB = -20 log10 |S11|
    #[inline]
    pub fn return_loss_db(&self) -> f64 {
        let mag = self.s11.abs().max(1e-12);
        -20.0 * mag.log10()
    }

    /// Forward Insertion Loss in decibels (dB):
    /// IL_dB = -20 log10 |S21|
    #[inline]
    pub fn insertion_loss_db(&self) -> f64 {
        let mag = self.s21.abs().max(1e-12);
        -20.0 * mag.log10()
    }

    /// Voltage Standing Wave Ratio (VSWR) at Port 1:
    /// VSWR = (1 + |S11|) / (1 - |S11|)
    #[inline]
    pub fn vswr(&self) -> f64 {
        let gamma = self.s11.abs().min(0.999_999);
        (1.0 + gamma) / (1.0 - gamma)
    }

    /// Voltage Standing Wave Ratio (VSWR) at Port 2:
    /// VSWR = (1 + |S22|) / (1 - |S22|)
    #[inline]
    pub fn vswr_port2(&self) -> f64 {
        let gamma = self.s22.abs().min(0.999_999);
        (1.0 + gamma) / (1.0 - gamma)
    }

    /// Matrix determinant: Delta = S11 * S22 - S12 * S21.
    #[inline]
    pub fn delta(&self) -> Complex64 {
        self.s11.mul(self.s22).sub(self.s12.mul(self.s21))
    }

    /// Rollett unconditional stability factor K:
    /// K = (1 - |S11|^2 - |S22|^2 + |Delta|^2) / (2 |S12 * S21|)
    pub fn stability_factor_k(&self) -> f64 {
        let s11_sq = self.s11.norm_sq();
        let s22_sq = self.s22.norm_sq();
        let delta_sq = self.delta().norm_sq();
        let denom = 2.0 * (self.s12.mul(self.s21)).abs();
        if denom < 1e-18 {
            f64::INFINITY
        } else {
            (1.0 - s11_sq - s22_sq + delta_sq) / denom
        }
    }

    /// Edwards-Sinsky mu1 stability factor:
    /// mu1 = (1 - |S11|^2) / (|S22 - Delta * S11*| + |S12 * S21|)
    /// A network is unconditionally stable if and only if mu1 > 1.
    pub fn mu1(&self) -> f64 {
        let s11_sq = self.s11.norm_sq();
        let num = 1.0 - s11_sq;
        let delta = self.delta();
        let s11_conj = self.s11.conj();
        let term1 = (self.s22 - delta * s11_conj).abs();
        let term2 = (self.s12 * self.s21).abs();
        let denom = term1 + term2;
        if denom < 1e-18 {
            f64::INFINITY
        } else {
            num / denom
        }
    }

    /// Edwards-Sinsky mu1 stability factor (alias).
    #[inline]
    pub fn mu1_factor(&self) -> f64 {
        self.mu1()
    }

    /// Edwards-Sinsky mu2 stability factor:
    /// mu2 = (1 - |S22|^2) / (|S11 - Delta * S22*| + |S12 * S21|)
    pub fn mu2(&self) -> f64 {
        let s22_sq = self.s22.norm_sq();
        let num = 1.0 - s22_sq;
        let delta = self.delta();
        let s22_conj = self.s22.conj();
        let term1 = (self.s11 - delta * s22_conj).abs();
        let term2 = (self.s12 * self.s21).abs();
        let denom = term1 + term2;
        if denom < 1e-18 {
            f64::INFINITY
        } else {
            num / denom
        }
    }

    /// Unconditional stability test:
    /// True if K > 1 and |Delta| < 1 (equivalent to mu1 > 1).
    pub fn is_unconditionally_stable(&self) -> bool {
        self.stability_factor_k() > 1.0 && self.delta().abs() < 1.0
    }

    /// Maximum Stable Gain (MSG):
    /// MSG = |S21 / S12|
    pub fn maximum_stable_gain(&self) -> f64 {
        let s12_mag = self.s12.abs();
        if s12_mag < 1e-18 {
            f64::INFINITY
        } else {
            self.s21.abs() / s12_mag
        }
    }

    /// Maximum Stable Gain in decibels (dB):
    /// MSG_dB = 10 log10(MSG)
    pub fn msg_db(&self) -> f64 {
        let msg = self.maximum_stable_gain();
        if msg.is_infinite() || msg <= 0.0 {
            0.0
        } else {
            10.0 * msg.log10()
        }
    }

    /// Maximum Available Gain (MAG):
    /// MAG = |S21 / S12| * (K - sqrt(K^2 - 1))
    /// Valid only when K > 1 and |Delta| < 1.
    pub fn maximum_available_gain(&self) -> Option<f64> {
        let k = self.stability_factor_k();
        let delta_mag = self.delta().abs();
        if k > 1.0 && delta_mag < 1.0 {
            let msg = self.maximum_stable_gain();
            let factor = k - (k * k - 1.0).sqrt();
            Some(msg * factor)
        } else {
            None
        }
    }

    /// Maximum Available Gain in decibels (dB).
    pub fn mag_db(&self) -> Option<f64> {
        self.maximum_available_gain().map(|mag| 10.0 * mag.log10())
    }

    /// Converts 2-port Impedance matrix [Z] to Scattering matrix [S] with reference Z0:
    /// S = (Z - Z0 * I)(Z + Z0 * I)^(-1)
    pub fn from_z_matrix(z: [[Complex64; 2]; 2], z0: f64, freq_hz: f64) -> Self {
        let z0_c = Complex64::new(z0, 0.0);

        let term1 = z[0][0].add(z0_c);
        let term2 = z[1][1].add(z0_c);
        let delta_z = term1.mul(term2).sub(z[0][1].mul(z[1][0]));

        let s11_num = z[0][0].sub(z0_c).mul(term2).sub(z[0][1].mul(z[1][0]));
        let s11 = s11_num.div(delta_z);

        let two_z0 = Complex64::new(2.0 * z0, 0.0);
        let s12 = two_z0.mul(z[0][1]).div(delta_z);
        let s21 = two_z0.mul(z[1][0]).div(delta_z);

        let s22_num = term1.mul(z[1][1].sub(z0_c)).sub(z[0][1].mul(z[1][0]));
        let s22 = s22_num.div(delta_z);

        Self::new(freq_hz, z0, s11, s12, s21, s22)
    }

    /// Converts 2-port Admittance matrix [Y] to Scattering matrix [S] with reference Z0.
    pub fn from_y_matrix(y: [[Complex64; 2]; 2], z0: f64, freq_hz: f64) -> Self {
        let y0 = 1.0 / z0;
        let y0_c = Complex64::new(y0, 0.0);

        let term1 = y[0][0].add(y0_c);
        let term2 = y[1][1].add(y0_c);
        let delta_y = term1.mul(term2).sub(y[0][1].mul(y[1][0]));

        let s11_num = y0_c.sub(y[0][0]).mul(term2).add(y[0][1].mul(y[1][0]));
        let s11 = s11_num.div(delta_y);

        let minus_two_y0 = Complex64::new(-2.0 * y0, 0.0);
        let s12 = minus_two_y0.mul(y[0][1]).div(delta_y);
        let s21 = minus_two_y0.mul(y[1][0]).div(delta_y);

        let s22_num = term1.mul(y0_c.sub(y[1][1])).add(y[0][1].mul(y[1][0]));
        let s22 = s22_num.div(delta_y);

        Self::new(freq_hz, z0, s11, s12, s21, s22)
    }

    /// Evaluates exact analytical S-parameters for an ideal lossless transmission line of impedance Zc
    /// and electrical delay tau, terminated into reference impedance Z0.
    pub fn for_transmission_line(zc: f64, tau: f64, z0: f64, freq_hz: f64) -> Self {
        let theta = 2.0 * PI * freq_hz * tau;
        let cos_th = theta.cos();
        let sin_th = theta.sin();

        let re_den = 2.0 * z0 * zc * cos_th;
        let im_den = (zc * zc + z0 * z0) * sin_th;
        let den = Complex64::new(re_den, im_den);

        let s11_num = Complex64::new(0.0, (zc * zc - z0 * z0) * sin_th);
        let s11 = s11_num.div(den);

        let s21_num = Complex64::new(2.0 * z0 * zc, 0.0);
        let s21 = s21_num.div(den);

        Self::new(freq_hz, z0, s11, s21, s21, s11)
    }

    /// Evaluates exact S-parameters for a symmetric Tee-attenuator network with series resistance R_s
    /// and shunt resistance R_p.
    pub fn for_tee_attenuator(r_series: f64, r_shunt: f64, z0: f64, freq_hz: f64) -> Self {
        let z11 = Complex64::new(r_series + r_shunt, 0.0);
        let z12 = Complex64::new(r_shunt, 0.0);
        let z = [[z11, z12], [z12, z11]];
        Self::from_z_matrix(z, z0, freq_hz)
    }

    /// Evaluates exact S-parameters for a series impedance Z_series connected between Port 1 and Port 2.
    pub fn for_series_impedance(z_series: Complex64, z0: f64, freq_hz: f64) -> Self {
        let two_z0 = Complex64::new(2.0 * z0, 0.0);
        let den = z_series + two_z0;
        let s11 = z_series / den;
        let s21 = two_z0 / den;
        Self::new(freq_hz, z0, s11, s21, s21, s11)
    }

    /// Evaluates exact S-parameters for a shunt admittance Y_shunt connected between the signal line and ground.
    pub fn for_shunt_admittance(y_shunt: Complex64, z0: f64, freq_hz: f64) -> Self {
        let two = Complex64::new(2.0, 0.0);
        let y_z0 = y_shunt * z0;
        let den = two + y_z0;
        let s11 = (-y_z0) / den;
        let s21 = two / den;
        Self::new(freq_hz, z0, s11, s21, s21, s11)
    }

    /// Converts 2-port Scattering matrix [S] back to Impedance matrix [Z]:
    /// Z = Z0 * (I + S)(I - S)^(-1)
    pub fn to_z_matrix(&self) -> Result<[[Complex64; 2]; 2], &'static str> {
        let one = Complex64::ONE;
        let delta_s = one
            .sub(self.s11)
            .mul(one.sub(self.s22))
            .sub(self.s12.mul(self.s21));
        if delta_s.norm_sq() < 1e-30 {
            return Err("Singular S-matrix: cannot invert (I - S)");
        }

        let z0_c = Complex64::from_real(self.z0_ref);
        let two_z0 = Complex64::from_real(2.0 * self.z0_ref);

        let z11_num = one
            .add(self.s11)
            .mul(one.sub(self.s22))
            .add(self.s12.mul(self.s21));
        let z11 = z0_c.mul(z11_num).div(delta_s);

        let z12 = two_z0.mul(self.s12).div(delta_s);
        let z21 = two_z0.mul(self.s21).div(delta_s);

        let z22_num = one
            .sub(self.s11)
            .mul(one.add(self.s22))
            .add(self.s12.mul(self.s21));
        let z22 = z0_c.mul(z22_num).div(delta_s);

        Ok([[z11, z12], [z21, z22]])
    }

    /// Touchstone S2P single-point string generator.
    pub fn to_touchstone_s2p(&self) -> String {
        format_touchstone_s2p(&[*self], self.z0_ref)
    }
}

/// Helper function to format a sequence of 2-port S-parameters into standard Touchstone S2P format.
pub fn format_touchstone_s2p(records: &[TwoPortSParameters], z0: f64) -> String {
    let mut out = String::with_capacity(records.len() * 128 + 256);
    out.push_str("! Touchstone 2-port S-parameters file generated by Phonon Studio RF Engine\n");
    out.push_str("! Format: Frequency (Hz), S11 (Re Im), S21 (Re Im), S12 (Re Im), S22 (Re Im)\n");
    out.push_str(&format!("# HZ S RI R {:.6}\n", z0));

    for r in records {
        out.push_str(&format!(
            "{:.6e} {:+.8e} {:+.8e} {:+.8e} {:+.8e} {:+.8e} {:+.8e} {:+.8e} {:+.8e}\n",
            r.freq_hz,
            r.s11.re,
            r.s11.im,
            r.s21.re,
            r.s21.im,
            r.s12.re,
            r.s12.im,
            r.s22.re,
            r.s22.im
        ));
    }
    out
}

/// Result of a multi-frequency 2-port S-parameter AC sweep.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiPortSSweepResult {
    pub z0: f64,
    pub sweep: FrequencySweep,
    pub frequencies: Vec<f64>,
    pub s_parameters: Vec<TwoPortSParameters>,
}

impl MultiPortSSweepResult {
    /// Number of sweep points.
    pub fn len(&self) -> usize {
        self.s_parameters.len()
    }

    /// Checks if sweep result is empty.
    pub fn is_empty(&self) -> bool {
        self.s_parameters.is_empty()
    }

    /// Generates Touchstone S2P representation for all frequency points.
    pub fn to_touchstone_s2p(&self) -> String {
        format_touchstone_s2p(&self.s_parameters, self.z0)
    }

    /// Extracts Rollett stability factors K across the sweep.
    pub fn k_factors(&self) -> Vec<f64> {
        self.s_parameters.iter().map(|s| s.stability_factor_k()).collect()
    }

    /// Extracts Edwards-Sinsky mu1 factors across the sweep.
    pub fn mu1_factors(&self) -> Vec<f64> {
        self.s_parameters.iter().map(|s| s.mu1()).collect()
    }

    /// Extracts determinants Delta across the sweep.
    pub fn delta_factors(&self) -> Vec<Complex64> {
        self.s_parameters.iter().map(|s| s.delta()).collect()
    }

    /// Extracts Maximum Available Gain in dB across the sweep.
    pub fn mag_db(&self) -> Vec<Option<f64>> {
        self.s_parameters.iter().map(|s| s.mag_db()).collect()
    }

    /// Extracts Maximum Stable Gain in dB across the sweep.
    pub fn msg_db(&self) -> Vec<f64> {
        self.s_parameters.iter().map(|s| s.msg_db()).collect()
    }

    /// Extracts Port 1 VSWR across the sweep.
    pub fn vswr_port1(&self) -> Vec<f64> {
        self.s_parameters.iter().map(|s| s.vswr()).collect()
    }

    /// Extracts Port 2 VSWR across the sweep.
    pub fn vswr_port2(&self) -> Vec<f64> {
        self.s_parameters.iter().map(|s| s.vswr_port2()).collect()
    }

    /// Extracts Input Return Loss in dB across the sweep.
    pub fn return_loss_db(&self) -> Vec<f64> {
        self.s_parameters.iter().map(|s| s.return_loss_db()).collect()
    }

    /// Extracts Forward Insertion Loss in dB across the sweep.
    pub fn insertion_loss_db(&self) -> Vec<f64> {
        self.s_parameters.iter().map(|s| s.insertion_loss_db()).collect()
    }
}

/// Multi-port linear AC solver computing S11, S21, S12, S22 over frequency with reference impedance Z0.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiPortSSolver {
    pub z0: f64,
    pub sweep: FrequencySweep,
}

impl Default for MultiPortSSolver {
    fn default() -> Self {
        Self {
            z0: 50.0,
            sweep: FrequencySweep::default(),
        }
    }
}

impl MultiPortSSolver {
    /// Creates a new multi-port S-parameter solver.
    pub fn new(z0: f64, sweep: FrequencySweep) -> Self {
        Self {
            z0: if z0 > 0.0 { z0 } else { 50.0 },
            sweep,
        }
    }

    /// Sets reference impedance Z0.
    pub fn with_z0(mut self, z0: f64) -> Self {
        if z0 > 0.0 {
            self.z0 = z0;
        }
        self
    }

    /// Sets frequency sweep configuration.
    pub fn with_sweep(mut self, sweep: FrequencySweep) -> Self {
        self.sweep = sweep;
        self
    }

    /// Solves 2-port S-parameters from an impedance matrix function Z(f).
    pub fn solve_z_matrix<F>(&self, z_fn: F) -> MultiPortSSweepResult
    where
        F: Fn(f64) -> [[Complex64; 2]; 2],
    {
        let freqs = self.sweep.points();
        let s_params: Vec<TwoPortSParameters> = freqs
            .iter()
            .map(|&f| {
                let z = z_fn(f);
                TwoPortSParameters::from_z_matrix(z, self.z0, f)
            })
            .collect();

        MultiPortSSweepResult {
            z0: self.z0,
            sweep: self.sweep.clone(),
            frequencies: freqs,
            s_parameters: s_params,
        }
    }

    /// Solves 2-port S-parameters from an admittance matrix function Y(f).
    pub fn solve_y_matrix<F>(&self, y_fn: F) -> MultiPortSSweepResult
    where
        F: Fn(f64) -> [[Complex64; 2]; 2],
    {
        let freqs = self.sweep.points();
        let s_params: Vec<TwoPortSParameters> = freqs
            .iter()
            .map(|&f| {
                let y = y_fn(f);
                TwoPortSParameters::from_y_matrix(y, self.z0, f)
            })
            .collect();

        MultiPortSSweepResult {
            z0: self.z0,
            sweep: self.sweep.clone(),
            frequencies: freqs,
            s_parameters: s_params,
        }
    }

    /// Solves 2-port S-parameters from a custom evaluation function (f, z0) -> TwoPortSParameters.
    pub fn solve_custom<F>(&self, eval_fn: F) -> MultiPortSSweepResult
    where
        F: Fn(f64, f64) -> TwoPortSParameters,
    {
        let freqs = self.sweep.points();
        let s_params: Vec<TwoPortSParameters> = freqs
            .iter()
            .map(|&f| eval_fn(f, self.z0))
            .collect();

        MultiPortSSweepResult {
            z0: self.z0,
            sweep: self.sweep.clone(),
            frequencies: freqs,
            s_parameters: s_params,
        }
    }

    /// Solves a transmission line network over the sweep.
    pub fn solve_transmission_line(&self, zc: f64, tau: f64) -> MultiPortSSweepResult {
        self.solve_custom(|f, z0| TwoPortSParameters::for_transmission_line(zc, tau, z0, f))
    }

    /// Solves a resistive symmetric Tee attenuator over the sweep.
    pub fn solve_tee_attenuator(&self, r_series: f64, r_shunt: f64) -> MultiPortSSweepResult {
        self.solve_custom(|f, z0| TwoPortSParameters::for_tee_attenuator(r_series, r_shunt, z0, f))
    }

    /// Solves a series RLC resonator network connected between Port 1 and Port 2.
    /// Z(omega) = R + j * omega * L + 1 / (j * omega * C)
    pub fn solve_series_rlc(&self, r: f64, l: f64, c: f64) -> MultiPortSSweepResult {
        self.solve_custom(|f, z0| {
            let omega = 2.0 * PI * f;
            let im_l = omega * l;
            let im_c = if c > 0.0 && omega > 0.0 { -1.0 / (omega * c) } else { 0.0 };
            let z_series = Complex64::new(r, im_l + im_c);
            TwoPortSParameters::for_series_impedance(z_series, z0, f)
        })
    }

    /// Solves a parallel RLC resonator network connected in shunt to ground.
    /// Y(omega) = 1 / R + j * omega * C + 1 / (j * omega * L)
    pub fn solve_shunt_rlc(&self, r: f64, l: f64, c: f64) -> MultiPortSSweepResult {
        self.solve_custom(|f, z0| {
            let omega = 2.0 * PI * f;
            let re_g = if r > 0.0 { 1.0 / r } else { 0.0 };
            let im_c = omega * c;
            let im_l = if l > 0.0 && omega > 0.0 { -1.0 / (omega * l) } else { 0.0 };
            let y_shunt = Complex64::new(re_g, im_c + im_l);
            TwoPortSParameters::for_shunt_admittance(y_shunt, z0, f)
        })
    }

    /// Solves a stylized RF amplifier model with small-signal gain, reverse isolation, and cutoff frequency.
    pub fn solve_amplifier(
        &self,
        gain_db: f64,
        reverse_isolation_db: f64,
        s11_re: f64,
        s11_im: f64,
        s22_re: f64,
        s22_im: f64,
        f_cutoff_hz: f64,
    ) -> MultiPortSSweepResult {
        let gain_lin = 10.0_f64.powf(gain_db / 20.0);
        let rev_iso_lin = 10.0_f64.powf(-reverse_isolation_db.abs() / 20.0);

        self.solve_custom(move |f, z0| {
            // Roll-off single-pole filter factor
            let roll = 1.0 / (1.0 + (f / f_cutoff_hz).powi(2)).sqrt();
            let phase = -(f / f_cutoff_hz).atan();
            let tf = Complex64::from_polar(roll, phase);

            let s21 = tf * gain_lin;
            let s12 = tf * rev_iso_lin;
            let s11 = Complex64::new(s11_re, s11_im);
            let s22 = Complex64::new(s22_re, s22_im);

            TwoPortSParameters::new(f, z0, s11, s12, s21, s22)
        })
    }

    /// Formats an external slice of S-parameters into standard Touchstone S2P format.
    pub fn to_touchstone_s2p(&self, s_params: &[TwoPortSParameters]) -> String {
        format_touchstone_s2p(s_params, self.z0)
    }
}
