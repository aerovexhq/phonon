#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for quantum acoustic
//! metasurface holography and chiral phonon beamforming arrays.

/// Physical parameter configuration for quantum acoustic metasurface holography
/// and chiral beamforming arrays.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralHolographicBeamformingParams {
    /// Number of active metasurface elements (clamp 16 to 128, default 48).
    pub metasurface_elements_count: usize,
    /// Element spacing in micrometers (clamp 0.20 to 5.0, default 0.85).
    pub element_spacing_um: f64,
    /// Operating acoustic frequency in GHz (clamp 1.0 to 10.0, default 3.8).
    pub operating_frequency_ghz: f64,
    /// Synthetic gauge phase gradient in radians per micrometer (clamp 0.5 to 8.0, default 3.2).
    pub synthetic_gauge_phase_gradient_rad_per_um: f64,
    /// Piezoelectric electromechanical coupling efficiency (clamp 0.50 to 0.99, default 0.91).
    pub piezoelectric_coupling_efficiency: f64,
    /// Sub-diffraction focusing enhancement ratio (clamp 1.10 to 3.50, default 2.10).
    pub sub_diffraction_focusing_ratio: f64,
    /// Cryogenic operating temperature in millikelvin (clamp 5.0 to 50.0, default 20.0).
    pub cryogenic_temperature_mk: f64,
    /// Chiral phonon isolation in dB (clamp 20.0 to 60.0, default 36.0).
    pub chiral_isolation_db: f64,
}

impl Default for ChiralHolographicBeamformingParams {
    fn default() -> Self {
        Self {
            metasurface_elements_count: 48,
            element_spacing_um: 0.85,
            operating_frequency_ghz: 3.8,
            synthetic_gauge_phase_gradient_rad_per_um: 3.2,
            piezoelectric_coupling_efficiency: 0.91,
            sub_diffraction_focusing_ratio: 2.10,
            cryogenic_temperature_mk: 20.0,
            chiral_isolation_db: 36.0,
        }
    }
}

impl ChiralHolographicBeamformingParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        metasurface_elements_count: usize,
        element_spacing_um: f64,
        operating_frequency_ghz: f64,
        synthetic_gauge_phase_gradient_rad_per_um: f64,
        piezoelectric_coupling_efficiency: f64,
        sub_diffraction_focusing_ratio: f64,
        cryogenic_temperature_mk: f64,
        chiral_isolation_db: f64,
    ) -> Self {
        Self {
            metasurface_elements_count: metasurface_elements_count.clamp(16, 128),
            element_spacing_um: element_spacing_um.clamp(0.20, 5.0),
            operating_frequency_ghz: operating_frequency_ghz.clamp(1.0, 10.0),
            synthetic_gauge_phase_gradient_rad_per_um: synthetic_gauge_phase_gradient_rad_per_um
                .clamp(0.5, 8.0),
            piezoelectric_coupling_efficiency: piezoelectric_coupling_efficiency.clamp(0.50, 0.99),
            sub_diffraction_focusing_ratio: sub_diffraction_focusing_ratio.clamp(1.10, 3.50),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(5.0, 50.0),
            chiral_isolation_db: chiral_isolation_db.clamp(20.0, 60.0),
        }
    }
}

/// Multi-physics performance evaluation metrics for quantum acoustic metasurface holography.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChiralHolographicBeamformingMetrics {
    /// Holographic target wavefront reconstruction fidelity (target >= 0.9960).
    pub holographic_reconstruction_fidelity: f64,
    /// Main acoustic beam directivity in dB (target >= 32.0).
    pub acoustic_beam_directivity_db: f64,
    /// Beam steering angular resolution in degrees (target <= 0.050).
    pub beam_steering_angular_resolution_deg: f64,
    /// Side-lobe suppression ratio in dB (target >= 28.0).
    pub side_lobe_suppression_ratio_db: f64,
    /// Total acoustic mode insertion loss in dB (target <= 1.20).
    pub acoustic_mode_insertion_loss_db: f64,
    /// Overall physical compliance flag across all roadmap targets.
    pub is_physically_compliant: bool,
}
