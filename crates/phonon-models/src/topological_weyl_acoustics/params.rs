//! Multi-physics parameter structures and metric definitions for
//! topological phononic Floquet Weyl semimetals and Fermi arc acoustics.

/// Physical parameter configuration for topological phononic Weyl semimetals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeylAcousticParams {
    /// 3D acoustic lattice constant $a$ in micrometers (default 450.0 um).
    pub lattice_constant_um: f64,
    /// Center acoustic frequency of the Weyl nodes $f_w$ in MHz (default 850.0 MHz).
    pub weyl_frequency_mhz: f64,
    /// Modulation amplitude breaking time-reversal symmetry $\delta_t$ (default 0.28).
    pub trs_breaking_modulation: f64,
    /// Spatial inversion symmetry breaking potential asymmetry $\delta_p$ (default 0.32).
    pub inversion_breaking_asymmetry: f64,
    /// Topological screw dislocation Burgers vector magnitude in lattice units (default 1.0).
    pub dislocation_burgers_vector: f64,
    /// Structural disorder disorder root-mean-square amplitude $\sigma_{\text{disorder}}$ (default 0.04).
    pub disorder_rms_amplitude: f64,
    /// Surface Fermi arc termination angle in degrees (default 0.0 deg for (001) surface).
    pub surface_termination_angle_deg: f64,
    /// Acoustic bulk quality factor $Q_{\text{bulk}}$ (default 1.2e4).
    pub acoustic_q_factor: f64,
    /// Ambient operating temperature in Kelvin (default 4.2 K).
    pub operating_temperature_k: f64,
}

impl Default for WeylAcousticParams {
    fn default() -> Self {
        Self {
            lattice_constant_um: 450.0,
            weyl_frequency_mhz: 850.0,
            trs_breaking_modulation: 0.28,
            inversion_breaking_asymmetry: 0.32,
            dislocation_burgers_vector: 1.0,
            disorder_rms_amplitude: 0.04,
            surface_termination_angle_deg: 0.0,
            acoustic_q_factor: 1.2e4,
            operating_temperature_k: 4.2,
        }
    }
}

impl WeylAcousticParams {
    /// Creates a new parameter configuration with bounds clamping.
    pub fn new(
        a_um: f64,
        f_w_mhz: f64,
        trs_mod: f64,
        inv_asym: f64,
        burgers: f64,
        disorder: f64,
        angle_deg: f64,
        q_bulk: f64,
        temp_k: f64,
    ) -> Self {
        Self {
            lattice_constant_um: a_um.clamp(10.0, 5000.0),
            weyl_frequency_mhz: f_w_mhz.clamp(10.0, 10000.0),
            trs_breaking_modulation: trs_mod.clamp(0.01, 1.0),
            inversion_breaking_asymmetry: inv_asym.clamp(0.01, 1.0),
            dislocation_burgers_vector: burgers.clamp(0.1, 5.0),
            disorder_rms_amplitude: disorder.clamp(0.0, 0.30),
            surface_termination_angle_deg: angle_deg.clamp(-90.0, 90.0),
            acoustic_q_factor: q_bulk.clamp(100.0, 1.0e7),
            operating_temperature_k: temp_k.clamp(0.01, 350.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological phononic Weyl semimetals.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeylAcousticMetrics {
    /// Normalized Weyl point separation distance $\Delta k / (\pi / a)$ (target >= 0.35).
    pub weyl_point_separation_norm: f64,
    /// Surface Fermi arc transmission efficiency $T_{\text{arc}}$ (target >= 94.0%).
    pub fermi_arc_transmission: f64,
    /// Screw dislocation topological mode purity $P_{\text{disloc}}$ (target >= 96.0%).
    pub dislocation_mode_purity: f64,
    /// Bulk bandgap acoustic isolation in dB (target >= 30.0 dB).
    pub bulk_bandgap_isolation_db: f64,
    /// Quantized topological chiral monopole charge magnitude $|C_w|$ (exact 1.0).
    pub chiral_monopole_charge: f64,
    /// Backscattering suppression ratio over disorder in dB.
    pub backscattering_suppression_db: f64,
    /// Physical compliance verification flag.
    pub is_physically_compliant: bool,
}
