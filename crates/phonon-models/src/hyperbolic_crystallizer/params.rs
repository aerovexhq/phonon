#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian chiral topological hyperbolic lattice anyon crystallizers and
//! fractal boundary engines.

/// Physical parameter configuration for quantum acoustic non-Abelian chiral topological
/// hyperbolic lattice anyon crystallizers and fractal boundary engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HyperbolicCrystallizerParams {
    /// Hyperbolic coupling energy in meV (clamp 1.0 to 35.0, default 16.5).
    pub hyperbolic_coupling_energy_mev: f64,
    /// Topological fractal gap in meV (clamp 2.0 to 45.0, default 22.0).
    pub topological_fractal_gap_mev: f64,
    /// Acoustic drive frequency in GHz (clamp 1.0 to 12.0, default 5.8).
    pub acoustic_drive_frequency_ghz: f64,
    /// Crystallization drift speed in m/s (clamp 200.0 to 3000.0, default 1400.0).
    pub crystallization_drift_speed_m_per_s: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Microwave pinning power in microwatts (clamp 0.5 to 30.0, default 5.8).
    pub microwave_pinning_power_uw: f64,
    /// Synthetic curvature radius in micrometers (clamp 0.1 to 5.0, default 1.6).
    pub synthetic_curvature_radius_um: f64,
    /// Hyperbolic tessellation pitch in micrometers (clamp 0.5 to 20.0, default 4.8).
    pub hyperbolic_tessellation_pitch_um: f64,
}

impl Default for HyperbolicCrystallizerParams {
    fn default() -> Self {
        Self {
            hyperbolic_coupling_energy_mev: 16.5,
            topological_fractal_gap_mev: 22.0,
            acoustic_drive_frequency_ghz: 5.8,
            crystallization_drift_speed_m_per_s: 1400.0,
            cryogenic_temperature_mk: 10.0,
            microwave_pinning_power_uw: 5.8,
            synthetic_curvature_radius_um: 1.6,
            hyperbolic_tessellation_pitch_um: 4.8,
        }
    }
}

impl HyperbolicCrystallizerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        hyperbolic_coupling_energy_mev: f64,
        topological_fractal_gap_mev: f64,
        acoustic_drive_frequency_ghz: f64,
        crystallization_drift_speed_m_per_s: f64,
        cryogenic_temperature_mk: f64,
        microwave_pinning_power_uw: f64,
        synthetic_curvature_radius_um: f64,
        hyperbolic_tessellation_pitch_um: f64,
    ) -> Self {
        Self {
            hyperbolic_coupling_energy_mev: hyperbolic_coupling_energy_mev.clamp(1.0, 35.0),
            topological_fractal_gap_mev: topological_fractal_gap_mev.clamp(2.0, 45.0),
            acoustic_drive_frequency_ghz: acoustic_drive_frequency_ghz.clamp(1.0, 12.0),
            crystallization_drift_speed_m_per_s: crystallization_drift_speed_m_per_s.clamp(200.0, 3000.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            microwave_pinning_power_uw: microwave_pinning_power_uw.clamp(0.5, 30.0),
            synthetic_curvature_radius_um: synthetic_curvature_radius_um.clamp(0.1, 5.0),
            hyperbolic_tessellation_pitch_um: hyperbolic_tessellation_pitch_um.clamp(0.5, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian chiral topological
/// hyperbolic lattice anyon crystallizers and fractal boundary engines.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HyperbolicCrystallizerMetrics {
    /// Crystallization fidelity across hyperbolic lattice anyon crystallizers (target >= 0.9980).
    pub crystallization_fidelity: f64,
    /// Crystallized anyon quantum state retention fraction (target >= 0.9970).
    pub crystallized_state_retention_fraction: f64,
    /// Topological protection energy gap in MHz (target >= 45.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-channel crosstalk acoustic isolation in decibels (target >= 55.0 dB).
    pub inter_channel_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 12.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
