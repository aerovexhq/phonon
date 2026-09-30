#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for non-Abelian
//! quantum acoustic fault-tolerant surface codes and chiral Majorana stabilizer simulators.

/// Physical parameter configuration for non-Abelian quantum acoustic fault-tolerant
/// surface codes and chiral Majorana stabilizer simulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumAcousticSurfaceCodeParams {
    /// Topological code distance d (clamp 3.0 to 15.0, default 5.0).
    pub code_distance: f64,
    /// Physical qubit/acoustic error rate per stabilizer cycle (clamp 1.0e-4 to 0.02, default 0.0025).
    pub physical_error_rate: f64,
    /// Syndrome extraction cycle time in nanoseconds (clamp 10.0 to 300.0, default 65.0).
    pub syndrome_extraction_time_ns: f64,
    /// Chiral Majorana mode topological coupling energy gap in MHz (clamp 10.0 to 80.0, default 38.0).
    pub majorana_coupling_gap_mhz: f64,
    /// Cryogenic operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 12.0).
    pub cryogenic_temperature_mk: f64,
    /// Acoustic parity readout cavity resonance frequency in GHz (clamp 2.0 to 15.0, default 5.8).
    pub acoustic_stabilizer_frequency_ghz: f64,
    /// Inter-stabilizer cavity pitch distance in micrometers (clamp 1.0 to 15.0, default 4.2).
    pub inter_stabilizer_pitch_um: f64,
    /// Decoder maximum weight iterations for matching (clamp 10.0 to 200.0, default 50.0).
    pub decoder_maximum_weight_iterations: f64,
}

impl Default for QuantumAcousticSurfaceCodeParams {
    fn default() -> Self {
        Self {
            code_distance: 5.0,
            physical_error_rate: 0.0025,
            syndrome_extraction_time_ns: 65.0,
            majorana_coupling_gap_mhz: 38.0,
            cryogenic_temperature_mk: 12.0,
            acoustic_stabilizer_frequency_ghz: 5.8,
            inter_stabilizer_pitch_um: 4.2,
            decoder_maximum_weight_iterations: 50.0,
        }
    }
}

impl QuantumAcousticSurfaceCodeParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        code_distance: f64,
        physical_error_rate: f64,
        syndrome_extraction_time_ns: f64,
        majorana_coupling_gap_mhz: f64,
        cryogenic_temperature_mk: f64,
        acoustic_stabilizer_frequency_ghz: f64,
        inter_stabilizer_pitch_um: f64,
        decoder_maximum_weight_iterations: f64,
    ) -> Self {
        Self {
            code_distance: code_distance.clamp(3.0, 15.0),
            physical_error_rate: physical_error_rate.clamp(1.0e-4, 0.02),
            syndrome_extraction_time_ns: syndrome_extraction_time_ns.clamp(10.0, 300.0),
            majorana_coupling_gap_mhz: majorana_coupling_gap_mhz.clamp(10.0, 80.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            acoustic_stabilizer_frequency_ghz: acoustic_stabilizer_frequency_ghz.clamp(2.0, 15.0),
            inter_stabilizer_pitch_um: inter_stabilizer_pitch_um.clamp(1.0, 15.0),
            decoder_maximum_weight_iterations: decoder_maximum_weight_iterations.clamp(10.0, 200.0),
        }
    }
}

/// Multi-physics evaluation metrics for non-Abelian quantum acoustic fault-tolerant
/// surface codes and chiral Majorana stabilizer simulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuantumAcousticSurfaceCodeMetrics {
    /// Protected logical state fidelity (target >= 0.9980).
    pub logical_state_fidelity: f64,
    /// Fault-tolerant threshold error rate (target <= 0.0075).
    pub fault_tolerant_threshold_error_rate: f64,
    /// Syndrome extraction and decoding latency in nanoseconds (target <= 120.0).
    pub syndrome_decoding_latency_ns: f64,
    /// Uncorrectable logical error rate per stabilizer round (target <= 1.0e-5).
    pub uncorrectable_logical_error_rate: f64,
    /// Inter-stabilizer crosstalk isolation in decibels (target >= 52.0).
    pub inter_stabilizer_crosstalk_isolation_db: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
