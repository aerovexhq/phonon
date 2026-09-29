//! Parameter configurations and multi-physics evaluation metrics for
//! quantum acoustic topological Chern insulators and chiral phonon diode circulators.

/// Physical parameter configuration for quantum acoustic topological Chern circulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalChernCirculatorParams {
    /// Unit cell acoustic lattice constant in micrometers (default 2.2 um).
    pub lattice_constant_um: f64,
    /// Operating acoustic center frequency in gigahertz (default 4.5 GHz).
    pub center_frequency_ghz: f64,
    /// Dynamic Coriolis modulation or synthetic angular momentum in megahertz (default 85.0 MHz).
    pub synthetic_angular_momentum_mhz: f64,
    /// Inter-site acoustic coupling rate in megahertz (default 40.0 MHz).
    pub inter_site_coupling_mhz: f64,
    /// Random lattice defect or disorder fraction (default 0.05).
    pub defect_disorder_fraction: f64,
    /// Operating cryostat temperature in milli-Kelvin (default 20.0 mK).
    pub operating_temp_m_k: f64,
    /// Number of circulator input/output acoustic waveguide ports (default 3).
    pub circulator_ports_count: usize,
    /// Intrinsic acoustic resonator quality factor (default 5.0e5).
    pub acoustic_intrinsic_q: f64,
}

impl Default for TopologicalChernCirculatorParams {
    fn default() -> Self {
        Self {
            lattice_constant_um: 2.2,
            center_frequency_ghz: 4.5,
            synthetic_angular_momentum_mhz: 85.0,
            inter_site_coupling_mhz: 40.0,
            defect_disorder_fraction: 0.05,
            operating_temp_m_k: 20.0,
            circulator_ports_count: 3,
            acoustic_intrinsic_q: 5.0e5,
        }
    }
}

impl TopologicalChernCirculatorParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        lattice_constant_um: f64,
        center_frequency_ghz: f64,
        synthetic_angular_momentum_mhz: f64,
        inter_site_coupling_mhz: f64,
        defect_disorder_fraction: f64,
        operating_temp_m_k: f64,
        circulator_ports_count: usize,
        acoustic_intrinsic_q: f64,
    ) -> Self {
        Self {
            lattice_constant_um: lattice_constant_um.clamp(0.5, 20.0),
            center_frequency_ghz: center_frequency_ghz.clamp(0.5, 15.0),
            synthetic_angular_momentum_mhz: synthetic_angular_momentum_mhz.clamp(10.0, 300.0),
            inter_site_coupling_mhz: inter_site_coupling_mhz.clamp(5.0, 150.0),
            defect_disorder_fraction: defect_disorder_fraction.clamp(0.0, 0.30),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 500.0),
            circulator_ports_count: circulator_ports_count.clamp(3, 8),
            acoustic_intrinsic_q: acoustic_intrinsic_q.clamp(1.0e4, 1.0e7),
        }
    }
}

/// Multi-physics evaluation metrics for topological Chern acoustic circulators.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalChernCirculatorMetrics {
    /// Forward acoustic transmission through chiral edge channels (target >= 0.950).
    pub forward_transmission: f64,
    /// Non-reciprocal backward acoustic isolation in decibels (target >= 35.0 dB).
    pub non_reciprocal_isolation_db: f64,
    /// Normalized topological bandgap ratio Delta omega / omega_0 (target >= 0.120).
    pub topological_bandgap_ratio: f64,
    /// Backscattering reflection at structural defect sites in decibels (target <= -40.0 dB).
    pub backscattering_reflection_db: f64,
    /// Acoustic insertion loss through the circulator waveguide in decibels (target <= 0.80 dB).
    pub insertion_loss_db: f64,
    /// Quantized first Chern number of the acoustic bulk band structure (target 1).
    pub topological_chern_number: i32,
    /// Physical compliance verification flag.
    pub is_physically_compliant: bool,
}
