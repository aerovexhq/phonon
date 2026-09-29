#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! chiral spin-mechanical frequency-bin entanglement and phononic Bell state analyzers.

/// Physical parameter configuration for chiral frequency-bin Bell state analyzers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralFrequencyBinBellAnalyzerParams {
    /// Parametric pump amplitude in MHz (clamp 2.0 to 50.0, default 16.5 MHz).
    pub parametric_pump_amplitude_mhz: f64,
    /// Frequency bin separation in MHz (clamp 10.0 to 200.0, default 65.0 MHz).
    pub bin_frequency_separation_mhz: f64,
    /// Spin-acoustic coupling rate in MHz (clamp 1.0 to 25.0, default 6.2 MHz).
    pub spin_acoustic_coupling_mhz: f64,
    /// Phononic cavity decay rate in kHz (clamp 10.0 to 300.0, default 75.0 kHz).
    pub cavity_decay_rate_khz: f64,
    /// Quantum detector efficiency (clamp 0.70 to 0.99, default 0.94).
    pub detector_quantum_efficiency: f64,
    /// Chiral isolation in dB (clamp 20.0 to 60.0, default 38.0 dB).
    pub chiral_isolation_db: f64,
    /// Dilution cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0 mK).
    pub cryogenic_temperature_mk: f64,
    /// Phonon measurement time window in microseconds (clamp 0.1 to 10.0, default 2.2 us).
    pub measurement_window_us: f64,
}

impl Default for ChiralFrequencyBinBellAnalyzerParams {
    fn default() -> Self {
        Self {
            parametric_pump_amplitude_mhz: 16.5,
            bin_frequency_separation_mhz: 65.0,
            spin_acoustic_coupling_mhz: 6.2,
            cavity_decay_rate_khz: 75.0,
            detector_quantum_efficiency: 0.94,
            chiral_isolation_db: 38.0,
            cryogenic_temperature_mk: 10.0,
            measurement_window_us: 2.2,
        }
    }
}

impl ChiralFrequencyBinBellAnalyzerParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        parametric_pump_amplitude_mhz: f64,
        bin_frequency_separation_mhz: f64,
        spin_acoustic_coupling_mhz: f64,
        cavity_decay_rate_khz: f64,
        detector_quantum_efficiency: f64,
        chiral_isolation_db: f64,
        cryogenic_temperature_mk: f64,
        measurement_window_us: f64,
    ) -> Self {
        Self {
            parametric_pump_amplitude_mhz: parametric_pump_amplitude_mhz.clamp(2.0, 50.0),
            bin_frequency_separation_mhz: bin_frequency_separation_mhz.clamp(10.0, 200.0),
            spin_acoustic_coupling_mhz: spin_acoustic_coupling_mhz.clamp(1.0, 25.0),
            cavity_decay_rate_khz: cavity_decay_rate_khz.clamp(10.0, 300.0),
            detector_quantum_efficiency: detector_quantum_efficiency.clamp(0.70, 0.99),
            chiral_isolation_db: chiral_isolation_db.clamp(20.0, 60.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            measurement_window_us: measurement_window_us.clamp(0.1, 10.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic chiral frequency-bin Bell state analyzers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralFrequencyBinBellAnalyzerMetrics {
    /// Bell state measurement fidelity (target >= 0.9950).
    pub bell_state_measurement_fidelity: f64,
    /// Frequency-bin mode indistinguishability (target >= 0.9980).
    pub frequency_bin_mode_indistinguishability: f64,
    /// Inter-bin cross-talk quantum dephasing rate in Hz (target <= 120.0 Hz).
    pub crosstalk_quantum_dephasing_rate_hz: f64,
    /// Dark count probability per measurement window (target <= 1.0e-5).
    pub dark_count_probability: f64,
    /// Two-phonon entanglement concurrence (target >= 0.980).
    pub two_phonon_entanglement_concurrence: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
