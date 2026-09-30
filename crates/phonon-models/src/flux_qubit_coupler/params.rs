#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for the Phonon
//! Universal Multi-Scale Visual Studio Autonomous Acoustically Driven Superconducting
//! Flux Qubit Coupler & Ultra-Low Jitter Clock Engine.

/// Physical parameter configuration for the universal multi-scale visual studio
/// autonomous acoustically driven superconducting flux qubit coupler and ultra-low jitter clock engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluxQubitCouplerParams {
    /// Acoustic-piezoelectric flux qubit coupling energy in meV (clamp 1.0 to 35.0, default 32.0).
    pub flux_coupling_mev: f64,
    /// Topological clock protection gap energy in meV (clamp 2.0 to 45.0, default 38.0).
    pub topological_clock_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 12.0).
    pub acoustic_drive_frequency_ghz: f64,
    /// Clock dispatch speed in m/s (clamp 200.0 to 3000.0, default 2900.0).
    pub clock_dispatch_speed_m_per_s: f64,
    /// Cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave control probe power in microwatts (clamp 0.5 to 30.0, default 13.5).
    pub microwave_probe_power_uw: f64,
    /// Synthetic multi-rail clock distribution nodes factor (clamp 1.0 to 8.0, default 4.0).
    pub synthetic_clock_nodes_factor: f64,
    /// Inter-node flux coupler pitch in micrometers (clamp 0.5 to 20.0, default 12.5).
    pub flux_coupler_pitch_um: f64,
}

impl Default for FluxQubitCouplerParams {
    fn default() -> Self {
        Self {
            flux_coupling_mev: 32.0,
            topological_clock_gap_mev: 38.0,
            acoustic_drive_frequency_ghz: 12.0,
            clock_dispatch_speed_m_per_s: 2900.0,
            cryogenic_temperature_mk: 10.0,
            microwave_probe_power_uw: 13.5,
            synthetic_clock_nodes_factor: 4.0,
            flux_coupler_pitch_um: 12.5,
        }
    }
}

impl FluxQubitCouplerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        flux_coupling_mev: f64,
        topological_clock_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        clock_dispatch_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_probe_power_uw: f64,
        synthetic_clock_nodes_factor: f64,
        flux_coupler_pitch_um: f64,
    ) -> Self {
        Self {
            flux_coupling_mev: flux_coupling_mev.clamp(1.0, 35.0),
            topological_clock_gap_mev: topological_clock_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            clock_dispatch_speed_m_per_s: clock_dispatch_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_probe_power_uw: microwave_probe_power_uw.clamp(0.5, 30.0),
            synthetic_clock_nodes_factor: synthetic_clock_nodes_factor.clamp(1.0, 8.0),
            flux_coupler_pitch_um: flux_coupler_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for the Phonon Universal Multi-Scale Visual Studio
/// Autonomous Acoustically Driven Superconducting Flux Qubit Coupler & Ultra-Low Jitter Clock Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FluxQubitCouplerMetrics {
    /// Coherent acoustic-mediated flux qubit coupling fidelity (target >= 0.9980).
    pub flux_coupling_fidelity: f64,
    /// Clock state retention fraction across synthetic distribution nodes (target >= 0.9970).
    pub clock_state_retention_fraction: f64,
    /// Topological clock protection gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-node crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_node_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
