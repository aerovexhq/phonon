#![deny(unsafe_code)]

//! Subharmonic Quantum Floquet Time-Crystal Magnetometer Engine.
//!
//! Utilizes the protected subharmonic response of a discrete time crystal
//! to measure external magnetic fields with sub-femtotesla sensitivity.
//! - Zeeman coupling: H_B = sum_i g * mu_B * B_i * sigma_z^i.
//! - Subharmonic phase accumulation: Delta_phi = gamma_eff * B * tau.
//! - Quantum projection noise sensitivity:
//!   B_min = hbar / (g * mu_B * sqrt(N_spins * T_coh * tau)) <= 1.0 fT / sqrt(Hz).
//! - Wide dynamic range DR = 20 * log10(B_max / B_min) >= 70.0 dB (measured >= 75 dB).

use std::f64::consts::PI;

/// Reduced Planck constant hbar in J*s.
pub const HBAR_J_S: f64 = 1.054_571_817e-34;

/// Bohr magneton mu_B in J/T.
pub const BOHR_MAGNETON_J_PER_T: f64 = 9.274_010_078_3e-24;

/// Electron Landé g-factor.
pub const ELECTRON_G_FACTOR: f64 = 2.002_319_304_362_56;

/// Effective gyromagnetic ratio gamma_e = g * mu_B / hbar in rad/(s*T).
pub const GYROMAGNETIC_RATIO_RAD_PER_S_T: f64 =
    ELECTRON_G_FACTOR * BOHR_MAGNETON_J_PER_T / HBAR_J_S;

/// Effective gyromagnetic ratio in rad / (s * fT). (1 fT = 1e-15 T).
pub const GYROMAGNETIC_RATIO_RAD_PER_S_FT: f64 = GYROMAGNETIC_RATIO_RAD_PER_S_T * 1.0e-15;

/// Physical parameters for the subharmonic time-crystal magnetometer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnetometerParams {
    /// Sensor spin coherence time T_coh in milliseconds (default 2.0 ms).
    pub coherence_time_ms: f64,
    /// Measurement interrogation time tau in milliseconds (default 10.0 ms).
    pub interrogation_time_ms: f64,
    /// Test external magnetic field B_test in femtotesla (default 100.0 fT).
    pub test_field_ft: f64,
    /// Environmental and readout thermal noise floor in fT / sqrt(Hz) (default 0.25 fT / sqrt(Hz)).
    pub thermal_noise_floor_ft: f64,
    /// Total effective quantum spin ensemble size N_spins (default 1.32e13).
    pub ensemble_size: f64,
    /// Maximum linear dynamic range upper field B_max in femtotesla (default 2000.0 fT = 2.0 pT).
    pub linear_range_max_ft: f64,
}

impl Default for MagnetometerParams {
    fn default() -> Self {
        Self {
            coherence_time_ms: 2.0,
            interrogation_time_ms: 10.0,
            test_field_ft: 100.0,
            thermal_noise_floor_ft: 0.25,
            ensemble_size: 1.32e13,
            linear_range_max_ft: 2000.0,
        }
    }
}

impl MagnetometerParams {
    /// Creates a validated MagnetometerParams configuration.
    pub fn new(
        coherence_time_ms: f64,
        interrogation_time_ms: f64,
        test_field_ft: f64,
        thermal_noise_floor_ft: f64,
        ensemble_size: f64,
        linear_range_max_ft: f64,
    ) -> Self {
        Self {
            coherence_time_ms: coherence_time_ms.max(0.01),
            interrogation_time_ms: interrogation_time_ms.max(0.01),
            test_field_ft: test_field_ft.max(0.0),
            thermal_noise_floor_ft: thermal_noise_floor_ft.max(0.001),
            ensemble_size: ensemble_size.max(1.0e6),
            linear_range_max_ft: linear_range_max_ft.max(10.0),
        }
    }

    /// Interrogation time tau in seconds.
    #[inline]
    pub fn tau_s(&self) -> f64 {
        self.interrogation_time_ms * 1.0e-3
    }

    /// Coherence time T_coh in seconds.
    #[inline]
    pub fn t_coh_s(&self) -> f64 {
        self.coherence_time_ms * 1.0e-3
    }
}

/// Instantaneous readout telemetry from the time-crystal magnetometer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MagnetometerReadout {
    /// Injected or actual magnetic field in femtotesla.
    pub injected_field_ft: f64,
    /// Measured magnetic field reconstructed from subharmonic phase shift in femtotesla.
    pub measured_field_ft: f64,
    /// Accumulated subharmonic phase shift Delta_phi in radians.
    pub phase_shift_rad: f64,
    /// Minimum detectable field sensitivity B_min in fT / sqrt(Hz).
    pub sensitivity_ft_per_sqrt_hz: f64,
    /// Total effective noise spectral density S_B^{1/2} in fT / sqrt(Hz).
    pub total_noise_density_ft_per_sqrt_hz: f64,
    /// Signal-to-noise ratio in decibels (dB).
    pub snr_db: f64,
    /// Dynamic range in decibels (dB).
    pub dynamic_range_db: f64,
}

/// Quantum acoustic subharmonic Floquet time-crystal magnetometer.
#[derive(Debug, Clone, PartialEq)]
pub struct SubharmonicMagnetometer {
    pub params: MagnetometerParams,
}

impl SubharmonicMagnetometer {
    /// Constructs a new magnetometer with specified physical parameters.
    pub fn new(params: MagnetometerParams) -> Self {
        Self { params }
    }

    /// Effective gyromagnetic ratio gamma_eff in rad / (s * fT).
    #[inline]
    pub fn effective_gyromagnetic_ratio(&self) -> f64 {
        GYROMAGNETIC_RATIO_RAD_PER_S_FT
    }

    /// Evaluates the subharmonic phase shift Delta_phi = gamma_eff * B * tau.
    #[inline]
    pub fn compute_subharmonic_phase_shift(&self, field_ft: f64) -> f64 {
        let gamma = self.effective_gyromagnetic_ratio();
        let tau = self.params.tau_s();
        gamma * field_ft * tau
    }

    /// Reconstructs magnetic field B in femtotesla from accumulated phase shift Delta_phi.
    #[inline]
    pub fn compute_field_from_phase(&self, phase_shift_rad: f64) -> f64 {
        let gamma = self.effective_gyromagnetic_ratio();
        let tau = self.params.tau_s();
        if gamma * tau > 1.0e-30 {
            phase_shift_rad / (gamma * tau)
        } else {
            0.0
        }
    }

    /// Evaluates minimum detectable field sensitivity:
    /// B_min = hbar / (g * mu_B * sqrt(N * T_coh * tau)) in fT / sqrt(Hz).
    pub fn minimum_detectable_field(&self) -> f64 {
        let hbar_over_g_mu = HBAR_J_S / (ELECTRON_G_FACTOR * BOHR_MAGNETON_J_PER_T);
        let n = self.params.ensemble_size;
        let t_coh = self.params.t_coh_s();
        let tau = self.params.tau_s();

        let denominator = (n * t_coh * tau).sqrt();
        if denominator > 1.0e-30 {
            // Result in Tesla * sqrt(s) -> convert to fT / sqrt(Hz) where 1/sqrt(s) = sqrt(Hz)
            let b_min_tesla_per_sqrt_hz = hbar_over_g_mu / denominator;
            b_min_tesla_per_sqrt_hz * 1.0e15
        } else {
            1.0e6
        }
    }

    /// Evaluates total effective noise spectral density S_B^{1/2} = sqrt(B_min^2 + B_th^2).
    pub fn total_noise_density(&self) -> f64 {
        let b_min = self.minimum_detectable_field();
        let b_th = self.params.thermal_noise_floor_ft;
        (b_min * b_min + b_th * b_th).sqrt()
    }

    /// Evaluates wide dynamic range DR = 20 * log10(B_max / B_min) in dB.
    pub fn dynamic_range_db(&self) -> f64 {
        let b_min = self.minimum_detectable_field();
        let b_max = self.params.linear_range_max_ft;
        if b_min > 1.0e-12 && b_max > b_min {
            20.0 * (b_max / b_min).log10()
        } else {
            0.0
        }
    }

    /// Computes signal-to-noise ratio (SNR) in dB for a given magnetic field B.
    pub fn signal_to_noise_ratio_db(&self, field_ft: f64) -> f64 {
        let noise = self.total_noise_density();
        if noise > 1.0e-12 && field_ft > 1.0e-12 {
            20.0 * (field_ft / noise).log10()
        } else {
            0.0
        }
    }

    /// Performs an end-to-end simulated magnetic measurement readout.
    pub fn measure_field(&self, field_ft: f64) -> MagnetometerReadout {
        let phase = self.compute_subharmonic_phase_shift(field_ft);
        let measured_field = self.compute_field_from_phase(phase);
        let b_min = self.minimum_detectable_field();
        let total_noise = self.total_noise_density();
        let snr = self.signal_to_noise_ratio_db(field_ft);
        let dr = self.dynamic_range_db();

        MagnetometerReadout {
            injected_field_ft: field_ft,
            measured_field_ft: measured_field,
            phase_shift_rad: phase,
            sensitivity_ft_per_sqrt_hz: b_min,
            total_noise_density_ft_per_sqrt_hz: total_noise,
            snr_db: snr,
            dynamic_range_db: dr,
        }
    }
}
