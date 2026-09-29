//! Parameters and metrics for topological acoustic axion polaritons,
//! synthetic chiral domain walls, and quantum dark matter resonant transducers.

/// Parameters for acoustic axion polariton hybridizations and synthetic gauge fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticAxionParams {
    /// Central acoustic cavity resonance frequency in GHz (nominal $1.0 - 10.0\text{ GHz}$).
    pub acoustic_frequency_ghz: f64,
    /// Applied static external magnetic bias field $B_0$ in Tesla (nominal $2.0 - 14.0\text{ T}$).
    pub static_magnetic_bias_t: f64,
    /// Dimensionless strain-axion dynamic coupling coefficient $\\xi_{\\mathrm{ax}}$ (nominal $1.0\\times 10^{-3} - 1.0\\times 10^{-2}$).
    pub axion_strain_coupling: f64,
    /// Loaded acoustic mechanical quality factor $Q_m$ (nominal $2.0\\times 10^4 - 5.0\\times 10^5$).
    pub cavity_acoustic_q: f64,
    /// Loaded electromagnetic microwave cavity quality factor $Q_e$ (nominal $1.0\\times 10^4 - 2.0\\times 10^5$).
    pub cavity_em_q: f64,
    /// Spatial width of the synthetic axion domain wall in nanometers (nominal $10.0 - 80.0\text{ nm}$).
    pub domain_wall_width_nm: f64,
}

impl Default for AcousticAxionParams {
    fn default() -> Self {
        Self {
            acoustic_frequency_ghz: 3.6,
            static_magnetic_bias_t: 8.0,
            axion_strain_coupling: 4.5e-3,
            cavity_acoustic_q: 120_000.0,
            cavity_em_q: 50_000.0,
            domain_wall_width_nm: 24.0,
        }
    }
}

/// Evaluated metrics for topological acoustic axion polaritons and dark matter haloscopes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticAxionMetrics {
    /// Non-reciprocal magnetoelectric acoustic isolation in dB ($\ge 30.0\text{ dB}$).
    pub magnetoelectric_isolation_db: f64,
    /// Dimensionless axion-phonon coupling cooperativity $\\mathcal{C}_{\\mathrm{ax}}$ ($\ge 50.0$).
    pub axion_cooperativity: f64,
    /// Quantum dark matter haloscopic readout signal-to-noise ratio in dB ($\ge 25.0\text{ dB}$).
    pub dark_matter_snr_db: f64,
    /// Chiral anomaly spectral asymmetry mode purity in percent ($\ge 95.0\%$).
    pub chiral_anomaly_purity_pct: f64,
    /// Forward acoustic transmission insertion loss in dB ($\le 1.0\text{ dB}$).
    pub insertion_loss_db: f64,
    /// Axion-polariton vacuum Rabi anti-crossing gap in GHz ($\ge 1.5\text{ GHz}$).
    pub anticrossing_gap_ghz: f64,
}

impl AcousticAxionParams {
    /// Creates a new parameter set for topological acoustic axion polaritons.
    pub fn new(
        freq_ghz: f64,
        bias_t: f64,
        coupling: f64,
        q_m: f64,
        q_e: f64,
        width_nm: f64,
    ) -> Self {
        Self {
            acoustic_frequency_ghz: freq_ghz.clamp(0.5, 30.0),
            static_magnetic_bias_t: bias_t.clamp(0.5, 30.0),
            axion_strain_coupling: coupling.clamp(1e-4, 0.1),
            cavity_acoustic_q: q_m.clamp(1000.0, 1e7),
            cavity_em_q: q_e.clamp(1000.0, 1e7),
            domain_wall_width_nm: width_nm.clamp(2.0, 500.0),
        }
    }
}
