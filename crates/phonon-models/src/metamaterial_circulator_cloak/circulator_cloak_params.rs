//! Parameters and metrics for topological acoustic metamaterial circulators
//! and non-reciprocal acoustic cloaking with angular-momentum bias.

/// Parameters for angular-momentum-biased topological acoustic metamaterial devices.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetamaterialCirculatorCloakParams {
    /// Device acoustic operating frequency in $\text{kHz}$ (nominal $2.0 - 25.0\text{ kHz}$).
    pub operating_frequency_khz: f64,
    /// Inner radius of cloaking shell / core scatterer in millimeters (nominal $25.0 - 100.0\text{ mm}$).
    pub cloak_inner_radius_mm: f64,
    /// Outer radius of cloaking shell in millimeters (nominal $60.0 - 250.0\text{ mm}$).
    pub cloak_outer_radius_mm: f64,
    /// Non-reciprocal fluid/modulation bias velocity Mach number $M = v_0 / c_s \in [0.08, 0.35]$.
    pub fluid_bias_mach_number: f64,
    /// Loaded acoustic resonator quality factor $\mathcal{Q}$ (nominal $150.0 - 1500.0$).
    pub resonator_q_factor: f64,
    /// Ambient background acoustic phase velocity $c_s$ in $\text{m/s}$ (nominal $343.0\text{ m/s}$ in air, $1500.0\text{ m/s}$ in water).
    pub sound_speed_m_s: f64,
    /// Number of circulation ports (nominal 3 for 3-port Y-circulator).
    pub waveguide_ports_count: usize,
}

impl Default for MetamaterialCirculatorCloakParams {
    fn default() -> Self {
        Self {
            operating_frequency_khz: 8.5,
            cloak_inner_radius_mm: 45.0,
            cloak_outer_radius_mm: 110.0,
            fluid_bias_mach_number: 0.18,
            resonator_q_factor: 450.0,
            sound_speed_m_s: 343.0,
            waveguide_ports_count: 3,
        }
    }
}

/// Evaluated metrics for topological acoustic circulators and cloaks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MetamaterialCirculatorCloakMetrics {
    /// Acoustic scattering cross-section reduction (cloaking contrast) in decibels ($\ge 20.0\text{ dB}$).
    pub cloaking_cross_section_reduction_db: f64,
    /// Multi-port acoustic circulator reverse isolation in decibels ($\ge 25.0\text{ dB}$).
    pub circulator_isolation_db: f64,
    /// Forward port transmission insertion loss in decibels ($\le 1.5\text{ dB}$).
    pub forward_insertion_loss_db: f64,
    /// Topological chiral boundary mode transmission through sharp waveguide bends in percent ($\ge 90.0\%$).
    pub topological_bend_transmission_pct: f64,
    /// Linear forward-to-backward transmission contrast ratio ($\ge 100.0$).
    pub non_reciprocal_contrast_ratio: f64,
    /// Topological Chern invariant integer ($\pm 1$).
    pub topological_chern_number: i32,
}

impl MetamaterialCirculatorCloakParams {
    /// Creates a new parameter set for topological circulator and cloak metamaterials.
    pub fn new(freq_khz: f64, inner_r_mm: f64, outer_r_mm: f64, mach: f64) -> Self {
        Self {
            operating_frequency_khz: freq_khz.max(0.5),
            cloak_inner_radius_mm: inner_r_mm.max(10.0),
            cloak_outer_radius_mm: outer_r_mm.max(inner_r_mm * 1.2),
            fluid_bias_mach_number: mach.clamp(0.01, 0.50),
            ..Default::default()
        }
    }
}
