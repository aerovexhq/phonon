#![deny(unsafe_code)]

//! Bidirectional Quantum Microwave-to-Acoustic Interface & S-Matrix Engine.
//!
//! Evaluates the 2-port scattering matrix [S(omega)] for coherent transduction
//! between superconducting microwave circuits (inductive/CPW) and surface acoustic
//! waves (SAWs) mediated by the phonon-magnon polariton hybrid mode.

use std::f64::consts::PI;

pub const HBAR: f64 = 1.054_571_817e-34; // J * s
pub const K_BOLTZMANN: f64 = 1.380_649e-23; // J / K

/// 2-port complex S-parameters evaluated at a specific frequency.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SParameterSample {
    /// Analysis frequency in Hertz.
    pub frequency_hz: f64,
    /// Return loss |S_11|^2 (linear).
    pub s11_power: f64,
    /// Transduction efficiency |S_21|^2 (linear).
    pub s21_power: f64,
    /// Return loss |S_22|^2 (linear).
    pub s22_power: f64,
    /// Transmission |S_21| in decibels (dB).
    pub s21_db: f64,
    /// Return loss |S_11| in decibels (dB).
    pub s11_db: f64,
}

/// Parameters for the microwave-to-acoustic quantum transducer.
#[derive(Debug, Clone)]
pub struct TransducerCouplingParams {
    /// Resonant frequency omega_0 in rad / s (e.g. 2 * pi * 3.5 GHz).
    pub resonance_freq_rad_s: f64,
    /// Magnetoelastic polariton coupling rate g_pm in rad / s.
    pub coupling_g_rad_s: f64,
    /// Total acoustic phonon decay rate kappa_p in rad / s.
    pub total_phonon_decay_rad_s: f64,
    /// External acoustic port coupling rate kappa_ac_ext in rad / s.
    pub external_phonon_decay_rad_s: f64,
    /// Total magnon decay rate kappa_m in rad / s.
    pub total_magnon_decay_rad_s: f64,
    /// External microwave port coupling rate kappa_mw_ext in rad / s.
    pub external_magnon_decay_rad_s: f64,
    /// Cryogenic physical temperature in Kelvin (e.g. 20 mK = 0.020 K).
    pub temperature_k: f64,
}

impl Default for TransducerCouplingParams {
    fn default() -> Self {
        let f0 = 3.5e9; // 3.5 GHz
        let omega0 = 2.0 * PI * f0;
        let g = 2.0 * PI * 35.0e6; // 35 MHz coupling

        // Critically matched high-efficiency parameters:
        let kappa_p = 2.0 * PI * 2.5e6; // 2.5 MHz total acoustic decay
        let kappa_p_ext = kappa_p * 0.85; // 85% external extraction efficiency

        let kappa_m = 2.0 * PI * 2.5e6; // 2.5 MHz total magnon decay
        let kappa_m_ext = kappa_m * 0.85; // 85% external microwave extraction efficiency

        Self {
            resonance_freq_rad_s: omega0,
            coupling_g_rad_s: g,
            total_phonon_decay_rad_s: kappa_p,
            external_phonon_decay_rad_s: kappa_p_ext,
            total_magnon_decay_rad_s: kappa_m,
            external_magnon_decay_rad_s: kappa_m_ext,
            temperature_k: 0.020, // 20 mK dilution fridge base temp
        }
    }
}

/// Transducer scattering matrix and quantum metrics solver.
#[derive(Debug, Clone)]
pub struct QuantumTransducerSolver {
    pub params: TransducerCouplingParams,
}

impl QuantumTransducerSolver {
    pub fn new(params: TransducerCouplingParams) -> Self {
        Self { params }
    }

    /// Evaluates complex 2-port S-parameters at drive frequency omega.
    ///
    /// Uses Heisenberg-Langevin input-output formalism:
    /// D(omega) = (kappa_p / 2 - i * delta_p) * (kappa_m / 2 - i * delta_m) + g^2
    /// S_21(omega) = i * sqrt(kappa_p_ext * kappa_m_ext) * g / D(omega)
    /// S_11(omega) = 1 - kappa_m_ext * (kappa_p / 2 - i * delta_p) / D(omega)
    /// S_22(omega) = 1 - kappa_ac_ext * (kappa_m / 2 - i * delta_m) / D(omega)
    pub fn evaluate_s_parameters(&self, freq_hz: f64) -> SParameterSample {
        let omega = 2.0 * PI * freq_hz;
        let omega0 = self.params.resonance_freq_rad_s;
        let delta = omega - omega0; // delta_p = delta_m = delta at resonance

        let kp = self.params.total_phonon_decay_rad_s;
        let kp_ext = self.params.external_phonon_decay_rad_s;
        let km = self.params.total_magnon_decay_rad_s;
        let km_ext = self.params.external_magnon_decay_rad_s;
        let g = self.params.coupling_g_rad_s;

        // Complex factors:
        // A = kp / 2 - i * delta = (re_a, im_a)
        let re_a = 0.5 * kp;
        let im_a = -delta;

        // B = km / 2 - i * delta = (re_b, im_b)
        let re_b = 0.5 * km;
        let im_b = -delta;

        // D = A * B + g^2
        // A * B = (re_a * re_b - im_a * im_b) + i * (re_a * im_b + im_a * re_b)
        let re_d = (re_a * re_b - im_a * im_b) + g * g;
        let im_d = re_a * im_b + im_a * re_b;
        let den_mag_sq = (re_d * re_d + im_d * im_d).max(1e-40);

        // Numerator of S_21: i * sqrt(kp_ext * km_ext) * g
        // (0.0, num_s21)
        let num_s21 = (kp_ext * km_ext).max(0.0).sqrt() * g;
        let s21_power = (num_s21 * num_s21) / den_mag_sq;

        // S_11 = 1 - km_ext * A / D
        // km_ext * A = (km_ext * re_a, km_ext * im_a)
        // (km_ext * A) / D = [ (km_ext * re_a) * re_d + (km_ext * im_a) * im_d + i(...) ] / den_mag_sq
        let term1_re = (km_ext * re_a * re_d + km_ext * im_a * im_d) / den_mag_sq;
        let term1_im = (km_ext * im_a * re_d - km_ext * re_a * im_d) / den_mag_sq;
        let s11_re = 1.0 - term1_re;
        let s11_im = -term1_im;
        let s11_power = (s11_re * s11_re + s11_im * s11_im).clamp(0.0, 1.0);

        // S_22 = 1 - kp_ext * B / D
        let term2_re = (kp_ext * re_b * re_d + kp_ext * im_b * im_d) / den_mag_sq;
        let term2_im = (kp_ext * im_b * re_d - kp_ext * re_b * im_d) / den_mag_sq;
        let s22_re = 1.0 - term2_re;
        let s22_im = -term2_im;
        let s22_power = (s22_re * s22_re + s22_im * s22_im).clamp(0.0, 1.0);

        // Enforce physical scattering conservation: |S_11|^2 + |S_21|^2 <= 1.0
        let s21_power_clamped = s21_power.min(1.0 - s11_power).max(0.0);

        let s21_db = if s21_power_clamped > 1e-12 {
            10.0 * s21_power_clamped.log10()
        } else {
            -120.0
        };

        let s11_db = if s11_power > 1e-12 {
            10.0 * s11_power.log10()
        } else {
            -120.0
        };

        SParameterSample {
            frequency_hz: freq_hz,
            s11_power,
            s21_power: s21_power_clamped,
            s22_power,
            s21_db,
            s11_db,
        }
    }

    /// Evaluates frequency sweep of S-parameters centered at resonance.
    pub fn sweep_s_parameters(
        &self,
        span_hz: f64,
        points: usize,
    ) -> Vec<SParameterSample> {
        let f0 = self.params.resonance_freq_rad_s / (2.0 * PI);
        let f_min = f0 - 0.5 * span_hz;
        let _f_max = f0 + 0.5 * span_hz;

        let mut samples = Vec::with_capacity(points);
        let step = span_hz / (points - 1).max(1) as f64;

        for i in 0..points {
            let f = f_min + i as f64 * step;
            samples.push(self.evaluate_s_parameters(f));
        }

        samples
    }

    /// Evaluates peak conversion efficiency: eta_max = max |S_21|^2 at polariton resonance.
    pub fn compute_peak_efficiency(&self) -> f64 {
        let f0 = self.params.resonance_freq_rad_s / (2.0 * PI);
        let g_hz = self.params.coupling_g_rad_s / (2.0 * PI);
        let s_upper = self.evaluate_s_parameters(f0 + g_hz).s21_power;
        let s_lower = self.evaluate_s_parameters(f0 - g_hz).s21_power;
        s_upper.max(s_lower)
    }

    /// Evaluates the 3-dB transduction bandwidth Delta f_3dB in Hertz around polariton resonance.
    pub fn compute_bandwidth_hz(&self) -> f64 {
        let eta_peak = self.compute_peak_efficiency();
        let target_eta = 0.5 * eta_peak;

        let f0 = self.params.resonance_freq_rad_s / (2.0 * PI);
        let g_hz = self.params.coupling_g_rad_s / (2.0 * PI);
        let f_polariton = f0 + g_hz;

        let span = 20.0e6; // 20 MHz search window around peak
        let f_min = f_polariton - 0.5 * span;
        let points = 200;
        let step = span / (points - 1) as f64;

        let mut f_low = f_polariton;
        let mut f_high = f_polariton;

        for i in 0..points {
            let f = f_min + i as f64 * step;
            let s = self.evaluate_s_parameters(f);
            if f < f_polariton && s.s21_power >= target_eta {
                f_low = f;
                break;
            }
        }

        for i in (0..points).rev() {
            let f = f_min + i as f64 * step;
            let s = self.evaluate_s_parameters(f);
            if f > f_polariton && s.s21_power >= target_eta {
                f_high = f;
                break;
            }
        }

        (f_high - f_low).abs().max(5.0e5)
    }

    /// Evaluates added thermal noise photons at frequency omega:
    /// n_add = [(1 - eta) / eta] * (n_th + 0.5)
    pub fn compute_added_noise_quanta(&self, freq_hz: f64) -> f64 {
        let sample = self.evaluate_s_parameters(freq_hz);
        let eta = sample.s21_power.clamp(1e-6, 0.9999);

        let energy = HBAR * 2.0 * PI * freq_hz;
        let k_t = K_BOLTZMANN * self.params.temperature_k.max(1e-4);
        let x = energy / k_t;
        let n_th = if x < 80.0 {
            1.0 / (x.exp() - 1.0)
        } else {
            0.0
        };

        ((1.0 - eta) / eta) * (n_th + 0.5)
    }

    /// Evaluates bidirectional quantum state fidelity: F = sqrt(eta).
    pub fn compute_quantum_fidelity(&self) -> f64 {
        self.compute_peak_efficiency().sqrt()
    }
}
