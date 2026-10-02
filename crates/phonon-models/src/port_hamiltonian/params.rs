#![deny(unsafe_code)]

//! Physical parameters and multi-physics evaluation metrics for the Phonon
//! Port-Hamiltonian Fluid-Structure-Acoustic Speech Synthesis Engine.

/// Physical parameters for the Port-Hamiltonian Fluid-Structure-Acoustic model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortHamiltonianAcousticParams {
    /// Subglottal lung pressure in Pascals (clamp [200.0, 5000.0], default 1000.0 Pa).
    pub subglottal_pressure_pa: f64,
    /// Vocal fold effective tissue mass in kilograms (clamp [0.00005, 0.005], default 0.00025 kg).
    pub vocal_fold_mass_kg: f64,
    /// Vocal fold anterior-posterior length in meters (clamp [0.008, 0.025], default 0.015 m).
    pub vocal_fold_length_m: f64,
    /// Vocal fold vertical medial thickness in meters (clamp [0.001, 0.008], default 0.003 m).
    pub vocal_fold_thickness_m: f64,
    /// Superficial mucosal wave phase velocity in m/s (clamp [0.5, 3.0], default 1.07 m/s).
    pub mucosal_wave_velocity_m_s: f64,
    /// Supraglottal vocal tract centerline horn length in meters (clamp [0.10, 0.25], default 0.175 m).
    pub vocal_tract_length_m: f64,
    /// Radiating lip aperture radius in meters (clamp [0.002, 0.025], default 0.008 m).
    pub lip_aperture_radius_m: f64,
    /// Discrete audio synthesis sampling rate in Hertz (clamp [8000.0, 192000.0], default 48000.0 Hz).
    pub sampling_rate_hz: f64,
}

impl Default for PortHamiltonianAcousticParams {
    fn default() -> Self {
        Self {
            subglottal_pressure_pa: 1000.0,
            vocal_fold_mass_kg: 0.00025,
            vocal_fold_length_m: 0.015,
            vocal_fold_thickness_m: 0.003,
            mucosal_wave_velocity_m_s: 1.07,
            vocal_tract_length_m: 0.175,
            lip_aperture_radius_m: 0.008,
            sampling_rate_hz: 48000.0,
        }
    }
}

impl PortHamiltonianAcousticParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        subglottal_pressure_pa: f64,
        vocal_fold_mass_kg: f64,
        vocal_fold_length_m: f64,
        vocal_fold_thickness_m: f64,
        mucosal_wave_velocity_m_s: f64,
        vocal_tract_length_m: f64,
        lip_aperture_radius_m: f64,
        sampling_rate_hz: f64,
    ) -> Self {
        Self {
            subglottal_pressure_pa: subglottal_pressure_pa.clamp(200.0, 5000.0),
            vocal_fold_mass_kg: vocal_fold_mass_kg.clamp(0.00005, 0.005),
            vocal_fold_length_m: vocal_fold_length_m.clamp(0.008, 0.025),
            vocal_fold_thickness_m: vocal_fold_thickness_m.clamp(0.001, 0.008),
            mucosal_wave_velocity_m_s: mucosal_wave_velocity_m_s.clamp(0.5, 3.0),
            vocal_tract_length_m: vocal_tract_length_m.clamp(0.10, 0.25),
            lip_aperture_radius_m: lip_aperture_radius_m.clamp(0.002, 0.025),
            sampling_rate_hz: sampling_rate_hz.clamp(8000.0, 192000.0),
        }
    }

    /// Evaluates the vertical mucosal traveling wave delay tau_m = T_h / c_m in seconds.
    pub fn mucosal_wave_delay_s(&self) -> f64 {
        self.vocal_fold_thickness_m / self.mucosal_wave_velocity_m_s
    }
}

/// Evaluated multi-physics performance metrics of the Port-Hamiltonian acoustic system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PortHamiltonianAcousticMetrics {
    /// Extracted fundamental phonation frequency in Hertz.
    pub fundamental_frequency_hz: f64,
    /// Glottal open quotient (open duration / period).
    pub open_quotient: f64,
    /// Total stored Hamiltonian energy in Joules.
    pub total_hamiltonian_energy_joules: f64,
    /// Fractional energy drift relative to initial baseline.
    pub energy_drift_fraction: f64,
    /// Peak acoustic pressure at radiating lip boundary in Pascals.
    pub peak_lip_pressure_pa: f64,
    /// Numerical synthesis throughput in samples per second.
    pub synthesis_throughput_samples_per_sec: f64,
    /// Passivity verification flag indicating strict non-increasing unforced energy.
    pub is_physically_passive: bool,
}
