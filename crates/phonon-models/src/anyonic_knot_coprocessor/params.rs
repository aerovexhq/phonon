#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological anyonic knot invariant quantum co-processors and
//! Chern-Simons calculators.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// anyonic knot invariant quantum co-processors and Chern-Simons calculators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnyonicKnotCoprocessorParams {
    /// Braid crossing coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub braid_crossing_coupling_mev: f64,
    /// Chern-Simons topological level k (clamp 1.0 to 12.0, default 4.0).
    pub chern_simons_level_k: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Knot braiding speed of chiral anyon modes in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub knot_braiding_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave interferometer readout power in micro-watts (clamp 0.5 to 30.0, default 5.8).
    pub microwave_interferometer_power_uw: f64,
    /// Anyon link closure radius in nanometers (clamp 20.0 to 200.0, default 75.0).
    pub anyon_link_closure_radius_nm: f64,
    /// Knot complexity crossings number (clamp 3.0 to 24.0, default 8.0).
    pub knot_complexity_crossings: f64,
}

impl Default for AnyonicKnotCoprocessorParams {
    fn default() -> Self {
        Self {
            braid_crossing_coupling_mev: 16.5,
            chern_simons_level_k: 4.0,
            acoustic_drive_frequency_ghz: 5.8,
            knot_braiding_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_interferometer_power_uw: 5.8,
            anyon_link_closure_radius_nm: 75.0,
            knot_complexity_crossings: 8.0,
        }
    }
}

impl AnyonicKnotCoprocessorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        braid_crossing_coupling_mev: f64,
        chern_simons_level_k: f64,
        acoustic_drive_frequency_ghz: f64,
        knot_braiding_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_interferometer_power_uw: f64,
        anyon_link_closure_radius_nm: f64,
        knot_complexity_crossings: f64,
    ) -> Self {
        Self {
            braid_crossing_coupling_mev: braid_crossing_coupling_mev.clamp(1.0, 35.0),
            chern_simons_level_k: chern_simons_level_k.clamp(1.0, 12.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            knot_braiding_speed_m_per_s: knot_braiding_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_interferometer_power_uw: microwave_interferometer_power_uw.clamp(0.5, 30.0),
            anyon_link_closure_radius_nm: anyon_link_closure_radius_nm.clamp(20.0, 200.0),
            knot_complexity_crossings: knot_complexity_crossings.clamp(3.0, 24.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// anyonic knot invariant quantum co-processors and Chern-Simons calculators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AnyonicKnotCoprocessorMetrics {
    /// Knot calculation fidelity (target >= 0.9980).
    pub knot_calculation_fidelity: f64,
    /// Anyon state retention fraction (target >= 0.9970).
    pub anyon_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-knot crosstalk acoustic isolation in decibels (target >= 54.0 dB).
    pub inter_knot_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
