#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for topological
//! acoustic higher-rank tensor gauge fields and chiral monopole-plaquette phononic sensors.

/// Physical parameter configuration for topological acoustic higher-rank tensor gauge
/// fields and chiral monopole-plaquette phononic sensors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TensorGaugeMonopoleSensorParams {
    /// Dimensionless higher-rank tensor gauge coupling constant g (clamp 0.20 to 5.0, default 1.65).
    pub tensor_gauge_coupling_constant: f64,
    /// Chiral plaquette ring-exchange coupling energy J_p in meV (clamp 1.0 to 30.0, default 12.5).
    pub chiral_plaquette_coupling_energy_mev: f64,
    /// Acoustic sensor resonance frequency in GHz (clamp 1.0 to 15.0, default 5.4).
    pub acoustic_sensor_frequency_ghz: f64,
    /// Operating cryogenic temperature in milli-Kelvin (clamp 1.0 to 50.0, default 14.0).
    pub cryogenic_temperature_mk: f64,
    /// Unit cell lattice constant a in nanometers (clamp 40.0 to 400.0, default 150.0).
    pub lattice_cell_dimension_nm: f64,
    /// Dipole conservation constraint weight alpha_d (clamp 0.50 to 0.99, default 0.94).
    pub dipole_conservation_constraint_weight: f64,
    /// Topological monopole pinning magnetic field B_p in Tesla (clamp 0.5 to 10.0, default 3.8).
    pub monopole_pinning_field_tesla: f64,
    /// Acoustic sensing cavity quality factor Q (clamp 1.0e4 to 5.0e5, default 1.2e5).
    pub sensing_cavity_quality_factor: f64,
}

impl Default for TensorGaugeMonopoleSensorParams {
    fn default() -> Self {
        Self {
            tensor_gauge_coupling_constant: 1.65,
            chiral_plaquette_coupling_energy_mev: 12.5,
            acoustic_sensor_frequency_ghz: 5.4,
            cryogenic_temperature_mk: 14.0,
            lattice_cell_dimension_nm: 150.0,
            dipole_conservation_constraint_weight: 0.94,
            monopole_pinning_field_tesla: 3.8,
            sensing_cavity_quality_factor: 1.2e5,
        }
    }
}

impl TensorGaugeMonopoleSensorParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        tensor_gauge_coupling_constant: f64,
        chiral_plaquette_coupling_energy_mev: f64,
        acoustic_sensor_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        lattice_cell_dimension_nm: f64,
        dipole_conservation_constraint_weight: f64,
        monopole_pinning_field_tesla: f64,
        sensing_cavity_quality_factor: f64,
    ) -> Self {
        Self {
            tensor_gauge_coupling_constant: tensor_gauge_coupling_constant.clamp(0.20, 5.0),
            chiral_plaquette_coupling_energy_mev: chiral_plaquette_coupling_energy_mev
                .clamp(1.0, 30.0),
            acoustic_sensor_frequency_ghz: acoustic_sensor_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            lattice_cell_dimension_nm: lattice_cell_dimension_nm.clamp(40.0, 400.0),
            dipole_conservation_constraint_weight: dipole_conservation_constraint_weight
                .clamp(0.50, 0.99),
            monopole_pinning_field_tesla: monopole_pinning_field_tesla.clamp(0.5, 10.0),
            sensing_cavity_quality_factor: sensing_cavity_quality_factor.clamp(1.0e4, 5.0e5),
        }
    }
}

/// Multi-physics evaluation metrics for topological acoustic higher-rank tensor gauge
/// fields and chiral monopole-plaquette phononic sensors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TensorGaugeMonopoleSensorMetrics {
    /// Sensor tensor charge sensitivity enhancement factor (target >= 75.0).
    pub tensor_charge_sensitivity_enhancement: f64,
    /// Plaquette phase stability error in radians (target <= 0.0015).
    pub plaquette_phase_stability_error_rad: f64,
    /// Sub-dimensional mobility leakage fraction (target <= 1.0e-5).
    pub sub_dimensional_leakage: f64,
    /// Topological monopole lifetime in milliseconds (target >= 25.0).
    pub topological_monopole_lifetime_ms: f64,
    /// Tensor gauge flux quantization fidelity (target >= 0.9970).
    pub tensor_gauge_flux_quantization_fidelity: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
