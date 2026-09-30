#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological anyon-condensed fractional Chern insulator simulators
//! and quantum heat engines.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// anyon-condensed fractional Chern insulator simulators and quantum heat engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChernHeatEngineParams {
    /// Chern coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub chern_coupling_energy_mev: f64,
    /// Topological Chern energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_chern_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Thermodynamic cycle speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub thermodynamic_cycle_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Thermodynamic work power output in microwatts (clamp 0.5 to 30.0, default 5.8).
    pub thermodynamic_work_power_uw: f64,
    /// Synthetic fractional Chern number C in dimensionless topological units (clamp 0.1 to 5.0, default 1.6).
    pub synthetic_fractional_chern_number_c: f64,
    /// Quantum heat engine cell pitch in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub heat_engine_cell_pitch_um: f64,
}

impl Default for ChernHeatEngineParams {
    fn default() -> Self {
        Self {
            chern_coupling_energy_mev: 16.5,
            topological_chern_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            thermodynamic_cycle_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            thermodynamic_work_power_uw: 5.8,
            synthetic_fractional_chern_number_c: 1.6,
            heat_engine_cell_pitch_um: 4.8,
        }
    }
}

impl ChernHeatEngineParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        chern_coupling_energy_mev: f64,
        topological_chern_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        thermodynamic_cycle_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        thermodynamic_work_power_uw: f64,
        synthetic_fractional_chern_number_c: f64,
        heat_engine_cell_pitch_um: f64,
    ) -> Self {
        Self {
            chern_coupling_energy_mev: chern_coupling_energy_mev.clamp(1.0, 35.0),
            topological_chern_gap_mev: topological_chern_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            thermodynamic_cycle_speed_m_per_s: thermodynamic_cycle_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            thermodynamic_work_power_uw: thermodynamic_work_power_uw.clamp(0.5, 30.0),
            synthetic_fractional_chern_number_c: synthetic_fractional_chern_number_c.clamp(0.1, 5.0),
            heat_engine_cell_pitch_um: heat_engine_cell_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// anyon-condensed fractional Chern insulator simulators and quantum heat engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChernHeatEngineMetrics {
    /// Thermodynamic cycle fidelity across anyon-condensed heat engines (target >= 0.9980).
    pub cycle_fidelity: f64,
    /// Condensed anyonic quantum state retention fraction (target >= 0.9970).
    pub condensed_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
