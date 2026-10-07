#![deny(unsafe_code)]

//! Master module for Phase 413: Phonon Studio Topological Acoustic Synthetic Dimension
//! Chern Insulator & High-Dimensional Multiplexed Router.
//!
//! Unifies Harper-Hofstadter synthetic lattices, 4D topological Weyl transport,
//! and multi-channel topological frequency multiplexing in safe pure Rust.

pub mod synthetic_lattice;
pub mod weyl_transport;
pub mod multiplexed_router;

pub use synthetic_lattice::{
    SyntheticBandPoint, SyntheticLatticeMetrics, SyntheticLatticeParams, SyntheticLatticePoint,
    SyntheticLatticeSolver,
};
pub use weyl_transport::{
    WeylArcPoint, WeylSyntheticParams, WeylTransportMetrics, WeylTransportSolver,
    WeylWavepacketPoint,
};
pub use multiplexed_router::{
    ChannelRoutingPoint, SyntheticMultiplexedRouter, SyntheticRouterMetrics, SyntheticRouterParams,
};

/// A single verification item in the synthetic dimension physics audit.
#[derive(Debug, Clone, PartialEq)]
pub struct SyntheticDimensionAuditCriterion {
    pub name: String,
    pub description: String,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}

/// Comprehensive physics audit report for the synthetic dimension router.
#[derive(Debug, Clone, PartialEq)]
pub struct SyntheticDimensionAuditReport {
    pub criteria: Vec<SyntheticDimensionAuditCriterion>,
    pub passed_count: usize,
    pub total_count: usize,
    pub all_passed: bool,
}

/// Master orchestrator coupling the synthetic 2D lattice, Weyl transport, and multiplexed router.
#[derive(Debug, Clone)]
pub struct SyntheticDimensionRouter {
    pub lattice_solver: SyntheticLatticeSolver,
    pub weyl_solver: WeylTransportSolver,
    pub router_solver: SyntheticMultiplexedRouter,
}

impl Default for SyntheticDimensionRouter {
    fn default() -> Self {
        Self {
            lattice_solver: SyntheticLatticeSolver::new(SyntheticLatticeParams::default()),
            weyl_solver: WeylTransportSolver::new(WeylSyntheticParams::default()),
            router_solver: SyntheticMultiplexedRouter::new(SyntheticRouterParams::default()),
        }
    }
}

impl SyntheticDimensionRouter {
    /// Creates a new router system with custom sub-solvers.
    pub fn new(
        lattice_params: SyntheticLatticeParams,
        weyl_params: WeylSyntheticParams,
        router_params: SyntheticRouterParams,
    ) -> Self {
        Self {
            lattice_solver: SyntheticLatticeSolver::new(lattice_params),
            weyl_solver: WeylTransportSolver::new(weyl_params),
            router_solver: SyntheticMultiplexedRouter::new(router_params),
        }
    }

    /// Evaluates the 10-point physics audit checklist.
    pub fn audit_synthetic_dimension_router(&self) -> SyntheticDimensionAuditReport {
        let mut criteria = Vec::with_capacity(10);

        let lattice_metrics = self.lattice_solver.evaluate_metrics();
        let weyl_metrics = self.weyl_solver.evaluate_metrics();
        let router_metrics = self.router_solver.evaluate_metrics();

        // 1. Synthetic Magnetic Flux Per Plaquette
        let flux = lattice_metrics.flux_per_plaquette_ratio;
        let c1_pass = flux > 0.05 && flux < 0.95;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "Synthetic Magnetic Flux".to_string(),
            description: "Modulation phase gradient synthesizes non-zero flux Phi / 2*pi in (0, 1)".to_string(),
            expected: "0.05 < Phi / 2*pi < 0.95".to_string(),
            actual: format!("{:.3}", flux),
            passed: c1_pass,
        });

        // 2. First Chern Number Quantization
        let c1 = lattice_metrics.first_chern_number;
        let c2_pass = (c1.abs() - 1.0).abs() < 1e-6;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "First Chern Number".to_string(),
            description: "Topological quantization of 2D synthetic band Chern number |C_1| = 1".to_string(),
            expected: "|C_1| = 1.0".to_string(),
            actual: format!("C_1 = {:.1}", c1),
            passed: c2_pass,
        });

        // 3. Chiral Synthetic Edge State Confinement
        let conf = lattice_metrics.edge_confinement_ratio;
        let c3_pass = conf >= 0.85;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "Chiral Edge Confinement".to_string(),
            description: "Acoustic probability localized on synthetic/physical boundaries >= 85%".to_string(),
            expected: ">= 85.0%".to_string(),
            actual: format!("{:.1}%", conf * 100.0),
            passed: c3_pass,
        });

        // 4. Unidirectional Frequency Ladder Pumping
        let v_m = lattice_metrics.frequency_ladder_velocity_modes_us;
        let dir_db = lattice_metrics.synthetic_directivity_db;
        let c4_pass = v_m.abs() >= 5.0 && dir_db >= 25.0;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "Frequency Ladder Pumping".to_string(),
            description: "Unidirectional frequency ladder climb |v_m| >= 5 modes/us, Directivity >= 25 dB".to_string(),
            expected: "|v_m| >= 5.0, Dir >= 25.0 dB".to_string(),
            actual: format!("|v_m| = {:.1} modes/us, Dir = {:.1} dB", v_m.abs(), dir_db),
            passed: c4_pass,
        });

        // 5. Harper-Hofstadter Bulk Bandgap
        let gap = lattice_metrics.bulk_bandgap_mhz;
        let c5_pass = gap >= 1.5;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "Hofstadter Bulk Gap".to_string(),
            description: "Topologically non-trivial bulk bandgap Delta_bulk >= 1.5 MHz".to_string(),
            expected: ">= 1.5 MHz".to_string(),
            actual: format!("{:.2} MHz", gap),
            passed: c5_pass,
        });

        // 6. Second Chern Number Quantization (4D Topology)
        let c2 = weyl_metrics.second_chern_number;
        let c6_pass = (c2.abs() - 1.0).abs() < 1e-6;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "Second Chern Number".to_string(),
            description: "Quantized 4D topological invariant |C_2| = 1.0".to_string(),
            expected: "|C_2| = 1.0".to_string(),
            actual: format!("C_2 = {:.1}", c2),
            passed: c6_pass,
        });

        // 7. Non-Local Fermi Arc Weyl Transport
        let weyl_t = weyl_metrics.non_local_transmission_ratio;
        let c7_pass = weyl_t >= 0.80;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "Non-Local Weyl Transport".to_string(),
            description: "Boundary-to-boundary transmission through synthetic dimensions >= 80%".to_string(),
            expected: ">= 80.0%".to_string(),
            actual: format!("{:.1}%", weyl_t * 100.0),
            passed: c7_pass,
        });

        // 8. Inter-Channel Routing Isolation
        let iso = router_metrics.inter_channel_isolation_db;
        let c8_pass = iso >= 35.0;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "Inter-Channel Isolation".to_string(),
            description: "Cross-talk suppression between orthogonal frequency channels >= 35.0 dB".to_string(),
            expected: ">= 35.0 dB".to_string(),
            actual: format!("{:.1} dB", iso),
            passed: c8_pass,
        });

        // 9. Router Insertion Loss
        let il = router_metrics.insertion_loss_db;
        let t_target = router_metrics.target_transmission_ratio;
        let c9_pass = il <= 0.80 && t_target >= 0.832;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "Router Insertion Loss".to_string(),
            description: "Target routed channel insertion loss <= 0.80 dB (T >= 83.2%)".to_string(),
            expected: "<= 0.80 dB, T >= 83.2%".to_string(),
            actual: format!("IL = {:.2} dB, T = {:.1}%", il, t_target * 100.0),
            passed: c9_pass,
        });

        // 10. Topological Defect Resilience
        let immunity = router_metrics.defect_immunity_ratio;
        let c10_pass = immunity >= 0.95;
        criteria.push(SyntheticDimensionAuditCriterion {
            name: "Defect Immunity".to_string(),
            description: "Routing transmission preservation under resonator defect T_defect / T_clean >= 95%".to_string(),
            expected: ">= 95.0%".to_string(),
            actual: format!("{:.1}%", immunity * 100.0),
            passed: c10_pass,
        });

        let passed_count = criteria.iter().filter(|c| c.passed).count();
        let total_count = criteria.len();
        let all_passed = passed_count == total_count;

        SyntheticDimensionAuditReport {
            criteria,
            passed_count,
            total_count,
            all_passed,
        }
    }
}
