#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for non-Abelian
//! anyon braiding in chiral acoustic Chern metamaterials and fault-tolerant
//! phononic topological qubits.

/// Physical parameter configuration for chiral Chern acoustic anyon braiding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralChernAnyonBraidingParams {
    /// Acoustic center frequency in GHz (clamp 1.0 to 15.0, default 4.8).
    pub acoustic_center_freq_ghz: f64,
    /// Chern bulk acoustic topological bandgap in MHz (clamp 20.0 to 200.0, default 80.0).
    pub chern_bandgap_mhz: f64,
    /// Dynamic acoustic strain modulation amplitude in MHz (clamp 5.0 to 50.0, default 22.0).
    pub strain_modulation_amplitude_mhz: f64,
    /// Spatial braiding interferometer arm length in micrometers (clamp 10.0 to 150.0, default 50.0).
    pub braiding_arm_length_um: f64,
    /// Anyonic wavepacket steering propagation velocity in m/s (clamp 1000.0 to 6000.0, default 3400.0).
    pub anyon_wavepacket_speed_m_per_s: f64,
    /// Intrinsic acoustic loss rate in kHz (clamp 0.1 to 20.0, default 2.5).
    pub acoustic_loss_rate_khz: f64,
    /// Cryogenic operating bath temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Parafermion topological order Z_m (clamp 2 to 6, default 2 for Majorana zero modes).
    pub anyon_type_parafermion_order: usize,
}

impl Default for ChiralChernAnyonBraidingParams {
    fn default() -> Self {
        Self {
            acoustic_center_freq_ghz: 4.8,
            chern_bandgap_mhz: 80.0,
            strain_modulation_amplitude_mhz: 22.0,
            braiding_arm_length_um: 50.0,
            anyon_wavepacket_speed_m_per_s: 3400.0,
            acoustic_loss_rate_khz: 2.5,
            operating_temp_m_k: 15.0,
            anyon_type_parafermion_order: 2,
        }
    }
}

impl ChiralChernAnyonBraidingParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        acoustic_center_freq_ghz: f64,
        chern_bandgap_mhz: f64,
        strain_modulation_amplitude_mhz: f64,
        braiding_arm_length_um: f64,
        anyon_wavepacket_speed_m_per_s: f64,
        acoustic_loss_rate_khz: f64,
        operating_temp_m_k: f64,
        anyon_type_parafermion_order: usize,
    ) -> Self {
        Self {
            acoustic_center_freq_ghz: acoustic_center_freq_ghz.clamp(1.0, 15.0),
            chern_bandgap_mhz: chern_bandgap_mhz.clamp(20.0, 200.0),
            strain_modulation_amplitude_mhz: strain_modulation_amplitude_mhz.clamp(5.0, 50.0),
            braiding_arm_length_um: braiding_arm_length_um.clamp(10.0, 150.0),
            anyon_wavepacket_speed_m_per_s: anyon_wavepacket_speed_m_per_s.clamp(1000.0, 6000.0),
            acoustic_loss_rate_khz: acoustic_loss_rate_khz.clamp(0.1, 20.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            anyon_type_parafermion_order: anyon_type_parafermion_order.clamp(2, 6),
        }
    }
}

/// Multi-physics performance evaluation metrics for chiral Chern anyon braiding.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralChernAnyonBraidingMetrics {
    /// Non-Abelian braiding gate fidelity (target >= 0.9980).
    pub braiding_gate_fidelity: f64,
    /// Dynamic topological protection gap in MHz (target >= 18.0).
    pub topological_protection_gap_mhz: f64,
    /// Anyon collision interferometric visibility (target >= 0.950).
    pub anyon_collision_visibility: f64,
    /// Non-adiabatic Landau-Zener state leakage rate (target <= 1.0e-5).
    pub non_adiabatic_leakage_rate: f64,
    /// Topological qubit dephasing coherence lifetime in ms (target >= 12.0).
    pub topological_qubit_coherence_ms: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
