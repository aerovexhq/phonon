#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for quantum acoustic
//! topological time crystals and Floquet-symmetry-enriched phononic memories.

/// Physical parameter configuration for quantum acoustic topological time crystals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalTimeCrystalParams {
    /// Floquet drive period in microseconds (clamp 0.1 to 10.0, default 1.5).
    pub floquet_drive_period_us: f64,
    /// Imperfect pulse rotation error epsilon_rot (clamp 0.001 to 0.10, default 0.02).
    pub imperfect_pulse_rotation_error: f64,
    /// Inter-resonator acoustic coupling interaction in MHz (clamp 5.0 to 80.0, default 35.0).
    pub inter_resonator_interaction_mhz: f64,
    /// Quasi-periodic disorder potential strength in MHz (clamp 10.0 to 150.0, default 65.0).
    pub disorder_potential_strength_mhz: f64,
    /// Acoustic dissipation loss rate in Hz (clamp 1.0 to 50.0, default 8.0).
    pub acoustic_loss_rate_hz: f64,
    /// Operating cryogenic temperature in millikelvin (clamp 1.0 to 50.0, default 10.0).
    pub operating_temp_m_k: f64,
    /// Phononic metamaterial chain length in resonators (clamp 8 to 64, default 24).
    pub phononic_chain_length: usize,
    /// Subharmonic discrete time translation symmetry period multiplier (clamp 2 to 4, default 2).
    pub subharmonic_period_multiplier: usize,
}

impl Default for TopologicalTimeCrystalParams {
    fn default() -> Self {
        Self {
            floquet_drive_period_us: 1.5,
            imperfect_pulse_rotation_error: 0.02,
            inter_resonator_interaction_mhz: 35.0,
            disorder_potential_strength_mhz: 65.0,
            acoustic_loss_rate_hz: 8.0,
            operating_temp_m_k: 10.0,
            phononic_chain_length: 24,
            subharmonic_period_multiplier: 2,
        }
    }
}

impl TopologicalTimeCrystalParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        floquet_drive_period_us: f64,
        imperfect_pulse_rotation_error: f64,
        inter_resonator_interaction_mhz: f64,
        disorder_potential_strength_mhz: f64,
        acoustic_loss_rate_hz: f64,
        operating_temp_m_k: f64,
        phononic_chain_length: usize,
        subharmonic_period_multiplier: usize,
    ) -> Self {
        Self {
            floquet_drive_period_us: floquet_drive_period_us.clamp(0.1, 10.0),
            imperfect_pulse_rotation_error: imperfect_pulse_rotation_error.clamp(0.001, 0.10),
            inter_resonator_interaction_mhz: inter_resonator_interaction_mhz.clamp(5.0, 80.0),
            disorder_potential_strength_mhz: disorder_potential_strength_mhz.clamp(10.0, 150.0),
            acoustic_loss_rate_hz: acoustic_loss_rate_hz.clamp(1.0, 50.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            phononic_chain_length: phononic_chain_length.clamp(8, 64),
            subharmonic_period_multiplier: subharmonic_period_multiplier.clamp(2, 4),
        }
    }
}

/// Multi-physics performance evaluation metrics for quantum acoustic
/// topological time crystals and Floquet-symmetry-enriched phononic memories.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalTimeCrystalMetrics {
    /// Time-crystalline order fidelity (target >= 0.9960).
    pub time_crystalline_order_fidelity: f64,
    /// Subharmonic frequency locking error delta omega_2T (target <= 0.0020).
    pub subharmonic_locking_error: f64,
    /// Temporal crystalline coherence lifetime in milliseconds (target >= 100.0).
    pub temporal_crystalline_lifetime_ms: f64,
    /// Non-volatile memory retention isolation in dB (target >= 45.0).
    pub memory_retention_isolation_db: f64,
    /// Many-body localization level statistics ratio (target >= 0.920).
    pub many_body_localization_ratio: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
