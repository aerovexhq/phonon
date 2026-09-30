#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! non-Abelian anyonic quantum memory and chiral Fibonacci braiding gate fabrics.

/// Physical parameter configuration for quantum acoustic non-Abelian Fibonacci anyonic
/// quantum memory and chiral braiding gate fabrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibonacciAnyonQuantumMemoryParams {
    /// Golden ratio quantum dimension parameter tau (clamp 1.50 to 1.70, default 1.618033988749895).
    pub golden_ratio_tau: f64,
    /// Topological protection gap energy in MHz (clamp 30.0 to 90.0, default 58.0).
    pub topological_gap_energy_mhz: f64,
    /// Fibonacci braid word decomposition length (clamp 10.0 to 100.0, default 32.0).
    pub braid_word_length: f64,
    /// Acoustic clock frequency in GHz (clamp 1.0 to 15.0, default 5.2).
    pub acoustic_clock_frequency_ghz: f64,
    /// Operating cryogenic dilution refrigerator temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Spatial separation between adjacent anyons in micrometers (clamp 0.5 to 8.0, default 2.6).
    pub inter_anyon_separation_um: f64,
    /// Quantum memory retention holding time in microseconds (clamp 10.0 to 500.0, default 120.0).
    pub memory_retention_time_us: f64,
    /// Dynamic strain-induced anyon shuttling velocity in m/s (clamp 200.0 to 3000.0, default 1250.0).
    pub strain_shuttling_velocity_mps: f64,
}

impl Default for FibonacciAnyonQuantumMemoryParams {
    fn default() -> Self {
        Self {
            golden_ratio_tau: 1.618033988749895,
            topological_gap_energy_mhz: 58.0,
            braid_word_length: 32.0,
            acoustic_clock_frequency_ghz: 5.2,
            cryogenic_temperature_mk: 10.0,
            inter_anyon_separation_um: 2.6,
            memory_retention_time_us: 120.0,
            strain_shuttling_velocity_mps: 1250.0,
        }
    }
}

impl FibonacciAnyonQuantumMemoryParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        golden_ratio_tau: f64,
        topological_gap_energy_mhz: f64,
        braid_word_length: f64,
        acoustic_clock_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        inter_anyon_separation_um: f64,
        memory_retention_time_us: f64,
        strain_shuttling_velocity_mps: f64,
    ) -> Self {
        Self {
            golden_ratio_tau: golden_ratio_tau.clamp(1.50, 1.70),
            topological_gap_energy_mhz: topological_gap_energy_mhz.clamp(30.0, 90.0),
            braid_word_length: braid_word_length.clamp(10.0, 100.0),
            acoustic_clock_frequency_ghz: acoustic_clock_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            inter_anyon_separation_um: inter_anyon_separation_um.clamp(0.5, 8.0),
            memory_retention_time_us: memory_retention_time_us.clamp(10.0, 500.0),
            strain_shuttling_velocity_mps: strain_shuttling_velocity_mps.clamp(200.0, 3000.0),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic non-Abelian Fibonacci anyonic
/// quantum memory and chiral braiding gate fabrics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FibonacciAnyonQuantumMemoryMetrics {
    /// Universal topological braiding gate fidelity (target >= 0.9980).
    pub braiding_gate_fidelity: f64,
    /// Anyon quantum memory retention fraction (target >= 0.9970).
    pub anyon_memory_retention_fraction: f64,
    /// Topological protection gap in MHz (target >= 44.0 MHz).
    pub topological_protection_gap_mhz: f64,
    /// Inter-qubit anyonic crosstalk isolation in decibels (target >= 54.0 dB).
    pub inter_qubit_crosstalk_isolation_db: f64,
    /// Topological mode dephasing rate in Hz (target <= 14.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
