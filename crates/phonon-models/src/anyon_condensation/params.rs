#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological anyon condensation networks and higher-form gauge transceivers.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// anyon condensation networks and higher-form gauge transceivers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnyonCondensationParams {
    /// Gauge coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub gauge_coupling_energy_mev: f64,
    /// Topological condensation energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_condensation_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Condensation drift speed of anyonic boundary states in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub condensation_drift_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave probe power in micro-watts (clamp 0.5 to 30.0, default 5.8).
    pub microwave_probe_power_uw: f64,
    /// Higher-form gauge flux quantum in units of Phi_0 (clamp 0.1 to 5.0, default 1.6).
    pub higher_form_flux_quantum_phi0: f64,
    /// Transceiver channel spatial pitch in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub transceiver_channel_pitch_um: f64,
}

impl Default for AnyonCondensationParams {
    fn default() -> Self {
        Self {
            gauge_coupling_energy_mev: 16.5,
            topological_condensation_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            condensation_drift_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 5.8,
            higher_form_flux_quantum_phi0: 1.6,
            transceiver_channel_pitch_um: 4.8,
        }
    }
}

impl AnyonCondensationParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        gauge_coupling_energy_mev: f64,
        topological_condensation_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        condensation_drift_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        higher_form_flux_quantum_phi0: f64,
        transceiver_channel_pitch_um: f64,
    ) -> Self {
        Self {
            gauge_coupling_energy_mev: gauge_coupling_energy_mev.clamp(1.0, 35.0),
            topological_condensation_gap_mev: topological_condensation_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            condensation_drift_speed_m_per_s: condensation_drift_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            higher_form_flux_quantum_phi0: higher_form_flux_quantum_phi0.clamp(0.1, 5.0),
            transceiver_channel_pitch_um: transceiver_channel_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// anyon condensation networks and higher-form gauge transceivers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnyonCondensationMetrics {
    /// Quantum transceiver fidelity across anyon condensation channels (target >= 0.9980).
    pub transceiver_fidelity: f64,
    /// Condensate quantum state retention fraction (target >= 0.9970).
    pub condensate_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-network crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_network_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
