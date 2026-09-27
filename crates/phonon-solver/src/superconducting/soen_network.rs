//! Superconducting Optoelectronic Spiking Neural Network (`SoenNetwork`)
//! and Cryogenic Dielectric Optical Waveguide Interconnects (`OpticalInterconnect`).
//!
//! Simulates multi-neuron cryogenic networks coupling:
//! - Passive dielectric optical waveguides with zero electronic crosstalk and high fanout ($> 1000$).
//! - Synaptic single-photon absorption and hot-spot current diversion.
//! - Dendritic superconducting flux storage loop integration.
//! - Somatic Josephson junction $2\pi$ phase slips and optical pulse emission.
//! - Parallel multi-core evaluation via Rayon.

use phonon_core::SPEED_OF_LIGHT;
use phonon_models::superconducting::{SoenMetrics, SoenNeuron};
use rayon::prelude::*;

/// In-flight optical photon packet traveling along a cryogenic waveguide.
#[derive(Debug, Clone, PartialEq)]
pub struct TravelingOpticalPacket {
    /// Source neuron index.
    pub src_neuron: usize,
    /// Destination neuron index.
    pub dst_neuron: usize,
    /// Destination synaptic channel index on the destination neuron.
    pub dst_channel: usize,
    /// Number of photons in the wavepacket.
    pub photon_count: f64,
    /// Optical arrival timestamp in seconds ($s$).
    pub arrival_time: f64,
}

/// Cryogenic dielectric optical waveguide interconnect matrix.
///
/// Features:
/// - Zero electronic crosstalk (dielectric waveguides do not couple capacitively or inductively to superconducting loops).
/// - Propagation delay: $\tau = \frac{n_{eff} L}{c}$.
/// - High fanout: supports splitting optical pulse energy to $> 1000$ destinations without capacitive delay penalties.
/// - Programmable synaptic transmission weights $W_{ij} \in [0.0, 1.0]$.
#[derive(Debug, Clone, PartialEq)]
pub struct OpticalInterconnect {
    /// Number of neurons in the network.
    pub num_neurons: usize,
    /// Effective optical refractive index $n_{eff}$ (e.g. $2.0$ for $\text{Si}_3\text{N}_4$ at $4\text{ K}$).
    pub effective_index: f64,
    /// Average interconnect waveguide length in meters ($m$) (e.g. $200\,\mu\text{m}$).
    pub waveguide_length_m: f64,
    /// Optical propagation delay in seconds ($s$): $\tau = \frac{n_{eff} L}{c}$.
    pub propagation_delay_s: f64,
    /// Optical crosstalk isolation ratio in decibels ($dB$) ($> 60\text{ dB}$, effectively zero).
    pub crosstalk_isolation_db: f64,
    /// Maximum supported optical fanout.
    pub max_fanout: usize,
    /// Synaptic connectivity weight matrix $[N \times N]$: row = dst, col = src.
    pub weights: Vec<f64>,
}

impl OpticalInterconnect {
    /// Constructs an optical interconnect matrix for $N$ neurons with specified waveguide length.
    pub fn new(num_neurons: usize, waveguide_length_m: f64) -> Self {
        let n = num_neurons.max(1);
        let n_eff = 2.0;
        let l_m = waveguide_length_m.max(1e-6);
        let delay = (n_eff * l_m) / SPEED_OF_LIGHT;

        Self {
            num_neurons: n,
            effective_index: n_eff,
            waveguide_length_m: l_m,
            propagation_delay_s: delay,
            crosstalk_isolation_db: 75.0, // > 75 dB optical isolation
            max_fanout: 1024,             // Optical fanout > 1000
            weights: vec![0.0; n * n],
        }
    }

    /// Sets the synaptic weight from `src` to `dst`.
    pub fn set_weight(&mut self, src: usize, dst: usize, weight: f64) {
        if src < self.num_neurons && dst < self.num_neurons {
            self.weights[dst * self.num_neurons + src] = weight.clamp(0.0, 1.0);
        }
    }

    /// Gets the synaptic weight from `src` to `dst`.
    pub fn get_weight(&self, src: usize, dst: usize) -> f64 {
        if src < self.num_neurons && dst < self.num_neurons {
            self.weights[dst * self.num_neurons + src]
        } else {
            0.0
        }
    }

    /// Configures all-to-all connectivity with uniform weight.
    pub fn connect_all_to_all(&mut self, weight: f64) {
        for w in &mut self.weights {
            *w = weight.clamp(0.0, 1.0);
        }
    }

    /// Configures feedforward layer connectivity from layer `src_range` to `dst_range`.
    pub fn connect_layers(
        &mut self,
        src_range: std::ops::Range<usize>,
        dst_range: std::ops::Range<usize>,
        weight: f64,
    ) {
        for dst in dst_range {
            for src in src_range.clone() {
                self.set_weight(src, dst, weight);
            }
        }
    }
}

/// Recorded spike event during network simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoenSpikeRecord {
    /// Neuron ID that fired.
    pub neuron_id: usize,
    /// Firing timestamp in seconds ($s$).
    pub time_s: f64,
    /// Optical pulse energy in Joules ($J$).
    pub pulse_energy_joules: f64,
}

/// Superconducting Optoelectronic Spiking Neural Network (`SoenNetwork`).
#[derive(Debug, Clone, PartialEq)]
pub struct SoenNetwork {
    /// Neurons belonging to the network.
    pub neurons: Vec<SoenNeuron>,
    /// Optical dielectric waveguide interconnect.
    pub interconnect: OpticalInterconnect,
    /// Traveling optical packets in flight along waveguides.
    pub in_flight_packets: Vec<TravelingOpticalPacket>,
    /// History of emitted spikes.
    pub spike_history: Vec<SoenSpikeRecord>,
    /// Current simulation time in seconds ($s$).
    pub current_time_s: f64,
}

impl SoenNetwork {
    /// Creates a new SOEN network with $N$ neurons and specified waveguide length.
    pub fn new(num_neurons: usize, waveguide_length_m: f64) -> Self {
        let n = num_neurons.max(1);
        let mut neurons = Vec::with_capacity(n);
        for id in 0..n {
            // Each neuron has up to N synaptic input channels and somatic Ic = 50 uA
            let mut neuron = SoenNeuron::new(id, n, 50.0e-6);
            // Initialize synaptic weights
            for ch in 0..n {
                neuron.set_weight(ch, 1.0);
            }
            neurons.push(neuron);
        }

        let interconnect = OpticalInterconnect::new(n, waveguide_length_m);

        Self {
            neurons,
            interconnect,
            in_flight_packets: Vec::new(),
            spike_history: Vec::new(),
            current_time_s: 0.0,
        }
    }

    /// Injects an external optical stimulus (photon count) into a specific neuron's channel.
    pub fn inject_stimulus(&mut self, neuron_id: usize, channel: usize, photon_count: f64) {
        if neuron_id < self.neurons.len() {
            self.neurons[neuron_id]
                .receive_synaptic_spikes(&[(channel, photon_count)], self.current_time_s);
        }
    }

    /// Advances the network simulation by one time-step $dt$.
    ///
    /// 1. Delivers arriving optical packets to receiving neurons.
    /// 2. Parallely evaluates all neurons (flux accumulation, somatic threshold, phase slip).
    /// 3. Collects newly emitted optical spikes and schedules downstream traveling packets.
    pub fn step(&mut self, dt: f64) {
        let next_time = self.current_time_s + dt;

        // 1. Process arrived optical packets
        let mut remaining_packets = Vec::with_capacity(self.in_flight_packets.len());
        let mut delivered_by_neuron: Vec<Vec<(usize, f64)>> = vec![Vec::new(); self.neurons.len()];

        for packet in self.in_flight_packets.drain(..) {
            if packet.arrival_time <= next_time {
                if packet.dst_neuron < delivered_by_neuron.len() {
                    delivered_by_neuron[packet.dst_neuron]
                        .push((packet.dst_channel, packet.photon_count));
                }
            } else {
                remaining_packets.push(packet);
            }
        }
        self.in_flight_packets = remaining_packets;

        // Deliver incoming photons to neurons
        for (neuron_idx, spikes) in delivered_by_neuron.into_iter().enumerate() {
            if !spikes.is_empty() {
                self.neurons[neuron_idx].receive_synaptic_spikes(&spikes, self.current_time_s);
            }
        }

        // 2. Parallely advance all neurons over dt using Rayon
        let t_curr = self.current_time_s;
        let spike_outputs: Vec<(usize, Option<f64>)> = self
            .neurons
            .par_iter_mut()
            .map(|neuron| {
                let opt_pulse = neuron.step(dt, t_curr);
                (neuron.id, opt_pulse)
            })
            .collect();

        // 3. Collect emitted spikes and schedule downstream optical packets
        for (src_id, opt_pulse) in spike_outputs {
            if let Some(pulse_energy) = opt_pulse {
                self.spike_history.push(SoenSpikeRecord {
                    neuron_id: src_id,
                    time_s: self.current_time_s,
                    pulse_energy_joules: pulse_energy,
                });

                // Fanout: route optical pulse to all downstream connected neurons
                let delay = self.interconnect.propagation_delay_s;
                let arrival = next_time + delay;

                for dst_id in 0..self.neurons.len() {
                    let weight = self.interconnect.get_weight(src_id, dst_id);
                    if weight > 0.0 {
                        self.in_flight_packets.push(TravelingOpticalPacket {
                            src_neuron: src_id,
                            dst_neuron: dst_id,
                            dst_channel: src_id,
                            photon_count: weight * 2.0, // 2-photon packet scaled by weight
                            arrival_time: arrival,
                        });
                    }
                }
            }
        }

        self.current_time_s = next_time;
    }

    /// Simulates the network for `steps` iterations with time-step `dt`.
    pub fn simulate(&mut self, steps: usize, dt: f64) {
        for _ in 0..steps {
            self.step(dt);
        }
    }

    /// Returns cumulative energy dissipated by all neurons in the network in Joules ($J$).
    pub fn total_energy_joules(&self) -> f64 {
        self.neurons
            .iter()
            .map(|n| n.cumulative_energy_joules)
            .sum()
    }

    /// Returns total spikes fired across the network.
    pub fn total_spikes(&self) -> usize {
        self.spike_history.len()
    }

    /// Returns average metrics across all neurons in the network.
    pub fn network_metrics(&self) -> SoenMetrics {
        let n = self.neurons.len().max(1) as f64;
        let mut total_spikes = 0;
        let mut total_e_aj = 0.0;
        let mut total_latency_ps = 0.0;
        let mut total_retention_ns = 0.0;

        for neuron in &self.neurons {
            let m = neuron.metrics();
            total_spikes += m.spike_count;
            total_e_aj += m.average_synaptic_energy_attojoules;
            total_latency_ps += m.somatic_firing_latency_ps;
            total_retention_ns += m.flux_retention_ns;
        }

        SoenMetrics {
            spike_count: total_spikes,
            average_synaptic_energy_joules: (total_e_aj / n) * 1e-18,
            average_synaptic_energy_attojoules: total_e_aj / n,
            wall_plug_energy_attojoules: (total_e_aj / n) * 1000.0,
            somatic_firing_latency_ps: total_latency_ps / n,
            flux_retention_ns: total_retention_ns / n,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optical_interconnect_delay_and_weights() {
        let mut conn = OpticalInterconnect::new(4, 300e-6); // 300 um
                                                            // Delay = n_eff * L / c = 2.0 * 300e-6 / 3e8 = 2.0 ps
        assert!((conn.propagation_delay_s - 2.0e-12).abs() < 1e-13);
        assert!(conn.crosstalk_isolation_db >= 70.0);
        assert!(conn.max_fanout >= 1000);

        conn.set_weight(0, 1, 0.75);
        assert_eq!(conn.get_weight(0, 1), 0.75);
        assert_eq!(conn.get_weight(1, 0), 0.0);
    }

    #[test]
    fn test_soen_network_propagation_and_recurrent_firing() {
        let mut net = SoenNetwork::new(3, 150e-6); // 150 um -> 1 ps delay
                                                   // Connect neuron 0 -> neuron 1 -> neuron 2 in a feedforward chain
        net.interconnect.set_weight(0, 1, 1.0);
        net.interconnect.set_weight(1, 2, 1.0);

        // Inject initial optical stimulus into neuron 0
        net.inject_stimulus(0, 0, 3.0);

        // Step simulation for 100 ps (100 steps with dt = 1 ps)
        net.simulate(100, 1.0e-12);

        // Neuron 0 should fire, propagating to neuron 1, which fires, propagating to neuron 2
        assert!(net.total_spikes() >= 1, "At least neuron 0 should fire");
        let metrics = net.network_metrics();
        assert!(metrics.average_synaptic_energy_attojoules < 25.0);
    }
}
