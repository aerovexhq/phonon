#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Hermitian Floquet exceptional-ring synthesizers and chiral skin sensors.

/// Physical parameter configuration for quantum acoustic non-Hermitian Floquet exceptional-ring
/// synthesizers and chiral skin sensors in dissipative phononic lattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetExceptionalRingSensorParams {
    /// Floquet drive modulation amplitude in MHz (clamp 5.0 to 80.0, default 32.0).
    pub floquet_drive_amplitude_mhz: f64,
    /// Floquet high-frequency modulation drive frequency in GHz (clamp 1.0 to 12.0, default 4.8).
    pub floquet_modulation_frequency_ghz: f64,
    /// Dimensionless non-reciprocal hopping asymmetry parameter (clamp 0.10 to 0.95, default 0.62).
    pub non_reciprocal_hopping_asymmetry: f64,
    /// Dissipative cavity loss contrast in kHz (clamp 20.0 to 500.0, default 140.0).
    pub cavity_loss_contrast_khz: f64,
    /// Number of spatial sensor array elements / resonators (clamp 8 to 64, default 24).
    pub sensor_array_elements: usize,
    /// External acoustic or inertial perturbation coupling strength in Hz (clamp 10.0 to 1000.0, default 150.0).
    pub perturbation_coupling_strength_hz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temperature_mk: f64,
    /// Transducer piezoelectric readout gain in dB (clamp 10.0 to 45.0, default 26.0).
    pub piezoelectric_gain_db: f64,
}

impl Default for FloquetExceptionalRingSensorParams {
    fn default() -> Self {
        Self {
            floquet_drive_amplitude_mhz: 32.0,
            floquet_modulation_frequency_ghz: 4.8,
            non_reciprocal_hopping_asymmetry: 0.62,
            cavity_loss_contrast_khz: 140.0,
            sensor_array_elements: 24,
            perturbation_coupling_strength_hz: 150.0,
            operating_temperature_mk: 15.0,
            piezoelectric_gain_db: 26.0,
        }
    }
}

impl FloquetExceptionalRingSensorParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        floquet_drive_amplitude_mhz: f64,
        floquet_modulation_frequency_ghz: f64,
        non_reciprocal_hopping_asymmetry: f64,
        cavity_loss_contrast_khz: f64,
        sensor_array_elements: usize,
        perturbation_coupling_strength_hz: f64,
        operating_temperature_mk: f64,
        piezoelectric_gain_db: f64,
    ) -> Self {
        Self {
            floquet_drive_amplitude_mhz: floquet_drive_amplitude_mhz.clamp(5.0, 80.0),
            floquet_modulation_frequency_ghz: floquet_modulation_frequency_ghz.clamp(1.0, 12.0),
            non_reciprocal_hopping_asymmetry: non_reciprocal_hopping_asymmetry.clamp(0.10, 0.95),
            cavity_loss_contrast_khz: cavity_loss_contrast_khz.clamp(20.0, 500.0),
            sensor_array_elements: sensor_array_elements.clamp(8, 64),
            perturbation_coupling_strength_hz: perturbation_coupling_strength_hz.clamp(10.0, 1000.0),
            operating_temperature_mk: operating_temperature_mk.clamp(1.0, 50.0),
            piezoelectric_gain_db: piezoelectric_gain_db.clamp(10.0, 45.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Hermitian Floquet
/// exceptional-ring synthesizers and chiral skin sensors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetExceptionalRingSensorMetrics {
    /// Ratio of skin-mode spatial localization on boundaries under open boundary conditions (target >= 0.940).
    pub skin_mode_localization_ratio: f64,
    /// Non-Hermitian exceptional-point eigenvalue sensitivity enhancement factor (target >= 85.0).
    pub sensitivity_enhancement_factor: f64,
    /// Directional reverse backscattering suppression ratio S21/S12 in dB (target >= 52.0 dB).
    pub reverse_backscattering_suppression_db: f64,
    /// Effective cryogenic sensor noise figure in dB (target <= 0.45 dB).
    pub sensor_noise_figure_db: f64,
    /// Quantized topological winding charge of the synthesized Floquet exceptional ring (target >= 0.990).
    pub exceptional_ring_topological_charge: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
