#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for topological acoustic
//! higher-order axion insulators and chiral hinge soliton networks.

use std::f64::consts::PI;

/// Physical parameter configuration for 3D topological acoustic higher-order axion insulators
/// and chiral hinge soliton networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralHingeAxionSolitonParams {
    /// Quantized axion electrodynamic angle theta in radians (clamp 0.0 to 2*PI, default PI).
    pub axion_angle_rad: f64,
    /// Dimensionless non-linear acoustic magnetoelectric coupling coefficient alpha (clamp 0.10 to 5.0, default 1.85).
    pub magnetoelectric_coupling_alpha: f64,
    /// 3D bulk topological acoustic bandgap in MHz (clamp 10.0 to 120.0, default 42.0 MHz).
    pub topological_bulk_gap_mhz: f64,
    /// 1D chiral hinge soliton temporal pulse width in nanoseconds (clamp 0.20 to 10.0, default 1.8 ns).
    pub hinge_soliton_pulse_width_ns: f64,
    /// Dimensionless acoustic non-linearity parameter beta (clamp 0.001 to 0.08, default 0.015).
    pub acoustic_non_linearity_parameter: f64,
    /// Operating acoustic frequency in GHz (clamp 1.0 to 12.0, default 4.5 GHz).
    pub operating_frequency_ghz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0 mK).
    pub cryogenic_temperature_mk: f64,
    /// 3D metamaterial lattice unit-cell dimension along each spatial axis (clamp 6 to 32, default 14).
    pub lattice_dimension_3d: usize,
}

impl Default for ChiralHingeAxionSolitonParams {
    fn default() -> Self {
        Self {
            axion_angle_rad: PI,
            magnetoelectric_coupling_alpha: 1.85,
            topological_bulk_gap_mhz: 42.0,
            hinge_soliton_pulse_width_ns: 1.8,
            acoustic_non_linearity_parameter: 0.015,
            operating_frequency_ghz: 4.5,
            cryogenic_temperature_mk: 15.0,
            lattice_dimension_3d: 14,
        }
    }
}

impl ChiralHingeAxionSolitonParams {
    /// Creates a new parameter configuration with rigorous physical boundary clamping.
    pub fn new(
        axion_angle_rad: f64,
        magnetoelectric_coupling_alpha: f64,
        topological_bulk_gap_mhz: f64,
        hinge_soliton_pulse_width_ns: f64,
        acoustic_non_linearity_parameter: f64,
        operating_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        lattice_dimension_3d: usize,
    ) -> Self {
        Self {
            axion_angle_rad: axion_angle_rad.clamp(0.0, 2.0 * PI),
            magnetoelectric_coupling_alpha: magnetoelectric_coupling_alpha.clamp(0.10, 5.0),
            topological_bulk_gap_mhz: topological_bulk_gap_mhz.clamp(10.0, 120.0),
            hinge_soliton_pulse_width_ns: hinge_soliton_pulse_width_ns.clamp(0.20, 10.0),
            acoustic_non_linearity_parameter: acoustic_non_linearity_parameter.clamp(0.001, 0.08),
            operating_frequency_ghz: operating_frequency_ghz.clamp(1.0, 12.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            lattice_dimension_3d: lattice_dimension_3d.clamp(6, 32),
        }
    }
}

/// Multi-physics evaluation metrics for topological acoustic higher-order axion insulators
/// and chiral hinge soliton networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralHingeAxionSolitonMetrics {
    /// Chiral hinge state transmission fidelity under backscattering suppression (target >= 0.9970).
    pub hinge_state_transmission_fidelity: f64,
    /// Effective topological axion mass gap protecting hinge modes in MHz (target >= 25.0 MHz).
    pub topological_axion_gap_mhz: f64,
    /// Non-linear harmonic distortion of hinge acoustic solitons in dB (target <= -48.0 dB).
    pub non_linear_harmonic_distortion_db: f64,
    /// Spatial cross-talk isolation between adjacent 1D hinge waveguides in dB (target >= 46.0 dB).
    pub inter_hinge_crosstalk_isolation_db: f64,
    /// Group velocity of 1D chiral hinge solitons in meters per second (target >= 2200.0 m/s).
    pub hinge_soliton_group_velocity_mps: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
