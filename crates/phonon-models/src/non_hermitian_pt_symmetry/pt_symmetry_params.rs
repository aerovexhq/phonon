//! Parameters and metrics for non-Hermitian phononic parity-time (PT)
//! symmetry breaking, exceptional points, and ultrasensitive acoustic sensors.

/// Parameters for coupled non-Hermitian PT-symmetric acoustic resonators and transmission lines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PtSymmetryParams {
    /// Resonator central acoustic frequency $f_0$ in $\text{GHz}$ (nominal $1.0 - 8.0\text{ GHz}$).
    pub acoustic_resonance_freq_ghz: f64,
    /// Inter-resonator acoustic coupling rate $J / (2\pi)$ in $\text{MHz}$ (nominal $5.0 - 60.0\text{ MHz}$).
    pub intercavity_coupling_mhz: f64,
    /// Balanced acoustic gain/loss rate $\gamma / (2\pi)$ in $\text{MHz}$ (nominal $5.0 - 60.0\text{ MHz}$).
    pub gain_loss_rate_mhz: f64,
    /// Perturbation frequency detuning $\delta f / (2\pi)$ in $\text{kHz}$ representing target analyte / mass loading (nominal $0.1 - 50.0\text{ kHz}$).
    pub perturbation_detuning_khz: f64,
    /// Intrinsic unloaded resonator quality factor $\mathcal{Q}_0$ (nominal $5.0\times 10^4 - 2.0\times 10^6$).
    pub intrinsic_q_factor: f64,
    /// Active piezoelectric pumping power required to sustain gain in milliwatts (nominal $0.1 - 3.0\text{ mW}$).
    pub active_gain_power_mw: f64,
    /// Synthetic non-reciprocal phase bias $\phi_{\mathrm{NR}}$ in radians (nominal $0.0 - \pi$).
    pub non_reciprocal_phase_rad: f64,
}

impl Default for PtSymmetryParams {
    fn default() -> Self {
        Self {
            acoustic_resonance_freq_ghz: 3.5,
            intercavity_coupling_mhz: 25.0,
            gain_loss_rate_mhz: 24.8, // tuned near exceptional point gamma ~ J
            perturbation_detuning_khz: 5.0,
            intrinsic_q_factor: 5.0e5,
            active_gain_power_mw: 0.85,
            non_reciprocal_phase_rad: std::f64::consts::FRAC_PI_2,
        }
    }
}

/// Evaluated metrics for acoustic PT-symmetry breaking, exceptional points, and sensing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PtSymmetryMetrics {
    /// Square-root sensitivity enhancement over classical linear sensors in decibels ($\ge 35.0\text{ dB}$).
    pub sensitivity_enhancement_db: f64,
    /// Threshold power required to sustain PT-symmetric balanced gain in milliwatts ($\le 2.0\text{ mW}$).
    pub threshold_power_mw: f64,
    /// Non-reciprocal acoustic circulator reverse isolation in decibels ($\ge 25.0\text{ dB}$).
    pub reverse_isolation_db: f64,
    /// Coherent directional perfect absorption efficiency in percent ($\ge 90.0\%$).
    pub directional_absorption_pct: f64,
    /// Petermann phase rigidity factor near the exceptional point $|\langle \psi_L | \psi_R \rangle| \le 0.20$.
    pub pt_phase_rigidity: f64,
    /// Exceptional point eigenvector coalescence fidelity in percent ($\ge 95.0\%$).
    pub coalescence_fidelity_pct: f64,
}

impl PtSymmetryParams {
    /// Creates a new parameter set for acoustic PT-symmetry.
    pub fn new(
        f0_ghz: f64,
        coupling_mhz: f64,
        gain_loss_mhz: f64,
        detuning_khz: f64,
        power_mw: f64,
    ) -> Self {
        Self {
            acoustic_resonance_freq_ghz: f0_ghz.clamp(0.5, 20.0),
            intercavity_coupling_mhz: coupling_mhz.clamp(1.0, 200.0),
            gain_loss_rate_mhz: gain_loss_mhz.clamp(1.0, 200.0),
            perturbation_detuning_khz: detuning_khz.clamp(0.01, 500.0),
            active_gain_power_mw: power_mw.clamp(0.01, 10.0),
            ..Default::default()
        }
    }
}
