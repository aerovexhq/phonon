#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological Majorana-driven transmon hybrid interfaces and
//! cryogenic quantum bus transceivers.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// Majorana-driven transmon hybrid interfaces and cryogenic quantum bus transceivers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaTransmonParams {
    /// Hybrid coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub hybrid_coupling_energy_mev: f64,
    /// Topological hybrid gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_hybrid_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Quantum bus propagation speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub bus_propagation_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Transmon microwave drive power in microwatts (clamp 0.5 to 30.0, default 5.8).
    pub transmon_microwave_drive_power_uw: f64,
    /// Synthetic Josephson energy ratio Ej/Ec (clamp 0.1 to 5.0, default 1.6).
    pub synthetic_josephson_energy_ratio_ej_ec: f64,
    /// Quantum bus channel pitch in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub quantum_bus_pitch_um: f64,
}

impl Default for MajoranaTransmonParams {
    fn default() -> Self {
        Self {
            hybrid_coupling_energy_mev: 16.5,
            topological_hybrid_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            bus_propagation_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            transmon_microwave_drive_power_uw: 5.8,
            synthetic_josephson_energy_ratio_ej_ec: 1.6,
            quantum_bus_pitch_um: 4.8,
        }
    }
}

impl MajoranaTransmonParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        hybrid_coupling_energy_mev: f64,
        topological_hybrid_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        bus_propagation_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        transmon_microwave_drive_power_uw: f64,
        synthetic_josephson_energy_ratio_ej_ec: f64,
        quantum_bus_pitch_um: f64,
    ) -> Self {
        Self {
            hybrid_coupling_energy_mev: hybrid_coupling_energy_mev.clamp(1.0, 35.0),
            topological_hybrid_gap_mev: topological_hybrid_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            bus_propagation_speed_m_per_s: bus_propagation_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            transmon_microwave_drive_power_uw: transmon_microwave_drive_power_uw.clamp(0.5, 30.0),
            synthetic_josephson_energy_ratio_ej_ec: synthetic_josephson_energy_ratio_ej_ec
                .clamp(0.1, 5.0),
            quantum_bus_pitch_um: quantum_bus_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// Majorana-driven transmon hybrid interfaces and cryogenic quantum bus transceivers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaTransmonMetrics {
    /// Interface fidelity across Majorana-transmon hybrid interfaces (target >= 0.9980).
    pub interface_fidelity: f64,
    /// Hybrid quantum state retention fraction (target >= 0.9970).
    pub hybrid_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
