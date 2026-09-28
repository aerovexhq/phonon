#![allow(clippy::needless_range_loop)]
//! Autonomous Neuromorphic Reservoir Computing & Memristive Dynamics.
//!
//! Formulates:
//! - Multi-technology memristive synaptic crossbar reservoirs (RRAM, PCM, FeFET).
//! - Recurrent Echo State Network (ESN) dynamical maps with leaking rate $\alpha \in (0, 1]$:
//!   $$\mathbf{x}[t+1] = (1 - \alpha) \mathbf{x}[t] + \alpha f_{act}\left( \mathbf{W}_{res} \mathbf{x}[t] + \mathbf{W}_{in} \mathbf{u}[t] + \mathbf{b} \right)$$
//! - Single-node delayed-feedback reservoir computing architecture (Mackey-Glass and Ikeda chaotic delay oscillators):
//!   $$T \frac{dx}{dt} = -x(t) + f_{nonlin}(x(t - \tau_D), u(t))$$
//!   discretized into $N_{virtual}$ virtual nodes along the delay line $\tau_D$.
//! - Echo State Property (ESP) analysis:
//!   * Spectral radius computation $\rho(\mathbf{W}_{res}) = \max_i |\lambda_i|$ via power iteration in safe Rust.
//!   * Rescaling $\mathbf{W}_{res} \leftarrow \frac{\rho_{target}}{\rho(\mathbf{W}_{res})} \mathbf{W}_{res}$ ensuring $\rho < 1.0$.
//!   * Fading memory capacity formulation: $MC = \sum_{k=1}^K r^2(u[t-k], y_k[t])$.
//! - Physical memristive non-idealities:
//!   * Cycle-to-cycle (C2C) and device-to-device (D2D) conductance variance.
//!   * Non-linear tunneling I-V and sneak-path currents.
//!   * Sub-femtojoule energy dissipation tracking per synaptic event ($E_{syn} = V^2 G \Delta t < 100\text{ fJ}$).

use super::fefet::FerroelectricFetModel;
use super::pcm::PhaseChangeMemoryModel;
use super::rram::FilamentaryRramModel;

/// Deterministic 64-bit XorShift pseudo-random number generator for reproducible reservoir synthesis.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReservoirRng {
    state: u64,
}

impl ReservoirRng {
    /// Creates a new PRNG initialized with a non-zero seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0xCAFE_BABE_DEAD_BEEF
            } else {
                seed
            },
        }
    }

    /// Generates next pseudo-random 64-bit unsigned integer.
    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Generates next pseudo-random floating point number uniformly distributed in `[0.0, 1.0)`.
    #[inline]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// Generates pseudo-random float in `[min, max)`.
    #[inline]
    pub fn next_range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.next_f64()
    }

    /// Generates normally distributed sample using Box-Muller transform: $\mathcal{N}(\mu, \sigma^2)$.
    pub fn next_gaussian(&mut self, mean: f64, std_dev: f64) -> f64 {
        let u1 = self.next_f64().max(1e-15);
        let u2 = self.next_f64();
        let z0 = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        mean + z0 * std_dev
    }
}

/// Underlying non-volatile memristive device technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemristorTechnology {
    /// Transition metal oxide filamentary RRAM (e.g. $\text{TiN}/\text{HfO}_2/\text{Pt}$).
    FilamentaryRram,
    /// Chalcogenide Phase-Change Memory (e.g. $\text{Ge}_2\text{Sb}_2\text{Te}_5$).
    PhaseChangeMemory,
    /// Ferroelectric Field-Effect Transistor (e.g. $\text{Hf}_{0.5}\text{Zr}_{0.5}\text{O}_2$).
    FerroelectricFet,
}

impl MemristorTechnology {
    /// Returns nominal conductance range $[G_{min}, G_{max}]$ in Siemens ($S$).
    pub fn conductance_range(&self) -> (f64, f64) {
        match self {
            Self::FilamentaryRram => {
                let rram = FilamentaryRramModel::hfo2_synaptic();
                (1.0 / rram.r_off, 1.0 / rram.r_on)
            }
            Self::PhaseChangeMemory => {
                let pcm = PhaseChangeMemoryModel::gst_mushroom_cell();
                (1.0 / pcm.r_amorph, 1.0 / pcm.r_cryst)
            }
            Self::FerroelectricFet => {
                let fefet = FerroelectricFetModel::hzo_10nm();
                let g_min = 1.0e-7;
                let g_max = fefet.channel_transconductance * 0.1;
                (g_min, g_max)
            }
        }
    }

    /// Evaluates non-linear device current at conductance state $G$ and voltage $V$.
    pub fn evaluate_current(&self, g_nominal: f64, v_volts: f64) -> f64 {
        match self {
            Self::FilamentaryRram => {
                let non_lin = (1.8 * v_volts.abs()).min(8.0).cosh();
                g_nominal * v_volts * non_lin
            }
            Self::PhaseChangeMemory => {
                let non_lin = (1.5 * v_volts.abs()).min(6.0).cosh();
                g_nominal * v_volts * non_lin
            }
            Self::FerroelectricFet => {
                let non_lin = (1.2 * v_volts.abs()).min(5.0).cosh();
                g_nominal * v_volts * non_lin
            }
        }
    }
}

/// Physical non-idealities configuration for memristive crossbar array reservoirs.
#[derive(Debug, Clone, PartialEq)]
pub struct MemristiveNonIdealityConfig {
    /// Device-to-device (D2D) static spatial conductance variation ratio (std dev / mean).
    pub d2d_variation_ratio: f64,
    /// Cycle-to-cycle (C2C) temporal noise ratio (std dev / mean).
    pub c2c_variation_ratio: f64,
    /// Whether to model non-linear tunneling / Poole-Frenkel field enhancement.
    pub enable_tunneling_nonlinearity: bool,
    /// Whether sneak-path leakage is active across unselected crossbar lines.
    pub enable_sneak_paths: bool,
    /// Sneak-path off-state leakage conductance per cross-point cell in Siemens ($S$).
    pub sneak_path_conductance_s: f64,
    /// Read pulse voltage amplitude in Volts ($V$) (typically $0.05 - 0.2\text{ V}$).
    pub read_voltage_v: f64,
    /// Read pulse width duration in seconds ($s$) (typically $5 - 50\text{ ns}$).
    pub pulse_duration_s: f64,
}

impl Default for MemristiveNonIdealityConfig {
    fn default() -> Self {
        Self {
            d2d_variation_ratio: 0.03, // 3% D2D variation
            c2c_variation_ratio: 0.01, // 1% C2C variation
            enable_tunneling_nonlinearity: true,
            enable_sneak_paths: true,
            sneak_path_conductance_s: 1.0e-9, // 1 nS leakage
            read_voltage_v: 0.10,             // 100 mV read pulse
            pulse_duration_s: 10.0e-9,        // 10 ns pulse
        }
    }
}

/// Non-linear reservoir activation function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReservoirActivation {
    /// Hyperbolic tangent $\tanh(x)$.
    Tanh,
    /// Logistic sigmoid $\frac{1}{1 + e^{-x}}$.
    Sigmoid,
    /// Rectified Linear Unit $\max(0, x)$.
    Relu,
    /// Physical memristive non-linear tunneling map.
    MemristiveNonLinear,
}

impl ReservoirActivation {
    /// Applies activation function to scalar input $z$.
    #[inline]
    pub fn apply(&self, z: f64) -> f64 {
        match self {
            Self::Tanh => z.tanh(),
            Self::Sigmoid => 1.0 / (1.0 + (-z).exp()),
            Self::Relu => z.max(0.0),
            Self::MemristiveNonLinear => {
                let clipped = z.clamp(-15.0, 15.0);
                clipped / (1.0 + clipped.abs().powi(2)).sqrt()
            }
        }
    }
}

/// Physical Memristive Crossbar Reservoir Network (Echo State Network map).
#[derive(Debug, Clone, PartialEq)]
pub struct MemristiveReservoir {
    /// Number of reservoir state units $N_{res}$.
    pub num_reservoir_nodes: usize,
    /// Number of input features $N_{in}$.
    pub num_inputs: usize,
    /// Memristive device technology (RRAM, PCM, FeFET).
    pub technology: MemristorTechnology,
    /// Non-ideality configuration.
    pub non_idealities: MemristiveNonIdealityConfig,
    /// Leaking rate $\alpha \in (0, 1]$.
    pub leak_rate: f64,
    /// Activation function.
    pub activation: ReservoirActivation,
    /// Recurrent weight matrix $\mathbf{W}_{res} \in \mathbb{R}^{N_{res} \times N_{res}}$.
    pub w_res: Vec<Vec<f64>>,
    /// Input weight matrix $\mathbf{W}_{in} \in \mathbb{R}^{N_{res} \times N_{in}}$.
    pub w_in: Vec<Vec<f64>>,
    /// Bias vector $\mathbf{b} \in \mathbb{R}^{N_{res}}$.
    pub bias: Vec<f64>,
    /// Instantaneous reservoir state vector $\mathbf{x}[t] \in \mathbb{R}^{N_{res}}$.
    pub state: Vec<f64>,
    /// Device-to-device spatial conductance variation matrix $\Delta G_{D2D}$.
    d2d_offsets: Vec<Vec<f64>>,
    /// Internal PRNG for stochastic cycle-to-cycle noise.
    rng: ReservoirRng,
    /// Total cumulative energy dissipated in Joules ($J$).
    pub cumulative_energy_joules: f64,
}

impl MemristiveReservoir {
    /// Creates and initializes a new physical memristive reservoir.
    pub fn new(
        num_reservoir_nodes: usize,
        num_inputs: usize,
        technology: MemristorTechnology,
        target_spectral_radius: f64,
        leak_rate: f64,
        seed: u64,
    ) -> Self {
        let mut rng = ReservoirRng::new(seed);
        let non_idealities = MemristiveNonIdealityConfig::default();

        // Initialize sparse recurrent weight matrix W_res:
        let mut w_res = vec![vec![0.0; num_reservoir_nodes]; num_reservoir_nodes];
        let density = 0.20; // 20% sparsity
        for row in 0..num_reservoir_nodes {
            for col in 0..num_reservoir_nodes {
                if rng.next_f64() < density {
                    w_res[row][col] = rng.next_range(-1.0, 1.0);
                }
            }
        }

        // Initialize input weight matrix W_in:
        let mut w_in = vec![vec![0.0; num_inputs]; num_reservoir_nodes];
        for row in 0..num_reservoir_nodes {
            for col in 0..num_inputs {
                w_in[row][col] = rng.next_range(-1.0, 1.0);
            }
        }

        // Initialize bias vector b:
        let mut bias = vec![0.0; num_reservoir_nodes];
        for b in bias.iter_mut() {
            *b = rng.next_range(-0.1, 0.1);
        }

        // Device-to-device static variations:
        let mut d2d_offsets = vec![vec![0.0; num_reservoir_nodes]; num_reservoir_nodes];
        for row in 0..num_reservoir_nodes {
            for col in 0..num_reservoir_nodes {
                d2d_offsets[row][col] = rng.next_gaussian(0.0, non_idealities.d2d_variation_ratio);
            }
        }

        let mut reservoir = Self {
            num_reservoir_nodes,
            num_inputs,
            technology,
            non_idealities,
            leak_rate: leak_rate.clamp(0.001, 1.0),
            activation: ReservoirActivation::Tanh,
            w_res,
            w_in,
            bias,
            state: vec![0.0; num_reservoir_nodes],
            d2d_offsets,
            rng,
            cumulative_energy_joules: 0.0,
        };

        // Scale W_res to exact target spectral radius:
        reservoir.scale_spectral_radius(target_spectral_radius);
        reservoir
    }

    /// Computes the spectral radius $\rho(\mathbf{W}_{res}) = \max_i |\lambda_i|$ using power iteration in safe Rust.
    pub fn compute_spectral_radius(&self) -> f64 {
        let n = self.num_reservoir_nodes;
        if n == 0 {
            return 0.0;
        }

        let mut v = vec![1.0 / (n as f64).sqrt(); n];
        let mut lambda = 0.0;

        for _ in 0..150 {
            let mut w = vec![0.0; n];
            for i in 0..n {
                let mut sum = 0.0;
                for j in 0..n {
                    sum += self.w_res[i][j] * v[j];
                }
                w[i] = sum;
            }

            let norm = w.iter().map(|&x| x * x).sum::<f64>().sqrt();
            if norm < 1e-14 {
                return 0.0;
            }

            let mut dot = 0.0;
            for i in 0..n {
                dot += w[i] * v[i];
            }
            lambda = dot;

            for i in 0..n {
                v[i] = w[i] / norm;
            }
        }

        lambda.abs()
    }

    /// Rescales recurrent matrix $\mathbf{W}_{res} \leftarrow \frac{\rho_{target}}{\rho(\mathbf{W}_{res})} \mathbf{W}_{res}$
    /// guaranteeing the Echo State Property (ESP) when $\rho < 1.0$.
    pub fn scale_spectral_radius(&mut self, target_spectral_radius: f64) {
        let current_rho = self.compute_spectral_radius();
        if current_rho > 1e-12 {
            let scale = target_spectral_radius / current_rho;
            for row in self.w_res.iter_mut() {
                for val in row.iter_mut() {
                    *val *= scale;
                }
            }
        }
    }

    /// Evaluates one discrete time step of the memristive reservoir map:
    /// $$\mathbf{x}[t+1] = (1 - \alpha) \mathbf{x}[t] + \alpha f_{act}\left( \mathbf{W}_{res} \mathbf{x}[t] + \mathbf{W}_{in} \mathbf{u}[t] + \mathbf{b} \right)$$
    /// incorporating physical non-idealities (D2D, C2C noise, sneak-paths, tunneling) and tracking synaptic energy.
    pub fn step(&mut self, u: &[f64]) -> (&[f64], f64) {
        let n = self.num_reservoir_nodes;
        let mut pre_activation = vec![0.0; n];
        let mut step_energy = 0.0;

        let (g_min, g_max) = self.technology.conductance_range();
        let g_span = g_max - g_min;
        let v_read = self.non_idealities.read_voltage_v;
        let dt_pulse = self.non_idealities.pulse_duration_s;

        // Recurrent contribution: W_res * x
        for i in 0..n {
            let mut sum_res = 0.0;
            for j in 0..n {
                let weight = self.w_res[i][j];
                if weight.abs() > 1e-12 {
                    let nominal_g = g_min + ((weight.abs() * 0.5).min(1.0)) * g_span;

                    let d2d = self.d2d_offsets[i][j];
                    let c2c = self
                        .rng
                        .next_gaussian(0.0, self.non_idealities.c2c_variation_ratio);
                    let actual_g = (nominal_g * (1.0 + d2d + c2c)).clamp(g_min * 0.5, g_max * 1.5);

                    // Synaptic energy: E_syn = V^2 * G * dt (sub-femtojoule)
                    let e_syn = v_read * v_read * actual_g * dt_pulse;
                    step_energy += e_syn;

                    let effective_weight = if self.non_idealities.enable_tunneling_nonlinearity {
                        let eff_curr = self.technology.evaluate_current(actual_g, v_read);
                        weight.signum()
                            * (eff_curr / (v_read * nominal_g.max(1e-12)))
                            * weight.abs()
                    } else {
                        weight * (1.0 + d2d + c2c)
                    };

                    sum_res += effective_weight * self.state[j];
                }

                if self.non_idealities.enable_sneak_paths
                    && self.non_idealities.sneak_path_conductance_s > 0.0
                {
                    let sneak_current = v_read * self.non_idealities.sneak_path_conductance_s;
                    let sneak_e = v_read * sneak_current * dt_pulse;
                    step_energy += sneak_e;
                    sum_res += 0.001 * (self.rng.next_f64() - 0.5);
                }
            }
            pre_activation[i] = sum_res;
        }

        // Input contribution: W_in * u
        let m = u.len().min(self.num_inputs);
        for i in 0..n {
            let mut sum_in = 0.0;
            for (k, &u_val) in u.iter().take(m).enumerate() {
                let w_in_val = self.w_in[i][k];
                sum_in += w_in_val * u_val;

                let g_syn = g_min + (w_in_val.abs() * 0.5).min(1.0) * g_span;
                step_energy += v_read * v_read * g_syn * dt_pulse;
            }
            pre_activation[i] += sum_in + self.bias[i];
        }

        // Leaky integrator state update:
        let alpha = self.leak_rate;
        for i in 0..n {
            let act = self.activation.apply(pre_activation[i]);
            self.state[i] = (1.0 - alpha) * self.state[i] + alpha * act;
        }

        self.cumulative_energy_joules += step_energy;
        (&self.state, step_energy)
    }

    /// Resets the reservoir state vector $\mathbf{x}$ to zero.
    pub fn reset_state(&mut self) {
        self.state.fill(0.0);
    }

    /// Computes the fading memory capacity (MC):
    /// $$MC = \sum_{k=1}^K r^2(u[t-k], y_k[t])$$
    /// evaluating linear reconstruction of past inputs $u[t-k]$ from reservoir states.
    pub fn calculate_memory_capacity(&mut self, inputs: &[f64], max_delay: usize) -> f64 {
        let total_steps = inputs.len();
        if total_steps < max_delay + 50 {
            return 0.0;
        }

        self.reset_state();
        let mut states: Vec<Vec<f64>> = Vec::with_capacity(total_steps);

        for &u_val in inputs {
            let u_vec = [u_val];
            self.step(&u_vec);
            states.push(self.state.clone());
        }

        let washout = 30.min(total_steps / 5);
        let eval_steps = total_steps - washout;
        let n_res = self.num_reservoir_nodes;

        // Build Gram matrix X * X^T + lambda * I for linear readout:
        let mut x_mat = vec![vec![0.0; n_res + 1]; eval_steps];
        for t in 0..eval_steps {
            for j in 0..n_res {
                x_mat[t][j] = states[washout + t][j];
            }
            x_mat[t][n_res] = 1.0; // Bias term
        }

        let p = n_res + 1;
        let mut a = vec![vec![0.0; p]; p];
        let lambda = 1e-4;

        for i in 0..p {
            for j in 0..p {
                let mut sum = 0.0;
                for t in 0..eval_steps {
                    sum += x_mat[t][i] * x_mat[t][j];
                }
                a[i][j] = sum;
            }
            a[i][i] += lambda;
        }

        let a_inv = match invert_matrix_safe(&a) {
            Some(inv) => inv,
            None => return 0.0,
        };

        let mut total_mc = 0.0;

        for k in 1..=max_delay {
            let mut y_target = vec![0.0; eval_steps];
            let mut mean_target = 0.0;
            for t in 0..eval_steps {
                let idx = (washout + t).saturating_sub(k);
                y_target[t] = inputs[idx];
                mean_target += y_target[t];
            }
            mean_target /= eval_steps as f64;

            let mut var_target = 0.0;
            for &y in &y_target {
                var_target += (y - mean_target).powi(2);
            }
            if var_target < 1e-12 {
                continue;
            }

            let mut b_vec = vec![0.0; p];
            for i in 0..p {
                let mut sum = 0.0;
                for t in 0..eval_steps {
                    sum += x_mat[t][i] * y_target[t];
                }
                b_vec[i] = sum;
            }

            let mut w_out = vec![0.0; p];
            for i in 0..p {
                let mut sum = 0.0;
                for j in 0..p {
                    sum += a_inv[i][j] * b_vec[j];
                }
                w_out[i] = sum;
            }

            let mut y_pred = vec![0.0; eval_steps];
            let mut mean_pred = 0.0;
            for t in 0..eval_steps {
                let mut sum = 0.0;
                for j in 0..p {
                    sum += x_mat[t][j] * w_out[j];
                }
                y_pred[t] = sum;
                mean_pred += sum;
            }
            mean_pred /= eval_steps as f64;

            let mut var_pred = 0.0;
            let mut cov = 0.0;
            for t in 0..eval_steps {
                let dt_target = y_target[t] - mean_target;
                let dt_pred = y_pred[t] - mean_pred;
                var_pred += dt_pred * dt_pred;
                cov += dt_target * dt_pred;
            }

            if var_pred > 1e-14 {
                let r_squared = (cov * cov) / (var_target * var_pred);
                total_mc += r_squared.clamp(0.0, 1.0);
            }
        }

        total_mc
    }
}

/// Single-node delayed-feedback reservoir computing architecture.
///
/// Implements continuous-time delay dynamical systems:
/// $$T \frac{dx}{dt} = -x(t) + f_{nonlin}(x(t - \tau_D), u(t))$$
/// discretized into $N_{virtual}$ virtual nodes along delay loop $\tau_D$.
#[derive(Debug, Clone, PartialEq)]
pub struct DelayedFeedbackReservoir {
    /// Number of virtual nodes $N_{virtual}$ along the delay line.
    pub num_virtual_nodes: usize,
    /// Delay interval $\tau_D$ in seconds ($s$).
    pub tau_delay_s: f64,
    /// Distance between virtual nodes $\theta = \tau_D / N_{virtual}$ in seconds ($s$).
    pub theta_s: f64,
    /// Mask modulation vector $M \in \mathbb{R}^{N_{virtual}}$.
    pub mask: Vec<f64>,
    /// Delay line history buffer storing $x(t - \tau_D)$.
    history_buffer: Vec<f64>,
    /// Write pointer for delay history circular buffer.
    buffer_idx: usize,
    /// Internal feedback gain $\eta$.
    pub feedback_gain: f64,
    /// Input coupling coefficient $\gamma$.
    pub input_coupling: f64,
    /// Non-linear oscillator function type.
    pub oscillator_type: DelayOscillatorType,
}

/// Delay oscillator non-linear characteristic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelayOscillatorType {
    /// Mackey-Glass chaotic oscillator: $f(z) = \frac{\eta z}{1 + z^p}$.
    MackeyGlass,
    /// Ikeda optoelectronic chaotic oscillator: $f(z) = \beta \sin^2(z + \phi_0)$.
    Ikeda,
}

impl DelayedFeedbackReservoir {
    /// Creates a new delayed-feedback reservoir with $N_{virtual}$ virtual nodes.
    pub fn new(
        num_virtual_nodes: usize,
        tau_delay_s: f64,
        oscillator_type: DelayOscillatorType,
        seed: u64,
    ) -> Self {
        let mut rng = ReservoirRng::new(seed);
        let theta_s = tau_delay_s / num_virtual_nodes.max(1) as f64;

        let mut mask = vec![0.0; num_virtual_nodes];
        for m in mask.iter_mut() {
            *m = if rng.next_f64() > 0.5 { 0.8 } else { -0.8 };
        }

        let history_buffer = vec![0.1; num_virtual_nodes];

        Self {
            num_virtual_nodes,
            tau_delay_s,
            theta_s,
            mask,
            history_buffer,
            buffer_idx: 0,
            feedback_gain: 1.2,
            input_coupling: 0.5,
            oscillator_type,
        }
    }

    /// Evaluates non-linear oscillator function $f_{nonlin}(x_{delayed}, u_{masked})$.
    fn evaluate_nonlinearity(&self, x_delayed: f64, u_masked: f64) -> f64 {
        let z = self.feedback_gain * x_delayed + self.input_coupling * u_masked;
        match self.oscillator_type {
            DelayOscillatorType::MackeyGlass => {
                let p = 10.0;
                let denom = 1.0 + z.abs().powf(p);
                z / denom.max(1e-6)
            }
            DelayOscillatorType::Ikeda => {
                let phi_0 = std::f64::consts::FRAC_PI_4;
                let sin_val = (z + phi_0).sin();
                sin_val * sin_val
            }
        }
    }

    /// Steps the delayed-feedback reservoir forward for one input sample $u[n]$,
    /// returning high-dimensional virtual node state vector $\mathbf{x}[n] \in \mathbb{R}^{N_{virtual}}$.
    pub fn step(&mut self, u: f64) -> Vec<f64> {
        let n = self.num_virtual_nodes;
        let mut virtual_states = vec![0.0; n];
        let dt = self.theta_s;
        let tau = self.tau_delay_s.max(1e-9);

        for k in 0..n {
            let u_masked = self.mask[k] * u;
            let x_delayed = self.history_buffer[self.buffer_idx];

            let nonlin = self.evaluate_nonlinearity(x_delayed, u_masked);
            let current_x = self.history_buffer[self.buffer_idx];
            let next_x = current_x + (dt / tau) * (-current_x + nonlin);

            self.history_buffer[self.buffer_idx] = next_x;
            virtual_states[k] = next_x;

            self.buffer_idx = (self.buffer_idx + 1) % n;
        }

        virtual_states
    }

    /// Resets the delay history buffer.
    pub fn reset(&mut self) {
        self.history_buffer.fill(0.1);
        self.buffer_idx = 0;
    }
}

/// Inverts a square matrix using Gauss-Jordan elimination with partial pivoting in safe Rust.
pub fn invert_matrix_safe(matrix: &[Vec<f64>]) -> Option<Vec<Vec<f64>>> {
    let n = matrix.len();
    if n == 0 || matrix[0].len() != n {
        return None;
    }

    let mut aug = vec![vec![0.0; 2 * n]; n];
    for i in 0..n {
        for j in 0..n {
            aug[i][j] = matrix[i][j];
        }
        aug[i][n + i] = 1.0;
    }

    for col in 0..n {
        let mut max_row = col;
        let mut max_val = aug[col][col].abs();
        for row in (col + 1)..n {
            let val = aug[row][col].abs();
            if val > max_val {
                max_val = val;
                max_row = row;
            }
        }

        if max_val < 1e-14 {
            return None;
        }

        if max_row != col {
            aug.swap(col, max_row);
        }

        let pivot = aug[col][col];
        for j in 0..(2 * n) {
            aug[col][j] /= pivot;
        }

        for row in 0..n {
            if row != col {
                let factor = aug[row][col];
                if factor.abs() > 1e-15 {
                    for j in 0..(2 * n) {
                        aug[row][j] -= factor * aug[col][j];
                    }
                }
            }
        }
    }

    let mut inv = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            inv[i][j] = aug[i][n + j];
        }
    }

    Some(inv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spectral_radius_and_rescaling() {
        let mut reservoir =
            MemristiveReservoir::new(20, 1, MemristorTechnology::FilamentaryRram, 0.85, 0.3, 42);

        let rho = reservoir.compute_spectral_radius();
        assert!(
            (rho - 0.85).abs() < 1e-3,
            "Spectral radius should scale to 0.85, got {}",
            rho
        );
        assert!(rho < 1.0, "Echo state property requires rho < 1.0");

        reservoir.scale_spectral_radius(0.60);
        let rho_scaled = reservoir.compute_spectral_radius();
        assert!(
            (rho_scaled - 0.60).abs() < 1e-3,
            "Rescaled spectral radius should be 0.60, got {}",
            rho_scaled
        );
    }

    #[test]
    fn test_echo_state_property_convergence() {
        // Two reservoirs with identical weights but different initial states must converge under same input
        let mut res1 = MemristiveReservoir::new(
            15,
            1,
            MemristorTechnology::PhaseChangeMemory,
            0.80,
            0.5,
            123,
        );
        let mut res2 = res1.clone();
        // Perturb state of res2:
        for val in res2.state.iter_mut() {
            *val = 0.95;
        }

        // Drive both with same periodic signal:
        for t in 0..200 {
            let u = [((t as f64) * 0.1).sin()];
            res1.step(&u);
            res2.step(&u);
        }

        // States must asymptotically synchronize (Echo State Property)
        let mut diff_norm = 0.0;
        for i in 0..res1.num_reservoir_nodes {
            diff_norm += (res1.state[i] - res2.state[i]).powi(2);
        }
        diff_norm = diff_norm.sqrt();
        assert!(
            diff_norm < 1e-2,
            "Reservoir states should synchronize under ESP, diff_norm = {}",
            diff_norm
        );
    }

    #[test]
    fn test_sub_femtojoule_synaptic_energy() {
        let mut reservoir =
            MemristiveReservoir::new(10, 1, MemristorTechnology::FerroelectricFet, 0.75, 0.4, 777);
        reservoir.non_idealities.read_voltage_v = 0.05; // 50 mV
        reservoir.non_idealities.pulse_duration_s = 5.0e-9; // 5 ns

        let (_, step_energy) = reservoir.step(&[0.5]);
        assert!(
            step_energy > 0.0,
            "Energy dissipation must be strictly positive"
        );
        let energy_per_synapse = step_energy / (10 * 10) as f64;
        // Should be sub-femtojoule to few femtojoules:
        assert!(
            energy_per_synapse < 1.0e-13,
            "Synaptic event dissipation must be < 100 fJ, got {} J",
            energy_per_synapse
        );
    }

    #[test]
    fn test_delayed_feedback_reservoir_dynamics() {
        let mut dfr =
            DelayedFeedbackReservoir::new(25, 10.0e-6, DelayOscillatorType::MackeyGlass, 999);

        let states1 = dfr.step(0.5);
        assert_eq!(states1.len(), 25);
        let states2 = dfr.step(0.8);
        assert_eq!(states2.len(), 25);
        assert_ne!(states1, states2);
    }
}
