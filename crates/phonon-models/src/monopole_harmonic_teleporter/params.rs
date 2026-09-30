#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological monopole-harmonic entanglement teleporters and
//! compactified quantum transceivers.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// monopole-harmonic entanglement teleporters and compactified quantum transceivers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonopoleHarmonicTeleporterParams {
    /// Monopole Berry curvature gauge strength in meV (clamp 1.0 to 35.0, default 16.5).
    pub monopole_berry_curvature_strength: f64,
    /// Topological superconducting pairing energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_superconducting_gap_mev: f64,
    /// Acoustic harmonic modulation frequency in GHz (clamp 1.0 to 12.0, default 5.7).
    pub acoustic_harmonic_frequency_ghz: f64,
    /// Teleportation drift velocity of chiral state modes in m/s (clamp 200.0 to 3000.0, default 1420.0).
    pub teleportation_drift_velocity_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave transceiver coherent pumping power in micro-watts (clamp 0.5 to 30.0, default 5.7).
    pub microwave_transceiver_power_uw: f64,
    /// Topological compactification radius in nanometers (clamp 10.0 to 200.0, default 65.0).
    pub compactification_radius_nm: f64,
    /// Inter-channel separation distance in micrometers (clamp 0.5 to 20.0, default 5.2).
    pub channel_separation_distance_um: f64,
}

impl Default for MonopoleHarmonicTeleporterParams {
    fn default() -> Self {
        Self {
            monopole_berry_curvature_strength: 16.5,
            topological_superconducting_gap_mev: 22.0,
            acoustic_harmonic_frequency_ghz: 5.7,
            teleportation_drift_velocity_m_per_s: 1420.0,
            cryogenic_temperature_mk: 10.0,
            microwave_transceiver_power_uw: 5.7,
            compactification_radius_nm: 65.0,
            channel_separation_distance_um: 5.2,
        }
    }
}

impl MonopoleHarmonicTeleporterParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        monopole_berry_curvature_strength: f64,
        topological_superconducting_gap_mev: f64,
        acoustic_harmonic_frequency_ghz: f64,
        teleportation_drift_velocity_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_transceiver_power_uw: f64,
        compactification_radius_nm: f64,
        channel_separation_distance_um: f64,
    ) -> Self {
        Self {
            monopole_berry_curvature_strength: monopole_berry_curvature_strength.clamp(1.0, 35.0),
            topological_superconducting_gap_mev: topological_superconducting_gap_mev
                .clamp(2.0, 45.0),
            acoustic_harmonic_frequency_ghz: acoustic_harmonic_frequency_ghz.clamp(1.0, 12.0),
            teleportation_drift_velocity_m_per_s: teleportation_drift_velocity_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_transceiver_power_uw: microwave_transceiver_power_uw.clamp(0.5, 30.0),
            compactification_radius_nm: compactification_radius_nm.clamp(10.0, 200.0),
            channel_separation_distance_um: channel_separation_distance_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// monopole-harmonic entanglement teleporters and compactified quantum transceivers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MonopoleHarmonicTeleporterMetrics {
    /// Quantum acoustic teleportation fidelity (target >= 0.9980).
    pub teleportation_fidelity: f64,
    /// Anyon non-Abelian quantum state retention fraction (target >= 0.9970).
    pub anyon_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
