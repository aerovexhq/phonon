#![deny(unsafe_code)]

//! Coherent Phonon-Magnon Polariton Dispersion & Hybridization Engine.
//!
//! Models coupled acoustic phonons (SAW/BAW) and spin waves (magnons) in
//! magnetoelastic heterostructures (such as YIG/GGG or Ni/LiNbO3).
//! Evaluates the avoided-crossing polariton gap, Hopfield mixing coefficients,
//! and quantum cooperativity.

use std::f64::consts::PI;

pub const GYROMAGNETIC_RATIO: f64 = 1.760_859_644e11; // rad / (s * T)
pub const MU_0: f64 = 1.256_637_061_4e-6; // T * m / A

/// Parameters governing the coupled phonon-magnon system.
#[derive(Debug, Clone)]
pub struct PhononMagnonParams {
    /// Acoustic phase velocity (speed of sound) v_s in m / s (e.g. 3840.0 m/s for YIG).
    pub sound_velocity_m_s: f64,
    /// Saturation magnetization M_s in A / m (e.g. 1.4e5 A/m for YIG).
    pub ms_a_m: f64,
    /// Exchange stiffness constant D_ex in T * m^2 (e.g. 5.4e-17 T * m^2 for YIG).
    pub exchange_stiffness_t_m2: f64,
    /// External static bias magnetic field B_ext in Tesla.
    pub b_ext_tesla: f64,
    /// Magnetoelastic coupling frequency g_pm in rad / s (typically 2 * pi * 30 MHz).
    pub coupling_g_rad_s: f64,
    /// Acoustic phonon resonator quality factor Q_p (e.g. 2000.0).
    pub acoustic_q: f64,
    /// Dimensionless Gilbert damping parameter alpha_G (e.g. 2.0e-4 for YIG).
    pub gilbert_damping: f64,
}

impl Default for PhononMagnonParams {
    fn default() -> Self {
        Self {
            sound_velocity_m_s: 3840.0,
            ms_a_m: 1.4e5,
            exchange_stiffness_t_m2: 5.4e-17,
            b_ext_tesla: 0.12, // 120 mT bias field
            coupling_g_rad_s: 2.0 * PI * 35.0e6, // 35 MHz coupling rate
            acoustic_q: 2500.0,
            gilbert_damping: 3.0e-4, // Low-damping YIG
        }
    }
}

/// Point on the polariton dispersion curve at wavenumber k.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonDispersionPoint {
    /// Acoustic/magnon wavenumber k in rad / m.
    pub wavenumber_rad_m: f64,
    /// Bare acoustic phonon frequency omega_p in rad / s.
    pub omega_phonon: f64,
    /// Bare spin-wave (magnon) frequency omega_m in rad / s.
    pub omega_magnon: f64,
    /// Lower hybridized polariton branch omega_- in rad / s.
    pub omega_lower_polariton: f64,
    /// Upper hybridized polariton branch omega_+ in rad / s.
    pub omega_upper_polariton: f64,
    /// Hopfield phonon fraction |u|^2 of lower polariton (0.0 to 1.0).
    pub hopfield_phonon_fraction: f64,
    /// Hopfield magnon fraction |v|^2 of lower polariton (0.0 to 1.0).
    pub hopfield_magnon_fraction: f64,
    /// Quantum cooperativity C = 4 * g^2 / (kappa_p * kappa_m).
    pub cooperativity: f64,
}

impl PolaritonDispersionPoint {
    /// Bare phonon frequency in GHz.
    pub fn phonon_freq_ghz(&self) -> f64 {
        self.omega_phonon / (2.0 * PI * 1e9)
    }

    /// Bare magnon frequency in GHz.
    pub fn magnon_freq_ghz(&self) -> f64 {
        self.omega_magnon / (2.0 * PI * 1e9)
    }

    /// Lower polariton frequency in GHz.
    pub fn lower_polariton_ghz(&self) -> f64 {
        self.omega_lower_polariton / (2.0 * PI * 1e9)
    }

    /// Upper polariton frequency in GHz.
    pub fn upper_polariton_ghz(&self) -> f64 {
        self.omega_upper_polariton / (2.0 * PI * 1e9)
    }

    /// Avoided crossing polariton splitting Delta f in MHz.
    pub fn splitting_mhz(&self) -> f64 {
        (self.omega_upper_polariton - self.omega_lower_polariton) / (2.0 * PI * 1e6)
    }
}

/// Engine evaluating phonon-magnon polariton dispersion, avoided crossings, and strong coupling.
#[derive(Debug, Clone)]
pub struct PolaritonDispersionEngine {
    pub params: PhononMagnonParams,
}

impl PolaritonDispersionEngine {
    pub fn new(params: PhononMagnonParams) -> Self {
        Self { params }
    }

    /// Evaluates bare acoustic phonon frequency: omega_p(k) = v_s * k.
    pub fn compute_phonon_omega(&self, k: f64) -> f64 {
        self.params.sound_velocity_m_s * k.abs()
    }

    /// Evaluates bare spin-wave frequency from dipole-exchange dispersion:
    /// omega_m(k) = gamma * sqrt[ B_eff * (B_eff + mu_0 * M_s) ]
    /// with B_eff(k) = B_ext + D_ex * k^2.
    pub fn compute_magnon_omega(&self, k: f64) -> f64 {
        let b_eff = self.params.b_ext_tesla + self.params.exchange_stiffness_t_m2 * k * k;
        let b_total = b_eff + MU_0 * self.params.ms_a_m;
        GYROMAGNETIC_RATIO * (b_eff * b_total).max(0.0).sqrt()
    }

    /// Evaluates acoustic phonon dissipation rate: kappa_p = omega_p / Q_p.
    pub fn compute_phonon_decay(&self, omega_p: f64) -> f64 {
        omega_p / self.params.acoustic_q.max(1.0)
    }

    /// Evaluates Gilbert-damped magnon dissipation rate: kappa_m = 2 * alpha_G * omega_m.
    pub fn compute_magnon_decay(&self, omega_m: f64) -> f64 {
        2.0 * self.params.gilbert_damping * omega_m
    }

    /// Evaluates the polariton eigenstate at wavenumber k.
    pub fn evaluate_point(&self, k: f64) -> PolaritonDispersionPoint {
        let omega_p = self.compute_phonon_omega(k);
        let omega_m = self.compute_magnon_omega(k);
        let g = self.params.coupling_g_rad_s;

        let omega_bar = 0.5 * (omega_p + omega_m);
        let detuning = omega_p - omega_m;
        let rabi_split = (detuning * detuning + 4.0 * g * g).sqrt();

        let omega_upper = omega_bar + 0.5 * rabi_split;
        let omega_lower = omega_bar - 0.5 * rabi_split;

        // Hopfield mixing coefficients for the lower polariton branch:
        // |u_-|^2 = 0.5 * (1 - detuning / rabi_split)
        // |v_-|^2 = 0.5 * (1 + detuning / rabi_split)
        let ratio = if rabi_split > 1e-12 {
            detuning / rabi_split
        } else {
            0.0
        };
        let hopfield_phonon = 0.5 * (1.0 - ratio);
        let hopfield_magnon = 0.5 * (1.0 + ratio);

        let kappa_p = self.compute_phonon_decay(omega_p);
        let kappa_m = self.compute_magnon_decay(omega_m);
        let cooperativity = if kappa_p * kappa_m > 1e-20 {
            (4.0 * g * g) / (kappa_p * kappa_m)
        } else {
            0.0
        };

        PolaritonDispersionPoint {
            wavenumber_rad_m: k,
            omega_phonon: omega_p,
            omega_magnon: omega_m,
            omega_lower_polariton: omega_lower,
            omega_upper_polariton: omega_upper,
            hopfield_phonon_fraction: hopfield_phonon,
            hopfield_magnon_fraction: hopfield_magnon,
            cooperativity,
        }
    }

    /// Numerically locates the exact resonance wavenumber k_0 where omega_p(k_0) = omega_m(k_0).
    pub fn find_resonance_wavenumber(&self) -> f64 {
        // Bisect between k = 1e4 rad/m and k = 2e7 rad/m
        let mut k_low = 1.0e4;
        let mut k_high = 2.0e7;

        for _ in 0..60 {
            let k_mid = 0.5 * (k_low + k_high);
            let diff = self.compute_phonon_omega(k_mid) - self.compute_magnon_omega(k_mid);
            if diff < 0.0 {
                k_low = k_mid;
            } else {
                k_high = k_mid;
            }
        }

        0.5 * (k_low + k_high)
    }

    /// Evaluates polariton dispersion curve across wavenumber range [k_min, k_max] with `sample_count` points.
    pub fn generate_dispersion_curve(
        &self,
        k_min: f64,
        k_max: f64,
        sample_count: usize,
    ) -> Vec<PolaritonDispersionPoint> {
        let mut points = Vec::with_capacity(sample_count);
        if sample_count < 2 {
            return points;
        }

        let step = (k_max - k_min) / (sample_count - 1) as f64;
        for i in 0..sample_count {
            let k = k_min + i as f64 * step;
            points.push(self.evaluate_point(k));
        }

        points
    }
}
