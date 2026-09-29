#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for quantum non-Abelian
//! holonomic acoustic gate processors and braided phonon circuit architectures.

/// Physical parameter configuration for holonomic acoustic quantum processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolonomicQuantumProcessorParams {
    /// Qubit acoustic resonance frequency in GHz (clamp 2.0 to 12.0, default 5.5).
    pub qubit_acoustic_freq_ghz: f64,
    /// Driving field Rabi amplitude in MHz (clamp 20.0 to 200.0, default 90.0).
    pub driving_field_amplitude_mhz: f64,
    /// Dynamical phase cancellation depth factor (clamp 0.92 to 1.00, default 0.995).
    pub dynamical_phase_cancellation_depth: f64,
    /// Inter-qubit acoustic exchange coupling rate in MHz (clamp 10.0 to 100.0, default 45.0).
    pub inter_qubit_coupling_mhz: f64,
    /// Acoustic phonon dephasing rate in kHz (clamp 0.2 to 20.0, default 2.0).
    pub acoustic_dephasing_rate_khz: f64,
    /// Cryostat operating temperature in millikelvin (clamp 1.0 to 50.0, default 12.0).
    pub operating_temp_m_k: f64,
    /// Pulse shaping truncation duration in nanoseconds (clamp 1.0 to 10.0, default 3.5).
    pub pulse_shaping_truncation_ns: f64,
    /// Qubit register size count (clamp 2 to 32, default 8).
    pub qubit_register_size: usize,
}

impl Default for HolonomicQuantumProcessorParams {
    fn default() -> Self {
        Self {
            qubit_acoustic_freq_ghz: 5.5,
            driving_field_amplitude_mhz: 90.0,
            dynamical_phase_cancellation_depth: 0.995,
            inter_qubit_coupling_mhz: 45.0,
            acoustic_dephasing_rate_khz: 2.0,
            operating_temp_m_k: 12.0,
            pulse_shaping_truncation_ns: 3.5,
            qubit_register_size: 8,
        }
    }
}

impl HolonomicQuantumProcessorParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        qubit_acoustic_freq_ghz: f64,
        driving_field_amplitude_mhz: f64,
        dynamical_phase_cancellation_depth: f64,
        inter_qubit_coupling_mhz: f64,
        acoustic_dephasing_rate_khz: f64,
        operating_temp_m_k: f64,
        pulse_shaping_truncation_ns: f64,
        qubit_register_size: usize,
    ) -> Self {
        Self {
            qubit_acoustic_freq_ghz: qubit_acoustic_freq_ghz.clamp(2.0, 12.0),
            driving_field_amplitude_mhz: driving_field_amplitude_mhz.clamp(20.0, 200.0),
            dynamical_phase_cancellation_depth: dynamical_phase_cancellation_depth.clamp(0.92, 1.00),
            inter_qubit_coupling_mhz: inter_qubit_coupling_mhz.clamp(10.0, 100.0),
            acoustic_dephasing_rate_khz: acoustic_dephasing_rate_khz.clamp(0.2, 20.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            pulse_shaping_truncation_ns: pulse_shaping_truncation_ns.clamp(1.0, 10.0),
            qubit_register_size: qubit_register_size.clamp(2, 32),
        }
    }
}

/// Multi-physics performance evaluation metrics for holonomic acoustic gate processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HolonomicQuantumProcessorMetrics {
    /// Holonomic non-Abelian quantum gate fidelity (target >= 0.9960).
    pub holonomic_gate_fidelity: f64,
    /// Two-qubit geometric entangling gate duration in nanoseconds (target <= 35.0).
    pub two_qubit_gate_duration_ns: f64,
    /// Geometric phase error in radians (target <= 0.0050).
    pub geometric_phase_error: f64,
    /// Fault-tolerant quantum acoustic logic circuit depth (target >= 100).
    pub fault_tolerant_logic_depth: usize,
    /// Inter-qubit crosstalk isolation in dB (target >= 40.0).
    pub crosstalk_isolation_db: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
