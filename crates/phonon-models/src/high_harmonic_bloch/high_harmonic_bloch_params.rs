//! Parameters and metrics for high-harmonic acoustic Bloch oscillations
//! and phononic frequency synthesizers in acoustic superlattices.

/// Parameters for acoustic superlattice Bloch oscillations and high-harmonic emission.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighHarmonicBlochParams {
    /// Acoustic superlattice spatial period $d_{\mathrm{SL}}$ in nanometers (nominal $10.0 - 40.0\text{ nm}$).
    pub superlattice_period_nm: f64,
    /// Acoustic miniband energy width $\Delta_n$ in meV (nominal $0.5 - 5.0\text{ meV}$).
    pub miniband_width_mev: f64,
    /// Inter-miniband energy bandgap $\Delta_{\mathrm{gap}}$ in meV (nominal $3.0 - 15.0\text{ meV}$).
    pub miniband_gap_mev: f64,
    /// Effective elastodynamic force field $F_{\mathrm{el}}$ in meV/nm (nominal $0.05 - 0.35\text{ meV/nm}$).
    pub effective_force_field_mev_nm: f64,
    /// Semiclassical acoustic wavepacket dephasing time $\tau_{\mathrm{deph}}$ in picoseconds (nominal $3.0 - 20.0\text{ ps}$).
    pub dephasing_time_ps: f64,
    /// Non-linear acoustic drive amplification factor (nominal $1.0 - 2.5$).
    pub non_linear_drive_factor: f64,
}

impl Default for HighHarmonicBlochParams {
    fn default() -> Self {
        Self {
            superlattice_period_nm: 16.0,
            miniband_width_mev: 1.8,
            miniband_gap_mev: 7.2,
            effective_force_field_mev_nm: 0.12,
            dephasing_time_ps: 9.0,
            non_linear_drive_factor: 1.45,
        }
    }
}

/// Evaluated metrics for high-harmonic acoustic Bloch oscillations and frequency synthesis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighHarmonicBlochMetrics {
    /// Fundamental acoustic Bloch oscillation frequency in GHz ($\ge 50.0\text{ GHz}$).
    pub bloch_frequency_ghz: f64,
    /// High-harmonic emission cutoff order $N_{\mathrm{cutoff}}$ ($\ge 25$).
    pub harmonic_cutoff_order: usize,
    /// Spectral purity / sideband suppression in dB ($\ge 45.0\text{ dB}$).
    pub spectral_purity_db: f64,
    /// Coherent Bloch oscillation count before dephasing $N_{\mathrm{osc}}$ ($\ge 3.0$).
    pub coherent_oscillations_count: f64,
    /// Broadband phononic frequency synthesizer conversion efficiency in percent ($\ge 15.0\%$).
    pub synthesizer_efficiency_pct: f64,
    /// Inter-miniband Landau-Zener tunneling leakage probability ($\le 0.05$).
    pub zener_leakage_prob: f64,
}

impl HighHarmonicBlochParams {
    /// Creates a new parameter set for high-harmonic acoustic Bloch oscillations.
    pub fn new(
        period_nm: f64,
        width_mev: f64,
        gap_mev: f64,
        force_mev_nm: f64,
        deph_ps: f64,
        drive_fac: f64,
    ) -> Self {
        Self {
            superlattice_period_nm: period_nm.clamp(5.0, 100.0),
            miniband_width_mev: width_mev.clamp(0.1, 20.0),
            miniband_gap_mev: gap_mev.clamp(1.0, 50.0),
            effective_force_field_mev_nm: force_mev_nm.clamp(0.01, 2.0),
            dephasing_time_ps: deph_ps.clamp(0.5, 100.0),
            non_linear_drive_factor: drive_fac.clamp(0.5, 5.0),
        }
    }
}
