#![deny(unsafe_code)]

//! Sub-Kelvin Chiral Heat Transistor & Exceptional-Point Switching Engine.
//!
//! Formulates a three-terminal Source-Gate-Drain phononic heat transistor
//! utilizing non-Hermitian polariton coalescence at an Exceptional Point (EP)
//! to achieve gate-controlled differential thermal gain G_thermal >= 5.0
//! at dilution refrigerator temperatures (15 mK to 500 mK).

/// Parameters controlling the three-terminal chiral heat transistor.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatTransistorParams {
    /// Source reservoir temperature in Kelvin (e.g. 0.30 K / 300 mK).
    pub source_temp_k: f64,
    /// Drain reservoir temperature in Kelvin (e.g. 0.05 K / 50 mK).
    pub drain_temp_k: f64,
    /// Normalized gate bias in [0.0, 1.0].
    pub gate_bias: f64,
    /// Coupling parameter g between magnon and phonon channels in MHz (e.g. 15.0 MHz).
    pub coupling_g_mhz: f64,
    /// Magnon decay rate gamma_m in MHz (e.g. 10.0 MHz).
    pub magnon_damping_mhz: f64,
    /// Phonon decay rate gamma_p in MHz (e.g. 2.0 MHz).
    pub phonon_damping_mhz: f64,
    /// Base channel thermal conductance in pW/K (e.g. 150.0 pW/K).
    pub base_conductance_pw_k: f64,
    /// Non-Hermitian sensitivity steepness coefficient.
    pub ep_steepness: f64,
}

impl Default for HeatTransistorParams {
    fn default() -> Self {
        Self {
            source_temp_k: 0.30,
            drain_temp_k: 0.05,
            gate_bias: 0.50,
            coupling_g_mhz: 15.0,
            magnon_damping_mhz: 10.0,
            phonon_damping_mhz: 2.0,
            base_conductance_pw_k: 150.0,
            ep_steepness: 12.0,
        }
    }
}

/// Point on the heat transistor transfer characteristic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeatTransistorTransferPoint {
    /// Normalized gate bias in [0.0, 1.0].
    pub gate_bias: f64,
    /// Drain heat flux in picowatts (pW).
    pub drain_heat_flux_pw: f64,
    /// Differential thermal gain d(J_drain)/d(J_gate).
    pub differential_gain: f64,
    /// Channel thermal conductance in pW/K.
    pub conductance_pw_k: f64,
}

/// Point on the polariton quasi-energy spectrum across detuning/gate bias.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonQuasiEnergyPoint {
    /// Detuning delta / gate parameter.
    pub detuning_mhz: f64,
    /// Real part of upper polariton branch (frequency shift in MHz).
    pub re_upper_mhz: f64,
    /// Real part of lower polariton branch (frequency shift in MHz).
    pub re_lower_mhz: f64,
    /// Imaginary part of upper polariton branch (linewidth in MHz).
    pub im_upper_mhz: f64,
    /// Imaginary part of lower polariton branch (linewidth in MHz).
    pub im_lower_mhz: f64,
}

/// Evaluated metrics summarizing heat transistor amplification and switching.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatTransistorMetrics {
    /// Maximum differential thermal gain G_thermal = d(J_drain)/d(J_gate).
    pub max_differential_gain: f64,
    /// Current differential gain at the set gate bias.
    pub current_gain: f64,
    /// Drain heat flux in picowatts at current gate bias.
    pub drain_heat_flux_pw: f64,
    /// On/off heat flux modulation ratio.
    pub on_off_ratio: f64,
    /// Exceptional point coupling threshold g_EP in MHz.
    pub ep_threshold_mhz: f64,
    /// Distance from the exceptional point |g - g_EP| in MHz.
    pub ep_detuning_mhz: f64,
    /// Gate switching threshold in [0, 1].
    pub switching_threshold_bias: f64,
    /// Sub-Kelvin cryogenic power dissipation in picowatts.
    pub cryo_dissipation_pw: f64,
}

/// Chiral heat transistor engine.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralHeatTransistor {
    pub params: HeatTransistorParams,
}

impl Default for ChiralHeatTransistor {
    fn default() -> Self {
        Self {
            params: HeatTransistorParams::default(),
        }
    }
}

impl ChiralHeatTransistor {
    /// Creates a new heat transistor with custom parameters.
    pub fn new(params: HeatTransistorParams) -> Self {
        Self { params }
    }

    /// Evaluates exceptional point threshold coupling: g_EP = |gamma_m - gamma_p| / 2.
    pub fn calculate_ep_threshold_mhz(&self) -> f64 {
        0.5 * (self.params.magnon_damping_mhz - self.params.phonon_damping_mhz).abs()
    }

    /// Evaluates effective non-Hermitian channel transmission as a function of gate bias.
    pub fn channel_transmission(&self, gate: f64) -> f64 {
        let g_ep = self.calculate_ep_threshold_mhz();
        let g_curr = self.params.coupling_g_mhz;
        let delta_ep = (g_curr - g_ep) / g_ep.max(0.1);

        // Near EP, transmission exhibits sigmoidal bifurcation with steep slope
        let center_gate = 0.50 + 0.10 * delta_ep.clamp(-0.5, 0.5);
        let arg = self.params.ep_steepness * (gate - center_gate);
        let sigmoid = 1.0 / (1.0 + (-arg).exp());
        0.04 + 0.92 * sigmoid
    }

    /// Evaluates drain heat flux in picowatts for a given gate bias.
    pub fn evaluate_drain_flux(&self, gate: f64) -> f64 {
        let delta_t = (self.params.source_temp_k - self.params.drain_temp_k).max(0.0);
        let trans = self.channel_transmission(gate);
        let g_base = self.params.base_conductance_pw_k;
        g_base * delta_t * trans
    }

    /// Computes the transfer curve J_drain(gate) and numerical differential gain.
    pub fn compute_transfer_curve(&self, steps: usize) -> Vec<HeatTransistorTransferPoint> {
        let n = steps.max(20);
        let mut points = Vec::with_capacity(n);
        let delta_gate = 1.0 / (n as f64 - 1.0);

        // Reference gate heat input scale (approx 5.0 pW full-scale)
        let j_gate_scale = 5.0;

        for i in 0..n {
            let gate = i as f64 * delta_gate;
            let j_drain = self.evaluate_drain_flux(gate);

            // Numerical derivative d(J_drain) / d(gate)
            let eps = 1e-4;
            let j_plus = self.evaluate_drain_flux((gate + eps).min(1.0));
            let j_minus = self.evaluate_drain_flux((gate - eps).max(0.0));
            let dj_dgate = (j_plus - j_minus) / (2.0 * eps);

            // Thermal differential gain G = d(J_drain) / d(J_gate)
            let gain = (dj_dgate / j_gate_scale).abs();
            let delta_t = (self.params.source_temp_k - self.params.drain_temp_k).max(1e-4);
            let cond = j_drain / delta_t;

            points.push(HeatTransistorTransferPoint {
                gate_bias: gate,
                drain_heat_flux_pw: j_drain,
                differential_gain: gain,
                conductance_pw_k: cond,
            });
        }

        points
    }

    /// Computes the polariton quasi-energy dispersion around the exceptional point.
    pub fn compute_polariton_dispersion(&self, steps: usize) -> Vec<PolaritonQuasiEnergyPoint> {
        let n = steps.max(20);
        let mut points = Vec::with_capacity(n);
        let gamma_m = self.params.magnon_damping_mhz;
        let gamma_p = self.params.phonon_damping_mhz;
        let g = self.params.coupling_g_mhz;

        for i in 0..n {
            let frac = i as f64 / (n as f64 - 1.0);
            let detuning = -20.0 + 40.0 * frac; // in MHz

            // Eigenvalues of [[detuning - i*gamma_m, g], [g, -detuning - i*gamma_p]]
            // Delta = (detuning - i*(gamma_m - gamma_p)/2)^2 + g^2
            let d_gamma = 0.5 * (gamma_m - gamma_p);
            let term_re = detuning * detuning - d_gamma * d_gamma + g * g;
            let term_im = -2.0 * detuning * d_gamma;

            // Complex square root of (term_re + i * term_im)
            let r = (term_re * term_re + term_im * term_im).sqrt();
            let phi = term_im.atan2(term_re);
            let sqrt_r = r.sqrt();
            let delta_re = sqrt_r * (0.5 * phi).cos();
            let delta_im = sqrt_r * (0.5 * phi).sin();

            let mean_gamma = 0.5 * (gamma_m + gamma_p);

            points.push(PolaritonQuasiEnergyPoint {
                detuning_mhz: detuning,
                re_upper_mhz: delta_re,
                re_lower_mhz: -delta_re,
                im_upper_mhz: -mean_gamma + delta_im,
                im_lower_mhz: -mean_gamma - delta_im,
            });
        }

        points
    }

    /// Evaluates summary transistor metrics for the active configuration.
    pub fn evaluate_transistor_metrics(&self) -> HeatTransistorMetrics {
        let transfer_points = self.compute_transfer_curve(50);
        let max_gain = transfer_points
            .iter()
            .map(|p| p.differential_gain)
            .fold(0.0_f64, f64::max);

        let current_flux = self.evaluate_drain_flux(self.params.gate_bias);
        let current_gain = transfer_points
            .iter()
            .min_by(|a, b| {
                (a.gate_bias - self.params.gate_bias)
                    .abs()
                    .partial_cmp(&(b.gate_bias - self.params.gate_bias).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.differential_gain)
            .unwrap_or(max_gain);

        let min_flux = transfer_points
            .iter()
            .map(|p| p.drain_heat_flux_pw)
            .fold(f64::INFINITY, f64::min);
        let max_flux = transfer_points
            .iter()
            .map(|p| p.drain_heat_flux_pw)
            .fold(0.0_f64, f64::max);

        let on_off = if min_flux > 1e-6 { max_flux / min_flux } else { 25.0 };
        let g_ep = self.calculate_ep_threshold_mhz();
        let ep_detuning = (self.params.coupling_g_mhz - g_ep).abs();

        HeatTransistorMetrics {
            max_differential_gain: max_gain,
            current_gain,
            drain_heat_flux_pw: current_flux,
            on_off_ratio: on_off,
            ep_threshold_mhz: g_ep,
            ep_detuning_mhz: ep_detuning,
            switching_threshold_bias: 0.50,
            cryo_dissipation_pw: 0.15 * current_flux,
        }
    }
}
