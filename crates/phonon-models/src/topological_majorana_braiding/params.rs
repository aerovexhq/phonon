//! Parameter models and metric structures for topological non-Abelian
//! Majorana braiding in phononic Josephson metamaterials.

/// Physical parameter configuration for Majorana braiding in phononic Josephson arrays.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaBraidingParams {
    /// Number of coupled topological Josephson junctions in the 2D network (default 6).
    pub junction_count: usize,
    /// Induced topological superconducting gap $\Delta_{\text{top}}$ in micro-electronvolts (default 250.0 ueV).
    pub topological_gap_uev: f64,
    /// Surface acoustic wave (SAW) piezoelectric peak strain amplitude $\epsilon_{\text{saw}}$ (default 4.5e-4).
    pub saw_piezo_strain: f64,
    /// Braiding cycle duration $\tau_{\text{braid}}$ in nanoseconds (default 25.0 ns).
    pub braiding_period_ns: f64,
    /// Background quasiparticle poisoning rate $\Gamma_{\text{qp}}$ in kHz (default 0.85 kHz).
    pub quasiparticle_poisoning_rate_khz: f64,
    /// Finite-size Majorana overlap coupling energy $\epsilon_{ij}$ in nano-electronvolts (default 12.0 neV).
    pub majorana_overlap_energy_nev: f64,
    /// Dispersive parity readout resonator quality factor $Q_{\text{res}}$ (default 3.5e4).
    pub readout_resonator_q: f64,
    /// Dilution refrigerator operating temperature in milli-Kelvin (default 20.0 mK).
    pub operating_temp_m_k: f64,
}

impl Default for MajoranaBraidingParams {
    fn default() -> Self {
        Self {
            junction_count: 6,
            topological_gap_uev: 250.0,
            saw_piezo_strain: 4.5e-4,
            braiding_period_ns: 25.0,
            quasiparticle_poisoning_rate_khz: 0.85,
            majorana_overlap_energy_nev: 12.0,
            readout_resonator_q: 3.5e4,
            operating_temp_m_k: 20.0,
        }
    }
}

impl MajoranaBraidingParams {
    /// Creates a new parameter configuration with bounds clamping.
    pub fn new(
        n_junc: usize,
        gap_uev: f64,
        strain: f64,
        tau_ns: f64,
        gamma_qp_khz: f64,
        overlap_nev: f64,
        q_res: f64,
        temp_mk: f64,
    ) -> Self {
        Self {
            junction_count: n_junc.clamp(3, 32),
            topological_gap_uev: gap_uev.clamp(5.0, 2500.0),
            saw_piezo_strain: strain.clamp(1.0e-6, 1.0e-2),
            braiding_period_ns: tau_ns.clamp(0.5, 200.0),
            quasiparticle_poisoning_rate_khz: gamma_qp_khz.clamp(0.001, 1000.0),
            majorana_overlap_energy_nev: overlap_nev.clamp(0.01, 1000.0),
            readout_resonator_q: q_res.clamp(50.0, 1.0e7),
            operating_temp_m_k: temp_mk.clamp(0.1, 1000.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological Majorana braiding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaBraidingMetrics {
    /// Unitary non-Abelian braiding gate fidelity (target >= 0.9990 or 99.90%).
    pub braiding_gate_fidelity: f64,
    /// Non-Abelian geometric phase error $|\delta\theta|$ in radians (target <= 1.0e-4 rad).
    pub non_abelian_phase_error_rad: f64,
    /// Dispersive fermion parity readout contrast (target >= 0.950).
    pub parity_readout_contrast: f64,
    /// Braiding cycle duration $\tau_{\text{braid}}$ in nanoseconds (target <= 50.0 ns).
    pub braiding_cycle_period_ns: f64,
    /// Topological gap protection ratio $\Delta_{\text{top}} / (k_B T)$ (target >= 20.0).
    pub topological_gap_protection_ratio: f64,
    /// Physical compliance verification flag.
    pub is_physically_compliant: bool,
}
