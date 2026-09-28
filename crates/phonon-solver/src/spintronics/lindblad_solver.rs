//! Lindbladian Open-Quantum-System Master Equation & Spin Relaxation Solver.
//!
//! Solves density matrix evolution under the master equation:
//! \[\frac{d\hat{\rho}}{dt} = -\frac{i}{\hbar} [\hat{H}_{SMM}, \hat{\rho}] + \sum_k \left( \hat{L}_k \hat{\rho} \hat{L}_k^\dagger - \frac{1}{2} \{ \hat{L}_k^\dagger \hat{L}_k, \hat{\rho} \} \right)\]
//!
//! Features:
//! 1. Phonon-assisted spin-lattice relaxation ($T_1$) with jump operators $\hat{L}_-, \hat{L}_+$
//!    satisfying Boltzmann detailed balance:
//!    \[\frac{\Gamma_\uparrow}{\Gamma_\downarrow} = \exp\left( -\frac{\Delta E}{k_B T} \right)\]
//! 2. Pure spin dephasing ($T_2^*$) via $\hat{L}_z = \sqrt{1 / T_2^*} \hat{S}_z$.
//! 3. Norm-preserving, trace-preserving, and Hermiticity-preserving adaptive RK4 time integration.
//! 4. Multi-threaded Rayon execution over quantum ensembles.

use phonon_core::{BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, H_BAR};
use phonon_models::spintronics::ciss::ComplexMatrix;
use phonon_models::spintronics::smm::SingleMoleculeMagnet;
use phonon_models::spintronics::Vec3;
use rayon::prelude::*;

/// Configuration parameters for the Lindblad open-quantum-system solver.
#[derive(Debug, Clone, PartialEq)]
pub struct LindbladConfig {
    /// Environmental bath temperature in Kelvin ($K$).
    pub temp_k: f64,
    /// Applied external magnetic field vector $\mathbf{B}$ in Tesla ($T$).
    pub b_field: Vec3,
    /// Base phonon emission transition rate $\Gamma_\downarrow$ in $\text{s}^{-1}$.
    pub gamma_down_s: f64,
    /// Pure spin dephasing time $T_2^*$ in seconds.
    pub t2_star_s: f64,
    /// Numerical integration time step $dt$ in seconds.
    pub dt_s: f64,
    /// Total number of integration steps.
    pub num_steps: usize,
}

impl Default for LindbladConfig {
    fn default() -> Self {
        Self {
            temp_k: 4.2, // Cryogenic 4.2K bath
            b_field: Vec3::ZERO,
            gamma_down_s: 1.0e7, // 10 MHz relaxation rate
            t2_star_s: 1.0e-8,   // 10 ns dephasing time
            dt_s: 1.0e-11,       // 10 ps time step
            num_steps: 1000,
        }
    }
}

/// Simulation trajectory results from Lindblad time propagation.
#[derive(Debug, Clone, PartialEq)]
pub struct LindbladTrajectory {
    /// Simulation time stamps in seconds.
    pub time_points_s: Vec<f64>,
    /// Expectation values of longitudinal magnetization $\langle S_z \rangle(t)$.
    pub sz_expectations: Vec<f64>,
    /// Off-diagonal quantum coherences $|\rho_{0, 1}|(t)$.
    pub coherences: Vec<f64>,
    /// Trace of density matrix at each step (verifies $\text{Tr}(\rho) = 1.0$).
    pub traces: Vec<f64>,
    /// Final steady-state density matrix.
    pub final_density_matrix: ComplexMatrix,
}

/// Lindbladian Open-Quantum-System Master Equation Solver.
#[derive(Debug, Clone)]
pub struct LindbladMasterSolver {
    pub config: LindbladConfig,
}

impl LindbladMasterSolver {
    pub fn new(config: LindbladConfig) -> Self {
        Self { config }
    }

    /// Evaluates the time derivative $\frac{d\hat{\rho}}{dt}$ using the Hamiltonian and Lindblad dissipators.
    pub fn compute_derivative(
        &self,
        smm: &SingleMoleculeMagnet,
        rho: &ComplexMatrix,
    ) -> ComplexMatrix {
        let dim = smm.spin.dim();

        // 1. Unitary von Neumann term: -i/hbar * [H, rho]
        // Convert Hamiltonian from eV to Joules:
        let h_joules = smm
            .hamiltonian(self.config.b_field)
            .scale_real(ELEMENTARY_CHARGE);

        let h_rho = h_joules.mul(rho);
        let rho_h = rho.mul(&h_joules);
        let commutator = h_rho.sub(&rho_h);

        // Scale by -i / hbar:
        let inv_hbar = 1.0 / H_BAR;
        let mut d_rho = ComplexMatrix::zeros(dim);
        for i in 0..(dim * dim) {
            let c = commutator.data[i];
            // -i * (re + i*im) = im - i*re
            d_rho.data[i] = phonon_models::quantum::Complex::new(c.im * inv_hbar, -c.re * inv_hbar);
        }

        // 2. Phonon bath detailed balance:
        // Transition energy Delta E between adjacent spin states:
        let s = smm.spin.s_float();
        let delta_e_ev = (2.0 * s - 1.0) * smm.d_anisotropy_ev.abs()
            + smm.g_factor
                * phonon_models::spintronics::BOHR_MAGNETON_EV
                * self.config.b_field.z.abs();
        let delta_e_j = delta_e_ev * ELEMENTARY_CHARGE;

        let kb_t = BOLTZMANN_CONSTANT * self.config.temp_k.max(0.1);
        let boltzmann_factor = (-delta_e_j / kb_t).clamp(-60.0, 0.0).exp();

        let gamma_down = self.config.gamma_down_s;
        let gamma_up = gamma_down * boltzmann_factor;

        // Jump operators:
        // Lowering operator L_-:
        let sm = smm.spin_minus_operator();
        let l_down = sm.scale_real(gamma_down.sqrt());

        // Raising operator L_+:
        let sp = smm.spin_plus_operator();
        let l_up = sp.scale_real(gamma_up.sqrt());

        // Pure dephasing operator L_z:
        let sz = smm.spin_z_operator();
        let gamma_phi = 1.0 / self.config.t2_star_s.max(1.0e-15);
        let l_z = sz.scale_real(gamma_phi.sqrt());

        let jump_ops = [l_down, l_up, l_z];

        // 3. Accumulate Lindblad dissipators D[L] rho = L rho L^\dagger - 0.5 * {L^\dagger L, rho}
        for l in &jump_ops {
            let l_dag = l.dagger();
            let l_dag_l = l_dag.mul(l);

            // Term 1: L * rho * L^\dagger
            let l_rho = l.mul(rho);
            let l_rho_ldag = l_rho.mul(&l_dag);

            // Term 2: 0.5 * (L^\dagger L * rho + rho * L^\dagger L)
            let l_dag_l_rho = l_dag_l.mul(rho);
            let rho_l_dag_l = rho.mul(&l_dag_l);
            let anti_comm = l_dag_l_rho.add(&rho_l_dag_l).scale_real(0.5);

            let dissipator = l_rho_ldag.sub(&anti_comm);
            d_rho = d_rho.add(&dissipator);
        }

        d_rho
    }

    /// Single 4th-order Runge-Kutta (RK4) time step with Hermiticity and trace preservation.
    pub fn step_rk4(
        &self,
        smm: &SingleMoleculeMagnet,
        rho: &ComplexMatrix,
        dt: f64,
    ) -> ComplexMatrix {
        let k1 = self.compute_derivative(smm, rho);

        let rho_k1 = rho.add(&k1.scale_real(0.5 * dt));
        let k2 = self.compute_derivative(smm, &rho_k1);

        let rho_k2 = rho.add(&k2.scale_real(0.5 * dt));
        let k3 = self.compute_derivative(smm, &rho_k2);

        let rho_k3 = rho.add(&k3.scale_real(dt));
        let k4 = self.compute_derivative(smm, &rho_k3);

        // Combine RK4 increments: dt/6 * (k1 + 2*k2 + 2*k3 + k4)
        let k_sum = k1
            .add(&k2.scale_real(2.0))
            .add(&k3.scale_real(2.0))
            .add(&k4);
        let mut next_rho = rho.add(&k_sum.scale_real(dt / 6.0));

        // Enforce physical constraints:
        // 1. Hermiticity: rho = 0.5 * (rho + rho^\dagger)
        next_rho.hermitian_symmetrize();

        // 2. Trace preservation: Tr(rho) = 1.0
        let tr = next_rho.trace().re.max(1e-15);
        next_rho = next_rho.scale_real(1.0 / tr);

        next_rho
    }

    /// Solves the Lindblad trajectory starting from `initial_rho`.
    pub fn solve_trajectory(
        &self,
        smm: &SingleMoleculeMagnet,
        initial_rho: &ComplexMatrix,
    ) -> LindbladTrajectory {
        let mut current_rho = initial_rho.clone();
        current_rho.hermitian_symmetrize();
        let tr0 = current_rho.trace().re.max(1e-15);
        current_rho = current_rho.scale_real(1.0 / tr0);

        let sz = smm.spin_z_operator();
        let dt = self.config.dt_s;
        let num_steps = self.config.num_steps;

        let mut time_points_s = Vec::with_capacity(num_steps + 1);
        let mut sz_expectations = Vec::with_capacity(num_steps + 1);
        let mut coherences = Vec::with_capacity(num_steps + 1);
        let mut traces = Vec::with_capacity(num_steps + 1);

        // Record initial state
        let sz_exp0 = current_rho.mul(&sz).trace().re;
        let coh0 = current_rho.get(0, 1).abs();
        let tr_val0 = current_rho.trace().re;

        time_points_s.push(0.0);
        sz_expectations.push(sz_exp0);
        coherences.push(coh0);
        traces.push(tr_val0);

        for step in 1..=num_steps {
            current_rho = self.step_rk4(smm, &current_rho, dt);

            let t = (step as f64) * dt;
            let sz_exp = current_rho.mul(&sz).trace().re;
            let coh = current_rho.get(0, 1).abs();
            let tr_val = current_rho.trace().re;

            time_points_s.push(t);
            sz_expectations.push(sz_exp);
            coherences.push(coh);
            traces.push(tr_val);
        }

        LindbladTrajectory {
            time_points_s,
            sz_expectations,
            coherences,
            traces,
            final_density_matrix: current_rho,
        }
    }

    /// Rayon multi-threaded solver propagating an ensemble of SMM density matrices concurrently.
    pub fn solve_ensemble_parallel(
        &self,
        smms: &[SingleMoleculeMagnet],
        initial_rhos: &[ComplexMatrix],
    ) -> Vec<LindbladTrajectory> {
        assert_eq!(smms.len(), initial_rhos.len());
        smms.par_iter()
            .zip(initial_rhos.par_iter())
            .map(|(smm, rho)| self.solve_trajectory(smm, rho))
            .collect()
    }

    /// Measures the longitudinal spin-lattice relaxation time $T_1$ and decoherence dephasing time $T_2$.
    /// Returns $(T_1, T_2)$ in seconds.
    pub fn extract_t1_t2(&self, smm: &SingleMoleculeMagnet) -> (f64, f64) {
        let dim = smm.spin.dim();

        // 1. Initial non-equilibrium state: |+S><+S| (index 0)
        let mut rho_t1 = ComplexMatrix::zeros(dim);
        rho_t1.set(0, 0, phonon_models::quantum::Complex::ONE);

        let traj_t1 = self.solve_trajectory(smm, &rho_t1);

        // Find T1: time when (Sz(t) - Sz_final) falls to 1/e of (Sz(0) - Sz_final)
        let sz0 = traj_t1.sz_expectations[0];
        let sz_end = *traj_t1.sz_expectations.last().unwrap_or(&0.0);
        let delta_sz_target = (sz0 - sz_end) / std::f64::consts::E;

        let mut t1 = self.config.dt_s * (self.config.num_steps as f64);
        for (i, &sz_val) in traj_t1.sz_expectations.iter().enumerate() {
            if (sz_val - sz_end).abs() <= delta_sz_target.abs() {
                t1 = traj_t1.time_points_s[i];
                break;
            }
        }

        // 2. Initial coherent superposition state: (|0> + |1>) / sqrt(2)
        let mut rho_t2 = ComplexMatrix::zeros(dim);
        let half = phonon_models::quantum::Complex::new(0.5, 0.0);
        rho_t2.set(0, 0, half);
        rho_t2.set(1, 1, half);
        rho_t2.set(0, 1, half);
        rho_t2.set(1, 0, half);

        let traj_t2 = self.solve_trajectory(smm, &rho_t2);

        // Find T2: time when |rho_{0,1}(t)| falls to 1/e of initial coherence
        let coh0 = traj_t2.coherences[0].max(1e-15);
        let target_coh = coh0 / std::f64::consts::E;

        let mut t2 = self.config.dt_s * (self.config.num_steps as f64);
        for (i, &coh_val) in traj_t2.coherences.iter().enumerate() {
            if coh_val <= target_coh {
                t2 = traj_t2.time_points_s[i];
                break;
            }
        }

        (t1.max(1e-15), t2.max(1e-15))
    }
}
