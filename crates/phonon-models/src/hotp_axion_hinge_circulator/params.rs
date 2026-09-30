#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! higher-order axion electrodynamics and chiral quadrupole-hinge polariton circulators.

/// Physical parameter configuration for quantum acoustic higher-order axion electrodynamics
/// and chiral quadrupole-hinge polariton circulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotpAxionHingeCirculatorParams {
    /// Axion angle normalized to pi (clamp 0.80 to 1.20, default 1.0).
    pub axion_angle_theta_pi: f64,
    /// Quantized bulk quadrupole polarization Q_xy (clamp 0.35 to 0.65, default 0.50).
    pub quadrupole_polarization_qxy: f64,
    /// Magnetoelectric hinge coupling coefficient alpha (clamp 0.10 to 0.95, default 0.72).
    pub magnetoelectric_hinge_coupling_alpha: f64,
    /// Acoustic hinge polariton resonance frequency in GHz (clamp 1.0 to 15.0, default 5.6).
    pub acoustic_hinge_frequency_ghz: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Hinge channel length in micrometers (clamp 1.0 to 20.0, default 6.0).
    pub hinge_channel_length_um: f64,
    /// Spatial separation between adjacent parallel hinges in micrometers (clamp 0.5 to 10.0, default 3.5).
    pub inter_hinge_separation_um: f64,
    /// Acoustic cavity resonance quality factor (clamp 1.0e4 to 5.0e5, default 1.1e5).
    pub cavity_resonance_quality_factor: f64,
}

impl Default for HotpAxionHingeCirculatorParams {
    fn default() -> Self {
        Self {
            axion_angle_theta_pi: 1.0,
            quadrupole_polarization_qxy: 0.50,
            magnetoelectric_hinge_coupling_alpha: 0.72,
            acoustic_hinge_frequency_ghz: 5.6,
            cryogenic_temperature_mk: 10.0,
            hinge_channel_length_um: 6.0,
            inter_hinge_separation_um: 3.5,
            cavity_resonance_quality_factor: 1.1e5,
        }
    }
}

impl HotpAxionHingeCirculatorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        axion_angle_theta_pi: f64,
        quadrupole_polarization_qxy: f64,
        magnetoelectric_hinge_coupling_alpha: f64,
        acoustic_hinge_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        hinge_channel_length_um: f64,
        inter_hinge_separation_um: f64,
        cavity_resonance_quality_factor: f64,
    ) -> Self {
        Self {
            axion_angle_theta_pi: axion_angle_theta_pi.clamp(0.80, 1.20),
            quadrupole_polarization_qxy: quadrupole_polarization_qxy.clamp(0.35, 0.65),
            magnetoelectric_hinge_coupling_alpha: magnetoelectric_hinge_coupling_alpha
                .clamp(0.10, 0.95),
            acoustic_hinge_frequency_ghz: acoustic_hinge_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            hinge_channel_length_um: hinge_channel_length_um.clamp(1.0, 20.0),
            inter_hinge_separation_um: inter_hinge_separation_um.clamp(0.5, 10.0),
            cavity_resonance_quality_factor: cavity_resonance_quality_factor
                .clamp(1.0e4, 5.0e5),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic higher-order axion electrodynamics
/// and chiral quadrupole-hinge polariton circulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotpAxionHingeCirculatorMetrics {
    /// Chiral hinge polariton transmission state fidelity (target >= 0.9980).
    pub hinge_polariton_transmission_fidelity: f64,
    /// Higher-order topological hinge protection gap in MHz (target >= 46.0 MHz).
    pub higher_order_topological_gap_mhz: f64,
    /// Dynamic non-reciprocal multi-port isolation in decibels (target >= 54.0 dB).
    pub dynamic_non_reciprocal_isolation_db: f64,
    /// Inter-hinge acoustic crosstalk isolation in decibels (target >= 53.0 dB).
    pub inter_hinge_crosstalk_isolation_db: f64,
    /// Topological hinge mode dephasing rate in Hz (target <= 15.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
