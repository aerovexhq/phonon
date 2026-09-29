//! Physical parameter models and multi-physics evaluation metrics for quantum
//! acoustic metasurface holography and dynamic phonon routing.

/// Physical parameter configuration for sub-wavelength reconfigurable acoustic
/// metasurfaces with voltage-tunable local phase gradients.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetasurfaceHolographyParams {
    /// Operating microwave acoustic frequency in GHz (clamp 0.5 to 15.0, default 3.5 GHz).
    pub operating_frequency_ghz: f64,
    /// Sub-wavelength unit cell pitch $d$ in micrometers (clamp 0.1 to 10.0, default 0.65 um).
    pub unit_cell_pitch_um: f64,
    /// Total number of metasurface array phase-gradient elements $N$ (clamp 16 to 256, default 64).
    pub array_elements_count: usize,
    /// Digital phase shifter resolution in bits (clamp 3 to 12, default 6 bits).
    pub phase_resolution_bits: usize,
    /// Piezoelectric local tuning bias voltage in Volts (clamp 0.5 to 15.0, default 3.3 V).
    pub piezoelectric_tuning_voltage_v: f64,
    /// Local tuning gate electrode series resistance in Ohms (clamp 10.0 to 500.0, default 45.0 Ohms).
    pub electrode_resistance_ohms: f64,
    /// Local tuning gate electrode capacitance in picofarads (clamp 0.01 to 20.0, default 0.05 pF).
    pub electrode_capacitance_pf: f64,
    /// Metasurface acoustic attenuation loss in dB per micrometer (clamp 1e-4 to 0.1, default 0.002 dB/um).
    pub acoustic_loss_db_per_um: f64,
}

impl Default for MetasurfaceHolographyParams {
    fn default() -> Self {
        Self {
            operating_frequency_ghz: 3.5,
            unit_cell_pitch_um: 0.65,
            array_elements_count: 64,
            phase_resolution_bits: 6,
            piezoelectric_tuning_voltage_v: 3.3,
            electrode_resistance_ohms: 45.0,
            electrode_capacitance_pf: 0.05,
            acoustic_loss_db_per_um: 0.002,
        }
    }
}

impl MetasurfaceHolographyParams {
    /// Creates a new parameter configuration with rigorous physical bounds clamping.
    pub fn new(
        operating_frequency_ghz: f64,
        unit_cell_pitch_um: f64,
        array_elements_count: usize,
        phase_resolution_bits: usize,
        piezoelectric_tuning_voltage_v: f64,
        electrode_resistance_ohms: f64,
        electrode_capacitance_pf: f64,
        acoustic_loss_db_per_um: f64,
    ) -> Self {
        Self {
            operating_frequency_ghz: operating_frequency_ghz.clamp(0.5, 15.0),
            unit_cell_pitch_um: unit_cell_pitch_um.clamp(0.1, 10.0),
            array_elements_count: array_elements_count.clamp(16, 256),
            phase_resolution_bits: phase_resolution_bits.clamp(3, 12),
            piezoelectric_tuning_voltage_v: piezoelectric_tuning_voltage_v.clamp(0.5, 15.0),
            electrode_resistance_ohms: electrode_resistance_ohms.clamp(10.0, 500.0),
            electrode_capacitance_pf: electrode_capacitance_pf.clamp(0.01, 20.0),
            acoustic_loss_db_per_um: acoustic_loss_db_per_um.clamp(1e-4, 0.1),
        }
    }
}

/// Multi-physics evaluation metrics for acoustic metasurface holography and dynamic routing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetasurfaceHolographyMetrics {
    /// Holographic beam steering efficiency into target receiver node (target >= 0.880 or 88.0%).
    pub beam_steering_efficiency: f64,
    /// Inter-channel acoustic crosstalk to unselected adjacent output channels in dB (target <= -35.0 dB).
    pub inter_channel_crosstalk_db: f64,
    /// Dynamic wavefront reconfiguration latency in nanoseconds (target <= 10.0 ns).
    pub reconfiguration_latency_ns: f64,
    /// Total acoustic transmission insertion loss in dB (target <= 1.20 dB).
    pub insertion_loss_db: f64,
    /// Multi-channel routing fidelity (target >= 0.960).
    pub routing_channel_fidelity: f64,
    /// Physical compliance verification flag across all design criteria.
    pub is_physically_compliant: bool,
}
