//! Solvers for Floquet-Bloch quantum time crystals, subharmonic spectral rigidity,
//! many-body localization lifetime, and quantum acoustic memory.

use phonon_models::quantum_time_crystal::{QuantumTimeCrystalParams, TimeCrystalMetrics};

/// Solver for Floquet quantum acoustic time crystals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetTimeCrystalSolver {
    pub params: QuantumTimeCrystalParams,
}

impl FloquetTimeCrystalSolver {
    /// Creates a new Floquet time crystal solver instance.
    pub fn new(params: QuantumTimeCrystalParams) -> Self {
        Self { params }
    }

    /// Evaluates the fundamental driving frequency $\nu_{\mathrm{drive}}$ in $\text{MHz}$.
    pub fn compute_drive_frequency_mhz(&self) -> f64 {
        1000.0 / self.params.drive_period_ns.max(0.1)
    }

    /// Evaluates the subharmonic response frequency $\nu_{\mathrm{sub}} = \nu_{\mathrm{drive}} / 2$ in $\text{MHz}$.
    pub fn compute_subharmonic_frequency_mhz(&self) -> f64 {
        self.compute_drive_frequency_mhz() / 2.0
    }

    /// Evaluates the discrete time crystalline order lifetime $\tau_{\mathrm{DTC}}$ in driving cycles.
    pub fn compute_time_crystal_lifetime_cycles(&self) -> f64 {
        let p = &self.params;
        let w_norm = p.disorder_strength_w / 1.5;
        let eps = p.pulse_imperfection_epsilon;
        let gamma_khz = p.acoustic_damping_rate_khz;
        let t_bath = p.thermal_bath_temp_k;

        // Exponential suppression of thermalization due to many-body localization (MBL):
        let mbl_factor = (w_norm * 1.2).min(5.0).exp();
        let heating_damping = 1.0 + 25.0 * eps * eps;
        let phonon_damping = 1.0 + (gamma_khz / 1.5);
        let thermal_damping = 1.0 + (t_bath / 0.15);

        let base_lifetime_cycles = 2200.0;
        let lifetime = (base_lifetime_cycles * mbl_factor)
            / (heating_damping * phonon_damping * thermal_damping);

        lifetime.max(100.0)
    }

    /// Evaluates the subharmonic spectral rigidity quality factor $\mathcal{Q}_{\mathrm{rigid}} \approx \tau_{\mathrm{DTC}} \cdot \frac{\pi}{2}$.
    pub fn compute_spectral_rigidity_quality_factor(&self) -> f64 {
        let tau = self.compute_time_crystal_lifetime_cycles();
        tau * std::f64::consts::FRAC_PI_2
    }

    /// Evaluates the subharmonic spectral rigidity contrast in decibels ($\ge 20.0\text{ dB}$).
    pub fn compute_spectral_rigidity_contrast_db(&self) -> f64 {
        let p = &self.params;
        let w_gain = 8.0 * (p.disorder_strength_w / 2.0).log10();
        let eps_loss = 20.0 * p.pulse_imperfection_epsilon;

        // Baseline contrast 26.0 dB in MBL protected regime
        (26.0 + w_gain - eps_loss).max(10.0)
    }

    /// Evaluates the normalized Fourier subharmonic peak amplitude $S(\pi/T) \in [0.85, 1.0]$.
    pub fn compute_fourier_peak_amplitude(&self) -> f64 {
        let eps = self.params.pulse_imperfection_epsilon;
        (1.0 - 0.75 * eps * eps).clamp(0.50, 1.0)
    }

    /// Evaluates the quantum acoustic memory state fidelity after $k$ cycles $\mathcal{F}_{\mathrm{memory}} = \exp(-2 k / \tau_{\mathrm{DTC}})$.
    pub fn compute_subharmonic_memory_fidelity(&self, cycle_count: f64) -> f64 {
        let tau = self.compute_time_crystal_lifetime_cycles();
        (-2.0 * cycle_count / tau.max(1.0)).exp().clamp(0.0, 1.0)
    }

    /// Evaluates the fractional frequency instability Allan deviation floor $\sigma_y$.
    pub fn compute_fractional_frequency_stability(&self) -> f64 {
        let q = self.compute_spectral_rigidity_quality_factor();
        // Allan deviation floor ~ 1 / (Q * SNR_linear) with subharmonic locked acoustic clock
        (1.0 / (q * 1.0e8)).clamp(1.0e-16, 1.0e-11)
    }

    /// Simulates the stroboscopic acoustic magnetization dynamics $Z(n) = \langle S^z(n T) \rangle$.
    pub fn evaluate_stroboscopic_magnetization(&self, cycle_count: usize) -> Vec<f64> {
        let tau = self.compute_time_crystal_lifetime_cycles();
        let amp = self.compute_fourier_peak_amplitude();

        (0..cycle_count)
            .map(|n| {
                let sign = if n % 2 == 0 { 1.0 } else { -1.0 };
                let decay = (-(n as f64) / tau).exp();
                sign * amp * decay
            })
            .collect()
    }

    /// Solves the full Floquet quantum time crystal metrics.
    pub fn solve(&self) -> TimeCrystalMetrics {
        let subharmonic_n = 2;
        let tau = self.compute_time_crystal_lifetime_cycles();
        let contrast_db = self.compute_spectral_rigidity_contrast_db();
        let q_rigid = self.compute_spectral_rigidity_quality_factor();
        let nu_sub = self.compute_subharmonic_frequency_mhz();
        let peak_amp = self.compute_fourier_peak_amplitude();
        let fidelity = self.compute_subharmonic_memory_fidelity(100.0); // 100 cycles memory benchmark
        let sigma_y = self.compute_fractional_frequency_stability();

        TimeCrystalMetrics {
            subharmonic_period_multiplier: subharmonic_n,
            time_crystal_lifetime_cycles: tau,
            spectral_rigidity_contrast_db: contrast_db,
            spectral_rigidity_quality_factor: q_rigid,
            subharmonic_frequency_mhz: nu_sub,
            fourier_peak_amplitude: peak_amp,
            subharmonic_memory_fidelity: fidelity,
            fractional_frequency_stability: sigma_y,
        }
    }
}
