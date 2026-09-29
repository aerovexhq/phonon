#![deny(unsafe_code)]

//! Parameter configurations and multi-physics evaluation metrics for
//! quantum phononic non-Abelian anyon colliders and multi-qubit topological
//! braiding interferometers.

/// Physical parameter configuration for quantum phononic non-Abelian anyon colliders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicAnyonColliderParams {
    /// Acoustic center frequency in gigahertz (clamp 1.0 to 15.0, default 4.2 GHz).
    pub acoustic_frequency_ghz: f64,
    /// Acoustic beam splitter power reflectivity (clamp 0.1 to 0.9, default 0.50).
    pub splitter_reflectivity: f64,
    /// Anyonic wavepacket temporal pulse width in picoseconds (clamp 10.0 to 500.0, default 85.0 ps).
    pub anyon_wavepacket_width_ps: f64,
    /// Braiding interferometer arm length in micrometers (clamp 5.0 to 200.0, default 45.0 um).
    pub interferometer_arm_length_um: f64,
    /// Bulk topological protecting bandgap in megahertz (clamp 50.0 to 1000.0, default 280.0 MHz).
    pub topological_gap_mhz: f64,
    /// Phononic edge mode dephasing rate in kilohertz (clamp 0.1 to 100.0, default 5.0 kHz).
    pub dephasing_rate_khz: f64,
    /// Cryostat dilution refrigerator operating ambient temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0 mK).
    pub operating_temp_m_k: f64,
    /// Number of topological anyonic qubits in braiding network (clamp 2 to 16, default 4).
    pub qubit_count: usize,
}

impl Default for PhononicAnyonColliderParams {
    fn default() -> Self {
        Self {
            acoustic_frequency_ghz: 4.2,
            splitter_reflectivity: 0.50,
            anyon_wavepacket_width_ps: 85.0,
            interferometer_arm_length_um: 45.0,
            topological_gap_mhz: 280.0,
            dephasing_rate_khz: 5.0,
            operating_temp_m_k: 15.0,
            qubit_count: 4,
        }
    }
}

impl PhononicAnyonColliderParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        acoustic_frequency_ghz: f64,
        splitter_reflectivity: f64,
        anyon_wavepacket_width_ps: f64,
        interferometer_arm_length_um: f64,
        topological_gap_mhz: f64,
        dephasing_rate_khz: f64,
        operating_temp_m_k: f64,
        qubit_count: usize,
    ) -> Self {
        Self {
            acoustic_frequency_ghz: acoustic_frequency_ghz.clamp(1.0, 15.0),
            splitter_reflectivity: splitter_reflectivity.clamp(0.1, 0.9),
            anyon_wavepacket_width_ps: anyon_wavepacket_width_ps.clamp(10.0, 500.0),
            interferometer_arm_length_um: interferometer_arm_length_um.clamp(5.0, 200.0),
            topological_gap_mhz: topological_gap_mhz.clamp(50.0, 1000.0),
            dephasing_rate_khz: dephasing_rate_khz.clamp(0.1, 100.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            qubit_count: qubit_count.clamp(2, 16),
        }
    }
}

/// Multi-physics evaluation metrics for quantum phononic non-Abelian anyon colliders.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhononicAnyonColliderMetrics {
    /// Two-particle Hong-Ou-Mandel anyonic collision visibility (target >= 0.920).
    pub collision_visibility: f64,
    /// Cross-correlation noise suppression in decibels (target >= 25.0 dB).
    pub cross_correlation_noise_suppression_db: f64,
    /// Non-Abelian braiding phase error in radians (target <= 1.0e-4 rad).
    pub braiding_phase_error_rad: f64,
    /// Multi-qubit non-demolition topological parity readout fidelity (target >= 0.998).
    pub topological_parity_readout_fidelity: f64,
    /// Anyonic collision Fano factor (target <= 0.35).
    pub anyonic_fano_factor: f64,
    /// Overall physical compliance flag across all anyon collider metrics.
    pub is_physically_compliant: bool,
}
