#![deny(unsafe_code)]

//! Physical parameters and metrics configuration for quantum opto-electro-phononic
//! frequency translators and millimeter-wave cavity interfaces.

/// Physical parameter configuration for opto-electro-phononic frequency translators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptoElectroPhononicTranslatorParams {
    /// Millimeter-wave cavity drive/resonance frequency in GHz (clamp 20.0 to 120.0, default 45.0).
    pub mmwave_frequency_ghz: f64,
    /// Telecom optical carrier wavelength in nm (clamp 1500.0 to 1600.0, default 1550.0).
    pub telecom_wavelength_nm: f64,
    /// Piezoelectric electromechanical cooperativity (clamp 5.0 to 100.0, default 35.0).
    pub piezoelectric_cooperativity: f64,
    /// Optomechanical radiation-pressure cooperativity (clamp 5.0 to 100.0, default 35.0).
    pub optomechanical_cooperativity: f64,
    /// Phononic crystal mechanical breathing mode damping rate in MHz (clamp 0.1 to 10.0, default 1.2).
    pub acoustic_damping_rate_mhz: f64,
    /// Optical cavity loaded quality factor Q (clamp 1.0e5 to 1.0e7, default 2.5e6).
    pub optical_q_factor: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Telecom pump laser drive power in mW (clamp 0.1 to 20.0, default 4.5).
    pub pump_laser_power_mw: f64,
}

impl Default for OptoElectroPhononicTranslatorParams {
    fn default() -> Self {
        Self {
            mmwave_frequency_ghz: 45.0,
            telecom_wavelength_nm: 1550.0,
            piezoelectric_cooperativity: 35.0,
            optomechanical_cooperativity: 35.0,
            acoustic_damping_rate_mhz: 1.2,
            optical_q_factor: 2.5e6,
            operating_temp_m_k: 15.0,
            pump_laser_power_mw: 4.5,
        }
    }
}

impl OptoElectroPhononicTranslatorParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        mmwave_frequency_ghz: f64,
        telecom_wavelength_nm: f64,
        piezoelectric_cooperativity: f64,
        optomechanical_cooperativity: f64,
        acoustic_damping_rate_mhz: f64,
        optical_q_factor: f64,
        operating_temp_m_k: f64,
        pump_laser_power_mw: f64,
    ) -> Self {
        Self {
            mmwave_frequency_ghz: mmwave_frequency_ghz.clamp(20.0, 120.0),
            telecom_wavelength_nm: telecom_wavelength_nm.clamp(1500.0, 1600.0),
            piezoelectric_cooperativity: piezoelectric_cooperativity.clamp(5.0, 100.0),
            optomechanical_cooperativity: optomechanical_cooperativity.clamp(5.0, 100.0),
            acoustic_damping_rate_mhz: acoustic_damping_rate_mhz.clamp(0.1, 10.0),
            optical_q_factor: optical_q_factor.clamp(1.0e5, 1.0e7),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            pump_laser_power_mw: pump_laser_power_mw.clamp(0.1, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum opto-electro-phononic frequency translators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OptoElectroPhononicTranslatorMetrics {
    /// Bidirectional quantum transduction efficiency between millimeter-wave and optical channels (target >= 0.800).
    pub transduction_efficiency: f64,
    /// Added thermal noise quanta referred to input (target <= 0.100).
    pub added_thermal_noise_quanta: f64,
    /// Instantaneous photon-phonon-photon conversion bandwidth in MHz (target >= 5.0).
    pub conversion_bandwidth_mhz: f64,
    /// Quantum state transfer fidelity across the translator bridge (target >= 0.9850).
    pub quantum_state_transfer_fidelity: f64,
    /// Optomechanical sideband ground-state cooling phonon occupancy (target <= 0.050).
    pub ground_state_cooling_occupancy: f64,
    /// Overall physical compliance flag across all roadmap performance targets.
    pub is_physically_compliant: bool,
}
