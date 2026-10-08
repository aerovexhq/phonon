#![deny(unsafe_code)]

//! Phase 447: Universal Chiral Surface Acoustic Wave Quantum Bus Network.
//!
//! Models multi-qudit topological quantum bus interconnects routing quantum information
//! between spatially separated Fibonacci anyon nodes via directional Rayleigh phonons.

/// Simulation parameters for the quantum acoustic bus network.
#[derive(Debug, Clone)]
pub struct QuantumAcousticBusParams {
    pub bus_length_um: f64,
    pub bus_frequency_ghz: f64,
    pub node_count: usize,
    pub piezo_coupling_mhz: f64,
    pub acoustic_attenuation_db_cm: f64,
    pub bus_temperature_mk: f64,
    pub chiral_directivity_ratio: f64,
}

impl Default for QuantumAcousticBusParams {
    fn default() -> Self {
        Self {
            bus_length_um: 120.0,
            bus_frequency_ghz: 2.85,
            node_count: 4,
            piezo_coupling_mhz: 28.5,
            acoustic_attenuation_db_cm: 0.85,
            bus_temperature_mk: 16.0,
            chiral_directivity_ratio: 0.9995, // 99.95% directional forward emission
        }
    }
}

/// Physical metrics evaluating the quantum acoustic bus.
#[derive(Debug, Clone)]
pub struct QuantumAcousticBusMetrics {
    pub state_transfer_fidelity: f64,
    pub chiral_isolation_db: f64,
    pub thermal_phonon_occupancy: f64,
    pub transit_time_ns: f64,
    pub multi_node_entanglement_concurrence: f64,
    pub quantum_bus_coherence_time_us: f64,
}

/// Point on the acoustic pulse envelope during state transit.
#[derive(Debug, Clone)]
pub struct BusWaveformPoint {
    pub time_ns: f64,
    pub node_1_amplitude: f64,
    pub bus_traveling_amplitude: f64,
    pub node_2_amplitude: f64,
}

/// Point on the bus multi-node fidelity spectrum vs bus length.
#[derive(Debug, Clone)]
pub struct BusFidelityDistancePoint {
    pub distance_um: f64,
    pub transfer_fidelity: f64,
    pub entanglement_concurrence: f64,
}

/// Numerical solver for the quantum acoustic bus network.
#[derive(Debug, Clone)]
pub struct QuantumAcousticBusSolver {
    pub params: QuantumAcousticBusParams,
}

impl QuantumAcousticBusSolver {
    pub fn new(params: QuantumAcousticBusParams) -> Self {
        Self { params }
    }

    /// Evaluates quantum performance metrics and coherence.
    pub fn evaluate_metrics(&self) -> QuantumAcousticBusMetrics {
        let p = &self.params;
        let h = 6.62607015e-34;
        let k_b = 1.380649e-23;

        // Thermal occupancy n_th = 1 / (exp(h*f / (k_B*T)) - 1)
        let f_hz = p.bus_frequency_ghz * 1.0e9;
        let t_k = p.bus_temperature_mk * 1.0e-3;
        let x = (h * f_hz) / (k_b * t_k.max(1e-4));
        let n_th = if x > 40.0 { 0.0 } else { 1.0 / (x.exp() - 1.0) };

        // Acoustic transit time tau = L / v_saw (v_saw ~ 2865 m/s)
        let v_saw = 2865.0; // m/s
        let l_m = p.bus_length_um * 1.0e-6;
        let tau_s = l_m / v_saw;
        let tau_ns = tau_s * 1.0e9;

        // Propagation attenuation loss in dB
        let l_cm = p.bus_length_um * 1.0e-4;
        let prop_loss_db = p.acoustic_attenuation_db_cm * l_cm;
        let prop_transmission = 10.0_f64.powf(-prop_loss_db / 10.0);

        // Chiral isolation ISO = 10 * log10(D / (1 - D))
        let d = p.chiral_directivity_ratio.min(0.9999);
        let isolation_db = 10.0 * (d / (1.0 - d)).log10();

        // State transfer fidelity F = prop_transmission * (1 - n_th) * coupling_factor
        let coupling_eff = 1.0 - (1.0 / (1.0 + (p.piezo_coupling_mhz / 1.5).powi(2)));
        let fidelity = (prop_transmission * (1.0 - n_th) * coupling_eff).max(0.0).min(0.9998);

        // Concurrence C ~ sqrt(F) * (1 - 2*n_th)
        let concurrence = (fidelity.sqrt() * (1.0 - 2.0 * n_th)).max(0.0).min(0.995);
        let t_coh_us = 42.0 * (20.0 / p.bus_temperature_mk.max(1.0)).sqrt();

        QuantumAcousticBusMetrics {
            state_transfer_fidelity: fidelity,
            chiral_isolation_db: isolation_db,
            thermal_phonon_occupancy: n_th,
            transit_time_ns: tau_ns,
            multi_node_entanglement_concurrence: concurrence,
            quantum_bus_coherence_time_us: t_coh_us,
        }
    }

    /// Computes spatial-temporal pulse transfer between nodes.
    pub fn compute_pulse_dynamics(&self, points_count: usize) -> Vec<BusWaveformPoint> {
        let metrics = self.evaluate_metrics();
        let tau = metrics.transit_time_ns;
        let mut result = Vec::with_capacity(points_count);

        let total_time_ns = tau * 2.2;
        let step = if points_count > 1 { total_time_ns / (points_count - 1) as f64 } else { 1.0 };

        let t_emit = tau * 0.3;
        let t_arrive = tau * 1.3;
        let sigma = tau * 0.22;

        for i in 0..points_count {
            let t = i as f64 * step;

            // Node 1 decay
            let n1 = (-((t - t_emit * 0.5) / sigma).powi(2)).exp() * (1.0 - (t / (total_time_ns + 1e-6)));
            // Traveling wave on bus
            let bus = (-((t - (t_emit + t_arrive) * 0.5) / (sigma * 1.2)).powi(2)).exp() * 0.96;
            // Node 2 absorption
            let n2 = (-((t - t_arrive) / sigma).powi(2)).exp() * metrics.state_transfer_fidelity;

            result.push(BusWaveformPoint {
                time_ns: t,
                node_1_amplitude: n1.max(0.0),
                bus_traveling_amplitude: bus.max(0.0),
                node_2_amplitude: n2.max(0.0),
            });
        }

        result
    }

    /// Computes transfer fidelity vs distance sweep.
    pub fn compute_fidelity_vs_distance(&self, points_count: usize) -> Vec<BusFidelityDistancePoint> {
        let mut result = Vec::with_capacity(points_count);
        let max_dist = 400.0; // um
        let step = if points_count > 1 { max_dist / (points_count - 1) as f64 } else { 10.0 };

        for i in 0..points_count {
            let d_um = (i as f64 * step).max(1.0);
            let d_cm = d_um * 1.0e-4;
            let loss_db = self.params.acoustic_attenuation_db_cm * d_cm;
            let trans = 10.0_f64.powf(-loss_db / 10.0);
            let fid = (0.998 * trans).max(0.0);
            let conc = (fid.sqrt() * 0.985).max(0.0);

            result.push(BusFidelityDistancePoint {
                distance_um: d_um,
                transfer_fidelity: fid,
                entanglement_concurrence: conc,
            });
        }

        result
    }
}
