/// Physical parameter configuration for topological Floquet-acoustic Chern insulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetAcousticChernParams {
    /// Acoustic phononic crystal lattice constant $a$ in micrometers (default 50.0 um).
    pub lattice_constant_um: f64,
    /// Base bulk acoustic wave velocity $v_{\\text{ac}}$ in m/s (default 3500.0 m/s).
    pub base_acoustic_velocity_m_s: f64,
    /// Acoustic center operating frequency $f_0$ in MHz (default 50.0 MHz).
    pub acoustic_center_freq_mhz: f64,
    /// High-frequency Floquet drive frequency $\\Omega / (2\\pi)$ in MHz (default 80.0 MHz).
    pub floquet_drive_freq_mhz: f64,
    /// Dynamic rotating acoustic strain field amplitude $\\epsilon_0$ (dimensionless, default 2.5e-4).
    pub dynamic_strain_amplitude: f64,
    /// Relative drive phase offset $\\phi_{\\text{drive}}$ in radians (default 0.0 rad).
    pub drive_phase_rad: f64,
    /// Intrinsic acoustic resonator quality factor $Q_{\\text{ac}}$ (default 5000.0).
    pub quality_factor: f64,
    /// Waveguide corner bend angle $\\theta_{\\text{bend}}$ in degrees (default 60.0 deg).
    pub bend_angle_deg: f64,
    /// Waveguide propagation length $L$ in micrometers (default 500.0 um).
    pub waveguide_length_um: f64,
}

impl Default for FloquetAcousticChernParams {
    fn default() -> Self {
        Self {
            lattice_constant_um: 50.0,
            base_acoustic_velocity_m_s: 3500.0,
            acoustic_center_freq_mhz: 50.0,
            floquet_drive_freq_mhz: 80.0,
            dynamic_strain_amplitude: 2.5e-4,
            drive_phase_rad: 0.0,
            quality_factor: 5000.0,
            bend_angle_deg: 60.0,
            waveguide_length_um: 500.0,
        }
    }
}

impl FloquetAcousticChernParams {
    /// Creates a new parameter configuration for Floquet-acoustic Chern insulators.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        lattice_um: f64,
        v_ac: f64,
        f0_mhz: f64,
        f_drive_mhz: f64,
        strain_amp: f64,
        phase_rad: f64,
        q_factor: f64,
        bend_deg: f64,
        length_um: f64,
    ) -> Self {
        Self {
            lattice_constant_um: lattice_um.clamp(1.0, 500.0),
            base_acoustic_velocity_m_s: v_ac.clamp(500.0, 12000.0),
            acoustic_center_freq_mhz: f0_mhz.clamp(1.0, 1000.0),
            floquet_drive_freq_mhz: f_drive_mhz.clamp(5.0, 2000.0),
            dynamic_strain_amplitude: strain_amp.clamp(1e-6, 1e-2),
            drive_phase_rad: phase_rad,
            quality_factor: q_factor.clamp(100.0, 1e7),
            bend_angle_deg: bend_deg.clamp(10.0, 170.0),
            waveguide_length_um: length_um.clamp(10.0, 10000.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological Floquet-acoustic Chern insulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloquetAcousticChernMetrics {
    /// Quantized acoustic Chern number $|C| = 1.0$.
    pub chern_number: f64,
    /// Topological minigap $\\Delta_{\\text{gap}}$ opened at Dirac cones in MHz (target >= 2.5 MHz).
    pub topological_minigap_mhz: f64,
    /// Forward sharp-bend transmission efficiency $T_{\\text{bend}}$ in % (target >= 92.0%).
    pub forward_bend_efficiency_pct: f64,
    /// Reverse backscattering isolation $\\mathcal{I}_{\\text{rev}}$ in dB (target >= 30.0 dB).
    pub reverse_isolation_db: f64,
    /// Chiral edge state group velocity $v_{\\text{edge}}$ in m/s.
    pub chiral_edge_velocity_m_s: f64,
    /// Non-reciprocal beam-steering angle $\\theta_{\\text{steer}}$ in degrees.
    pub beam_steering_angle_deg: f64,
    /// Physical compliance flag verifying all engineering thresholds are fulfilled.
    pub is_physically_compliant: bool,
}
