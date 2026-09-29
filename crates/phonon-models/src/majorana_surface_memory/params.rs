#![deny(unsafe_code)]

//! Parameter configurations and multi-physics evaluation metrics for
//! topological quantum acoustic memory and Majorana surface code decoders.

/// Physical parameter configuration for topological quantum acoustic memory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaSurfaceMemoryParams {
    /// Surface code distance d, representing stabilizer lattice dimension (clamp 3 to 15, default 5).
    pub code_distance: usize,
    /// High-Q phononic defect cavity resonance frequency in gigahertz (clamp 1.0 to 12.0, default 4.8 GHz).
    pub cavity_resonance_ghz: f64,
    /// Intrinsic acoustic defect cavity quality factor (clamp 1.0e6 to 1.0e9, default 2.5e7).
    pub acoustic_quality_factor: f64,
    /// Physical qubit/mode error rate per cycle (clamp 1.0e-4 to 0.05, default 0.0015).
    pub physical_error_rate: f64,
    /// Surface code stabilizer syndrome extraction time in nanoseconds (clamp 50.0 to 1000.0, default 250.0 ns).
    pub syndrome_extraction_time_ns: f64,
    /// Phonon-Majorana zero mode hybrid coupling strength in megahertz (clamp 5.0 to 100.0, default 35.0 MHz).
    pub majorana_coupling_mhz: f64,
    /// Cryostat dilution refrigerator operating ambient temperature in milli-Kelvin (clamp 1.0 to 50.0, default 12.0 mK).
    pub operating_temp_m_k: f64,
    /// Dispersive cavity shift for single-shot parity readout in megahertz (clamp 1.0 to 30.0, default 8.5 MHz).
    pub readout_dispersive_shift_mhz: f64,
}

impl Default for MajoranaSurfaceMemoryParams {
    fn default() -> Self {
        Self {
            code_distance: 5,
            cavity_resonance_ghz: 4.8,
            acoustic_quality_factor: 2.5e7,
            physical_error_rate: 0.0015,
            syndrome_extraction_time_ns: 250.0,
            majorana_coupling_mhz: 35.0,
            operating_temp_m_k: 12.0,
            readout_dispersive_shift_mhz: 8.5,
        }
    }
}

impl MajoranaSurfaceMemoryParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        code_distance: usize,
        cavity_resonance_ghz: f64,
        acoustic_quality_factor: f64,
        physical_error_rate: f64,
        syndrome_extraction_time_ns: f64,
        majorana_coupling_mhz: f64,
        operating_temp_m_k: f64,
        readout_dispersive_shift_mhz: f64,
    ) -> Self {
        Self {
            code_distance: code_distance.clamp(3, 15),
            cavity_resonance_ghz: cavity_resonance_ghz.clamp(1.0, 12.0),
            acoustic_quality_factor: acoustic_quality_factor.clamp(1.0e6, 1.0e9),
            physical_error_rate: physical_error_rate.clamp(1.0e-4, 0.05),
            syndrome_extraction_time_ns: syndrome_extraction_time_ns.clamp(50.0, 1000.0),
            majorana_coupling_mhz: majorana_coupling_mhz.clamp(5.0, 100.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            readout_dispersive_shift_mhz: readout_dispersive_shift_mhz.clamp(1.0, 30.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological quantum acoustic memory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MajoranaSurfaceMemoryMetrics {
    /// Quantum memory coherence dephasing time T2 in milliseconds (target >= 10.0 ms).
    pub quantum_coherence_t2_ms: f64,
    /// Fault-tolerant surface code physical error threshold (target >= 0.010 or 1.0%).
    pub fault_tolerant_threshold: f64,
    /// Minimum-Weight Perfect Matching (MWPM) syndrome decoding latency in microseconds (target <= 2.50 us).
    pub syndrome_decoding_latency_us: f64,
    /// Logical qubit error rate per syndrome cycle (target <= 1.0e-5).
    pub logical_error_rate: f64,
    /// Single-shot acoustic qubit storage fidelity (target >= 0.995 or 99.5%).
    pub acoustic_qubit_storage_fidelity: f64,
    /// Overall physical compliance flag across all fault-tolerant memory thresholds.
    pub is_physically_compliant: bool,
}
