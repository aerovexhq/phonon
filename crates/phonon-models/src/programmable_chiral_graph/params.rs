#![deny(unsafe_code)]

//! Parameter configurations and multi-physics evaluation metrics for
//! programmable chiral phonon networks and high-dimensional quantum acoustic graph states.

/// Physical parameter configuration for programmable chiral phonon networks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgrammableChiralGraphParams {
    /// Number of coupled phononic resonator nodes in the graph network (default 64).
    pub network_nodes_count: usize,
    /// Acoustic center resonance frequency in gigahertz (default 3.6 GHz).
    pub acoustic_resonance_ghz: f64,
    /// Initial phononic state quadrature squeezing in decibels (default 12.0 dB).
    pub initial_squeezing_db: f64,
    /// Inter-site chiral phononic hopping coupling rate in megahertz (default 28.0 MHz).
    pub inter_site_coupling_mhz: f64,
    /// Voltage-controlled phase shifter reconfigurable switching time in nanoseconds (default 12.5 ns).
    pub phase_shifter_switching_time_ns: f64,
    /// Chiral topological edge channel non-reciprocal isolation in decibels (default 38.0 dB).
    pub chiral_isolation_db: f64,
    /// Operating cryostat ambient temperature in milli-Kelvin (default 15.0 mK).
    pub operating_temp_m_k: f64,
    /// Topological chiral waveguide propagation loss in dB/cm (default 0.025 dB/cm).
    pub waveguide_propagation_loss_db_per_cm: f64,
}

impl Default for ProgrammableChiralGraphParams {
    fn default() -> Self {
        Self {
            network_nodes_count: 64,
            acoustic_resonance_ghz: 3.6,
            initial_squeezing_db: 12.0,
            inter_site_coupling_mhz: 28.0,
            phase_shifter_switching_time_ns: 12.5,
            chiral_isolation_db: 38.0,
            operating_temp_m_k: 15.0,
            waveguide_propagation_loss_db_per_cm: 0.025,
        }
    }
}

impl ProgrammableChiralGraphParams {
    /// Creates a new parameter configuration with physical boundary clamping.
    pub fn new(
        network_nodes_count: usize,
        acoustic_resonance_ghz: f64,
        initial_squeezing_db: f64,
        inter_site_coupling_mhz: f64,
        phase_shifter_switching_time_ns: f64,
        chiral_isolation_db: f64,
        operating_temp_m_k: f64,
        waveguide_propagation_loss_db_per_cm: f64,
    ) -> Self {
        Self {
            network_nodes_count: network_nodes_count.clamp(16, 256),
            acoustic_resonance_ghz: acoustic_resonance_ghz.clamp(0.5, 15.0),
            initial_squeezing_db: initial_squeezing_db.clamp(3.0, 18.0),
            inter_site_coupling_mhz: inter_site_coupling_mhz.clamp(5.0, 100.0),
            phase_shifter_switching_time_ns: phase_shifter_switching_time_ns.clamp(1.0, 50.0),
            chiral_isolation_db: chiral_isolation_db.clamp(20.0, 60.0),
            operating_temp_m_k: operating_temp_m_k.clamp(1.0, 100.0),
            waveguide_propagation_loss_db_per_cm: waveguide_propagation_loss_db_per_cm.clamp(0.005, 0.10),
        }
    }
}

/// Multi-physics evaluation metrics for high-dimensional quantum acoustic graph states.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgrammableChiralGraphMetrics {
    /// Multi-partite quantum acoustic graph state entanglement fidelity (target >= 0.940).
    pub graph_entanglement_fidelity: f64,
    /// Topological chiral edge routing channel purity (target >= 0.960).
    pub topological_edge_purity: f64,
    /// Scalable network graph nodes count (target >= 64).
    pub network_nodes_count: usize,
    /// Reconfigurable phase switching time in nanoseconds (target <= 20.0 ns).
    pub switching_time_ns: f64,
    /// Continuous-variable cluster state nullifier variance in decibels (target <= -4.5 dB).
    pub nullifier_variance_db: f64,
    /// Multi-partite quantum acoustic stabilizer generator fidelity (target >= 0.950).
    pub stabilizer_generator_fidelity: f64,
    /// Overall physical compliance flag across all design thresholds.
    pub is_physically_compliant: bool,
}
