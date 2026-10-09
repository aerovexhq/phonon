#![deny(unsafe_code)]

//! Phase 460: Topological Acoustic Floquet Higher-Order Weyl Semimetal Vortex Transceiver & Multi-Terminal Quantum Acoustic Router.
//!
//! Master coordinator and physics invariant audit engine integrating:
//! 1. 3D Higher-Order Weyl Semimetal (HOWSM) with quantized Berry monopole charge |C| = 1.0,
//!    bulk Weyl node momentum separation Delta_kz, 1D chiral gapless hinge mode dispersion,
//!    and spatial acoustic energy confinement >= 85.0%.
//! 2. Acoustic Vortex Beam Transceiver converting chiral hinge states into collimated acoustic
//!    vortex beams carrying quantized Orbital Angular Momentum (OAM l = +/- 1, +/- 2), with
//!    modal purity P_oam >= 90.0%, generation efficiency >= 80.0%, and core null depth >= 25.0 dB.
//! 3. 6-terminal non-reciprocal chiral circulator and router steering vortex modes with forward
//!    loss IL <= 0.40 dB, backward isolation ISO >= 38.0 dB, return loss RL >= 22.0 dB, and
//!    topological defect retention >= 95.0% around sharp corner obstacles.

pub mod acoustic_vortex_transceiver;
pub mod higher_order_weyl_lattice;
pub mod multi_terminal_chiral_router;

pub use acoustic_vortex_transceiver::{
    WeylVortexGridPoint, WeylVortexMetrics, WeylVortexParams, WeylVortexRadialPoint,
    WeylVortexSolver,
};
pub use higher_order_weyl_lattice::{
    HingeModeSpatialPoint, HigherOrderWeylMetrics, HigherOrderWeylParams, HigherOrderWeylSolver,
    WeylDispersionPoint,
};
pub use multi_terminal_chiral_router::{
    MultiTerminalRouterMetrics, MultiTerminalRouterParams, MultiTerminalRouterSolver,
    RouterSpectrumPoint,
};

/// 10-point rigorous physics audit report for Phase 460.
#[derive(Debug, Clone)]
pub struct WeylVortexRouterAuditReport {
    /// 1. Berry curvature monopole charge quantization |C| = 1.0 (|C - 1.0| <= 0.02).
    pub quantized_berry_monopole: bool,
    /// 2. Bulk Weyl node momentum separation Delta_kz >= 0.015 1/um.
    pub weyl_node_separation: bool,
    /// 3. 1D chiral hinge state group velocity v_hinge >= 500.0 m/s.
    pub chiral_hinge_dispersion: bool,
    /// 4. Spatial acoustic energy confinement ratio at the hinges >= 85.0%.
    pub hinge_energy_confinement: bool,
    /// 5. Acoustic vortex quantized orbital angular momentum |l| >= 1.
    pub quantized_oam_charge: bool,
    /// 6. Vortex OAM modal purity P_oam >= 90.0%.
    pub vortex_mode_purity: bool,
    /// 7. Vortex beam generation efficiency >= 80.0%.
    pub vortex_beam_efficiency: bool,
    /// 8. Central vortex phase singularity core null depth >= 25.0 dB.
    pub vortex_core_null_depth: bool,
    /// 9. Multi-terminal forward loss IL <= 0.40 dB and backward isolation ISO >= 38.0 dB.
    pub multi_terminal_insertion_isolation: bool,
    /// 10. Corner defect obstacle retention >= 95.0% and return loss RL >= 22.0 dB.
    pub topological_defect_immunity: bool,
}

impl WeylVortexRouterAuditReport {
    /// Returns the (passed_count, total_count) score.
    pub fn score(&self) -> (usize, usize) {
        let items = [
            self.quantized_berry_monopole,
            self.weyl_node_separation,
            self.chiral_hinge_dispersion,
            self.hinge_energy_confinement,
            self.quantized_oam_charge,
            self.vortex_mode_purity,
            self.vortex_beam_efficiency,
            self.vortex_core_null_depth,
            self.multi_terminal_insertion_isolation,
            self.topological_defect_immunity,
        ];
        let passed = items.iter().filter(|&&v| v).count();
        (passed, items.len())
    }

    /// Returns true if all 10 physics audit criteria scored PASS.
    pub fn is_pass(&self) -> bool {
        let (passed, total) = self.score();
        passed == total
    }
}

/// Master coordinator processor for Higher-Order Weyl Vortex Transceiver & Multi-Terminal Router (Phase 460).
#[derive(Debug, Clone)]
pub struct WeylVortexRouterProcessor {
    pub weyl_params: HigherOrderWeylParams,
    pub vortex_params: WeylVortexParams,
    pub router_params: MultiTerminalRouterParams,
}

impl WeylVortexRouterProcessor {
    /// Creates a new master processor instance.
    pub fn new(
        weyl_params: HigherOrderWeylParams,
        vortex_params: WeylVortexParams,
        router_params: MultiTerminalRouterParams,
    ) -> Self {
        Self {
            weyl_params,
            vortex_params,
            router_params,
        }
    }

    /// Runs a comprehensive 10-point physics audit across all subsystems.
    pub fn audit_system(&self) -> WeylVortexRouterAuditReport {
        let weyl_solver = HigherOrderWeylSolver::new(self.weyl_params.clone());
        let weyl_m = weyl_solver.evaluate_metrics();

        let vortex_solver = WeylVortexSolver::new(self.vortex_params.clone());
        let vortex_m = vortex_solver.evaluate_metrics();

        let router_solver = MultiTerminalRouterSolver::new(self.router_params.clone());
        let router_m = router_solver.evaluate_metrics();

        WeylVortexRouterAuditReport {
            quantized_berry_monopole: (weyl_m.monopole_charge.abs() - 1.0).abs() <= 0.02
                && weyl_m.monopole_quantization_error <= 0.02,
            weyl_node_separation: weyl_m.weyl_node_separation_inv_um >= 0.015,
            chiral_hinge_dispersion: weyl_m.hinge_group_velocity_ms >= 500.0,
            hinge_energy_confinement: weyl_m.hinge_confinement_percent >= 85.0,
            quantized_oam_charge: vortex_m.measured_topological_charge.abs() >= 1,
            vortex_mode_purity: vortex_m.oam_mode_purity_percent >= 90.0,
            vortex_beam_efficiency: vortex_m.vortex_generation_efficiency_percent >= 80.0,
            vortex_core_null_depth: vortex_m.core_null_depth_db >= 25.0,
            multi_terminal_insertion_isolation: router_m.forward_insertion_loss_db <= 0.40
                && router_m.backward_isolation_db >= 38.0,
            topological_defect_immunity: router_m.corner_defect_retention_percent >= 95.0
                && router_m.port_return_loss_db >= 22.0,
        }
    }
}
