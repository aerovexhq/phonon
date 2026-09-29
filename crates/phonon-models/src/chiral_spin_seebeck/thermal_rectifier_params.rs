//! Topological acoustic heat rectifiers, thermal diodes,
//! and non-reciprocal phononic waveguides.

/// Parameters for multi-stage topological acoustic heat rectifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalThermalRectifierParams {
    /// Number of cascaded thermal diode stages $N$ (nominal $4 - 16$).
    pub diode_stages_count: usize,
    /// Single-stage forward acoustic phonon transmission $T_{\mathrm{fwd}}$ (nominal $0.80 - 0.96$).
    pub forward_phonon_transmission: f64,
    /// Single-stage backward acoustic phonon transmission $T_{\mathrm{bwd}}$ (nominal $0.05 - 0.20$).
    pub backward_phonon_transmission: f64,
    /// Characteristic chiral phonon center frequency in Terahertz ($\text{THz}$) (nominal $0.5 - 5.0\text{ THz}$).
    pub phonon_frequency_thz: f64,
    /// Structural lattice asymmetry factor $\eta_{\mathrm{asym}} \in [0.2, 0.9]$.
    pub lattice_asymmetry_factor: f64,
}

impl Default for TopologicalThermalRectifierParams {
    fn default() -> Self {
        Self {
            diode_stages_count: 6,
            forward_phonon_transmission: 0.90,
            backward_phonon_transmission: 0.12,
            phonon_frequency_thz: 1.5,
            lattice_asymmetry_factor: 0.65,
        }
    }
}

/// Evaluated metrics for topological thermal rectifiers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalRectifierMetrics {
    /// Overall cascaded thermal rectification ratio $\mathcal{R}_{\mathrm{th}} \ge 10.0\times$.
    pub rectification_ratio: f64,
    /// Directional thermal contrast in decibels $10 \log_{10}(\mathcal{R}_{\mathrm{th}})$.
    pub thermal_contrast_db: f64,
    /// Cascaded forward total transmission probability.
    pub forward_total_transmission: f64,
    /// Cascaded backward total transmission probability.
    pub backward_total_transmission: f64,
    /// Reverse thermal isolation in decibels.
    pub reverse_isolation_db: f64,
}

impl TopologicalThermalRectifierParams {
    /// Creates a new parameter set for topological thermal rectifiers.
    pub fn new(
        diode_stages_count: usize,
        forward_transmission: f64,
        backward_transmission: f64,
    ) -> Self {
        Self {
            diode_stages_count: diode_stages_count.max(1),
            forward_phonon_transmission: forward_transmission.clamp(0.1, 0.99),
            backward_phonon_transmission: backward_transmission.clamp(0.01, 0.5),
            ..Default::default()
        }
    }
}
