#![deny(unsafe_code)]

//! Physical parameters and metrics configuration for topological phononic
//! Floquet-Majorana braiding processors and non-Abelian topological logic.

/// Physical parameter configuration for Floquet-Majorana braiding processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetMajoranaBraidingProcessorParams {
    /// Floquet periodic drive frequency in GHz (clamp 1.0 to 20.0, default 8.0).
    pub floquet_drive_freq_ghz: f64,
    /// Periodic acoustic strain modulation amplitude in MHz (clamp 10.0 to 150.0, default 60.0).
    pub floquet_modulation_amplitude_mhz: f64,
    /// Synthetic non-Abelian gauge flux per plaquette in radians (clamp 0.1 to 3.14159, default 1.5708).
    pub synthetic_gauge_flux_rad: f64,
    /// Phononic crystal waveguide braiding channel length in micrometers (clamp 5.0 to 100.0, default 25.0).
    pub phononic_waveguide_length_um: f64,
    /// Inter-Majorana boundary coupling gap in MHz (clamp 5.0 to 80.0, default 30.0).
    pub majorana_coupling_gap_mhz: f64,
    /// Acoustic dissipation and phonon loss rate in kHz (clamp 0.5 to 50.0, default 5.0).
    pub acoustic_loss_rate_khz: f64,
    /// Cryogenic dilution refrigerator operating temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Number of topological braiding junction nodes in the phononic network (clamp 3 to 12, default 4).
    pub braiding_nodes_count: usize,
}

impl Default for FloquetMajoranaBraidingProcessorParams {
    fn default() -> Self {
        Self {
            floquet_drive_freq_ghz: 8.0,
            floquet_modulation_amplitude_mhz: 60.0,
            synthetic_gauge_flux_rad: 1.5708,
            phononic_waveguide_length_um: 25.0,
            majorana_coupling_gap_mhz: 30.0,
            acoustic_loss_rate_khz: 5.0,
            operating_temp_m_k: 15.0,
            braiding_nodes_count: 4,
        }
    }
}

impl FloquetMajoranaBraidingProcessorParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        floquet_drive_freq_ghz: f64,
        floquet_modulation_amplitude_mhz: f64,
        synthetic_gauge_flux_rad: f64,
        phononic_waveguide_length_um: f64,
        majorana_coupling_gap_mhz: f64,
        acoustic_loss_rate_khz: f64,
        operating_temp_m_k: f64,
        braiding_nodes_count: usize,
    ) -> Self {
        Self {
            floquet_drive_freq_ghz: floquet_drive_freq_ghz.clamp(1.0, 20.0),
            floquet_modulation_amplitude_mhz: floquet_modulation_amplitude_mhz.clamp(10.0, 150.0),
            synthetic_gauge_flux_rad: synthetic_gauge_flux_rad.clamp(0.1, 3.14159),
            phononic_waveguide_length_um: phononic_waveguide_length_um.clamp(5.0, 100.0),
            majorana_coupling_gap_mhz: majorana_coupling_gap_mhz.clamp(5.0, 80.0),
            acoustic_loss_rate_khz: acoustic_loss_rate_khz.clamp(0.5, 50.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            braiding_nodes_count: braiding_nodes_count.clamp(3, 12),
        }
    }
}

/// Multi-physics evaluation metrics for topological phononic Floquet-Majorana braiding processors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetMajoranaBraidingProcessorMetrics {
    /// Floquet-Majorana adiabatic braiding gate fidelity (target >= 0.9980).
    pub braiding_gate_fidelity: f64,
    /// Dynamic topological protection gap isolating boundary Majorana modes in MHz (target >= 15.0).
    pub topological_protection_gap_mhz: f64,
    /// Adiabatic braiding operation cycle latency in ns (target <= 150.0).
    pub operation_latency_ns: f64,
    /// Continuous topological edge state isolation against bulk scattering in dB (target >= 40.0).
    pub edge_state_isolation_db: f64,
    /// Non-Abelian topological quantum state purity under thermal acoustic bath coupling (target >= 0.9950).
    pub non_abelian_state_purity: f64,
    /// Overall physical compliance flag across all roadmap performance targets.
    pub is_physically_compliant: bool,
}
