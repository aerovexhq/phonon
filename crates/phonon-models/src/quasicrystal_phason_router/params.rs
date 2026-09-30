#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological quasicrystal phason-defect routers and higher-dimensional
//! state concentrators.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// quasicrystal phason-defect routers and higher-dimensional state concentrators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuasicrystalPhasonRouterParams {
    /// Phason strain coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub phason_strain_coupling_mev: f64,
    /// Quasicrystal topological bulk energy gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub quasicrystal_topological_gap_mev: f64,
    /// Acoustic phason drive frequency in GHz (clamp 1.0 to 12.0, default 5.7).
    pub acoustic_phason_frequency_ghz: f64,
    /// Phason flip propagation speed of localized defect modes in m/s (clamp 200.0 to 3000.0, default 1420.0).
    pub phason_flip_propagation_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave pump drive power in micro-watts (clamp 0.5 to 30.0, default 5.7).
    pub microwave_pump_power_uw: f64,
    /// Quasicrystal inflation ratio (e.g. golden ratio tau for Penrose or silver ratio for Ammann-Beenker, clamp 1.2 to 3.5, default 1.618).
    pub quasicrystal_inflation_ratio: f64,
    /// Router acoustic channel separation distance in micrometers (clamp 0.5 to 20.0, default 5.0).
    pub router_channel_separation_um: f64,
}

impl Default for QuasicrystalPhasonRouterParams {
    fn default() -> Self {
        Self {
            phason_strain_coupling_mev: 16.5,
            quasicrystal_topological_gap_mev: 22.0,
            acoustic_phason_frequency_ghz: 5.7,
            phason_flip_propagation_speed_m_per_s: 1420.0,
            cryogenic_temperature_mk: 10.0,
            microwave_pump_power_uw: 5.7,
            quasicrystal_inflation_ratio: 1.618,
            router_channel_separation_um: 5.0,
        }
    }
}

impl QuasicrystalPhasonRouterParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        phason_strain_coupling_mev: f64,
        quasicrystal_topological_gap_mev: f64,
        acoustic_phason_frequency_ghz: f64,
        phason_flip_propagation_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_pump_power_uw: f64,
        quasicrystal_inflation_ratio: f64,
        router_channel_separation_um: f64,
    ) -> Self {
        Self {
            phason_strain_coupling_mev: phason_strain_coupling_mev.clamp(1.0, 35.0),
            quasicrystal_topological_gap_mev: quasicrystal_topological_gap_mev.clamp(2.0, 45.0),
            acoustic_phason_frequency_ghz: acoustic_phason_frequency_ghz.clamp(1.0, 12.0),
            phason_flip_propagation_speed_m_per_s: phason_flip_propagation_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_pump_power_uw: microwave_pump_power_uw.clamp(0.5, 30.0),
            quasicrystal_inflation_ratio: quasicrystal_inflation_ratio.clamp(1.2, 3.5),
            router_channel_separation_um: router_channel_separation_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// quasicrystal phason-defect routers and higher-dimensional state concentrators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuasicrystalPhasonRouterMetrics {
    /// Routing fidelity of chiral topological phason defect states (target >= 0.9980).
    pub routing_fidelity: f64,
    /// Phason quantum state retention fraction (target >= 0.9970).
    pub phason_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
