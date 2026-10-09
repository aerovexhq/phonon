#![deny(unsafe_code)]

pub mod valley_qubit;
pub mod flatband_memory_cell;
pub mod cryogenic_valley_bus;

pub use valley_qubit::{
    MoireValleyQubitParams, ValleyPseudospinState, ValleyBlochVector,
    ValleyRabiPoint, ValleyQubitMetrics, ValleyQubitEngine,
};
pub use flatband_memory_cell::{
    FlatBandMemoryParams, FlatBandDispersionPoint, MemoryDecayPoint,
    MemoryStorageMetrics, PhononMemoryCell,
};
pub use cryogenic_valley_bus::{
    ValleyBusParams, BusSpectrumPoint, BusEntanglementMetrics,
    ValleyBusRoutingMetrics, MultiNodeValleyBus,
};

/// 10-point comprehensive physics audit report for the moire valley qubit and memory bus system.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireValleyQubitAuditReport {
    /// 1. Magic-angle Dirac velocity quenching: v_g / v_0 <= 0.015 and bandwidth <= 0.50 MHz.
    pub magic_angle_quenching_pass: bool,
    /// 2. AA-stacking site acoustic energy confinement: eta_AA >= 90.0%.
    pub aa_site_confinement_pass: bool,
    /// 3. Valley pseudospin coherence: T2* >= 100.0 us and gate fidelity >= 99.9%.
    pub valley_coherence_pass: bool,
    /// 4. Higher moire sub-band leakage suppression: P_leak <= 1.0e-4.
    pub valley_leakage_pass: bool,
    /// 5. Flat-band memory storage lifetime: tau_store >= 1.5 ms and Q >= 100,000.
    pub storage_lifetime_pass: bool,
    /// 6. Phonon write/retrieval efficiency: eta_retrieve >= 92.0%.
    pub retrieval_efficiency_pass: bool,
    /// 7. Dilution refrigerator thermal phonon occupancy: n_th <= 1.0e-4 at 15 mK.
    pub thermal_occupancy_pass: bool,
    /// 8. Valley-momentum locked bus chiral reverse isolation: ISO >= 40.0 dB.
    pub valley_chiral_isolation_pass: bool,
    /// 9. Multi-node end-to-end bus insertion loss: IL <= 0.35 dB.
    pub bus_insertion_loss_pass: bool,
    /// 10. Multi-node entanglement routing concurrence: C >= 0.90.
    pub multi_node_concurrence_pass: bool,
    /// Total passed audit criteria (out of 10).
    pub passed_count: usize,
    /// Total evaluated audit criteria (10).
    pub total_count: usize,
}

impl MoireValleyQubitAuditReport {
    /// Check if all 10 physics audit criteria passed.
    pub fn is_all_pass(&self) -> bool {
        self.passed_count == self.total_count && self.total_count == 10
    }
}

/// Unified coordinator for twisted-bilayer acoustic moire valley qubits and memory bus.
#[derive(Debug, Clone)]
pub struct MoireValleyQubitProcessor {
    /// Valley qubit simulation engine.
    pub qubit_engine: ValleyQubitEngine,
    /// Flat-band phonon memory cell solver.
    pub memory_cell: PhononMemoryCell,
    /// Cryogenic multi-node valley bus solver.
    pub valley_bus: MultiNodeValleyBus,
}

impl Default for MoireValleyQubitProcessor {
    fn default() -> Self {
        Self::new(
            MoireValleyQubitParams::default(),
            FlatBandMemoryParams::default(),
            ValleyBusParams::default(),
        )
    }
}

impl MoireValleyQubitProcessor {
    /// Create a new processor with the specified subsystem parameters.
    pub fn new(
        qubit_params: MoireValleyQubitParams,
        memory_params: FlatBandMemoryParams,
        bus_params: ValleyBusParams,
    ) -> Self {
        Self {
            qubit_engine: ValleyQubitEngine::new(qubit_params),
            memory_cell: PhononMemoryCell::new(memory_params),
            valley_bus: MultiNodeValleyBus::new(bus_params),
        }
    }

    /// Perform a rigorous 10-point physics audit.
    pub fn audit(&self) -> MoireValleyQubitAuditReport {
        let q_metrics = self.qubit_engine.compute_metrics();
        let m_metrics = self.memory_cell.compute_metrics();
        let b_metrics = self.valley_bus.compute_metrics();

        // 1. Magic-angle Dirac velocity quenching
        let magic_angle_quenching_pass = m_metrics.group_velocity_ratio <= 0.015
            && m_metrics.flatband_bandwidth_mhz <= 0.50;

        // 2. AA-stacking site acoustic energy confinement
        let aa_site_confinement_pass = m_metrics.aa_confinement_ratio >= 0.90;

        // 3. Valley pseudospin coherence
        let valley_coherence_pass = q_metrics.coherence_time_t2_star_us >= 100.0
            && q_metrics.gate_fidelity >= 0.999;

        // 4. Higher moire sub-band leakage suppression
        let valley_leakage_pass = q_metrics.leakage_probability <= 1.0e-4;

        // 5. Flat-band memory storage lifetime
        let storage_lifetime_pass = m_metrics.storage_lifetime_ms >= 1.5
            && self.memory_cell.params().quality_factor >= 100_000.0;

        // 6. Phonon write/retrieval efficiency
        let retrieval_efficiency_pass = m_metrics.retrieval_efficiency >= 0.92;

        // 7. Dilution refrigerator thermal phonon occupancy
        let thermal_occupancy_pass = m_metrics.thermal_phonon_occupancy <= 1.0e-4;

        // 8. Valley-momentum locked bus chiral reverse isolation
        let valley_chiral_isolation_pass = b_metrics.reverse_isolation_db >= 40.0;

        // 9. Multi-node end-to-end bus insertion loss
        let bus_insertion_loss_pass = b_metrics.end_to_end_insertion_loss_db <= 0.35;

        // 10. Multi-node entanglement routing concurrence
        let multi_node_concurrence_pass = b_metrics.entanglement.concurrence >= 0.90;

        let checks = [
            magic_angle_quenching_pass,
            aa_site_confinement_pass,
            valley_coherence_pass,
            valley_leakage_pass,
            storage_lifetime_pass,
            retrieval_efficiency_pass,
            thermal_occupancy_pass,
            valley_chiral_isolation_pass,
            bus_insertion_loss_pass,
            multi_node_concurrence_pass,
        ];

        let passed_count = checks.iter().filter(|&&c| c).count();

        MoireValleyQubitAuditReport {
            magic_angle_quenching_pass,
            aa_site_confinement_pass,
            valley_coherence_pass,
            valley_leakage_pass,
            storage_lifetime_pass,
            retrieval_efficiency_pass,
            thermal_occupancy_pass,
            valley_chiral_isolation_pass,
            bus_insertion_loss_pass,
            multi_node_concurrence_pass,
            passed_count,
            total_count: 10,
        }
    }
}
