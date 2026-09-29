#![deny(unsafe_code)]

//! Physical parameters and metrics configuration for non-Abelian quantum acoustic
//! holonomic gates and geometric phase processors in phononic resonator networks.

/// Physical parameter configuration for acoustic holonomic processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticHolonomicProcessorParams {
    /// Phononic crystal acoustic resonance frequency in GHz (clamp 2.0 to 15.0, default 6.0).
    pub acoustic_resonance_ghz: f64,
    /// Piezoelectric microwave drive amplitude in MHz (clamp 10.0 to 100.0, default 35.0).
    pub piezoelectric_drive_amplitude_mhz: f64,
    /// Non-Abelian Wilczek-Zee geometric phase angle in radians (clamp 0.1 to 3.14159, default 1.5708).
    pub wilczek_zee_phase_rad: f64,
    /// Dynamical phase cancellation efficiency ratio (clamp 0.90 to 1.00, default 0.99).
    pub dynamical_phase_cancellation_ratio: f64,
    /// Phonon acoustic damping dissipation rate in kHz (clamp 0.5 to 50.0, default 5.0).
    pub acoustic_damping_rate_khz: f64,
    /// Cryogenic dilution refrigerator operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Multi-mode inter-qubit acoustic coupling rate in MHz (clamp 5.0 to 50.0, default 20.0).
    pub qubit_coupling_rate_mhz: f64,
    /// Microwave control pulse rise time in ns (clamp 1.0 to 20.0, default 5.0).
    pub pulse_rise_time_ns: f64,
}

impl Default for AcousticHolonomicProcessorParams {
    fn default() -> Self {
        Self {
            acoustic_resonance_ghz: 6.0,
            piezoelectric_drive_amplitude_mhz: 35.0,
            wilczek_zee_phase_rad: 1.5708,
            dynamical_phase_cancellation_ratio: 0.99,
            acoustic_damping_rate_khz: 5.0,
            operating_temp_m_k: 15.0,
            qubit_coupling_rate_mhz: 20.0,
            pulse_rise_time_ns: 5.0,
        }
    }
}

impl AcousticHolonomicProcessorParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        acoustic_resonance_ghz: f64,
        piezoelectric_drive_amplitude_mhz: f64,
        wilczek_zee_phase_rad: f64,
        dynamical_phase_cancellation_ratio: f64,
        acoustic_damping_rate_khz: f64,
        operating_temp_m_k: f64,
        qubit_coupling_rate_mhz: f64,
        pulse_rise_time_ns: f64,
    ) -> Self {
        Self {
            acoustic_resonance_ghz: acoustic_resonance_ghz.clamp(2.0, 15.0),
            piezoelectric_drive_amplitude_mhz: piezoelectric_drive_amplitude_mhz.clamp(10.0, 100.0),
            wilczek_zee_phase_rad: wilczek_zee_phase_rad.clamp(0.1, 3.14159),
            dynamical_phase_cancellation_ratio: dynamical_phase_cancellation_ratio.clamp(0.90, 1.00),
            acoustic_damping_rate_khz: acoustic_damping_rate_khz.clamp(0.5, 50.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            qubit_coupling_rate_mhz: qubit_coupling_rate_mhz.clamp(5.0, 50.0),
            pulse_rise_time_ns: pulse_rise_time_ns.clamp(1.0, 20.0),
        }
    }
}

/// Multi-physics evaluation metrics for non-Abelian quantum acoustic holonomic processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AcousticHolonomicProcessorMetrics {
    /// Non-adiabatic holonomic gate fidelity (target >= 0.9950).
    pub holonomic_gate_fidelity: f64,
    /// Total gate operation cycle time in ns (target <= 200.0).
    pub gate_operation_time_ns: f64,
    /// Environmental gate dephasing error rate under thermal phonon noise (target <= 1.0e-3).
    pub gate_error_rate: f64,
    /// Two-qubit entangling geometric gate fidelity (target >= 0.9920).
    pub two_qubit_entangling_fidelity: f64,
    /// Geometric phase purity reflecting immunity against dynamical phase drift (target >= 0.9900).
    pub geometric_purity: f64,
    /// Overall physical compliance flag across all roadmap performance targets.
    pub is_physically_compliant: bool,
}
