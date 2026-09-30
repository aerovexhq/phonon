#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Photonic-Phononic Quantum Transceiver &
//! Terahertz Frequency Comb Metrology Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous photonic-phononic quantum transceiver and terahertz frequency comb metrology engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumTransceiverParams {
    /// Transceiver electro-optic/optomechanical coupling energy in meV (clamp 1.0 to 35.0, default 21.5).
    pub transceiver_coupling_mev: f64,
    /// Topological transceiver bandgap energy in meV (clamp 2.0 to 45.0, default 27.5).
    pub topological_transceiver_gap_mev: f64,
    /// Acoustic drive carrier frequency in GHz (clamp 1.0 to 12.0, default 8.2).
    pub acoustic_drive_frequency_ghz: f64,
    /// Optical carrier telemetry dispatch speed in m/s (clamp 200.0 to 3000.0, default 1850.0).
    pub optical_carrier_dispatch_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Dispersive microwave readout probe power in microwatts (clamp 0.5 to 30.0, default 8.2).
    pub microwave_probe_power_uw: f64,
    /// Synthetic soliton comb lines scaling factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_comb_lines_factor: f64,
    /// Quantum transceiver routing pitch in micrometers (clamp 0.5 to 20.0, default 7.2).
    pub transceiver_pitch_um: f64,
}

impl Default for QuantumTransceiverParams {
    fn default() -> Self {
        Self {
            transceiver_coupling_mev: 21.5,
            topological_transceiver_gap_mev: 27.5,
            acoustic_drive_frequency_ghz: 8.2,
            optical_carrier_dispatch_speed_m_per_s: 1850.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 8.2,
            synthetic_comb_lines_factor: 4.0,
            transceiver_pitch_um: 7.2,
        }
    }
}

impl QuantumTransceiverParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        transceiver_coupling_mev: f64,
        topological_transceiver_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        optical_carrier_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_comb_lines_factor: f64,
        transceiver_pitch_um: f64,
    ) -> Self {
        Self {
            transceiver_coupling_mev: transceiver_coupling_mev.clamp(1.0, 35.0),
            topological_transceiver_gap_mev: topological_transceiver_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            optical_carrier_dispatch_speed_m_per_s: optical_carrier_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_comb_lines_factor: synthetic_comb_lines_factor.clamp(1.0, 8.0),
            transceiver_pitch_um: transceiver_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Photonic-Phononic Quantum Transceiver & Terahertz Frequency Comb Metrology Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumTransceiverMetrics {
    /// Transceiver fidelity across photonic-phononic quantum interfaces (target >= 0.9980).
    pub transceiver_fidelity: f64,
    /// Quantum state transfer retention fraction across optical-acoustic conversions (target >= 0.9970).
    pub state_transfer_retention_fraction: f64,
    /// Multi-physics topological protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-comb crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_comb_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
