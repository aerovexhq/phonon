#![deny(unsafe_code)]

//! Cryogenic Quantum Qubit Dispersive Readout Engine.
//!
//! Models a dilution-fridge quantum processor readout interface using the non-reciprocal
//! topological Floquet acoustic circulator. Operates at millikelvin temperatures (15 mK),
//! suppressing amplifier back-action dephasing while achieving high readout SNR (>= 18.0 dB)
//! and quantum-limited added noise (<= 0.55 quanta).

use std::f64::consts::PI;

/// Physical constants.
const H_PLANCK: f64 = 6.62607015e-34;
const K_BOLTZMANN: f64 = 1.380649e-23;

/// Parameters for cryogenic qubit readout.
#[derive(Debug, Clone)]
pub struct CryogenicReadoutParams {
    /// Operating temperature in Kelvin (default ~0.015 K / 15 mK).
    pub temperature_k: f64,
    /// Probe readout center frequency in GHz (default ~1.0 GHz).
    pub readout_freq_ghz: f64,
    /// Dispersive cavity shift 2*chi in MHz (default ~16.0 MHz).
    pub dispersive_shift_2chi_mhz: f64,
    /// Readout cavity linewidth kappa in MHz (default ~6.0 MHz).
    pub cavity_linewidth_mhz: f64,
    /// Circulator insertion loss in dB (default ~0.60 dB).
    pub circulator_insertion_loss_db: f64,
    /// Circulator reverse isolation in dB (default ~32.0 dB).
    pub circulator_isolation_db: f64,
    /// Post-amplifier noise temperature in K (default ~4.0 K HEMT).
    pub amplifier_noise_temp_k: f64,
    /// Number of probe photons/phonons per measurement pulse (default ~15.0).
    pub probe_photon_count: f64,
}

impl Default for CryogenicReadoutParams {
    fn default() -> Self {
        Self {
            temperature_k: 0.015,
            readout_freq_ghz: 1.0,
            dispersive_shift_2chi_mhz: 16.0,
            cavity_linewidth_mhz: 6.0,
            circulator_insertion_loss_db: 0.60,
            circulator_isolation_db: 32.0,
            amplifier_noise_temp_k: 4.0,
            probe_photon_count: 15.0,
        }
    }
}

/// Dispersive readout transmission point for |0> and |1> qubit eigenstates.
#[derive(Debug, Clone)]
pub struct CryogenicReadoutPoint {
    /// Probe frequency in GHz.
    pub freq_ghz: f64,
    /// Cavity transmission magnitude for |0> ground state in dB.
    pub trans_state_0_db: f64,
    /// Cavity transmission magnitude for |1> excited state in dB.
    pub trans_state_1_db: f64,
    /// Dispersive phase difference between |0> and |1> in degrees.
    pub phase_difference_deg: f64,
    /// Resolved readout Signal-to-Noise Ratio (SNR) in dB.
    pub readout_snr_db: f64,
}

/// Physical simulation engine for cryogenic non-reciprocal qubit readout.
#[derive(Debug, Clone)]
pub struct CryogenicReadoutEngine {
    pub params: CryogenicReadoutParams,
}

impl CryogenicReadoutEngine {
    /// Creates a new cryogenic readout engine.
    pub fn new(params: CryogenicReadoutParams) -> Self {
        Self { params }
    }

    /// Evaluates equilibrium thermal phonon occupancy n_th via Bose-Einstein statistics.
    pub fn thermal_phonon_occupancy(&self) -> f64 {
        let h_nu = H_PLANCK * self.params.readout_freq_ghz * 1e9;
        let k_t = K_BOLTZMANN * self.params.temperature_k.max(1e-4);
        let exponent = (h_nu / k_t).clamp(0.01, 80.0);
        1.0 / (exponent.exp() - 1.0)
    }

    /// Evaluates quantum added noise in quanta.
    /// In a phase-insensitive non-reciprocal acoustic circulator, the standard quantum
    /// limit (SQL) imposes n_add >= 0.5. At 15 mK, n_th ~ 0.04, yielding n_add <= 0.55.
    pub fn quantum_added_noise_quanta(&self) -> f64 {
        0.50 + self.thermal_phonon_occupancy()
    }

    /// Evaluates residual back-action noise photons leaking into the qubit.
    /// Reverse isolation attenuates the 4K HEMT amplifier noise floor by >= 30 dB.
    pub fn amplifier_backaction_leakage_quanta(&self) -> f64 {
        let h_nu = H_PLANCK * self.params.readout_freq_ghz * 1e9;
        let k_t = K_BOLTZMANN * self.params.amplifier_noise_temp_k;
        let amp_noise_quanta = k_t / (h_nu + 1e-30);

        let isolation_factor = 10.0_f64.powf(-self.params.circulator_isolation_db / 10.0);
        amp_noise_quanta * isolation_factor
    }

    /// Evaluates readout SNR in dB at the cavity center frequency.
    pub fn peak_readout_snr_db(&self) -> f64 {
        let chi = self.params.dispersive_shift_2chi_mhz * 0.5;
        let kappa = self.params.cavity_linewidth_mhz;
        let separation_factor = (4.0 * chi * chi) / (kappa * kappa + 4.0 * chi * chi);

        let n_sig = self.params.probe_photon_count;
        let n_noise = self.quantum_added_noise_quanta() + 0.5;
        let il_factor = 10.0_f64.powf(-self.params.circulator_insertion_loss_db / 10.0);

        // Dispersive measurement integration gain over readout pulse duration (tau_meas ~ 800 ns)
        let integration_gain = 8.0;
        let raw_snr = separation_factor * (n_sig / n_noise) * il_factor * integration_gain;
        10.0 * raw_snr.max(1e-4).log10()
    }

    /// Evaluates dispersive cavity transmission spectrum for |0> and |1> states.
    pub fn compute_readout_spectrum(&self, num_points: usize, span_mhz: f64) -> Vec<CryogenicReadoutPoint> {
        let f0 = self.params.readout_freq_ghz;
        let chi_ghz = (self.params.dispersive_shift_2chi_mhz * 0.5) * 1e-3;
        let kappa_ghz = self.params.cavity_linewidth_mhz * 1e-3;

        let span_ghz = span_mhz * 1e-3;
        let f_min = f0 - span_ghz * 0.5;
        let f_max = f0 + span_ghz * 0.5;

        let mut points = Vec::with_capacity(num_points);
        let il_db = -self.params.circulator_insertion_loss_db;
        let peak_snr = self.peak_readout_snr_db();

        for i in 0..num_points {
            let frac = (i as f64) / ((num_points - 1).max(1) as f64);
            let freq = f_min + frac * (f_max - f_min);

            // State |0>: cavity shifted by -chi
            let delta_0 = freq - (f0 - chi_ghz);
            let denom_0 = delta_0 * delta_0 + (kappa_ghz * 0.5).powi(2);
            let t0_mag = (kappa_ghz * 0.5) / denom_0.sqrt();
            let trans_0_db = (20.0 * t0_mag.clamp(1e-4, 1.0).log10() + il_db).clamp(-60.0, 0.0);

            // State |1>: cavity shifted by +chi
            let delta_1 = freq - (f0 + chi_ghz);
            let denom_1 = delta_1 * delta_1 + (kappa_ghz * 0.5).powi(2);
            let t1_mag = (kappa_ghz * 0.5) / denom_1.sqrt();
            let trans_1_db = (20.0 * t1_mag.clamp(1e-4, 1.0).log10() + il_db).clamp(-60.0, 0.0);

            // Phase difference
            let phi_0 = (-delta_0 / (kappa_ghz * 0.5)).atan();
            let phi_1 = (-delta_1 / (kappa_ghz * 0.5)).atan();
            let phase_diff_deg = (phi_1 - phi_0).to_degrees().abs();

            let snr_factor = (t0_mag - t1_mag).abs() / (t0_mag.max(t1_mag) + 1e-6);
            let snr_db = (peak_snr * snr_factor).clamp(0.0, 30.0);

            points.push(CryogenicReadoutPoint {
                freq_ghz: freq,
                trans_state_0_db: trans_0_db,
                trans_state_1_db: trans_1_db,
                phase_difference_deg: phase_diff_deg,
                readout_snr_db: snr_db,
            });
        }

        points
    }
}
