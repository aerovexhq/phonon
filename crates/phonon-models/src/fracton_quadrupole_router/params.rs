#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! topological non-Abelian fracton gauge-matter ensembles and chiral quadrupole entanglement routers.

/// Physical parameter configuration for quantum acoustic topological non-Abelian fracton
/// gauge-matter ensembles and chiral quadrupole entanglement routers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractonQuadrupoleRouterParams {
    /// Higher-rank symmetric tensor gauge coupling energy in meV (clamp 2.0 to 45.0, default 22.0).
    pub higher_rank_tensor_coupling_mev: f64,
    /// Chiral acoustic quadrupole polarization intensity (clamp 0.10 to 0.95, default 0.65).
    pub quadrupole_polarization_intensity: f64,
    /// Fundamental acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Sub-dimensional quasiparticle mobility fraction (clamp 0.05 to 0.85, default 0.35).
    pub sub_dimensional_mobility_fraction: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave entanglement routing drive power in micro-watts (clamp 0.5 to 30.0, default 5.0).
    pub microwave_routing_power_uw: f64,
    /// Spatial separation distance between quadrupole routers in micrometers (clamp 0.5 to 15.0, default 3.8).
    pub router_separation_um: f64,
    /// Metamaterial substrate acoustic shear modulus in GPa (clamp 10.0 to 120.0, default 45.0).
    pub acoustic_shear_modulus_gpa: f64,
}

impl Default for FractonQuadrupoleRouterParams {
    fn default() -> Self {
        Self {
            higher_rank_tensor_coupling_mev: 22.0,
            quadrupole_polarization_intensity: 0.65,
            acoustic_drive_frequency_ghz: 5.8,
            sub_dimensional_mobility_fraction: 0.35,
            cryogenic_temperature_mk: 10.0,
            microwave_routing_power_uw: 5.0,
            router_separation_um: 3.8,
            acoustic_shear_modulus_gpa: 45.0,
        }
    }
}

impl FractonQuadrupoleRouterParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        higher_rank_tensor_coupling_mev: f64,
        quadrupole_polarization_intensity: f64,
        acoustic_drive_frequency_ghz: f64,
        sub_dimensional_mobility_fraction: f64,
        cryogenic_temperature_mk: f64,
        microwave_routing_power_uw: f64,
        router_separation_um: f64,
        acoustic_shear_modulus_gpa: f64,
    ) -> Self {
        Self {
            higher_rank_tensor_coupling_mev: higher_rank_tensor_coupling_mev.clamp(2.0, 45.0),
            quadrupole_polarization_intensity: quadrupole_polarization_intensity.clamp(0.10, 0.95),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            sub_dimensional_mobility_fraction: sub_dimensional_mobility_fraction.clamp(0.05, 0.85),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_routing_power_uw: microwave_routing_power_uw.clamp(0.5, 30.0),
            router_separation_um: router_separation_um.clamp(0.5, 15.0),
            acoustic_shear_modulus_gpa: acoustic_shear_modulus_gpa.clamp(10.0, 120.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic topological non-Abelian fracton
/// gauge-matter ensembles and chiral quadrupole entanglement routers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractonQuadrupoleRouterMetrics {
    /// Quantum acoustic entanglement routing fidelity (target >= 0.9980).
    pub routing_fidelity: f64,
    /// Immobile fracton bound-state quantum retention fraction (target >= 0.9970).
    pub fracton_state_retention_fraction: f64,
    /// Higher-rank topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-router acoustic and multipole crosstalk isolation in decibels (target >= 55.0 dB).
    pub inter_router_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
