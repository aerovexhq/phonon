#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for Floquet-Bloch
//! synthetic gauge acoustic fields and dynamically reconfigurable phononic quantum simulators.

/// Physical parameter configuration for Floquet-Bloch synthetic gauge phononic lattices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetSyntheticGaugeParams {
    /// Acoustic center frequency in GHz (clamp 1.0 to 12.0, default 4.6).
    pub acoustic_center_freq_ghz: f64,
    /// Floquet parametric drive frequency in MHz (clamp 10.0 to 200.0, default 80.0).
    pub floquet_drive_freq_mhz: f64,
    /// Parametric modulation depth (clamp 0.05 to 0.60, default 0.28).
    pub parametric_modulation_depth: f64,
    /// Lattice plaquette count (clamp 4 to 64, default 16).
    pub lattice_plaquette_count: usize,
    /// Synthetic phase gradient per unit cell in radians (clamp 0.2 to 3.14159, default 1.5708).
    pub synthetic_phase_gradient_rad: f64,
    /// Inter-site acoustic coupling rate in MHz (clamp 5.0 to 60.0, default 25.0).
    pub inter_site_coupling_mhz: f64,
    /// Operating cryostat temperature in millikelvin (clamp 1.0 to 50.0, default 15.0).
    pub operating_temp_m_k: f64,
    /// Acoustic phonon damping rate in kHz (clamp 0.5 to 50.0, default 5.0).
    pub acoustic_damping_rate_khz: f64,
}

impl Default for FloquetSyntheticGaugeParams {
    fn default() -> Self {
        Self {
            acoustic_center_freq_ghz: 4.6,
            floquet_drive_freq_mhz: 80.0,
            parametric_modulation_depth: 0.28,
            lattice_plaquette_count: 16,
            synthetic_phase_gradient_rad: 1.5708,
            inter_site_coupling_mhz: 25.0,
            operating_temp_m_k: 15.0,
            acoustic_damping_rate_khz: 5.0,
        }
    }
}

impl FloquetSyntheticGaugeParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        acoustic_center_freq_ghz: f64,
        floquet_drive_freq_mhz: f64,
        parametric_modulation_depth: f64,
        lattice_plaquette_count: usize,
        synthetic_phase_gradient_rad: f64,
        inter_site_coupling_mhz: f64,
        operating_temp_m_k: f64,
        acoustic_damping_rate_khz: f64,
    ) -> Self {
        Self {
            acoustic_center_freq_ghz: acoustic_center_freq_ghz.clamp(1.0, 12.0),
            floquet_drive_freq_mhz: floquet_drive_freq_mhz.clamp(10.0, 200.0),
            parametric_modulation_depth: parametric_modulation_depth.clamp(0.05, 0.60),
            lattice_plaquette_count: lattice_plaquette_count.clamp(4, 64),
            synthetic_phase_gradient_rad: synthetic_phase_gradient_rad.clamp(0.2, 3.14159),
            inter_site_coupling_mhz: inter_site_coupling_mhz.clamp(5.0, 60.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 50.0),
            acoustic_damping_rate_khz: acoustic_damping_rate_khz.clamp(0.5, 50.0),
        }
    }
}

/// Multi-physics performance evaluation metrics for Floquet-Bloch synthetic gauge fields.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetSyntheticGaugeMetrics {
    /// Dynamical state transfer fidelity (target >= 0.9950).
    pub dynamical_state_fidelity: f64,
    /// Synthetic magnetic flux ratio per plaquette Phi / Phi_0 (target >= 0.500).
    pub synthetic_magnetic_flux_ratio: f64,
    /// Synthetic flux quantization error delta Phi (target <= 0.010).
    pub flux_quantization_error: f64,
    /// Dynamic Chern invariant switching time in nanoseconds (target <= 20.0).
    pub chern_switching_time_ns: f64,
    /// Topological band isolation gap in dB (target >= 30.0).
    pub topological_band_isolation_db: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
