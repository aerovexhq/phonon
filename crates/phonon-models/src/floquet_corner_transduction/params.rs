//! Physical parameter configuration and evaluation metrics for Floquet second-order
//! topological phononic corner states and quantum transduction.

/// Parameter configuration for Floquet second-order topological phononic corner states
/// and bidirectional microwave-to-optical quantum transduction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerTransductionParams {
    /// 2D phononic crystal unit cell lattice constant $a$ in micrometers (default 1.8 um).
    pub lattice_constant_um: f64,
    /// Acoustic breathing corner mode resonance frequency in GHz (default 4.2 GHz).
    pub acoustic_resonance_ghz: f64,
    /// Inter-cell to intra-cell hopping amplitude ratio $t_{\text{inter}} / t_{\text{intra}}$ (default 2.4).
    pub inter_intra_hopping_ratio: f64,
    /// Piezoelectric electro-mechanical cooperativity $C_{em}$ (default 28.0).
    pub piezoelectric_cooperativity: f64,
    /// Optomechanical radiation pressure cooperativity $C_{om}$ (default 26.0).
    pub optomechanical_cooperativity: f64,
    /// Intrinsic acoustic quality factor $Q_m$ of the corner mode (default 2.8e5).
    pub acoustic_quality_factor: f64,
    /// Optical nanocavity decay rate $\kappa_o / (2\pi)$ in MHz (default 45.0 MHz).
    pub optical_cavity_decay_rate_mhz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (default 18.0 mK).
    pub operating_temp_m_k: f64,
}

impl Default for CornerTransductionParams {
    fn default() -> Self {
        Self {
            lattice_constant_um: 1.8,
            acoustic_resonance_ghz: 4.2,
            inter_intra_hopping_ratio: 2.4,
            piezoelectric_cooperativity: 28.0,
            optomechanical_cooperativity: 26.0,
            acoustic_quality_factor: 2.8e5,
            optical_cavity_decay_rate_mhz: 45.0,
            operating_temp_m_k: 18.0,
        }
    }
}

impl CornerTransductionParams {
    /// Creates a new configuration with rigorous physical bounds clamping.
    pub fn new(
        lattice_constant_um: f64,
        acoustic_resonance_ghz: f64,
        inter_intra_hopping_ratio: f64,
        piezoelectric_cooperativity: f64,
        optomechanical_cooperativity: f64,
        acoustic_quality_factor: f64,
        optical_cavity_decay_rate_mhz: f64,
        operating_temp_m_k: f64,
    ) -> Self {
        Self {
            lattice_constant_um: lattice_constant_um.clamp(0.5, 10.0),
            acoustic_resonance_ghz: acoustic_resonance_ghz.clamp(0.5, 20.0),
            inter_intra_hopping_ratio: inter_intra_hopping_ratio.clamp(1.05, 10.0),
            piezoelectric_cooperativity: piezoelectric_cooperativity.clamp(1.0, 200.0),
            optomechanical_cooperativity: optomechanical_cooperativity.clamp(1.0, 200.0),
            acoustic_quality_factor: acoustic_quality_factor.clamp(1.0e4, 1.0e7),
            optical_cavity_decay_rate_mhz: optical_cavity_decay_rate_mhz.clamp(1.0, 500.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 500.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological corner states and quantum transduction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CornerTransductionMetrics {
    /// Corner mode wavefunction localization purity within corner unit cell domain (target >= 0.960).
    pub corner_mode_localization_purity: f64,
    /// Bidirectional microwave-to-optical power transduction efficiency (target >= 0.450).
    pub bidirectional_transduction_efficiency: f64,
    /// Added quantum noise photons referred to the input (target <= 0.20).
    pub added_noise_photons: f64,
    /// Acoustic quality factor of the corner mode (target >= 1.5e5).
    pub corner_acoustic_quality_factor: f64,
    /// Quantized bulk quadrupole topological invariant $q_{xy}$ (target quantized 0.500).
    pub quadrupole_topological_invariant: f64,
    /// Strict physical compliance verification flag across all design criteria.
    pub is_physically_compliant: bool,
}
