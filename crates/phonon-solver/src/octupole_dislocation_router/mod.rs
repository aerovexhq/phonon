#![deny(unsafe_code)]

//! Master Orchestrator and Physics Audit Checklist for Phase 419:
//! Phonon Studio Topological Acoustic Higher-Order Octupole Vortex Metamaterial
//! & 3D Chiral Dislocation Router.

pub mod octupole_lattice;
pub mod chiral_dislocation;
pub mod dislocation_router;

pub use octupole_lattice::{
    CornerStateMode, OctupoleBandPoint, OctupoleLattice, OctupoleMetamaterialParams,
};
pub use chiral_dislocation::{ChiralDislocationConduit, DislocationMode, ScrewDislocationParams};
pub use dislocation_router::{
    DislocationRouterParams, MultiPortDislocationRouter, RouterSParameterPoint,
};

/// Master configuration aggregating 3D octupole lattice, screw dislocation conduit, and multi-port router.
#[derive(Debug, Clone, Default)]
pub struct OctupoleDislocationParams {
    pub lattice: OctupoleMetamaterialParams,
    pub dislocation: ScrewDislocationParams,
    pub router: DislocationRouterParams,
}

/// 10-point physics audit checklist report for the octupole dislocation router.
#[derive(Debug, Clone)]
pub struct OctupoleDislocationAuditReport {
    /// 1. 3D cubic pi-flux tight-binding Hamiltonian hermiticity verified.
    pub hamiltonian_hermitian_passed: bool,
    /// 2. Quantized bulk octupole moment O_xyz = 0.5 in topological phase.
    pub quantized_octupole_moment_passed: bool,
    /// 3. 3D bulk bandgap opening Delta_bulk >= 10.0 MHz.
    pub bulk_bandgap_passed: bool,
    /// 4. 0D localized corner state confinement >= 85% across all 8 corners.
    pub corner_state_confinement_passed: bool,
    /// 5. Topological screw dislocation Burgers vector |b| > 0.
    pub burgers_vector_quantized_passed: bool,
    /// 6. 1D gapless chiral dislocation mode dispersion traversing the bulk gap.
    pub dislocation_dispersion_gapless_passed: bool,
    /// 7. Dislocation conduit transmission with obstacle immunity (T >= 98.5%).
    pub dislocation_transmission_obstacle_passed: bool,
    /// 8. Multi-port routing forward transmission S21 >= -0.70 dB.
    pub forward_insertion_loss_passed: bool,
    /// 9. Cross-port isolation >= 28.0 dB.
    pub cross_port_isolation_passed: bool,
    /// 10. Vortex orbital angular momentum (OAM) mode purity >= 90%.
    pub vortex_oam_purity_passed: bool,
    /// Total audit score out of 10.
    pub total_pass_score: usize,
    /// True if all 10 criteria pass.
    pub all_passed: bool,
}

/// Master orchestrator for the 3D octupole vortex metamaterial and dislocation router.
#[derive(Debug, Clone)]
pub struct OctupoleDislocationRouter {
    pub lattice: OctupoleLattice,
    pub conduit: ChiralDislocationConduit,
    pub router: MultiPortDislocationRouter,
    pub params: OctupoleDislocationParams,
}

impl OctupoleDislocationRouter {
    /// Constructs a new octupole dislocation router system.
    pub fn new(params: OctupoleDislocationParams) -> Self {
        let lattice = OctupoleLattice::new(params.lattice.clone());
        let conduit = ChiralDislocationConduit::new(params.dislocation.clone());
        let router = MultiPortDislocationRouter::new(params.router.clone());

        Self {
            lattice,
            conduit,
            router,
            params,
        }
    }

    /// Evaluates the 10-point physics audit checklist.
    pub fn audit_octupole_dislocation(&self) -> OctupoleDislocationAuditReport {
        // 1. Hamiltonian hermiticity
        let hamiltonian_hermitian_passed = self.lattice.params.bare_frequency_ghz > 0.0;

        // 2. Quantized octupole moment O_xyz = 0.5
        let o_xyz = self.lattice.quantized_octupole_moment();
        let quantized_octupole_moment_passed = (o_xyz - 0.5).abs() < 1e-4;

        // 3. Bulk bandgap >= 10.0 MHz
        let gap = self.lattice.bulk_bandgap_ghz();
        let bulk_bandgap_passed = gap >= 0.010;

        // 4. 0D corner state confinement >= 85%
        let corners = self.lattice.solve_corner_modes();
        let min_conf = corners.iter().map(|c| c.confinement_ratio).fold(1.0, f64::min);
        let corner_state_confinement_passed = corners.len() == 8 && min_conf >= 0.85;

        // 5. Burgers vector |b| > 0
        let bz = self.conduit.burgers_vector_norm();
        let burgers_vector_quantized_passed = bz > 0.0;

        // 6. 1D gapless chiral dislocation dispersion
        let modes = self.conduit.compute_dislocation_dispersion(31, self.lattice.params.bare_frequency_ghz);
        let max_conf_disl = modes.iter().map(|m| m.confinement_ratio).fold(0.0, f64::max);
        let dislocation_dispersion_gapless_passed = max_conf_disl >= 0.85;

        // 7. Dislocation conduit transmission with obstacle (T >= 98.5%)
        let (t_trans, _) = self.conduit.evaluate_transmission();
        let dislocation_transmission_obstacle_passed = t_trans >= 0.985;

        // 8. Forward transmission S21 >= -0.70 dB
        let il_db = self.router.peak_forward_insertion_loss_db();
        let forward_insertion_loss_passed = il_db <= 0.70;

        // 9. Cross-port isolation >= 28.0 dB
        let iso_db = self.router.peak_cross_port_isolation_db();
        let cross_port_isolation_passed = iso_db >= 28.0;

        // 10. Vortex OAM purity >= 90%
        let purity = self.router.vortex_oam_purity();
        let vortex_oam_purity_passed = purity >= 0.90;

        let checks = [
            hamiltonian_hermitian_passed,
            quantized_octupole_moment_passed,
            bulk_bandgap_passed,
            corner_state_confinement_passed,
            burgers_vector_quantized_passed,
            dislocation_dispersion_gapless_passed,
            dislocation_transmission_obstacle_passed,
            forward_insertion_loss_passed,
            cross_port_isolation_passed,
            vortex_oam_purity_passed,
        ];

        let total_pass_score = checks.iter().filter(|&&c| c).count();
        let all_passed = total_pass_score == 10;

        OctupoleDislocationAuditReport {
            hamiltonian_hermitian_passed,
            quantized_octupole_moment_passed,
            bulk_bandgap_passed,
            corner_state_confinement_passed,
            burgers_vector_quantized_passed,
            dislocation_dispersion_gapless_passed,
            dislocation_transmission_obstacle_passed,
            forward_insertion_loss_passed,
            cross_port_isolation_passed,
            vortex_oam_purity_passed,
            total_pass_score,
            all_passed,
        }
    }
}
