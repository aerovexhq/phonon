//! Floquet-Bloch quantum time crystals, subharmonic phonon states,
//! many-body localization, and non-equilibrium symmetry breaking.

/// Parameters for Floquet-Bloch quantum acoustic time crystals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumTimeCrystalParams {
    /// Number of acoustic lattice sites in the 1D chain $L$ (nominal $12 - 32$).
    pub lattice_size: usize,
    /// Floquet driving period $T_{\mathrm{drive}}$ in nanoseconds (nominal $2.0 - 20.0\text{ ns}$).
    pub drive_period_ns: f64,
    /// Pulse rotation angle imperfection $\epsilon$ away from ideal $\pi$-flip (nominal $0.01 - 0.15$).
    pub pulse_imperfection_epsilon: f64,
    /// Random on-site disorder strength $W / J_0$ protecting many-body localization (nominal $1.5 - 5.0$).
    pub disorder_strength_w: f64,
    /// Disordered nearest-neighbor acoustic spin/phonon coupling $J_0$ in $\text{MHz}$ (nominal $5.0 - 50.0\text{ MHz}$).
    pub interaction_coupling_j0_mhz: f64,
    /// Fractional coupling disorder fluctuation $\delta J / J_0$ (nominal $0.10 - 0.40$).
    pub interaction_fluctuation_dj: f64,
    /// Intrinsic acoustic phonon damping rate $\gamma_{\mathrm{damp}}$ in $\text{kHz}$ (nominal $0.2 - 5.0\text{ kHz}$).
    pub acoustic_damping_rate_khz: f64,
    /// Sub-Kelvin cryogenic operating temperature $T_{\mathrm{bath}}$ in Kelvin (nominal $0.01 - 0.50\text{ K}$).
    pub thermal_bath_temp_k: f64,
}

impl Default for QuantumTimeCrystalParams {
    fn default() -> Self {
        Self {
            lattice_size: 20,
            drive_period_ns: 8.0,
            pulse_imperfection_epsilon: 0.05,
            disorder_strength_w: 2.8,
            interaction_coupling_j0_mhz: 25.0,
            interaction_fluctuation_dj: 0.20,
            acoustic_damping_rate_khz: 1.0,
            thermal_bath_temp_k: 0.08,
        }
    }
}

/// Evaluated metrics for Floquet quantum time crystals and subharmonic states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimeCrystalMetrics {
    /// Subharmonic response period multiplier $n$ (nominal $2$ for period-doubled discrete time crystals).
    pub subharmonic_period_multiplier: usize,
    /// Time crystalline order lifetime in driving cycles $\tau_{\mathrm{DTC}} \ge 1000\text{ cycles}$.
    pub time_crystal_lifetime_cycles: f64,
    /// Subharmonic spectral rigidity peak sharpness in decibels ($\ge 20.0\text{ dB}$).
    pub spectral_rigidity_contrast_db: f64,
    /// Subharmonic spectral peak quality factor $\mathcal{Q}_{\mathrm{rigid}} \ge 100.0$.
    pub spectral_rigidity_quality_factor: f64,
    /// Subharmonic response frequency in $\text{MHz}$ ($\nu_{\mathrm{drive}} / n$).
    pub subharmonic_frequency_mhz: f64,
    /// Normalized subharmonic Fourier peak amplitude $S(\pi/T) \in [0.85, 1.0]$.
    pub fourier_peak_amplitude: f64,
    /// Subharmonic quantum acoustic memory state fidelity $\mathcal{F}_{\mathrm{memory}} \ge 90.0\%$.
    pub subharmonic_memory_fidelity: f64,
    /// Fractional frequency instability Allan deviation floor $\sigma_y \le 10^{-11}$.
    pub fractional_frequency_stability: f64,
}

impl QuantumTimeCrystalParams {
    /// Creates a new parameter set for quantum acoustic time crystals.
    pub fn new(
        drive_period_ns: f64,
        pulse_imperfection_epsilon: f64,
        disorder_strength_w: f64,
    ) -> Self {
        Self {
            drive_period_ns: drive_period_ns.max(0.5),
            pulse_imperfection_epsilon: pulse_imperfection_epsilon.clamp(0.001, 0.30),
            disorder_strength_w: disorder_strength_w.max(0.5),
            ..Default::default()
        }
    }
}
