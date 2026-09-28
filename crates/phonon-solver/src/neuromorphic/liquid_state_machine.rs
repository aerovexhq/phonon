#![allow(clippy::needless_range_loop)]
//! Spiking Liquid State Machine (LSM) & Memristive Synaptic Plasticity.
//!
//! Formulates:
//! - Recurrent 3D liquid pool of Leaky Integrate-and-Fire (LIF) neurons (`SpikingNeuronModel`, `NeuronState`).
//! - 3D Euclidean distance-dependent cortical microcircuit connectivity:
//!   $$P(i \to j) = C \cdot \exp\left(-\frac{d_{ij}^2}{\lambda^2}\right)$$
//! - Continuous-time synaptic current convolution:
//!   $$I_{syn, i}(t) = \sum_j W_{ij} \sum_k \exp(-(t - t_{j,k})/\tau_s)$$
//! - Online Spike-Timing-Dependent Plasticity (STDP) synaptic weight adaptation (`SpikeTimingPlasticityModel`).
//! - High-dimensional liquid state readout vector $\mathbf{x}(t)$ via low-pass filtering of neuron spike events:
//!   $$x_i(t) = \int_0^t \exp(-(t - s)/\tau_{liquid}) S_i(s) ds$$

use phonon_models::memristor::neuron::{NeuronState, SpikingNeuronModel};
use phonon_models::memristor::reservoir::ReservoirRng;
use phonon_models::memristor::stdp::SpikeTimingPlasticityModel;

/// Configuration for 3D Spiking Liquid State Machine.
#[derive(Debug, Clone, PartialEq)]
pub struct LsmConfig {
    /// 3D liquid pool dimensions $(D_x, D_y, D_z)$.
    pub dimensions: (usize, usize, usize),
    /// Number of input channels.
    pub num_inputs: usize,
    /// Distance scale factor $\lambda$ for connection probability.
    pub lambda_distance: f64,
    /// Baseline connection probability prefactor $C$.
    pub connection_prob_c: f64,
    /// Synaptic current exponential decay time constant $\tau_s$ in seconds ($s$) (e.g. $5\text{ ms}$).
    pub tau_synaptic_s: f64,
    /// Readout state low-pass filter time constant $\tau_{liquid}$ in seconds ($s$) (e.g. $20\text{ ms}$).
    pub tau_liquid_s: f64,
    /// Fraction of inhibitory neurons (typically $0.20$ for 20% GABAergic).
    pub inhibitory_ratio: f64,
    /// Peak post-synaptic current injection per unit synaptic conductance in Amperes ($A$).
    pub synaptic_current_scale: f64,
    /// Enable online STDP learning.
    pub enable_stdp: bool,
    /// Synapse read pulse voltage for energy accounting.
    pub read_voltage_v: f64,
}

impl Default for LsmConfig {
    fn default() -> Self {
        Self {
            dimensions: (3, 3, 3), // 27 liquid neurons
            num_inputs: 1,
            lambda_distance: 2.0,
            connection_prob_c: 0.35,
            tau_synaptic_s: 5.0e-3, // 5 ms
            tau_liquid_s: 20.0e-3,  // 20 ms
            inhibitory_ratio: 0.20, // 20% inhibitory
            synaptic_current_scale: 1.0e-6,
            enable_stdp: true,
            read_voltage_v: 0.10,
        }
    }
}

/// 3D Spiking Liquid State Machine (LSM) with memristive synaptic plasticity.
#[derive(Debug, Clone)]
pub struct LiquidStateMachine {
    /// Configuration parameters.
    pub config: LsmConfig,
    /// Total number of liquid neurons $N = D_x \times D_y \times D_z$.
    pub num_neurons: usize,
    /// Physical neuron models.
    pub neurons: Vec<SpikingNeuronModel>,
    /// Instantaneous states of liquid neurons.
    pub neuron_states: Vec<NeuronState>,
    /// 3D spatial coordinates $(x, y, z)$ of neurons.
    pub neuron_coords: Vec<(f64, f64, f64)>,
    /// Excitatory (true) vs Inhibitory (false) flags.
    pub is_excitatory: Vec<bool>,
    /// Recurrent memristive synaptic weight matrix $W_{rec} \in \mathbb{R}^{N \times N}$ in Siemens ($S$).
    pub w_rec: Vec<Vec<f64>>,
    /// Input synaptic weight matrix $W_{in} \in \mathbb{R}^{N \times N_{in}}$ in Siemens ($S$).
    pub w_in: Vec<Vec<f64>>,
    /// Instantaneous synaptic input currents $I_{syn, i}(t)$ in Amperes ($A$).
    pub i_syn: Vec<f64>,
    /// Low-pass filtered liquid state vector $\mathbf{x}(t) \in \mathbb{R}^N$.
    pub liquid_state: Vec<f64>,
    /// Most recent spike emission timestamps per neuron in seconds ($s$).
    pub last_spike_times_s: Vec<Option<f64>>,
    /// STDP plasticity model.
    pub stdp_model: SpikeTimingPlasticityModel,
    /// Current simulation time in seconds ($s$).
    pub current_time_s: f64,
    /// Total spikes emitted across lifetime.
    pub total_spikes: usize,
    /// Total cumulative energy dissipated in Joules ($J$).
    pub cumulative_energy_joules: f64,
}

impl LiquidStateMachine {
    /// Creates and connects a new 3D Spiking Liquid State Machine.
    pub fn new(config: LsmConfig, seed: u64) -> Self {
        let (dx, dy, dz) = config.dimensions;
        let num_neurons = dx * dy * dz;
        let mut rng = ReservoirRng::new(seed);

        let mut neuron_coords = Vec::with_capacity(num_neurons);
        let mut is_excitatory = Vec::with_capacity(num_neurons);
        let mut neurons = Vec::with_capacity(num_neurons);
        let mut neuron_states = Vec::with_capacity(num_neurons);

        let base_neuron = SpikingNeuronModel::neuromorphic_cmos();

        for z in 0..dz {
            for y in 0..dy {
                for x in 0..dx {
                    neuron_coords.push((x as f64, y as f64, z as f64));
                    let excit = rng.next_f64() >= config.inhibitory_ratio;
                    is_excitatory.push(excit);

                    // Slightly randomize membrane parameters for biological heterogeneity:
                    let mut n = base_neuron.clone();
                    n.v_thresh_volts *= 0.95 + 0.10 * rng.next_f64();
                    n.r_mem_ohms *= 0.90 + 0.20 * rng.next_f64();

                    neurons.push(n.clone());
                    neuron_states.push(NeuronState::new(n.v_rest_volts));
                }
            }
        }

        // Recurrent synaptic connectivity based on 3D Euclidean distance:
        let mut w_rec = vec![vec![0.0; num_neurons]; num_neurons];
        let lambda_sq = config.lambda_distance * config.lambda_distance;
        let g_min = 1.0e-6;
        let g_max = 50.0e-6;

        for i in 0..num_neurons {
            let (xi, yi, zi) = neuron_coords[i];
            for j in 0..num_neurons {
                if i == j {
                    continue; // No direct self-connections
                }
                let (xj, yj, zj) = neuron_coords[j];
                let dist_sq = (xi - xj).powi(2) + (yi - yj).powi(2) + (zi - zj).powi(2);
                let p_conn = config.connection_prob_c * (-dist_sq / lambda_sq).exp();

                if rng.next_f64() < p_conn {
                    let conductance = g_min + (g_max - g_min) * rng.next_f64();
                    w_rec[i][j] = conductance;
                }
            }
        }

        // Input connectivity: map each input channel to a fraction of liquid neurons
        let mut w_in = vec![vec![0.0; config.num_inputs]; num_neurons];
        for i in 0..num_neurons {
            for k in 0..config.num_inputs {
                if rng.next_f64() < 0.40 {
                    w_in[i][k] = g_min + (g_max - g_min) * rng.next_f64();
                }
            }
        }

        let stdp_model = SpikeTimingPlasticityModel::cortical_stdp(g_min, g_max);

        Self {
            config,
            num_neurons,
            neurons,
            neuron_states,
            neuron_coords,
            is_excitatory,
            w_rec,
            w_in,
            i_syn: vec![0.0; num_neurons],
            liquid_state: vec![0.0; num_neurons],
            last_spike_times_s: vec![None; num_neurons],
            stdp_model,
            current_time_s: 0.0,
            total_spikes: 0,
            cumulative_energy_joules: 0.0,
        }
    }

    /// Advances the Liquid State Machine forward by time-step $dt$, returning the continuous liquid state vector $\mathbf{x}(t)$.
    pub fn step(&mut self, input_currents: &[f64], dt_s: f64) -> &[f64] {
        let n = self.num_neurons;
        let decay_syn = (-dt_s / self.config.tau_synaptic_s.max(1e-6)).exp();
        let decay_liq = (-dt_s / self.config.tau_liquid_s.max(1e-6)).exp();
        let mut fired_this_step = vec![false; n];
        let mut step_energy = 0.0;
        let v_read = self.config.read_voltage_v;

        // 1. Decay previous synaptic currents:
        for i in 0..n {
            self.i_syn[i] *= decay_syn;
        }

        // 2. Inject external input currents:
        let m = input_currents.len().min(self.config.num_inputs);
        for i in 0..n {
            let mut ext_i = 0.0;
            for (k, &u_val) in input_currents.iter().take(m).enumerate() {
                let g = self.w_in[i][k];
                if g > 0.0 {
                    let injected = g * u_val * self.config.synaptic_current_scale;
                    ext_i += injected;
                    step_energy += v_read * v_read * g * dt_s;
                }
            }
            self.i_syn[i] += ext_i;
        }

        // 3. Update each LIF neuron state:
        for i in 0..n {
            let total_i = self.i_syn[i];
            let (next_state, spike) = self.neurons[i].step(
                self.neuron_states[i],
                total_i,
                i as u32,
                self.current_time_s,
                dt_s,
            );
            self.neuron_states[i] = next_state;

            if spike.is_some() {
                fired_this_step[i] = true;
                self.total_spikes += 1;
            }
        }

        // 4. Propagate emitted spikes through recurrent memristive synapses:
        for pre in 0..n {
            if fired_this_step[pre] {
                let sign = if self.is_excitatory[pre] { 1.0 } else { -1.0 };
                let t_pre = self.current_time_s;

                for post in 0..n {
                    let g_syn = self.w_rec[post][pre];
                    if g_syn > 0.0 {
                        // Current injection kick to post-synaptic neuron:
                        let delta_i = sign * g_syn * self.config.synaptic_current_scale;
                        self.i_syn[post] += delta_i;

                        // Energy accounting: E_syn = V^2 * G * dt
                        let e_pulse = v_read * v_read * g_syn * 10.0e-9;
                        step_energy += e_pulse;

                        // Online STDP weight adaptation:
                        if self.config.enable_stdp {
                            if let Some(t_post) = self.last_spike_times_s[post] {
                                let delta_t = t_post - t_pre;
                                self.w_rec[post][pre] =
                                    self.stdp_model.update_weight(g_syn, delta_t);
                            }
                        }
                    }
                }
                self.last_spike_times_s[pre] = Some(t_pre);
            }
        }

        // 5. Update high-dimensional liquid state readout vector:
        // x_i[t + dt] = x_i[t] * exp(-dt / tau_liquid) + (1.0 if spike else 0.0)
        for i in 0..n {
            let spike_inc = if fired_this_step[i] { 1.0 } else { 0.0 };
            self.liquid_state[i] = self.liquid_state[i] * decay_liq + spike_inc;
        }

        self.cumulative_energy_joules += step_energy;
        self.current_time_s += dt_s;

        &self.liquid_state
    }

    /// Resets all neurons and internal traces to resting baseline.
    pub fn reset(&mut self) {
        for i in 0..self.num_neurons {
            self.neuron_states[i] = NeuronState::new(self.neurons[i].v_rest_volts);
            self.i_syn[i] = 0.0;
            self.liquid_state[i] = 0.0;
            self.last_spike_times_s[i] = None;
        }
        self.current_time_s = 0.0;
    }
}
