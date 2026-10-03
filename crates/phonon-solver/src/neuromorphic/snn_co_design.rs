#![deny(unsafe_code)]

//! Memristor Dynamic State Variable Solver, Crossbar Array, and STDP SNN Engine.
//!
//! Provides non-linear boundary window functions (Linear, Joglekar, Biolek),
//! memristor state variable drift integration, Leaky Integrate-and-Fire (LIF)
//! spiking neuron dynamics, Vector-Matrix Multiplication (VMM) dendritic currents,
//! and online Spike-Timing-Dependent Plasticity (STDP) for neuromorphic hardware co-design.

/// Window function model for non-linear boundary effects in memristor ion drift.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowFunction {
    /// Linear ion drift without boundary suppression.
    Linear,
    /// Joglekar window function: f(w) = 1 - (2w - 1)^(2p).
    Joglekar { p: f64 },
    /// Biolek window function: f(w, v) = 1 - (w - stp(-v))^(2p), resolving boundary lock.
    Biolek { p: f64 },
}

impl Default for WindowFunction {
    fn default() -> Self {
        Self::Biolek { p: 2.0 }
    }
}

impl WindowFunction {
    /// Evaluates the window factor f(w, v) \in [0.0, 1.0].
    pub fn evaluate(&self, w: f64, voltage: f64) -> f64 {
        let w_clamped = w.clamp(0.0, 1.0);
        match *self {
            Self::Linear => 1.0,
            Self::Joglekar { p } => {
                let term = (2.0 * w_clamped - 1.0).abs();
                let f = 1.0 - term.powf(2.0 * p.max(0.5));
                f.clamp(0.0, 1.0)
            }
            Self::Biolek { p } => {
                // stp(-v) is 1.0 if voltage < 0, 0.0 if voltage >= 0
                let stp = if voltage < 0.0 { 1.0 } else { 0.0 };
                let term = (w_clamped - stp).abs();
                let f = 1.0 - term.powf(2.0 * p.max(0.5));
                f.clamp(0.0, 1.0)
            }
        }
    }
}

/// Dynamic state variable model for a non-volatile memristor device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MemristorState {
    /// Normalized state variable w \in [0.0, 1.0] representing conductive filament length.
    pub w: f64,
    /// Minimum resistance (low-resistance state LRS / on-state) in Ohms.
    pub r_on: f64,
    /// Maximum resistance (high-resistance state HRS / off-state) in Ohms.
    pub r_off: f64,
    /// Dopant ion mobility parameter mu_v in m^2 / (V * s).
    pub mobility: f64,
    /// Oxide device active layer thickness D in meters.
    pub thickness: f64,
    /// Selected boundary window function.
    pub window_func: WindowFunction,
    /// Programming threshold voltage V_th in Volts.
    pub v_th: f64,
}

impl Default for MemristorState {
    fn default() -> Self {
        Self {
            w: 0.2,
            r_on: 1_000.0,      // 1 kOhm
            r_off: 100_000.0,   // 100 kOhm
            mobility: 1.0e-14,  // 1e-14 m^2 / (V*s)
            thickness: 10.0e-9, // 10 nm
            window_func: WindowFunction::Biolek { p: 2.0 },
            v_th: 0.1,          // 100 mV threshold
        }
    }
}

impl MemristorState {
    /// Creates a new memristor state variable instance.
    pub fn new(
        w: f64,
        r_on: f64,
        r_off: f64,
        mobility: f64,
        thickness: f64,
        window_func: WindowFunction,
        v_th: f64,
    ) -> Self {
        Self {
            w: w.clamp(0.0, 1.0),
            r_on: r_on.max(1e-3),
            r_off: r_off.max(r_on),
            mobility,
            thickness: thickness.max(1e-12),
            window_func,
            v_th: v_th.max(0.0),
        }
    }

    /// Computes instantaneous chord conductance G(w) = w / R_on + (1 - w) / R_off in Siemens (S).
    #[inline]
    pub fn conductance(&self) -> f64 {
        self.w / self.r_on + (1.0 - self.w) / self.r_off
    }

    /// Computes instantaneous resistance R(w) = 1 / G(w) in Ohms.
    #[inline]
    pub fn resistance(&self) -> f64 {
        1.0 / self.conductance().max(1e-18)
    }

    /// Advances the memristor state variable w via continuous ion drift dw/dt with window function and threshold.
    pub fn step(&mut self, voltage: f64, dt: f64) {
        if voltage.abs() <= self.v_th {
            return;
        }

        let v_eff = if voltage > 0.0 {
            voltage - self.v_th
        } else {
            voltage + self.v_th
        };

        let f_w = self.window_func.evaluate(self.w, voltage);
        let d_sq = (self.thickness * self.thickness).max(1e-24);
        let dw_dt = (self.mobility / d_sq) * v_eff * f_w;
        self.w = (self.w + dw_dt * dt).clamp(0.0, 1.0);
    }

    /// Directly applies an STDP synaptic state delta \Delta w \in [-1.0, 1.0].
    pub fn apply_weight_change(&mut self, delta_w: f64) {
        self.w = (self.w + delta_w).clamp(0.0, 1.0);
    }
}

/// Leaky Integrate-and-Fire (LIF) neuromorphic neuron model with refractory dynamics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LifNeuron {
    /// Membrane capacitance C_m in Farads (F).
    pub c_m: f64,
    /// Membrane leak resistance R_leak in Ohms (\Omega).
    pub r_leak: f64,
    /// Resting membrane potential V_rest in Volts (V).
    pub v_rest: f64,
    /// Action potential firing threshold V_th in Volts (V).
    pub v_th: f64,
    /// Post-spike reset membrane potential V_reset in Volts (V).
    pub v_reset: f64,
    /// Absolute refractory period \tau_ref in seconds (s).
    pub tau_ref: f64,
    /// Instantaneous membrane potential V in Volts (V).
    pub v: f64,
    /// Remaining refractory time in seconds (s).
    pub refractory_remaining: f64,
}

impl Default for LifNeuron {
    fn default() -> Self {
        Self {
            c_m: 1.0e-7,      // 100 nF
            r_leak: 10_000.0, // 10 kOhm -> tau_m = 1 ms
            v_rest: 0.0,      // 0 V
            v_th: 1.0,        // 1 V
            v_reset: 0.0,     // 0 V
            tau_ref: 0.002,   // 2 ms refractory
            v: 0.0,
            refractory_remaining: 0.0,
        }
    }
}

impl LifNeuron {
    /// Creates a new LIF neuron instance.
    pub fn new(
        c_m: f64,
        r_leak: f64,
        v_rest: f64,
        v_th: f64,
        v_reset: f64,
        tau_ref: f64,
    ) -> Self {
        Self {
            c_m: c_m.max(1e-15),
            r_leak: r_leak.max(1.0),
            v_rest,
            v_th,
            v_reset,
            tau_ref: tau_ref.max(0.0),
            v: v_rest,
            refractory_remaining: 0.0,
        }
    }

    /// Advances the membrane potential by integration step dt with injected dendritic current input_current (A).
    /// Returns true if an action potential spike was emitted.
    pub fn step(&mut self, input_current: f64, dt: f64) -> bool {
        if self.refractory_remaining > 0.0 {
            self.refractory_remaining = (self.refractory_remaining - dt).max(0.0);
            self.v = self.v_reset;
            return false;
        }

        // Leaky integration: C_m * dV/dt = -(V - V_rest)/R_leak + I
        let tau_m = (self.r_leak * self.c_m).max(1e-12);
        let dv = (-(self.v - self.v_rest) / tau_m + input_current / self.c_m.max(1e-15)) * dt;
        self.v += dv;

        if self.v >= self.v_th {
            self.v = self.v_reset;
            self.refractory_remaining = self.tau_ref;
            true
        } else {
            false
        }
    }

    /// Resets neuron potential to resting level and clears refractory timer.
    pub fn reset(&mut self) {
        self.v = self.v_rest;
        self.refractory_remaining = 0.0;
    }
}

/// Online Spike-Timing-Dependent Plasticity (STDP) exponential kernel parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StdpParams {
    /// Maximum potentiation amplitude A_+ for pre-before-post spikes.
    pub a_plus: f64,
    /// Maximum depression amplitude A_- for post-before-pre spikes.
    pub a_minus: f64,
    /// Time constant \tau_+ for potentiation in seconds.
    pub tau_plus: f64,
    /// Time constant \tau_- for depression in seconds.
    pub tau_minus: f64,
}

impl Default for StdpParams {
    fn default() -> Self {
        Self {
            a_plus: 0.05,
            a_minus: 0.025,
            tau_plus: 0.020, // 20 ms
            tau_minus: 0.020, // 20 ms
        }
    }
}

impl StdpParams {
    /// Creates a new STDP parameter instance.
    pub fn new(a_plus: f64, a_minus: f64, tau_plus: f64, tau_minus: f64) -> Self {
        Self {
            a_plus: a_plus.max(0.0),
            a_minus: a_minus.max(0.0),
            tau_plus: tau_plus.max(1e-6),
            tau_minus: tau_minus.max(1e-6),
        }
    }

    /// Computes \Delta w given \Delta t = t_post - t_pre.
    /// If \Delta t > 0: A_+ * exp(-\Delta t / \tau_+) (LTP)
    /// If \Delta t < 0: -A_- * exp(\Delta t / \tau_-) (LTD)
    pub fn delta_w(&self, dt: f64) -> f64 {
        if dt > 0.0 {
            self.a_plus * (-dt / self.tau_plus).exp()
        } else if dt < 0.0 {
            -self.a_minus * (dt / self.tau_minus).exp()
        } else {
            0.0
        }
    }
}

/// Simulation trajectory results from an SNN crossbar run.
#[derive(Debug, Clone, PartialEq)]
pub struct SnnTrajectory {
    /// Discrete simulation timestamps in seconds.
    pub timestamps: Vec<f64>,
    /// Membrane potential traces V_j(t) for each output neuron [neuron_idx][time_step].
    pub membrane_potentials: Vec<Vec<f64>>,
    /// Input spike events: (timestamp, input_channel).
    pub input_spikes: Vec<(f64, usize)>,
    /// Output spike events: (timestamp, output_neuron_idx).
    pub output_spikes: Vec<(f64, usize)>,
    /// Initial synaptic conductances before simulation run [M][N].
    pub initial_conductances: Vec<Vec<f64>>,
    /// Final synaptic conductances after STDP adaptation [M][N].
    pub final_conductances: Vec<Vec<f64>>,
    /// Total energy dissipated per synaptic operation in femtojoules (fJ/SOP).
    pub energy_per_sop_fj: f64,
    /// Total energy dissipated across all junctions during simulation in Joules.
    pub total_energy_joules: f64,
    /// Total synaptic operations executed across all crossbar junctions.
    pub total_synaptic_ops: usize,
    /// Average firing frequency for each output neuron in Hertz (Hz).
    pub output_spike_rates_hz: Vec<f64>,
    /// Mean synaptic conductance across all junctions in Siemens (S).
    pub mean_conductance_s: f64,
}

/// Memristor crossbar array coupled to LIF output neurons with STDP plasticity.
#[derive(Debug, Clone, PartialEq)]
pub struct SpikingCrossbarNetwork {
    /// Number of pre-synaptic input lines M.
    pub num_inputs: usize,
    /// Number of post-synaptic output lines N.
    pub num_outputs: usize,
    /// M x N matrix of memristive synaptic junction devices.
    pub crossbar: Vec<Vec<MemristorState>>,
    /// N post-synaptic LIF neurons.
    pub neurons: Vec<LifNeuron>,
    /// STDP learning parameters.
    pub stdp_params: StdpParams,
    /// Pre-synaptic spike pulse amplitude in Volts.
    pub pulse_voltage: f64,
    /// Pre-synaptic spike pulse duration in seconds.
    pub pulse_duration: f64,
    /// Plasticity enable flag.
    pub enable_stdp: bool,
}

impl SpikingCrossbarNetwork {
    /// Creates a new spiking crossbar network with M inputs and N outputs.
    pub fn new(num_inputs: usize, num_outputs: usize) -> Self {
        let m = num_inputs.max(1);
        let n = num_outputs.max(1);
        let mut crossbar = Vec::with_capacity(m);
        for i in 0..m {
            let mut row = Vec::with_capacity(n);
            for j in 0..n {
                let base_w = 0.2 + 0.1 * ((i + j) % 3) as f64;
                let mut cell = MemristorState::default();
                cell.w = base_w.clamp(0.05, 0.95);
                row.push(cell);
            }
            crossbar.push(row);
        }

        let neurons = (0..n).map(|_| LifNeuron::default()).collect();

        Self {
            num_inputs: m,
            num_outputs: n,
            crossbar,
            neurons,
            stdp_params: StdpParams::default(),
            pulse_voltage: 1.0,
            pulse_duration: 0.001,
            enable_stdp: true,
        }
    }

    /// Sets custom STDP parameters.
    pub fn with_stdp_params(mut self, params: StdpParams) -> Self {
        self.stdp_params = params;
        self
    }

    /// Sets pre-synaptic pulse voltage amplitude in Volts.
    pub fn with_pulse_voltage(mut self, v: f64) -> Self {
        self.pulse_voltage = v;
        self
    }

    /// Sets pre-synaptic pulse duration in seconds.
    pub fn with_pulse_duration(mut self, d: f64) -> Self {
        self.pulse_duration = d.max(1e-6);
        self
    }

    /// Computes Vector-Matrix Multiplication (VMM): dendritic currents I_j = \sum_i V_{pre, i} G_{ij}.
    pub fn vmm(&self, v_pre: &[f64]) -> Vec<f64> {
        let mut currents = vec![0.0; self.num_outputs];
        let m = self.num_inputs.min(v_pre.len());
        for i in 0..m {
            let v_i = v_pre[i];
            if v_i.abs() > 1e-12 {
                for j in 0..self.num_outputs {
                    let g_ij = self.crossbar[i][j].conductance();
                    currents[j] += v_i * g_ij;
                }
            }
        }
        currents
    }

    /// Returns the M x N matrix of instantaneous conductances in Siemens.
    pub fn conductance_matrix(&self) -> Vec<Vec<f64>> {
        self.crossbar
            .iter()
            .map(|row| row.iter().map(|c| c.conductance()).collect())
            .collect()
    }

    /// Returns the M x N matrix of memristor state variables w \in [0.0, 1.0].
    pub fn weight_matrix(&self) -> Vec<Vec<f64>> {
        self.crossbar
            .iter()
            .map(|row| row.iter().map(|c| c.w).collect())
            .collect()
    }

    /// Overwrites memristor state variables with given matrix.
    pub fn set_weights(&mut self, weights: &[Vec<f64>]) {
        for (i, row) in weights.iter().enumerate().take(self.num_inputs) {
            for (j, &w_val) in row.iter().enumerate().take(self.num_outputs) {
                self.crossbar[i][j].w = w_val.clamp(0.0, 1.0);
            }
        }
    }

    /// Resets all output LIF neurons to resting state.
    pub fn reset_neurons(&mut self) {
        for neuron in &mut self.neurons {
            neuron.reset();
        }
    }

    /// Resets all crossbar junction state variables to a uniform default value.
    pub fn reset_weights(&mut self, default_w: f64) {
        let w_clamped = default_w.clamp(0.0, 1.0);
        for row in &mut self.crossbar {
            for cell in row {
                cell.w = w_clamped;
            }
        }
    }

    /// Configures the window function across all junctions.
    pub fn set_window_function(&mut self, window_func: WindowFunction) {
        for row in &mut self.crossbar {
            for cell in row {
                cell.window_func = window_func;
            }
        }
    }

    /// Generates a structured multi-channel pattern stimulus.
    pub fn generate_pattern_stimulus(&self, duration_s: f64) -> Vec<Vec<f64>> {
        let mut trains = vec![Vec::new(); self.num_inputs];
        let interval = 0.010; // 10 ms interval
        let mut t = 0.005;
        while t < duration_s {
            let cycle = (t * 100.0).round() as usize;
            for i in 0..self.num_inputs {
                if (i % 2 == 0 && cycle % 2 == 0) || (i % 2 == 1 && cycle % 2 == 1) {
                    trains[i].push(t);
                }
            }
            t += interval;
        }
        trains
    }

    /// Simulates the crossbar network under given input spike trains for duration_s with step dt.
    pub fn simulate(
        &mut self,
        duration_s: f64,
        dt: f64,
        input_spike_trains: &[Vec<f64>],
    ) -> SnnTrajectory {
        let dt = dt.max(1e-6);
        let duration = duration_s.max(dt);
        let steps = (duration / dt).round() as usize;

        let initial_conductances = self.conductance_matrix();
        let mut timestamps = Vec::with_capacity(steps);
        let mut membrane_potentials = vec![Vec::with_capacity(steps); self.num_outputs];
        let mut input_spikes: Vec<(f64, usize)> = Vec::new();
        let mut output_spikes: Vec<(f64, usize)> = Vec::new();

        let mut recent_pre_spikes: Vec<Vec<f64>> = vec![Vec::new(); self.num_inputs];
        let mut recent_post_spikes: Vec<Vec<f64>> = vec![Vec::new(); self.num_outputs];

        let mut total_energy_joules = 0.0;
        let mut total_synaptic_ops = 0;

        let max_stdp_window = 5.0 * self.stdp_params.tau_plus.max(self.stdp_params.tau_minus);

        self.reset_neurons();

        for step_idx in 0..steps {
            let t = step_idx as f64 * dt;
            timestamps.push(t);

            // 1. Identify newly arrived input spikes in this time slice [t, t + dt)
            let mut v_pre = vec![0.0; self.num_inputs];
            for i in 0..self.num_inputs {
                let has_new_spike = if i < input_spike_trains.len() {
                    input_spike_trains[i].iter().any(|&st| st >= t && st < t + dt)
                } else {
                    false
                };

                if has_new_spike {
                    input_spikes.push((t, i));
                    recent_pre_spikes[i].push(t);

                    // Online STDP: Pre fires after recent Post -> LTD
                    if self.enable_stdp {
                        for j in 0..self.num_outputs {
                            for &post_t in &recent_post_spikes[j] {
                                let delta_t = post_t - t; // delta_t < 0
                                if delta_t.abs() <= max_stdp_window {
                                    let delta_w = self.stdp_params.delta_w(delta_t);
                                    self.crossbar[i][j].apply_weight_change(delta_w);
                                }
                            }
                        }
                    }
                }

                // Check active pulse window [st, st + pulse_duration]
                let is_pulsing = if i < input_spike_trains.len() {
                    input_spike_trains[i]
                        .iter()
                        .any(|&st| t >= st && t < st + self.pulse_duration)
                } else {
                    false
                };

                if is_pulsing {
                    v_pre[i] = self.pulse_voltage;
                }
            }

            // 2. Compute synaptic energy dissipation and SOP count
            for i in 0..self.num_inputs {
                if v_pre[i] > 0.0 {
                    for j in 0..self.num_outputs {
                        let g_ij = self.crossbar[i][j].conductance();
                        total_energy_joules += (v_pre[i] * v_pre[i]) * g_ij * dt;
                        total_synaptic_ops += 1;
                    }
                }
            }

            // 3. VMM dendritic current summation: I_j = \sum_i V_{pre, i} G_{ij}
            let dendritic_currents = self.vmm(&v_pre);

            // 4. LIF neurons integration and spike emission
            for j in 0..self.num_outputs {
                let spiked = self.neurons[j].step(dendritic_currents[j], dt);
                membrane_potentials[j].push(self.neurons[j].v);

                if spiked {
                    output_spikes.push((t, j));
                    recent_post_spikes[j].push(t);

                    // Online STDP: Post fires after recent Pre -> LTP
                    if self.enable_stdp {
                        for i in 0..self.num_inputs {
                            for &pre_t in &recent_pre_spikes[i] {
                                let delta_t = t - pre_t; // delta_t > 0
                                if delta_t <= max_stdp_window {
                                    let delta_w = self.stdp_params.delta_w(delta_t);
                                    self.crossbar[i][j].apply_weight_change(delta_w);
                                }
                            }
                        }
                    }
                }
            }

            // 5. Prune spike histories older than STDP interaction window
            let cutoff = t - max_stdp_window;
            for pre_list in &mut recent_pre_spikes {
                pre_list.retain(|&st| st >= cutoff);
            }
            for post_list in &mut recent_post_spikes {
                post_list.retain(|&st| st >= cutoff);
            }
        }

        let final_conductances = self.conductance_matrix();

        let energy_per_sop_fj = if total_synaptic_ops > 0 {
            (total_energy_joules / total_synaptic_ops as f64) * 1.0e15
        } else {
            0.0
        };

        let mut output_spike_rates_hz = vec![0.0; self.num_outputs];
        for &(_, j) in &output_spikes {
            if j < self.num_outputs {
                output_spike_rates_hz[j] += 1.0;
            }
        }
        for rate in &mut output_spike_rates_hz {
            *rate /= duration.max(1e-9);
        }

        let mut mean_conductance_s = 0.0;
        for row in &final_conductances {
            for &g in row {
                mean_conductance_s += g;
            }
        }
        mean_conductance_s /= (self.num_inputs * self.num_outputs).max(1) as f64;

        SnnTrajectory {
            timestamps,
            membrane_potentials,
            input_spikes,
            output_spikes,
            initial_conductances,
            final_conductances,
            energy_per_sop_fj,
            total_energy_joules,
            total_synaptic_ops,
            output_spike_rates_hz,
            mean_conductance_s,
        }
    }
}
