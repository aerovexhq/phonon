#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological optomechanical polariton switchyards and multi-channel
//! routing networks.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// optomechanical polariton switchyards and multi-channel routing networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptomechanicalSwitchyardParams {
    /// Optomechanical coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub optomechanical_coupling_energy_mev: f64,
    /// Topological polariton energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_polariton_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Polariton routing speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub polariton_routing_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Optical control pump power in microwatts (clamp 0.5 to 30.0, default 5.8).
    pub optical_control_pump_power_uw: f64,
    /// Synthetic optomechanical cooperativity C in dimensionless units (clamp 0.1 to 5.0, default 1.6).
    pub synthetic_optomechanical_cooperativity_c: f64,
    /// Switchyard channel pitch in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub switchyard_channel_pitch_um: f64,
}

impl Default for OptomechanicalSwitchyardParams {
    fn default() -> Self {
        Self {
            optomechanical_coupling_energy_mev: 16.5,
            topological_polariton_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            polariton_routing_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            optical_control_pump_power_uw: 5.8,
            synthetic_optomechanical_cooperativity_c: 1.6,
            switchyard_channel_pitch_um: 4.8,
        }
    }
}

impl OptomechanicalSwitchyardParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        optomechanical_coupling_energy_mev: f64,
        topological_polariton_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        polariton_routing_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        optical_control_pump_power_uw: f64,
        synthetic_optomechanical_cooperativity_c: f64,
        switchyard_channel_pitch_um: f64,
    ) -> Self {
        Self {
            optomechanical_coupling_energy_mev: optomechanical_coupling_energy_mev.clamp(1.0, 35.0),
            topological_polariton_gap_mev: topological_polariton_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            polariton_routing_speed_m_per_s: polariton_routing_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            optical_control_pump_power_uw: optical_control_pump_power_uw.clamp(0.5, 30.0),
            synthetic_optomechanical_cooperativity_c: synthetic_optomechanical_cooperativity_c.clamp(0.1, 5.0),
            switchyard_channel_pitch_um: switchyard_channel_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// optomechanical polariton switchyards and multi-channel routing networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptomechanicalSwitchyardMetrics {
    /// Polariton routing fidelity across multi-channel switchyards (target >= 0.9980).
    pub routing_fidelity: f64,
    /// Polariton quantum state retention fraction (target >= 0.9970).
    pub polariton_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
