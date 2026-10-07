#![deny(unsafe_code)]

//! Phase 411: Phonon Studio Topological Chiral Acoustic Edge-Magnetoplasmon Circulator
//! & Non-Reciprocal Quantum Hall Router.
//!
//! Provides quantized Hall conductance, chiral edge-magnetoplasmon (EMP) dispersion,
//! 3-port cyclic circulator scattering matrix, and backscattering-immune topological routing.

pub mod chiral_dispersion;
pub mod circulator_s_matrix;
pub mod quantum_hall_router;

pub use chiral_dispersion::{
    ChiralDispersionSolver, ChiralEmpParams, EmpDispersionPoint, BOLTZMANN_K,
    CONDUCTANCE_QUANTUM, ELECTRON_MASS, ELEMENTARY_CHARGE, EPSILON_0, HBAR, PLANCK_CONSTANT,
};
pub use circulator_s_matrix::{
    EmpCirculator, EmpCirculatorParams, EmpSMatrix3x3, EmpSpectrumPoint,
};
pub use quantum_hall_router::{
    DefectParams, QuantumHallRouter, QuantumHallRouterParams, RouterChannel,
    RouterTransportMetrics,
};

/// An individual criterion in the 10-point physics audit checklist.
#[derive(Debug, Clone, PartialEq)]
pub struct EmpAuditCriterion {
    pub name: String,
    pub description: String,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}

/// Comprehensive physics audit report for the Chiral EMP Router system.
#[derive(Debug, Clone, PartialEq)]
pub struct EmpAuditReport {
    pub criteria: Vec<EmpAuditCriterion>,
    pub passed_count: usize,
    pub total_count: usize,
    pub all_passed: bool,
}

/// Master orchestrator uniting chiral dispersion, 3-port circulator, and topological router.
#[derive(Debug, Clone)]
pub struct ChiralEdgeMagnetoplasmonRouter {
    pub dispersion_solver: ChiralDispersionSolver,
    pub circulator: EmpCirculator,
    pub hall_router: QuantumHallRouter,
}

impl Default for ChiralEdgeMagnetoplasmonRouter {
    fn default() -> Self {
        let disp_params = ChiralEmpParams::default();
        let circ_params = EmpCirculatorParams {
            center_freq_ghz: disp_params.center_freq_ghz,
            ..Default::default()
        };
        let router_params = QuantumHallRouterParams::default();

        Self {
            dispersion_solver: ChiralDispersionSolver::new(disp_params),
            circulator: EmpCirculator::new(circ_params),
            hall_router: QuantumHallRouter::new(router_params),
        }
    }
}

impl ChiralEdgeMagnetoplasmonRouter {
    /// Creates a new router system with custom configuration parameters.
    pub fn new(
        disp_params: ChiralEmpParams,
        circ_params: EmpCirculatorParams,
        router_params: QuantumHallRouterParams,
    ) -> Self {
        Self {
            dispersion_solver: ChiralDispersionSolver::new(disp_params),
            circulator: EmpCirculator::new(circ_params),
            hall_router: QuantumHallRouter::new(router_params),
        }
    }

    /// Evaluates the 10-point physics audit checklist for the chiral EMP router system.
    pub fn audit_chiral_emp(&self) -> EmpAuditReport {
        let mut criteria = Vec::with_capacity(10);

        // 1. Quantized Hall conductance
        let sigma_xy = self.dispersion_solver.quantized_hall_conductance();
        let expected_sigma = self.dispersion_solver.params.filling_factor_nu as f64 * CONDUCTANCE_QUANTUM;
        let sigma_err = (sigma_xy - expected_sigma).abs();
        let c1_pass = sigma_err < 1.0e-12;
        criteria.push(EmpAuditCriterion {
            name: "Quantized Hall Conductance".to_string(),
            description: "Conductance quantized to nu * e^2 / h".to_string(),
            expected: format!("{:.6e} S", expected_sigma),
            actual: format!("{:.6e} S", sigma_xy),
            passed: c1_pass,
        });

        // 2. Broken time-reversal symmetry & cyclotron gap
        let fc_ghz = self.dispersion_solver.cyclotron_frequency_ghz();
        let gap_mev = self.dispersion_solver.cyclotron_energy_mev();
        let thermal_mev = self.dispersion_solver.thermal_energy_mev();
        let c2_pass = self.dispersion_solver.is_quantum_hall_regime_valid() && fc_ghz > 1.0e3;
        criteria.push(EmpAuditCriterion {
            name: "Broken TRS & Cyclotron Gap".to_string(),
            description: "Cyclotron gap hbar*omega_c >> k_B*T breaking TRS".to_string(),
            expected: format!("Gap > 5 * k_B*T ({:.2} meV)", 5.0 * thermal_mev),
            actual: format!("Gap = {:.2} meV (f_c = {:.1} GHz)", gap_mev, fc_ghz),
            passed: c2_pass,
        });

        // 3. Chiral edge velocity non-reciprocity
        let eta_nr = self.dispersion_solver.velocity_non_reciprocity_ratio();
        let c3_pass = eta_nr >= 0.02;
        criteria.push(EmpAuditCriterion {
            name: "Acoustic Velocity Non-Reciprocity".to_string(),
            description: "Non-reciprocal SAW velocity shift eta_nr >= 2.0%".to_string(),
            expected: "eta_nr >= 0.020 (2.0%)".to_string(),
            actual: format!("eta_nr = {:.4} ({:.2}%)", eta_nr, eta_nr * 100.0),
            passed: c3_pass,
        });

        // 4. Circulator forward insertion loss
        let il_db = self.circulator.forward_insertion_loss_db();
        let c4_pass = il_db <= 0.50;
        criteria.push(EmpAuditCriterion {
            name: "Circulator Forward Insertion Loss".to_string(),
            description: "Forward insertion loss S21 <= 0.50 dB".to_string(),
            expected: "<= 0.50 dB".to_string(),
            actual: format!("{:.2} dB", il_db),
            passed: c4_pass,
        });

        // 5. Circulator reverse isolation
        let iso_db = self.circulator.reverse_isolation_db();
        let c5_pass = iso_db >= 35.0;
        criteria.push(EmpAuditCriterion {
            name: "Circulator Reverse Isolation".to_string(),
            description: "Reverse isolation S12 >= 35.0 dB".to_string(),
            expected: ">= 35.0 dB".to_string(),
            actual: format!("{:.2} dB", iso_db),
            passed: c5_pass,
        });

        // 6. Circulator return loss
        let rl_db = self.circulator.return_loss_db();
        let c6_pass = rl_db >= 25.0;
        criteria.push(EmpAuditCriterion {
            name: "Circulator Return Loss".to_string(),
            description: "Port return loss S11 >= 25.0 dB".to_string(),
            expected: ">= 25.0 dB".to_string(),
            actual: format!("{:.2} dB", rl_db),
            passed: c6_pass,
        });

        // 7. Cyclic 3-port circulator symmetry
        let s_mat = self.circulator.evaluate_s_matrix(self.circulator.params.center_freq_ghz);
        let sym_diff1 = (s_mat.s21_mag - s_mat.s32_mag).abs();
        let sym_diff2 = (s_mat.s21_mag - s_mat.s13_mag).abs();
        let c7_pass = sym_diff1 < 1.0e-4 && sym_diff2 < 1.0e-4;
        criteria.push(EmpAuditCriterion {
            name: "Cyclic 3-Port Symmetry".to_string(),
            description: "Equal cyclic forward transmission |S21| = |S32| = |S13|".to_string(),
            expected: "Diff < 1.0e-4".to_string(),
            actual: format!("Max diff = {:.2e}", sym_diff1.max(sym_diff2)),
            passed: c7_pass,
        });

        // 8. Sharp 90-degree corner transmission
        let transport = self.hall_router.evaluate_transport_metrics();
        let c8_pass = transport.corner_power_transmission >= 0.95;
        criteria.push(EmpAuditCriterion {
            name: "Sharp 90-Degree Corner Transmission".to_string(),
            description: "Corner power transmission T_corner >= 95.0%".to_string(),
            expected: ">= 95.0%".to_string(),
            actual: format!("{:.2}% ({:.2} dB)", transport.corner_power_transmission * 100.0, transport.corner_transmission_db),
            passed: c8_pass,
        });

        // 9. Boundary defect backscattering suppression
        let c9_pass = transport.defect_backscattering_suppression_db >= 30.0;
        criteria.push(EmpAuditCriterion {
            name: "Defect Backscattering Suppression".to_string(),
            description: "Boundary defect backscattering suppression >= 30.0 dB".to_string(),
            expected: ">= 30.0 dB".to_string(),
            actual: format!("{:.2} dB", transport.defect_backscattering_suppression_db),
            passed: c9_pass,
        });

        // 10. Split-gate multi-terminal routing isolation
        let c10_pass = transport.cross_channel_isolation_db >= 35.0;
        criteria.push(EmpAuditCriterion {
            name: "Split-Gate Cross-Channel Isolation".to_string(),
            description: "Active-to-inactive channel isolation >= 35.0 dB".to_string(),
            expected: ">= 35.0 dB".to_string(),
            actual: format!("{:.2} dB", transport.cross_channel_isolation_db),
            passed: c10_pass,
        });

        let passed_count = criteria.iter().filter(|c| c.passed).count();
        let total_count = criteria.len();
        let all_passed = passed_count == total_count;

        EmpAuditReport {
            criteria,
            passed_count,
            total_count,
            all_passed,
        }
    }
}
