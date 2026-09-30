#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for chiral acoustic
//! axion electrodynamics and dynamic magnetoelectric phonon circulators.

use std::f64::consts::PI;

/// Physical parameter configuration for chiral acoustic axion electrodynamics and
/// dynamic magnetoelectric phonon circulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralAxionCirculatorParams {
    /// Axion coupling constant theta in radians (clamp 1.0 to 3.5, default PI ~ 3.141592653589793).
    pub axion_coupling_constant_theta: f64,
    /// Dynamic magnetoelectric polarizability alpha (clamp 0.05 to 0.95, default 0.65).
    pub magnetoelectric_polarizability_alpha: f64,
    /// Acoustic circulation operating frequency in GHz (clamp 1.0 to 15.0, default 4.6).
    pub acoustic_circulation_frequency_ghz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub cryogenic_temperature_mk: f64,
    /// Magnetic heterostructure thickness in nanometers (clamp 20.0 to 250.0, default 85.0).
    pub magnetic_heterostructure_thickness_nm: f64,
    /// Inter-port angular spacing in degrees (clamp 100.0 to 140.0, default 120.0).
    pub inter_port_angular_spacing_deg: f64,
    /// Acoustic power drive in microwatts (clamp 0.1 to 50.0, default 5.0).
    pub acoustic_power_drive_uw: f64,
    /// Cavity resonance quality factor (clamp 1.0e4 to 5.0e5, default 8.5e4).
    pub cavity_resonance_quality_factor: f64,
}

impl Default for ChiralAxionCirculatorParams {
    fn default() -> Self {
        Self {
            axion_coupling_constant_theta: PI,
            magnetoelectric_polarizability_alpha: 0.65,
            acoustic_circulation_frequency_ghz: 4.6,
            cryogenic_temperature_mk: 15.0,
            magnetic_heterostructure_thickness_nm: 85.0,
            inter_port_angular_spacing_deg: 120.0,
            acoustic_power_drive_uw: 5.0,
            cavity_resonance_quality_factor: 8.5e4,
        }
    }
}

impl ChiralAxionCirculatorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        axion_coupling_constant_theta: f64,
        magnetoelectric_polarizability_alpha: f64,
        acoustic_circulation_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        magnetic_heterostructure_thickness_nm: f64,
        inter_port_angular_spacing_deg: f64,
        acoustic_power_drive_uw: f64,
        cavity_resonance_quality_factor: f64,
    ) -> Self {
        Self {
            axion_coupling_constant_theta: axion_coupling_constant_theta.clamp(1.0, 3.5),
            magnetoelectric_polarizability_alpha: magnetoelectric_polarizability_alpha
                .clamp(0.05, 0.95),
            acoustic_circulation_frequency_ghz: acoustic_circulation_frequency_ghz
                .clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            magnetic_heterostructure_thickness_nm: magnetic_heterostructure_thickness_nm
                .clamp(20.0, 250.0),
            inter_port_angular_spacing_deg: inter_port_angular_spacing_deg.clamp(100.0, 140.0),
            acoustic_power_drive_uw: acoustic_power_drive_uw.clamp(0.1, 50.0),
            cavity_resonance_quality_factor: cavity_resonance_quality_factor
                .clamp(1.0e4, 5.0e5),
        }
    }
}

/// Multi-physics evaluation metrics for chiral acoustic axion electrodynamics and
/// dynamic magnetoelectric phonon circulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralAxionCirculatorMetrics {
    /// Dynamic non-reciprocal isolation in decibels (target >= 52.0 dB).
    pub non_reciprocal_isolation_db: f64,
    /// Axion polariton transmission fidelity (target >= 0.9970).
    pub axion_polariton_transmission_fidelity: f64,
    /// Circulator insertion loss in decibels (target <= 0.35 dB).
    pub circulator_insertion_loss_db: f64,
    /// Axionic phase stability error in radians (target <= 0.0018 rad).
    pub axionic_phase_stability_error_rad: f64,
    /// Harmonic distortion suppression in decibels (target >= 54.0 dB).
    pub harmonic_distortion_suppression_db: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
