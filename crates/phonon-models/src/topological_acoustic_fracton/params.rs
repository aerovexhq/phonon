#![deny(unsafe_code)]

//! Physical parameter models and multi-physics evaluation metrics for topological acoustic
//! fracton dynamics and sub-system symmetry-protected phononic multipole routers.

/// Physical parameter configuration for topological acoustic fracton dynamics
/// and sub-system symmetry-protected phononic multipole routers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalAcousticFractonParams {
    /// Higher-rank tensor gauge coupling constant g (clamp 0.10 to 5.0, default 1.45).
    pub higher_rank_gauge_coupling_g: f64,
    /// Sub-dimensional phononic crystal lattice constant in nanometers (clamp 50.0 to 500.0, default 160.0).
    pub sub_dimensional_lattice_constant_nm: f64,
    /// Operating acoustic phonon driving frequency in GHz (clamp 1.0 to 15.0, default 4.8).
    pub acoustic_phonon_frequency_ghz: f64,
    /// Multipole conservation moment order (1.0 = dipole, 2.0 = quadrupole, clamp 1.0 to 4.0, default 2.0).
    pub multipole_moment_order: f64,
    /// Operating cryogenic temperature in milli-Kelvin (clamp 1.0 to 50.0, default 15.0).
    pub cryogenic_temperature_mk: f64,
    /// Number of sub-system symmetry protecting phononic lattice layers (clamp 4.0 to 64.0, default 24.0).
    pub sub_system_layer_count: f64,
    /// Fracton pinning potential depth in meV (clamp 0.5 to 20.0, default 6.8).
    pub fracton_pinning_potential_mev: f64,
    /// Spatial inter-router separation distance in micrometers (clamp 0.5 to 12.0, default 3.2).
    pub inter_router_separation_um: f64,
}

impl Default for TopologicalAcousticFractonParams {
    fn default() -> Self {
        Self {
            higher_rank_gauge_coupling_g: 1.45,
            sub_dimensional_lattice_constant_nm: 160.0,
            acoustic_phonon_frequency_ghz: 4.8,
            multipole_moment_order: 2.0,
            cryogenic_temperature_mk: 15.0,
            sub_system_layer_count: 24.0,
            fracton_pinning_potential_mev: 6.8,
            inter_router_separation_um: 3.2,
        }
    }
}

impl TopologicalAcousticFractonParams {
    /// Creates a new parameter configuration with strict physical boundary clamping.
    pub fn new(
        higher_rank_gauge_coupling_g: f64,
        sub_dimensional_lattice_constant_nm: f64,
        acoustic_phonon_frequency_ghz: f64,
        multipole_moment_order: f64,
        cryogenic_temperature_mk: f64,
        sub_system_layer_count: f64,
        fracton_pinning_potential_mev: f64,
        inter_router_separation_um: f64,
    ) -> Self {
        Self {
            higher_rank_gauge_coupling_g: higher_rank_gauge_coupling_g.clamp(0.10, 5.0),
            sub_dimensional_lattice_constant_nm: sub_dimensional_lattice_constant_nm
                .clamp(50.0, 500.0),
            acoustic_phonon_frequency_ghz: acoustic_phonon_frequency_ghz.clamp(1.0, 15.0),
            multipole_moment_order: multipole_moment_order.clamp(1.0, 4.0),
            cryogenic_temperature_mk: cryogenic_temperature_mk.clamp(1.0, 50.0),
            sub_system_layer_count: sub_system_layer_count.clamp(4.0, 64.0),
            fracton_pinning_potential_mev: fracton_pinning_potential_mev.clamp(0.5, 20.0),
            inter_router_separation_um: inter_router_separation_um.clamp(0.5, 12.0),
        }
    }
}

/// Multi-physics evaluation metrics for topological acoustic fracton dynamics
/// and sub-system symmetry-protected phononic multipole routers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalAcousticFractonMetrics {
    /// Immobile fracton acoustic confinement state fidelity (target >= 0.9970).
    pub fracton_confinement_fidelity: f64,
    /// Sub-dimensional edge channel isolation in dB (target >= 50.0 dB).
    pub sub_dimensional_edge_channel_isolation_db: f64,
    /// Multipole charge conservation relative numerical error (target <= 1.0e-5).
    pub multipole_charge_conservation_error: f64,
    /// Thermal fracton diffusion and dephasing rate in Hz (target <= 25.0 Hz).
    pub fracton_diffusion_dephasing_rate_hz: f64,
    /// Sub-system symmetry boundary mode purity ratio (target >= 0.990).
    pub sub_system_boundary_mode_purity: f64,
    /// Overall physical compliance flag across all roadmap design criteria.
    pub is_physically_compliant: bool,
}
