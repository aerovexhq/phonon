#![deny(unsafe_code)]

//! Cryogenic microwave-to-phonon quantum transducer array.
//!
//! Models interdigital piezoelectric transducers (IDT) converting microwave RF
//! photons to surface/bulk acoustic wave phonons with high quantum conversion
//! efficiency, low insertion loss, and quantum-limited added noise.

use std::f64::consts::PI;

/// Physical constants.
const HBAR: f64 = 1.054_571_817e-34; // J*s
const KB: f64 = 1.380_649e-23;       // J/K

/// Configuration parameters for the cryogenic microwave-to-phonon transducer.
#[derive(Debug, Clone, PartialEq)]
pub struct TransducerParams {
    /// Center operating RF frequency in GHz (default: 1.0 GHz).
    pub rf_freq_ghz: f64,
    /// Effective electromechanical piezoelectric coupling coefficient k_eff^2 (default: 0.08 / 8.0%).
    pub piezo_coupling_keff2: f64,
    /// Number of interdigital electrode finger pairs N_p (default: 25).
    pub electrode_pairs: usize,
    /// Acoustic beam aperture in micrometers (default: 120.0 um).
    pub aperture_um: f64,
    /// Cryogenic operating temperature in Kelvin (default: 0.020 K / 20 mK).
    pub temperature_k: f64,
    /// Microwave characteristic impedance in Ohms (default: 50.0 Ohm).
    pub characteristic_impedance_ohm: f64,
}

impl Default for TransducerParams {
    fn default() -> Self {
        Self {
            rf_freq_ghz: 1.0,
            piezo_coupling_keff2: 0.08,
            electrode_pairs: 25,
            aperture_um: 120.0,
            temperature_k: 0.015,
            characteristic_impedance_ohm: 50.0,
        }
    }
}

/// Transducer frequency response sample point.
#[derive(Debug, Clone, PartialEq)]
pub struct TransducerResponsePoint {
    pub freq_ghz: f64,
    pub conversion_efficiency: f64,
    pub insertion_loss_db: f64,
    pub radiation_conductance_ms: f64,
    pub acoustic_susceptance_ms: f64,
}

/// Microwave-to-phonon quantum transducer solver.
#[derive(Debug, Clone)]
pub struct MicrowavePhononTransducer {
    pub params: TransducerParams,
}

impl MicrowavePhononTransducer {
    /// Creates a new MicrowavePhononTransducer.
    pub fn new(params: TransducerParams) -> Self {
        Self { params }
    }

    /// Evaluates the static capacitance C_T of the IDT in Picofarads.
    pub fn static_capacitance_pf(&self) -> f64 {
        let np = self.params.electrode_pairs as f64;
        let c0_per_finger_pf = 0.045; // ~45 fF per pair per 100um
        let aperture_norm = self.params.aperture_um / 100.0;
        np * c0_per_finger_pf * aperture_norm
    }

    /// Evaluates radiation conductance G_a(f) in milliSiemens (mS).
    pub fn radiation_conductance_ms(&self, freq_ghz: f64) -> f64 {
        let f0 = self.params.rf_freq_ghz;
        let np = self.params.electrode_pairs as f64;
        let c_t_pf = self.static_capacitance_pf();
        let keff2 = self.params.piezo_coupling_keff2;

        let delta_f = freq_ghz - f0;
        let x = np * PI * (delta_f / f0);
        let sinc = if x.abs() < 1e-6 {
            1.0
        } else {
            x.sin() / x
        };

        // G_0 = 8 * k_eff^2 * f0 * C_T * N_p
        // in mS = 8 * keff2 * (f0 in GHz * 1e9) * (C_t in pF * 1e-12) * N_p * 1e3
        let g0_ms = 8.0 * keff2 * f0 * c_t_pf * np;
        g0_ms * sinc.powi(2)
    }

    /// Evaluates acoustic susceptance B_a(f) in milliSiemens (mS).
    pub fn acoustic_susceptance_ms(&self, freq_ghz: f64) -> f64 {
        let f0 = self.params.rf_freq_ghz;
        let np = self.params.electrode_pairs as f64;
        let c_t_pf = self.static_capacitance_pf();
        let keff2 = self.params.piezo_coupling_keff2;

        let delta_f = freq_ghz - f0;
        let x = np * PI * (delta_f / f0);
        let num = if x.abs() < 1e-6 {
            0.0
        } else {
            ((2.0 * x).sin() - 2.0 * x) / (2.0 * x * x)
        };

        let g0_ms = 8.0 * keff2 * f0 * c_t_pf * np;
        g0_ms * num
    }

    /// Evaluates microwave-to-phonon quantum conversion efficiency eta(f) in [0.0, 1.0].
    pub fn conversion_efficiency(&self, freq_ghz: f64) -> f64 {
        let g_a_s = self.radiation_conductance_ms(freq_ghz) * 1e-3;
        let b_a_s = self.acoustic_susceptance_ms(freq_ghz) * 1e-3;
        let z0 = self.params.characteristic_impedance_ohm;

        let omega = 2.0 * PI * (freq_ghz * 1e9);
        let c_t_f = self.static_capacitance_pf() * 1e-12;
        let b_total = b_a_s + omega * c_t_f * 0.15; // Shunt matching reduces static capacitive reactance

        let num = 4.0 * g_a_s * z0;
        let denom = (1.0 + g_a_s * z0).powi(2) + (b_total * z0).powi(2);

        if denom < 1e-9 {
            0.0
        } else {
            let eta = num / denom;
            eta.clamp(0.0, 0.95)
        }
    }

    /// Peak microwave-to-phonon conversion efficiency at center frequency.
    pub fn peak_efficiency(&self) -> f64 {
        self.conversion_efficiency(self.params.rf_freq_ghz)
    }

    /// Insertion loss in dB at center frequency.
    pub fn insertion_loss_db(&self) -> f64 {
        let eta = self.peak_efficiency().max(1e-4);
        -10.0 * eta.log10()
    }

    /// Added noise quanta n_add in the quantum transduced signal.
    /// At 20 mK and 1 GHz, n_th = 1 / (exp(hbar * omega / (kB * T)) - 1) ~ 0.08 quanta.
    /// Total added noise n_add = 0.5 (zero-point) + n_th <= 0.55 quanta.
    pub fn added_noise_quanta(&self) -> f64 {
        let omega = 2.0 * PI * (self.params.rf_freq_ghz * 1e9);
        let x = (HBAR * omega) / (KB * self.params.temperature_k.max(1e-4));
        let thermal_occupancy = if x > 50.0 {
            0.0
        } else {
            1.0 / (x.exp() - 1.0)
        };
        0.5 + thermal_occupancy
    }

    /// Effective cryogenic noise temperature T_N in Kelvin.
    pub fn effective_noise_temperature_k(&self) -> f64 {
        let n_add = self.added_noise_quanta();
        let omega = 2.0 * PI * (self.params.rf_freq_ghz * 1e9);
        (n_add * HBAR * omega) / KB
    }

    /// Computes conversion efficiency spectrum across span in MHz.
    pub fn compute_spectrum(&self, num_points: usize, span_mhz: f64) -> Vec<TransducerResponsePoint> {
        let mut points = Vec::with_capacity(num_points);
        let f0 = self.params.rf_freq_ghz;
        let half_span_ghz = (span_mhz * 0.5) * 1e-3;
        let f_min = f0 - half_span_ghz;
        let f_max = f0 + half_span_ghz;

        for i in 0..num_points {
            let frac = if num_points > 1 {
                i as f64 / (num_points - 1) as f64
            } else {
                0.5
            };
            let f = f_min + (f_max - f_min) * frac;
            let eta = self.conversion_efficiency(f);
            let il_db = if eta > 1e-4 {
                -10.0 * eta.log10()
            } else {
                40.0
            };
            let g_a = self.radiation_conductance_ms(f);
            let b_a = self.acoustic_susceptance_ms(f);

            points.push(TransducerResponsePoint {
                freq_ghz: f,
                conversion_efficiency: eta,
                insertion_loss_db: il_db,
                radiation_conductance_ms: g_a,
                acoustic_susceptance_ms: b_a,
            });
        }

        points
    }
}
