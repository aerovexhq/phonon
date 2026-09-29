//! Parameters and metrics for quantum topological soliton microcavities
//! and octave-spanning phonon frequency comb synthesis.

/// Parameters for topological acoustic Kerr soliton microcavity resonators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalSolitonCombParams {
    /// Resonator central acoustic mode frequency $f_0$ in $\text{GHz}$ (nominal $2.0 - 10.0\text{ GHz}$).
    pub cavity_resonance_freq_ghz: f64,
    /// Cavity free spectral range $\mathrm{FSR} = D_1 / (2\pi)$ in $\text{MHz}$ (nominal $40.0 - 300.0\text{ MHz}$).
    pub free_spectral_range_mhz: f64,
    /// Loaded topological boundary mode quality factor $\mathcal{Q}$ (nominal $1.0\times 10^5 - 8.0\times 10^6$).
    pub loaded_q_factor: f64,
    /// Anomalous second-order group velocity dispersion $D_2 / (2\pi)$ in $\text{kHz}$ (nominal $15.0 - 90.0\text{ kHz}$).
    pub dispersion_d2_khz: f64,
    /// Non-linear phononic Kerr frequency shift $g_{\mathrm{Kerr}} / (2\pi)$ in $\text{Hz}$ (nominal $10.0 - 80.0\text{ Hz}$).
    pub kerr_coefficient_hz: f64,
    /// Continuous-wave acoustic pump power in milliwatts (nominal $0.5 - 12.0\text{ mW}$).
    pub pump_power_mw: f64,
    /// Normalized cavity detuning $\Delta / (\kappa / 2) \in [1.5, 7.0]$.
    pub normalized_detuning: f64,
}

impl Default for TopologicalSolitonCombParams {
    fn default() -> Self {
        Self {
            cavity_resonance_freq_ghz: 4.5,
            free_spectral_range_mhz: 120.0,
            loaded_q_factor: 1.5e6,
            dispersion_d2_khz: 35.0,
            kerr_coefficient_hz: 30.0,
            pump_power_mw: 4.0,
            normalized_detuning: 3.5,
        }
    }
}

/// Evaluated metrics for topological phononic soliton frequency combs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalSolitonCombMetrics {
    /// Comb spectral bandwidth span in octaves ($\ge 2.0\text{ octaves}$).
    pub comb_span_octaves: f64,
    /// Repetition-rate timing jitter in femtoseconds ($\le 10.0\text{ fs}$).
    pub timing_jitter_fs: f64,
    /// Metrological beat-note signal-to-noise ratio in decibels ($\ge 30.0\text{ dB}$).
    pub beat_note_snr_db: f64,
    /// Temporal dissipative acoustic soliton duration in picoseconds ($\le 15.0\text{ ps}$).
    pub soliton_pulse_duration_ps: f64,
    /// Total phase-locked frequency comb lines count ($\ge 100$).
    pub comb_lines_count: usize,
    /// Fractional frequency instability Allan deviation floor ($\le 1.0\times 10^{-12}$).
    pub allan_deviation_floor: f64,
}

impl TopologicalSolitonCombParams {
    /// Creates a new parameter set for topological soliton combs.
    pub fn new(f0_ghz: f64, fsr_mhz: f64, q_factor: f64, d2_khz: f64, power_mw: f64) -> Self {
        Self {
            cavity_resonance_freq_ghz: f0_ghz.max(0.5),
            free_spectral_range_mhz: fsr_mhz.max(10.0),
            loaded_q_factor: q_factor.max(1.0e4),
            dispersion_d2_khz: d2_khz.max(1.0),
            pump_power_mw: power_mw.max(0.1),
            ..Default::default()
        }
    }
}
