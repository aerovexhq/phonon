#![deny(unsafe_code)]

//! Phononic Lugiato-Lefever Equation (LLE) & Dissipative Kerr Soliton Engine.
//!
//! Solves the normalized Lugiato-Lefever equation governing nonlinear microresonator
//! acoustic and optical Kerr frequency combs:
//!   d(psi)/dt = -(1 + i*alpha)*psi - i*(D_2 / (2 * kappa)) * d^2(psi)/d(theta)^2 + i * |psi|^2 * psi + F_0
//!
//! Uses a symmetric split-step Fourier method: exact linear dispersion, detuning,
//! and dissipation in the modal Fourier domain via radix-2 FFT, and 4th-order
//! Runge-Kutta / exact phase rotation for the Kerr nonlinearity and drive in real space.

use phonon_models::quantum::Complex;
use std::f64::consts::PI;

/// Physical parameters for the phononic microresonator Kerr comb.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MicroresonatorParams {
    /// Cavity loaded quality factor Q (e.g. 1.5e6).
    pub q_factor: f64,
    /// Free spectral range (FSR) in Hz (repetition rate f_rep, e.g. 100.0 MHz).
    pub fsr_hz: f64,
    /// Intrinsic/loaded cavity decay rate kappa in rad/s (kappa = 2 * pi * f_0 / Q).
    pub kappa: f64,
    /// Second-order chromatic dispersion D_2 in rad/s (anomalous D_2 > 0 supports bright solitons).
    pub d2: f64,
    /// Nonlinear Kerr coupling parameter g_kerr in rad/s.
    pub g_kerr: f64,
    /// Normalized pump detuning alpha (dimensionless, range -2.0 to 12.0).
    pub alpha: f64,
    /// Normalized pump drive amplitude F_0 (dimensionless, F_0^2 in 1.0 to 8.0).
    pub f_drive: f64,
    /// Grid size N for azimuthal discretization (e.g. 256 or 512 points).
    pub grid_size: usize,
}

impl MicroresonatorParams {
    /// Default parameters supporting single bright dissipative Kerr solitons.
    pub fn default_bright_soliton() -> Self {
        let q_factor = 1.5e6;
        let fsr_hz = 100.0e6;
        // Cavity decay kappa ~ 2 * pi * 250 kHz = 1.5708e6 rad/s
        let kappa = 2.0 * PI * 250.0e3;
        // Anomalous dispersion D_2 ~ 10 kHz * 2 * pi rad/s = 6.2832e4 rad/s
        let d2 = 2.0 * PI * 10.0e3;
        let g_kerr = 1.2e-3;
        let alpha = 3.5;
        let f_drive = 2.0; // F_0^2 = 4.0 (above soliton threshold ~ 0.81 * alpha = 2.84)
        let grid_size = 256;

        Self {
            q_factor,
            fsr_hz,
            kappa,
            d2,
            g_kerr,
            alpha,
            f_drive,
            grid_size,
        }
    }

    /// Default parameters supporting stable Turing pattern rolls.
    pub fn default_turing_roll() -> Self {
        let mut p = Self::default_bright_soliton();
        p.alpha = 1.8;
        p.f_drive = 1.6; // F_0^2 ~ 2.56
        p
    }

    /// Normalized second-order chromatic dispersion parameter d_2 / 2 = D_2 / (2 * kappa).
    #[inline]
    pub fn normalized_dispersion(&self) -> f64 {
        if self.kappa.abs() < 1e-12 {
            0.02
        } else {
            self.d2 / (2.0 * self.kappa)
        }
    }

    /// Dimensionless pump drive power F_0^2.
    #[inline]
    pub fn f_squared(&self) -> f64 {
        self.f_drive * self.f_drive
    }

    /// Builder: update detuning alpha.
    pub fn with_alpha(mut self, alpha: f64) -> Self {
        self.alpha = alpha;
        self
    }

    /// Builder: update pump drive F_0.
    pub fn with_f_drive(mut self, f_drive: f64) -> Self {
        self.f_drive = f_drive;
        self
    }

    /// Builder: update dispersion D_2 in rad/s.
    pub fn with_d2(mut self, d2: f64) -> Self {
        self.d2 = d2;
        self
    }

    /// Builder: update grid size N.
    pub fn with_grid_size(mut self, grid_size: usize) -> Self {
        self.grid_size = grid_size;
        self
    }
}

impl Default for MicroresonatorParams {
    fn default() -> Self {
        Self::default_bright_soliton()
    }
}

/// Dynamical operating regime of the Kerr microcomb state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MicrocombRegime {
    /// Homogeneous low-power continuous wave state.
    LowPowerCw,
    /// Periodic Turing pattern roll structure (primary comb sidebands).
    TuringRolls,
    /// Chaotic modulation instability with turbulent pulses.
    ModulationInstabilityChaos,
    /// Localized dissipative Kerr bright soliton pulse.
    DissipativeSoliton,
    /// Periodically oscillating breathing soliton.
    BreatherSoliton,
}

impl MicrocombRegime {
    /// Human-readable label for the microcomb regime.
    pub fn label(&self) -> &'static str {
        match self {
            Self::LowPowerCw => "Low-Power CW",
            Self::TuringRolls => "Turing Pattern Rolls",
            Self::ModulationInstabilityChaos => "Modulation Instability Chaos",
            Self::DissipativeSoliton => "Dissipative Kerr Soliton",
            Self::BreatherSoliton => "Breather Soliton",
        }
    }

    /// Short description of the physical regime.
    pub fn description(&self) -> &'static str {
        match self {
            Self::LowPowerCw => "Homogeneous flat intracavity field without sideband generation",
            Self::TuringRolls => "Multiple periodic azimuthal rolls with wide mode spacing",
            Self::ModulationInstabilityChaos => "Turbulent multi-frequency chaos with strong fluctuations",
            Self::DissipativeSoliton => "Ultra-stable single localized pulse packet with sech^2 envelope",
            Self::BreatherSoliton => "Soliton with periodic temporal oscillation in peak intensity",
        }
    }
}

/// Dynamic state of the intracavity field around the microresonator ring.
#[derive(Debug, Clone, PartialEq)]
pub struct MicrocombState {
    /// Azimuthal complex field values psi(theta) at discrete grid points.
    pub psi: Vec<Complex>,
    /// Azimuthal grid angles theta in [-pi, pi).
    pub theta: Vec<f64>,
    /// Elapsed normalized simulation time.
    pub time: f64,
    /// Mean intracavity power: (1 / N) * sum(|psi_k|^2).
    pub mean_power: f64,
    /// Peak intracavity power: max(|psi_k|^2).
    pub peak_power: f64,
    /// Minimum intracavity power: min(|psi_k|^2).
    pub min_power: f64,
    /// Classified microcomb regime.
    pub regime: MicrocombRegime,
    /// Normalized pump detuning at this state.
    pub alpha: f64,
    /// Normalized pump drive amplitude at this state.
    pub f_drive: f64,
}

impl MicrocombState {
    /// Creates a zero/empty microcomb state for grid size N.
    pub fn new(n: usize) -> Self {
        let theta: Vec<f64> = (0..n)
            .map(|k| -PI + (k as f64) * (2.0 * PI / n as f64))
            .collect();
        let psi = vec![Complex::ZERO; n];
        Self {
            psi,
            theta,
            time: 0.0,
            mean_power: 0.0,
            peak_power: 0.0,
            min_power: 0.0,
            regime: MicrocombRegime::LowPowerCw,
            alpha: 0.0,
            f_drive: 0.0,
        }
    }

    /// Computes and updates power metrics from the current complex field.
    pub fn recalculate_metrics(&mut self) {
        let n = self.psi.len();
        if n == 0 {
            return;
        }
        let mut sum_power = 0.0;
        let mut peak = 0.0;
        let mut min = f64::MAX;

        for p in &self.psi {
            let p_sq = p.norm_sq();
            sum_power += p_sq;
            if p_sq > peak {
                peak = p_sq;
            }
            if p_sq < min {
                min = p_sq;
            }
        }

        self.mean_power = sum_power / (n as f64);
        self.peak_power = peak;
        self.min_power = if min == f64::MAX { 0.0 } else { min };
    }
}

/// Result of a continuous detuning sweep scan across alpha values.
#[derive(Debug, Clone, PartialEq)]
pub struct DetuningScanResult {
    /// Scanned detuning alpha values.
    pub alphas: Vec<f64>,
    /// Mean intracavity powers vs detuning alpha.
    pub powers: Vec<f64>,
    /// Peak intracavity powers vs detuning alpha.
    pub peak_powers: Vec<f64>,
    /// Operating regime identified at each detuning alpha step.
    pub regimes: Vec<MicrocombRegime>,
    /// Soliton existence step/plateau alpha range [alpha_start, alpha_end].
    pub soliton_step_range: Option<(f64, f64)>,
    /// Turing roll alpha range [alpha_start, alpha_end].
    pub turing_roll_range: Option<(f64, f64)>,
    /// Chaos alpha range [alpha_start, alpha_end].
    pub chaos_range: Option<(f64, f64)>,
    /// Representative steady-state soliton state if found during scan.
    pub best_soliton_state: Option<MicrocombState>,
}

/// Pure safe Rust 1D Cooley-Tukey radix-2 Fast Fourier Transform with DFT fallback.
pub fn fft_1d(input: &[Complex], inverse: bool) -> Vec<Complex> {
    let n = input.len();
    if n <= 1 {
        return input.to_vec();
    }

    if (n & (n - 1)) == 0 {
        let mut out = input.to_vec();
        radix2_fft_in_place(&mut out, inverse);
        if inverse {
            let inv_n = 1.0 / n as f64;
            for x in &mut out {
                *x = x.scale(inv_n);
            }
        }
        out
    } else {
        let mut out = Vec::with_capacity(n);
        let sign = if inverse { 1.0 } else { -1.0 };
        let factor = 2.0 * PI * sign / n as f64;
        for k in 0..n {
            let mut sum = Complex::ZERO;
            for (j, item) in input.iter().enumerate() {
                let angle = factor * (k * j) as f64;
                let twiddle = Complex::new(angle.cos(), angle.sin());
                sum = sum + (*item * twiddle);
            }
            if inverse {
                sum = sum.scale(1.0 / n as f64);
            }
            out.push(sum);
        }
        out
    }
}

fn radix2_fft_in_place(data: &mut [Complex], inverse: bool) {
    let n = data.len();
    let mut j = 0;
    for i in 0..n {
        if i < j {
            data.swap(i, j);
        }
        let mut m = n >> 1;
        while m >= 1 && j >= m {
            j -= m;
            m >>= 1;
        }
        j += m;
    }

    let sign = if inverse { 1.0 } else { -1.0 };
    let mut len = 2;
    while len <= n {
        let half = len / 2;
        let angle = sign * 2.0 * PI / len as f64;
        let w_step = Complex::new(angle.cos(), angle.sin());

        for i in (0..n).step_by(len) {
            let mut w = Complex::ONE;
            for k in 0..half {
                let u = data[i + k];
                let v = data[i + k + half] * w;
                data[i + k] = u + v;
                data[i + k + half] = u - v;
                w = w * w_step;
            }
        }
        len <<= 1;
    }
}

/// High-performance Split-Step Fourier solver for the normalized Lugiato-Lefever Equation.
#[derive(Debug, Clone, PartialEq)]
pub struct LleSplitStepSolver {
    /// Resonator parameters.
    pub params: MicroresonatorParams,
    /// Intracavity field state.
    pub state: MicrocombState,
    /// Conservative / lossless limit mode (no linear dissipation and no drive).
    pub conservative: bool,
}

impl LleSplitStepSolver {
    /// Creates a new LLE solver instance with the given resonator parameters.
    pub fn new(params: MicroresonatorParams) -> Self {
        let mut state = MicrocombState::new(params.grid_size);
        state.alpha = params.alpha;
        state.f_drive = params.f_drive;
        let mut solver = Self {
            params,
            state,
            conservative: false,
        };
        solver.init_soliton(0.0);
        solver
    }

    /// Initializes a continuous wave (CW) state with small thermal noise.
    pub fn init_cw(&mut self) {
        let alpha = self.params.alpha;
        let f0 = self.params.f_drive;

        // Linear CW response: psi_0 = F_0 / (1 + i * alpha)
        let denom = 1.0 + alpha * alpha;
        let psi_cw = Complex::new(f0 / denom, -f0 * alpha / denom);

        for (idx, p) in self.state.psi.iter_mut().enumerate() {
            // Deterministic pseudo-noise seed
            let phase = (idx as f64 * 1.6180339887).sin() * 2.0 * PI;
            let noise_amp = 1.0e-4;
            *p = psi_cw + Complex::new(noise_amp * phase.cos(), noise_amp * phase.sin());
        }
        self.state.time = 0.0;
        self.state.recalculate_metrics();
        self.state.regime = self.classify_regime();
    }

    /// Initializes a bright dissipative Kerr soliton localized pulse at theta_0.
    pub fn init_soliton(&mut self, theta_0: f64) {
        let alpha = self.params.alpha.max(0.1);
        let f0 = self.params.f_drive;
        let d2_eff = self.params.normalized_dispersion();

        // Analytical soliton parameters
        // Soliton width: theta_s = sqrt(d2_eff / (2 * alpha))
        let theta_s = if d2_eff > 0.0 {
            (d2_eff / (2.0 * alpha)).sqrt().max(0.01)
        } else {
            0.05
        };

        // Peak soliton amplitude: B ~ sqrt(2 * alpha)
        let peak_amp = (2.0 * alpha).sqrt();

        // Low-power CW background: psi_0 ~ -i * F_0 / alpha
        let psi_bg = Complex::new(0.0, -f0 / alpha);

        // Approximate soliton phase: phi_s = 0.5 * arccos(2 * sqrt(2*alpha) / (pi * F_0))
        let phase_arg = (2.0 * (2.0 * alpha).sqrt() / (PI * f0.max(0.1))).clamp(-1.0, 1.0);
        let soliton_phase = 0.5 * phase_arg.acos();
        let phase_factor = Complex::cis(soliton_phase);

        for (idx, theta) in self.state.theta.iter().enumerate() {
            // Circular distance on [-pi, pi)
            let mut diff = (theta - theta_0).abs();
            if diff > PI {
                diff = 2.0 * PI - diff;
            }
            let x = diff / theta_s;
            let sech = 2.0 / (x.exp() + (-x).exp());

            let pulse = phase_factor.scale(peak_amp * sech);
            self.state.psi[idx] = psi_bg + pulse;
        }

        self.state.time = 0.0;
        self.state.recalculate_metrics();
        self.state.regime = self.classify_regime();
    }

    /// Initializes a Turing pattern roll state with `rolls` azimuthal periods.
    pub fn init_turing(&mut self, rolls: usize) {
        let m = rolls.max(2) as f64;
        let alpha = self.params.alpha;
        let f0 = self.params.f_drive;

        // Upper bistable branch CW field: intensity I_0 ~ alpha - 0.3 (> 1.0 threshold for MI)
        let i0 = (alpha - 0.3).max(1.2);
        let det_eff = alpha - i0;
        let denom = 1.0 + det_eff * det_eff;
        let psi_cw = Complex::new(f0 / denom, -f0 * det_eff / denom);
        let mod_amp = 0.75;

        for (idx, theta) in self.state.theta.iter().enumerate() {
            let modulation = mod_amp * (m * theta).cos();
            self.state.psi[idx] = psi_cw + Complex::new(modulation, 0.2 * modulation);
        }

        self.state.time = 0.0;
        self.state.recalculate_metrics();
        self.state.regime = self.classify_regime();
    }

    /// Toggles conservative / Hamiltonian lossless limit mode for energy testing.
    pub fn set_conservative(&mut self, conservative: bool) {
        self.conservative = conservative;
    }

    /// Integrates the LLE forward by one time step `dt` using the split-step Fourier method.
    pub fn step(&mut self, dt: f64) {
        let n = self.params.grid_size;
        let alpha = self.params.alpha;
        let d2_eff = self.params.normalized_dispersion();
        let f0 = if self.conservative { 0.0 } else { self.params.f_drive };

        // 1. Half-step linear propagation in Fourier domain
        let half_dt = 0.5 * dt;
        let mut k_modes = fft_1d(&self.state.psi, false);

        for (k, val) in k_modes.iter_mut().enumerate() {
            let mu = if k < n / 2 {
                k as f64
            } else {
                (k as f64) - (n as f64)
            };
            let gamma = if self.conservative { 0.0 } else { 1.0 };
            let lin_decay = (-gamma * half_dt).exp();
            let lin_phase = -(alpha + d2_eff * mu * mu) * half_dt;
            let propagator = Complex::cis(lin_phase).scale(lin_decay);
            *val = *val * propagator;
        }

        let mut real_psi = fft_1d(&k_modes, true);

        // 2. Full-step nonlinear phase rotation and pump drive in real space
        for p in &mut real_psi {
            *p = if f0 == 0.0 {
                // Exact unitary phase rotation in zero-drive / conservative limit
                let phase = p.norm_sq() * dt;
                *p * Complex::cis(phase)
            } else {
                // RK4 integration of d(psi)/dt = i * |psi|^2 * psi + F_0
                rk4_step(*p, f0, dt)
            };
        }

        // 3. Second half-step linear propagation in Fourier domain
        let mut k_modes2 = fft_1d(&real_psi, false);
        for (k, val) in k_modes2.iter_mut().enumerate() {
            let mu = if k < n / 2 {
                k as f64
            } else {
                (k as f64) - (n as f64)
            };
            let gamma = if self.conservative { 0.0 } else { 1.0 };
            let lin_decay = (-gamma * half_dt).exp();
            let lin_phase = -(alpha + d2_eff * mu * mu) * half_dt;
            let propagator = Complex::cis(lin_phase).scale(lin_decay);
            *val = *val * propagator;
        }

        self.state.psi = fft_1d(&k_modes2, true);
        self.state.time += dt;
        self.state.recalculate_metrics();
        self.state.regime = self.classify_regime();
    }

    /// Runs the split-step Fourier solver until steady state (for `steps` steps).
    pub fn run_to_steady_state(&mut self, steps: usize) -> MicrocombState {
        let dt = 0.01;
        for _ in 0..steps {
            self.step(dt);
        }
        self.state.clone()
    }

    /// Sweeps the normalized detuning alpha from `start_alpha` to `end_alpha` over `steps`.
    ///
    /// Identifies the low-power CW region, Turing pattern rolls, modulation instability
    /// chaos, and the characteristic dissipative soliton step / plateau.
    pub fn sweep_detuning(
        &mut self,
        start_alpha: f64,
        end_alpha: f64,
        steps: usize,
    ) -> DetuningScanResult {
        let num_steps = steps.max(20);
        let d_alpha = (end_alpha - start_alpha) / (num_steps - 1) as f64;

        let mut alphas = Vec::with_capacity(num_steps);
        let mut powers = Vec::with_capacity(num_steps);
        let mut peak_powers = Vec::with_capacity(num_steps);
        let mut regimes = Vec::with_capacity(num_steps);

        // Reset to initial alpha and run transient warm-up
        self.params.alpha = start_alpha;
        self.init_soliton(0.0);
        self.run_to_steady_state(60);

        let mut soliton_start = None;
        let mut soliton_end = None;
        let mut turing_start = None;
        let mut turing_end = None;
        let mut chaos_start = None;
        let mut chaos_end = None;
        let mut best_soliton: Option<MicrocombState> = None;
        let mut max_soliton_contrast = 0.0;

        for step_idx in 0..num_steps {
            let current_alpha = start_alpha + (step_idx as f64) * d_alpha;
            self.params.alpha = current_alpha;
            self.state.alpha = current_alpha;

            // Evolve for short settling interval at this alpha
            for _ in 0..25 {
                self.step(0.015);
            }

            let reg = self.classify_regime();
            let p_mean = self.state.mean_power;
            let p_peak = self.state.peak_power;

            alphas.push(current_alpha);
            powers.push(p_mean);
            peak_powers.push(p_peak);
            regimes.push(reg);

            match reg {
                MicrocombRegime::TuringRolls => {
                    if turing_start.is_none() {
                        turing_start = Some(current_alpha);
                    }
                    turing_end = Some(current_alpha);
                }
                MicrocombRegime::ModulationInstabilityChaos => {
                    if chaos_start.is_none() {
                        chaos_start = Some(current_alpha);
                    }
                    chaos_end = Some(current_alpha);
                }
                MicrocombRegime::DissipativeSoliton | MicrocombRegime::BreatherSoliton => {
                    if soliton_start.is_none() {
                        soliton_start = Some(current_alpha);
                    }
                    soliton_end = Some(current_alpha);

                    let contrast = p_peak / (p_mean.max(1e-6));
                    if contrast > max_soliton_contrast {
                        max_soliton_contrast = contrast;
                        best_soliton = Some(self.state.clone());
                    }
                }
                _ => {}
            }
        }

        let soliton_step_range = match (soliton_start, soliton_end) {
            (Some(s), Some(e)) if e >= s => Some((s, e)),
            _ => None,
        };
        let turing_roll_range = match (turing_start, turing_end) {
            (Some(s), Some(e)) if e >= s => Some((s, e)),
            _ => None,
        };
        let chaos_range = match (chaos_start, chaos_end) {
            (Some(s), Some(e)) if e >= s => Some((s, e)),
            _ => None,
        };

        DetuningScanResult {
            alphas,
            powers,
            peak_powers,
            regimes,
            soliton_step_range,
            turing_roll_range,
            chaos_range,
            best_soliton_state: best_soliton,
        }
    }

    /// Classifies the current intracavity state into a physical microcomb regime.
    pub fn classify_regime(&self) -> MicrocombRegime {
        let p_peak = self.state.peak_power;
        let p_mean = self.state.mean_power;
        let p_min = self.state.min_power;

        if p_mean < 1e-6 {
            return MicrocombRegime::LowPowerCw;
        }

        let contrast = (p_peak - p_min) / (p_peak + p_min + 1e-12);
        let peak_to_avg = p_peak / p_mean;

        // Flat or low contrast: continuous wave
        if contrast < 0.12 || peak_to_avg < 1.25 {
            return MicrocombRegime::LowPowerCw;
        }

        // Count significant local intensity peaks across the azimuthal grid
        let threshold = p_min + 0.25 * (p_peak - p_min);
        let n = self.state.psi.len();
        let mut peak_indices = Vec::new();

        for i in 0..n {
            let prev = (i + n - 1) % n;
            let next = (i + 1) % n;
            let val = self.state.psi[i].norm_sq();
            if val > threshold
                && val >= self.state.psi[prev].norm_sq()
                && val >= self.state.psi[next].norm_sq()
            {
                peak_indices.push(i);
            }
        }

        let num_peaks = peak_indices.len();

        if num_peaks == 1 || (num_peaks <= 3 && peak_to_avg > 3.0) {
            MicrocombRegime::DissipativeSoliton
        } else if num_peaks >= 4 && num_peaks <= 32 {
            // Check peak height uniformity for Turing rolls
            let mut peak_vals = Vec::with_capacity(num_peaks);
            for &idx in &peak_indices {
                peak_vals.push(self.state.psi[idx].norm_sq());
            }
            let mean_pk: f64 = peak_vals.iter().sum::<f64>() / num_peaks as f64;
            let variance: f64 = peak_vals
                .iter()
                .map(|&v| (v - mean_pk).powi(2))
                .sum::<f64>()
                / num_peaks as f64;
            let std_rel = variance.sqrt() / mean_pk.max(1e-6);

            if std_rel < 0.42 {
                MicrocombRegime::TuringRolls
            } else {
                MicrocombRegime::ModulationInstabilityChaos
            }
        } else {
            MicrocombRegime::ModulationInstabilityChaos
        }
    }
}

/// 4th-order Runge-Kutta step for the local nonlinear equation:
///   d(psi)/dt = i * |psi|^2 * psi + F_0
#[inline]
fn rk4_step(psi: Complex, f0: f64, dt: f64) -> Complex {
    let rhs = |p: Complex| -> Complex {
        let n_sq = p.norm_sq();
        // i * |p|^2 * p + F_0 = (-n_sq * p.im + F_0) + i * (n_sq * p.re)
        Complex::new(-n_sq * p.im + f0, n_sq * p.re)
    };

    let k1 = rhs(psi);
    let k2 = rhs(psi + k1.scale(0.5 * dt));
    let k3 = rhs(psi + k2.scale(0.5 * dt));
    let k4 = rhs(psi + k3.scale(dt));

    psi + (k1 + k2.scale(2.0) + k3.scale(2.0) + k4).scale(dt / 6.0)
}
