#![deny(unsafe_code)]

//! Phase 444: Topological Acoustic Boundary-Mode Valley-Hall Quantum Router & Entanglement Concentrator.
//!
//! Master module integrating topological acoustic valley-Hall phononic lattices,
//! dynamic multi-channel valley pseudospin quantum routers, and non-linear boundary
//! entanglement concentration with cryogenic dispersive readout.

pub mod valley_hall_lattice;
pub mod valley_quantum_router;
pub mod entanglement_concentrator;

pub use valley_hall_lattice::{
    ValleyHallLatticeMetrics, ValleyHallLatticeParams, ValleyHallLatticeSolver,
    ValleyRouterDispersionPoint, ValleySpatialFieldPoint,
};
pub use valley_quantum_router::{
    RouterChannelTarget, RouterDynamicTracePoint, RouterSpectralPoint,
    ValleyQuantumRouterMetrics, ValleyQuantumRouterParams, ValleyQuantumRouterSolver,
};
pub use entanglement_concentrator::{
    BellDensityMatrix, CavityReadoutSpectrumPoint, DistillationYieldPoint,
    EntanglementConcentratorMetrics, EntanglementConcentratorParams,
    EntanglementConcentratorSolver,
};

/// 10-Point rigorous physical audit report for the topological valley quantum system.
#[derive(Debug, Clone)]
pub struct ValleyRouterAuditReport {
    pub valley_bulk_gap_pass: bool,
    pub valley_chern_difference_pass: bool,
    pub edge_group_velocity_pass: bool,
    pub corner_transmission_pass: bool,
    pub crosstalk_isolation_pass: bool,
    pub target_transmission_pass: bool,
    pub switching_latency_pass: bool,
    pub concentrated_concurrence_pass: bool,
    pub bell_fidelity_pass: bool,
    pub dispersive_readout_snr_pass: bool,

    pub total_score: usize,
    pub all_passed: bool,
}

/// Unified master orchestrator for Phase 444.
#[derive(Debug, Clone)]
pub struct TopologicalValleyQuantumProcessor {
    pub lattice_solver: ValleyHallLatticeSolver,
    pub router_solver: ValleyQuantumRouterSolver,
    pub concentrator_solver: EntanglementConcentratorSolver,
}

impl Default for TopologicalValleyQuantumProcessor {
    fn default() -> Self {
        Self {
            lattice_solver: ValleyHallLatticeSolver::new(ValleyHallLatticeParams::default()),
            router_solver: ValleyQuantumRouterSolver::new(ValleyQuantumRouterParams::default()),
            concentrator_solver: EntanglementConcentratorSolver::new(
                EntanglementConcentratorParams::default(),
            ),
        }
    }
}

impl TopologicalValleyQuantumProcessor {
    pub fn new(
        lattice_params: ValleyHallLatticeParams,
        router_params: ValleyQuantumRouterParams,
        concentrator_params: EntanglementConcentratorParams,
    ) -> Self {
        Self {
            lattice_solver: ValleyHallLatticeSolver::new(lattice_params),
            router_solver: ValleyQuantumRouterSolver::new(router_params),
            concentrator_solver: EntanglementConcentratorSolver::new(concentrator_params),
        }
    }

    /// Executes the 10-point physics audit checklist.
    pub fn audit_system(&self) -> ValleyRouterAuditReport {
        let lm = self.lattice_solver.evaluate_metrics();
        let rm = self.router_solver.evaluate_metrics();
        let cm = self.concentrator_solver.evaluate_metrics();

        let valley_bulk_gap_pass = lm.valley_bulk_gap_mhz >= 3.0;
        let valley_chern_difference_pass = (lm.valley_chern_difference - 1.0).abs() < 1e-4;
        let edge_group_velocity_pass = lm.edge_group_velocity_ms >= 1200.0;
        let corner_transmission_pass = lm.corner_transmission_ratio >= 0.95;

        let crosstalk_isolation_pass = rm.crosstalk_isolation_db >= 35.0;
        let target_transmission_pass = rm.target_transmission_percent >= 90.0;
        let switching_latency_pass = rm.switching_latency_ns <= 3.0;

        let concentrated_concurrence_pass = cm.concentrated_concurrence >= 0.96;
        let bell_fidelity_pass = cm.bell_state_fidelity >= 0.995;
        let dispersive_readout_snr_pass = cm.dispersive_readout_snr_db >= 18.0;

        let items = [
            valley_bulk_gap_pass,
            valley_chern_difference_pass,
            edge_group_velocity_pass,
            corner_transmission_pass,
            crosstalk_isolation_pass,
            target_transmission_pass,
            switching_latency_pass,
            concentrated_concurrence_pass,
            bell_fidelity_pass,
            dispersive_readout_snr_pass,
        ];

        let total_score = items.iter().filter(|&&p| p).count();
        let all_passed = total_score == 10;

        ValleyRouterAuditReport {
            valley_bulk_gap_pass,
            valley_chern_difference_pass,
            edge_group_velocity_pass,
            corner_transmission_pass,
            crosstalk_isolation_pass,
            target_transmission_pass,
            switching_latency_pass,
            concentrated_concurrence_pass,
            bell_fidelity_pass,
            dispersive_readout_snr_pass,
            total_score,
            all_passed,
        }
    }
}
