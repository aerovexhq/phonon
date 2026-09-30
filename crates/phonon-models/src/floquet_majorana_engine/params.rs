#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological Floquet-Majorana engines and non-equilibrium
//! time-translational simulators.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// Floquet-Majorana engines and non-equilibrium time-translational simulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetMajoranaEngineParams {
    /// Floquet periodic drive amplitude in meV (clamp 1.0 to 35.0, default 16.5).
    pub floquet_drive_amplitude_mev: f64,
    /// Topological quasiparticle pairing energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_quasiparticle_gap_mev: f64,
    /// Floquet acoustic modulation frequency in GHz (clamp 1.0 to 12.0, default 5.6).
    pub floquet_modulation_frequency_ghz: f64,
    /// Stroboscopic shuttling speed of Majorana edge modes in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub stroboscopic_shuttling_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave coherent pumping power in micro-watts (clamp 0.5 to 30.0, default 5.8).
    pub microwave_pumping_power_uw: f64,
    /// Floquet stroboscopic drive period in nanoseconds (clamp 0.1 to 10.0, default 2.5).
    pub floquet_drive_period_ns: f64,
    /// Majorana nanowire length in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub majorana_wire_length_um: f64,
}

impl Default for FloquetMajoranaEngineParams {
    fn default() -> Self {
        Self {
            floquet_drive_amplitude_mev: 16.5,
            topological_quasiparticle_gap_mev: 22.0,
            floquet_modulation_frequency_ghz: 5.6,
            stroboscopic_shuttling_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_pumping_power_uw: 5.8,
            floquet_drive_period_ns: 2.5,
            majorana_wire_length_um: 4.8,
        }
    }
}

impl FloquetMajoranaEngineParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        floquet_drive_amplitude_mev: f64,
        topological_quasiparticle_gap_mev: f64,
        floquet_modulation_frequency_ghz: f64,
        stroboscopic_shuttling_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_pumping_power_uw: f64,
        floquet_drive_period_ns: f64,
        majorana_wire_length_um: f64,
    ) -> Self {
        Self {
            floquet_drive_amplitude_mev: floquet_drive_amplitude_mev.clamp(1.0, 35.0),
            topological_quasiparticle_gap_mev: topological_quasiparticle_gap_mev.clamp(2.0, 45.0),
            floquet_modulation_frequency_ghz: floquet_modulation_frequency_ghz.clamp(1.0, 12.0),
            stroboscopic_shuttling_speed_m_per_s: stroboscopic_shuttling_speed_m_per_s
                .clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_pumping_power_uw: microwave_pumping_power_uw.clamp(0.5, 30.0),
            floquet_drive_period_ns: floquet_drive_period_ns.clamp(0.1, 10.0),
            majorana_wire_length_um: majorana_wire_length_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// Floquet-Majorana engines and non-equilibrium time-translational simulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetMajoranaEngineMetrics {
    /// Quantum acoustic Floquet engine fidelity (target >= 0.9980).
    pub floquet_engine_fidelity: f64,
    /// Floquet-Majorana state retention fraction under stroboscopic drive cycles (target >= 0.9970).
    pub floquet_majorana_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-mode crosstalk acoustic isolation in decibels (target >= 54.0 dB).
    pub inter_mode_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
