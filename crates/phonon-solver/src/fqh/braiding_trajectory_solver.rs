//! Quantum trajectory master equation solver for anyon braiding sequences and quasiparticle poisoning.
//!
//! Evaluates unitary braiding gate sequences, Lindblad-type parity dephasing, and target state fidelity.

use phonon_models::fqh::{c_abs_sq, mat2_dagger, mat2_mul, NonAbelianBraidingModel, Unitary2x2};

/// Discrete elementary braiding operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BraidOperation {
    /// R_12 braid exchanging anyons 1 and 2.
    R12,
    /// B_23 braid exchanging anyons 2 and 3.
    B23,
    /// R_12^\u{2020} inverse braid.
    R12Dagger,
    /// B_23^\u{2020} inverse braid.
    B23Dagger,
}

/// Trajectory result of a sequence of anyonic braid gates.
#[derive(Debug, Clone, PartialEq)]
pub struct BraidingTrajectoryResult {
    /// Final 2x2 density matrix [[re, im]; 4].
    pub final_density_matrix: Unitary2x2,
    /// State purity Tr(\u{03c1}^2) \u{2208} [0.5, 1.0].
    pub purity: f64,
    /// State fidelity relative to the target pure state.
    pub fidelity: f64,
    /// Number of applied braid gates.
    pub gate_count: usize,
    /// Whether target fidelity meets threshold (>= 99.9%).
    pub passed_fidelity: bool,
}

/// Quantum trajectory solver for anyon braiding.
pub struct BraidingTrajectorySolver {
    /// Quasiparticle poisoning error probability per gate p_p \u{2208} [0, 1].
    pub poisoning_prob_per_gate: f64,
}

impl BraidingTrajectorySolver {
    /// Constructs a braiding solver with a specified poisoning dephasing rate.
    pub fn new(poisoning_prob_per_gate: f64) -> Self {
        Self {
            poisoning_prob_per_gate: poisoning_prob_per_gate.clamp(0.0, 0.5),
        }
    }

    /// Default solver under ultra-low cryogenic poisoning (p_p = 1.0e-5).
    pub fn cryogenic_low_noise() -> Self {
        Self::new(1.0e-5)
    }

    /// Evaluates the 2x2 unitary matrix corresponding to a braid operation.
    pub fn operation_matrix(op: BraidOperation) -> Unitary2x2 {
        match op {
            BraidOperation::R12 => NonAbelianBraidingModel::braid_r12(),
            BraidOperation::B23 => NonAbelianBraidingModel::braid_b23(),
            BraidOperation::R12Dagger => mat2_dagger(&NonAbelianBraidingModel::braid_r12()),
            BraidOperation::B23Dagger => mat2_dagger(&NonAbelianBraidingModel::braid_b23()),
        }
    }

    /// Simulates a sequence of braid operations starting from |0\u{27e9} state: \u{03c1}_0 = [[1,0], [0,0]].
    pub fn run_braid_sequence(&self, sequence: &[BraidOperation]) -> BraidingTrajectoryResult {
        // Initial pure state |0\u{27e9}: \u{03c1}_0 = [[1, 0], [0, 0]]
        let mut rho: Unitary2x2 = [(1.0, 0.0), (0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];

        // Track ideal target unitary
        let mut u_total: Unitary2x2 = [(1.0, 0.0), (0.0, 0.0), (0.0, 0.0), (1.0, 0.0)];

        let p_p = self.poisoning_prob_per_gate;

        for &op in sequence {
            let u_op = Self::operation_matrix(op);
            u_total = mat2_mul(&u_op, &u_total);

            // Coherent unitary step: \u{03c1} \u{2192} U * \u{03c1} * U\u{2020}
            rho = NonAbelianBraidingModel::evolve_density_matrix(&rho, &u_op);

            // Incoherent poisoning dephasing channel: \u{03c3}_z dephasing
            // \u{03c3}_z * \u{03c1} * \u{03c3}_z flips sign of off-diagonal elements \u{03c1}_01 and \u{03c1}_10
            if p_p > 0.0 {
                let diag0 = rho[0];
                let diag1 = rho[3];
                let off01 = rho[1];
                let off10 = rho[2];

                // (1 - p) * off + p * (-off) = (1 - 2p) * off
                let dephase_factor = 1.0 - 2.0 * p_p;
                rho[0] = diag0;
                rho[1] = (off01.0 * dephase_factor, off01.1 * dephase_factor);
                rho[2] = (off10.0 * dephase_factor, off10.1 * dephase_factor);
                rho[3] = diag1;
            }
        }

        // Target ideal pure state is u_total * |0\u{27e9}
        let target_state = [u_total[0], u_total[2]];

        let fidelity = NonAbelianBraidingModel::state_fidelity(&rho, target_state);

        // Purity Tr(\u{03c1}^2) = |\u{03c1}00|^2 + |\u{03c1}11|^2 + 2 * |\u{03c1}01|^2
        let purity = c_abs_sq(rho[0]) + c_abs_sq(rho[3]) + 2.0 * c_abs_sq(rho[1]);

        BraidingTrajectoryResult {
            final_density_matrix: rho,
            purity: purity.clamp(0.5, 1.0),
            fidelity,
            gate_count: sequence.len(),
            passed_fidelity: fidelity >= 0.999,
        }
    }
}
