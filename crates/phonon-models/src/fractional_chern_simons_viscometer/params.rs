#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! chiral fractional Chern-Simons hydrodynamics and anyonic holographic edge viscometers.

/// Physical parameter configuration for quantum acoustic chiral fractional Chern-Simons
/// hydrodynamics and anyonic holographic edge viscometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractionalChernSimonsViscometerParams {
    /// Fractional quantum Hall filling factor nu (clamp 0.20 to 1.00, default 0.3333333333333333).
    pub fractional_filling_factor_nu: f64,
    /// Perpendicular quantizing magnetic field in Tesla (clamp 2.0 to 16.0, default 9.5).
    pub magnetic_field_tesla: f64,
    /// Piezoelectric stress-strain coupling coefficient (clamp 0.10 to 0.95, default 0.68).
    pub piezoelectric_stress_coupling_coefficient: f64,
    /// Acoustic shear excitation frequency in GHz (clamp 1.0 to 15.0, default 4.2).
    pub acoustic_shear_frequency_ghz: f64,
    /// Operating cryogenic temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Viscometer acoustic channel interaction length in micrometers (clamp 2.0 to 30.0, default 8.5).
    pub viscometer_channel_length_um: f64,
    /// Quantum Hall chiral edge channel width in nanometers (clamp 20.0 to 200.0, default 65.0).
    pub edge_channel_width_nm: f64,
    /// Electron effective mass ratio m* / m_0 (clamp 0.05 to 0.50, default 0.067).
    pub electron_effective_mass_ratio: f64,
}

impl Default for FractionalChernSimonsViscometerParams {
    fn default() -> Self {
        Self {
            fractional_filling_factor_nu: 0.3333333333333333,
            magnetic_field_tesla: 9.5,
            piezoelectric_stress_coupling_coefficient: 0.68,
            acoustic_shear_frequency_ghz: 4.2,
            cryogenic_temperature_mk: 10.0,
            viscometer_channel_length_um: 8.5,
            edge_channel_width_nm: 65.0,
            electron_effective_mass_ratio: 0.067,
        }
    }
}

impl FractionalChernSimonsViscometerParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        fractional_filling_factor_nu: f64,
        magnetic_field_tesla: f64,
        piezoelectric_stress_coupling_coefficient: f64,
        acoustic_shear_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        viscometer_channel_length_um: f64,
        edge_channel_width_nm: f64,
        electron_effective_mass_ratio: f64,
    ) -> Self {
        Self {
            fractional_filling_factor_nu: fractional_filling_factor_nu.clamp(0.20, 1.00),
            magnetic_field_tesla: magnetic_field_tesla.clamp(2.0, 16.0),
            piezoelectric_stress_coupling_coefficient: piezoelectric_stress_coupling_coefficient
                .clamp(0.10, 0.95),
            acoustic_shear_frequency_ghz: acoustic_shear_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            viscometer_channel_length_um: viscometer_channel_length_um.clamp(2.0, 30.0),
            edge_channel_width_nm: edge_channel_width_nm.clamp(20.0, 200.0),
            electron_effective_mass_ratio: electron_effective_mass_ratio.clamp(0.05, 0.50),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic chiral fractional Chern-Simons
/// hydrodynamics and anyonic holographic edge viscometers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FractionalChernSimonsViscometerMetrics {
    /// Hall viscosity extraction measurement fidelity (target >= 0.9980).
    pub hall_viscosity_measurement_fidelity: f64,
    /// Edge-to-bulk acoustic crosstalk isolation in decibels (target >= 55.0 dB).
    pub edge_to_bulk_acoustic_isolation_db: f64,
    /// Chiral edge mode velocity stability fraction (target >= 0.9970).
    pub edge_mode_velocity_stability_fraction: f64,
    /// Anomalous edge acoustic dissipation rate in dB/um (target <= 0.0015 dB/um).
    pub anomalous_edge_acoustic_dissipation_db_per_um: f64,
    /// Non-equilibrium hydrodynamic entropy generation rate in W/K (target <= 1.0e-5 W/K).
    pub hydrodynamic_entropy_generation_rate_w_per_k: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
