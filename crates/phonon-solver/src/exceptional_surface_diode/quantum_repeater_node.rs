#![deny(unsafe_code)]

//! Unidirectional Quantum Repeater Node & Memory Protection Solver.
//!
//! Models an acoustic quantum repeater node protected by the non-Hermitian chiral diode,
//! shielding cryogenic acoustic quantum memories from backscattered thermal and control noise,
//! maximizing entangled Bell pair fidelity F >= 98.0%, memory coherence T2* >= 50.0 us,
//! and secret key distillation rate R_key >= 1.0 kbps.

/// Parameters for the quantum repeater station.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumRepeaterParams {
    /// Inter-node physical link distance in kilometers (default ~25.0 km).
    pub node_distance_km: f64,
    /// Electromechanical transducer single-phonon coupling rate in kHz (default ~750.0 kHz).
    pub single_phonon_coupling_khz: f64,
    /// Intrinsic acoustic quantum memory coherence time T2* in microseconds (default ~65.0 us).
    pub coherence_time_t2_us: f64,
    /// Channel link attenuation in dB/km (default ~0.18 dB/km).
    pub fiber_attenuation_db_per_km: f64,
    /// Single-phonon detector quantum efficiency in [0.0, 1.0] (default ~0.92).
    pub detector_efficiency: f64,
    /// Cryogenic dilution refrigerator temperature in milliKelvin (default ~20.0 mK).
    pub ambient_temperature_mk: f64,
    /// Acoustic mode center frequency in GHz (default ~5.0 GHz).
    pub center_freq_ghz: f64,
    /// Number of discrete time steps for memory decay profile (default 51).
    pub time_steps: usize,
}

impl Default for QuantumRepeaterParams {
    fn default() -> Self {
        Self {
            node_distance_km: 25.0,
            single_phonon_coupling_khz: 750.0,
            coherence_time_t2_us: 65.0,
            fiber_attenuation_db_per_km: 0.18,
            detector_efficiency: 0.92,
            ambient_temperature_mk: 20.0,
            center_freq_ghz: 5.0,
            time_steps: 51,
        }
    }
}

/// Dynamic quantum repeater performance metrics at timestamp t.
#[derive(Debug, Clone, PartialEq)]
pub struct RepeaterTimePoint {
    pub time_us: f64,
    /// Entangled Bell state fidelity F(t).
    pub bell_fidelity: f64,
    /// Acoustic quantum memory survival probability.
    pub memory_survival_prob: f64,
    /// Instantaneous secret key rate in bits per second.
    pub secret_key_rate_bps: f64,
}

/// Summary metrics of the solved quantum repeater node.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumRepeaterMetrics {
    /// Initial entangled Bell pair fidelity (F >= 0.980 required).
    pub bell_pair_fidelity: f64,
    /// Entanglement generation rate in Hertz.
    pub entanglement_generation_rate_hz: f64,
    /// Effective acoustic quantum memory coherence time T2* in microseconds (>= 50.0 us required).
    pub quantum_memory_t2_us: f64,
    /// Distilled secret key generation rate in kilobits per second (>= 1.0 kbps required).
    pub secret_key_rate_kbps: f64,
    /// Entanglement swapping process fidelity in [0.0, 1.0].
    pub swapping_fidelity: f64,
    /// Suppression of backscattered control and thermal noise photons in dB (>= 35.0 dB required).
    pub backscatter_noise_suppression_db: f64,
    /// Ambient thermal phonon occupancy n_th at 20 mK.
    pub cryogenic_thermal_occupancy: f64,
    /// Single-photon / single-phonon link transmission fraction.
    pub link_transmission_fraction: f64,
}

/// Solver for unidirectional quantum repeater dynamics and noise shielding.
#[derive(Debug, Clone, PartialEq)]
pub struct QuantumRepeaterSolver {
    pub params: QuantumRepeaterParams,
}

impl Default for QuantumRepeaterSolver {
    fn default() -> Self {
        Self {
            params: QuantumRepeaterParams::default(),
        }
    }
}

impl QuantumRepeaterSolver {
    pub fn new(params: QuantumRepeaterParams) -> Self {
        Self { params }
    }

    /// Evaluates binary entropy function h2(p) = -p*log2(p) - (1-p)*log2(1-p).
    fn binary_entropy(p: f64) -> f64 {
        if p <= 0.0 || p >= 1.0 {
            0.0
        } else {
            -p * p.log2() - (1.0 - p) * (1.0 - p).log2()
        }
    }

    /// Evaluates Bose-Einstein thermal phonon distribution at center frequency f0 and temperature T.
    pub fn thermal_phonon_occupancy(&self) -> f64 {
        let h = 6.62607015e-34; // J*s
        let kb = 1.380649e-23; // J/K
        let f0 = self.params.center_freq_ghz * 1e9;
        let t_kelvin = (self.params.ambient_temperature_mk * 1e-3).max(1e-6);

        let exponent = (h * f0) / (kb * t_kelvin);
        if exponent > 80.0 {
            0.0
        } else {
            1.0 / (exponent.exp() - 1.0)
        }
    }

    /// Solves the quantum repeater node protected by the non-Hermitian chiral diode.
    pub fn solve_repeater(&self, diode_isolation_db: f64) -> (QuantumRepeaterMetrics, Vec<RepeaterTimePoint>) {
        let dist_km = self.params.node_distance_km;
        let loss_db = self.params.fiber_attenuation_db_per_km * dist_km;
        let link_trans = 10.0f64.powf(-loss_db / 10.0);
        let det_eff = self.params.detector_efficiency;
        let t2_us = self.params.coherence_time_t2_us;
        let n_th = self.thermal_phonon_occupancy();

        // Speed of light in optical/phononic waveguide ~ 2.0e8 m/s
        let c_link = 2.0e5; // km / s
        let round_trip_time_s = (2.0 * dist_km) / c_link;
        let rep_rate_hz = 1.0 / round_trip_time_s.max(1e-6);

        // Heradled entanglement generation probability per attempt
        let p_pair = link_trans * det_eff * det_eff;
        let ent_rate_hz = rep_rate_hz * p_pair;

        // Backscatter isolation impact:
        // An unshielded node suffers backscattered pump photons causing dephasing:
        // delta_dephasing = 10^(-diode_isolation_db / 10)
        let backscatter_leakage = 10.0f64.powf(-diode_isolation_db / 10.0);
        let effective_t2 = t2_us / (1.0 + backscatter_leakage * 200.0);

        // Intrinsic Bell state fidelity protected by diode isolation
        let raw_fidelity = 0.995;
        let bell_fid = (raw_fidelity * (1.0 - backscatter_leakage * 0.5) - n_th).min(0.999).max(0.50);

        // Entanglement swapping process fidelity
        let swap_fid = (bell_fid * 0.985).min(0.99);

        // Secret key distillation rate via Shor-Preskill / Devetak-Winter bound:
        // R_key = R_ent * max(0, 1 - 2*h2(e_bit))
        let e_bit = (1.0 - bell_fid) / 2.0;
        let key_fraction = (1.0 - 2.0 * Self::binary_entropy(e_bit)).max(0.0);
        let key_rate_bps = ent_rate_hz * key_fraction;
        let key_rate_kbps = key_rate_bps / 1000.0;

        // Dynamic time trace over [0, 2*T2]
        let n_time = self.params.time_steps.max(11);
        let t_max = 2.0 * effective_t2;
        let dt = t_max / ((n_time - 1) as f64);
        let mut time_points = Vec::with_capacity(n_time);

        for step in 0..n_time {
            let t = (step as f64) * dt;
            let decay = (-t / effective_t2).exp();
            let fid_t = 0.5 + (bell_fid - 0.5) * decay;
            let p_surv = decay;
            let inst_ebit = (1.0 - fid_t) / 2.0;
            let inst_key_bps = ent_rate_hz * (1.0 - 2.0 * Self::binary_entropy(inst_ebit)).max(0.0);

            time_points.push(RepeaterTimePoint {
                time_us: t,
                bell_fidelity: fid_t,
                memory_survival_prob: p_surv,
                secret_key_rate_bps: inst_key_bps,
            });
        }

        let metrics = QuantumRepeaterMetrics {
            bell_pair_fidelity: bell_fid,
            entanglement_generation_rate_hz: ent_rate_hz,
            quantum_memory_t2_us: effective_t2,
            secret_key_rate_kbps: key_rate_kbps,
            swapping_fidelity: swap_fid,
            backscatter_noise_suppression_db: diode_isolation_db,
            cryogenic_thermal_occupancy: n_th,
            link_transmission_fraction: link_trans,
        };

        (metrics, time_points)
    }
}
