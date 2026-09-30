#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for quantum acoustic
//! higher-order topological quadrupole-octupole superlattices and non-Hermitian
//! corner metasurfaces.

/// Physical parameter configuration for quantum acoustic higher-order topological
/// quadrupole-octupole superlattices and non-Hermitian corner metasurfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotpQuadrupoleOctupoleParams {
    /// Intra-cell hopping amplitude gamma (clamp 0.10 to 0.90, default 0.35).
    pub intra_cell_hopping_gamma: f64,
    /// Inter-cell hopping amplitude lambda (clamp 0.80 to 2.50, default 1.45).
    pub inter_cell_hopping_lambda: f64,
    /// Non-Hermitian gain-loss rate gamma_NH (clamp 0.01 to 0.40, default 0.12).
    pub non_hermitian_gain_loss_gamma: f64,
    /// Acoustic corner resonance frequency in GHz (clamp 1.0 to 15.0, default 5.2).
    pub acoustic_corner_frequency_ghz: f64,
    /// Operating cryogenic temperature in milli-Kelvin (clamp 1.0 to 50.0, default 10.0).
    pub cryogenic_temperature_mk: f64,
    /// Multipole topological order: 2.0 for quadrupole, 3.0 for octupole (clamp 2.0 to 3.0, default 2.0).
    pub multipole_order: f64,
    /// Superlattice dimension cell count L (clamp 6.0 to 32.0, default 12.0).
    pub superlattice_dimension_cells: f64,
    /// Synthetic gauge flux per plaquette in units of pi (clamp 0.80 to 1.20, default 1.0).
    pub synthetic_gauge_flux_pi: f64,
}

impl Default for HotpQuadrupoleOctupoleParams {
    fn default() -> Self {
        Self {
            intra_cell_hopping_gamma: 0.35,
            inter_cell_hopping_lambda: 1.45,
            non_hermitian_gain_loss_gamma: 0.12,
            acoustic_corner_frequency_ghz: 5.2,
            cryogenic_temperature_mk: 10.0,
            multipole_order: 2.0,
            superlattice_dimension_cells: 12.0,
            synthetic_gauge_flux_pi: 1.0,
        }
    }
}

impl HotpQuadrupoleOctupoleParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        intra_cell_hopping_gamma: f64,
        inter_cell_hopping_lambda: f64,
        non_hermitian_gain_loss_gamma: f64,
        acoustic_corner_frequency_ghz: f64,
        cryogenic_temperature_mk: f64,
        multipole_order: f64,
        superlattice_dimension_cells: f64,
        synthetic_gauge_flux_pi: f64,
    ) -> Self {
        Self {
            intra_cell_hopping_gamma: intra_cell_hopping_gamma.clamp(0.10, 0.90),
            inter_cell_hopping_lambda: inter_cell_hopping_lambda.clamp(0.80, 2.50),
            non_hermitian_gain_loss_gamma: non_hermitian_gain_loss_gamma.clamp(0.01, 0.40),
            acoustic_corner_frequency_ghz: acoustic_corner_frequency_ghz.clamp(1.0, 15.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            multipole_order: multipole_order.clamp(2.0, 3.0),
            superlattice_dimension_cells: superlattice_dimension_cells.clamp(6.0, 32.0),
            synthetic_gauge_flux_pi: synthetic_gauge_flux_pi.clamp(0.80, 1.20),
        }
    }
}

/// Multi-physics evaluation metrics for quantum acoustic higher-order topological
/// quadrupole-octupole superlattices and non-Hermitian corner metasurfaces.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HotpQuadrupoleOctupoleMetrics {
    /// Corner state spatial localization fidelity (target >= 0.9980).
    pub corner_state_localization_fidelity: f64,
    /// Higher-order topological bulk and edge bandgap in MHz (target >= 45.0 MHz).
    pub higher_order_topological_gap_mhz: f64,
    /// Quantized multipole topological charge / polarization (target >= 0.990).
    pub multipole_topological_charge: f64,
    /// Corner-to-bulk crosstalk isolation in decibels (target >= 54.0 dB).
    pub corner_to_bulk_crosstalk_isolation_db: f64,
    /// Topological corner mode dephasing rate in Hz (target <= 15.0 Hz).
    pub topological_mode_dephasing_rate_hz: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
