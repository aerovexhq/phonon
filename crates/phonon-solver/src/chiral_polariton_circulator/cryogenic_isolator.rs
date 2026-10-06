#![deny(unsafe_code)]

//! Cryogenic Non-Reciprocal Microwave Isolator & Quantum-Limited Noise Engine.
//!
//! Models non-reciprocal microwave isolation operating at dilution refrigerator
//! temperatures (e.g. 20 mK), evaluating:
//! - Noise temperature T_noise = (hbar * omega / (2 * k_B)) / tanh(hbar * omega / (2 * k_B * T))
//! - Added thermal noise quanta n_add approaching the quantum limit (n_add <= 0.05 at 20 mK and 5 GHz)
//! - Directivity D = ISO_dB - IL_dB >= 35.0 dB
//! - 1-dB compression point P_1dB >= -20.0 dBm

use std::f64::consts::PI;
use super::circulator_s_matrix::SParameters;
use super::chiral_dispersion::{BOLTZMANN_K_J_K, HBAR_J_S};

/// Parameters for cryogenic microwave isolator operation.
#[derive(Debug, Clone, PartialEq)]
pub struct CryogenicIsolatorParams {
    /// Physical operating temperature in Kelvin (default 0.020 K / 20 mK dilution fridge base).
    pub operating_temp_k: f64,
    /// Operating microwave center frequency in GHz (default ~5.0 GHz).
    pub frequency_ghz: f64,
    /// Incident microwave probe power in dBm (default -90.0 dBm in quantum readout regime).
    pub incident_power_dbm: f64,
    /// 1-dB compression power handling limit in dBm (default -20.0 dBm).
    pub power_handling_1db_dbm: f64,
}

impl Default for CryogenicIsolatorParams {
    fn default() -> Self {
        Self {
            operating_temp_k: 0.020, // 20 mK dilution refrigerator base
            frequency_ghz: 5.0,
            incident_power_dbm: -90.0,
            power_handling_1db_dbm: -20.0,
        }
    }
}

/// Evaluated physical performance metrics for the cryogenic isolator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CryogenicIsolatorMetrics {
    /// Effective noise temperature in Kelvin including quantum vacuum fluctuations.
    pub noise_temperature_k: f64,
    /// Added thermal noise quanta n_add referred to input.
    pub added_noise_quanta: f64,
    /// Directivity D = ISO_dB - IL_dB in decibels.
    pub directivity_db: f64,
    /// 1-dB compression power in dBm.
    pub power_1db_compression_dbm: f64,
    /// Forward insertion loss in decibels.
    pub insertion_loss_db: f64,
    /// Reverse isolation in decibels.
    pub isolation_db: f64,
    /// Thermal Bose-Einstein occupation of the 50-Ohm internal termination load.
    pub load_thermal_occupation: f64,
    /// True if device operates within 5% of the standard quantum limit at 20 mK.
    pub is_quantum_limited: bool,
}

impl CryogenicIsolatorParams {
    /// Evaluates angular frequency omega in rad/s: omega = 2 * pi * f.
    #[inline]
    pub fn angular_frequency_rad_s(&self) -> f64 {
        2.0 * PI * self.frequency_ghz * 1.0e9
    }

    /// Evaluates half-quantum zero-point energy scale T_Q = hbar * omega / (2 * k_B) in Kelvin.
    #[inline]
    pub fn quantum_temperature_scale_k(&self) -> f64 {
        let omega = self.angular_frequency_rad_s();
        (HBAR_J_S * omega) / (2.0 * BOLTZMANN_K_J_K)
    }

    /// Evaluates effective noise temperature T_noise:
    ///   T_noise = (hbar * omega / (2 * k_B)) / tanh(hbar * omega / (2 * k_B * T))
    pub fn compute_noise_temperature(&self) -> f64 {
        let t_q = self.quantum_temperature_scale_k();
        let temp = self.operating_temp_k.max(1.0e-4);
        let arg = t_q / temp;

        let tanh_val = if arg > 30.0 {
            1.0
        } else if arg < 1.0e-5 {
            arg
        } else {
            arg.tanh()
        };

        t_q / tanh_val
    }

    /// Evaluates the Bose-Einstein thermal photon occupation:
    ///   n_th = 1 / (exp(hbar * omega / (k_B * T)) - 1)
    pub fn compute_thermal_occupation(&self) -> f64 {
        let omega = self.angular_frequency_rad_s();
        let temp = self.operating_temp_k.max(1.0e-4);
        let x = (HBAR_J_S * omega) / (BOLTZMANN_K_J_K * temp);

        if x > 50.0 {
            0.0
        } else if x < 1.0e-5 {
            1.0 / x
        } else {
            1.0 / (x.exp() - 1.0)
        }
    }

    /// Evaluates added thermal noise quanta n_add referred to input,
    /// accounting for internal termination loss and finite directivity.
    pub fn compute_added_noise_quanta(&self, insertion_loss_db: f64) -> f64 {
        let n_th = self.compute_thermal_occupation();
        // Transmission power ratio eta = 10^(-IL / 10)
        let eta = 10.0_f64.powf(-insertion_loss_db / 10.0);
        let loss_fraction = (1.0 - eta).max(0.0);
        // Added thermal noise emitted by lossy termination into the channel
        loss_fraction * n_th + n_th * 1.0e-4
    }

    /// Evaluates comprehensive isolator performance metrics using current S-parameters.
    pub fn evaluate_metrics(&self, s_params: &SParameters) -> CryogenicIsolatorMetrics {
        let il_db = s_params.insertion_loss_db();
        let iso_db = s_params.isolation_db();
        let directivity_db = iso_db - il_db;
        let t_noise = self.compute_noise_temperature();
        let n_th = self.compute_thermal_occupation();
        let n_add = self.compute_added_noise_quanta(il_db);
        let is_quantum_limited = n_add <= 0.05;

        CryogenicIsolatorMetrics {
            noise_temperature_k: t_noise,
            added_noise_quanta: n_add,
            directivity_db,
            power_1db_compression_dbm: self.power_handling_1db_dbm,
            insertion_loss_db: il_db,
            isolation_db: iso_db,
            load_thermal_occupation: n_th,
            is_quantum_limited,
        }
    }
}
