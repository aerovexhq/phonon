#![deny(unsafe_code)]

//! Virtual Phonon-Mediated Remote Qubit Entanglement & Beam Splitter Routing.
//!
//! Solves the coherent interaction between two remote transmon qubits dispersively
//! coupled to a shared Surface Acoustic Wave (SAW) bus mode, synthesizing maximally
//! entangled Bell states and phononic beam splitter routing.

use super::cqa_master_equation::Complex64;
use phonon_models::quantum_acoustic::{SawBeamSplitter, VirtualPhononBus};
use std::f64::consts::PI;

/// Two-Qubit Hilbert Space Dimension ($2 \times 2 = 4$).
pub const TWO_QUBIT_DIM: usize = 4;

/// Two-Qubit Density Matrix $\rho_{2q} \in \mathbb{C}^{4 \times 4}$.
///
/// Basis state indexing:
/// - $0 \implies |gg\rangle$
/// - $1 \implies |ge\rangle$
/// - $2 \implies |eg\rangle$
/// - $3 \implies |ee\rangle$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TwoQubitDensityMatrix {
    /// Row-major array of elements ($4 \times 4 = 16$).
    pub data: [Complex64; TWO_QUBIT_DIM * TWO_QUBIT_DIM],
}

impl Default for TwoQubitDensityMatrix {
    fn default() -> Self {
        Self {
            data: [Complex64::ZERO; TWO_QUBIT_DIM * TWO_QUBIT_DIM],
        }
    }
}

impl TwoQubitDensityMatrix {
    /// Maps $(q_1, q_2)$ to index $\in [0, 3]$.
    #[inline]
    pub fn state_index(q1: usize, q2: usize) -> usize {
        debug_assert!(q1 < 2 && q2 < 2);
        q1 * 2 + q2
    }

    /// Gets element $\rho_{i, j}$.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> Complex64 {
        self.data[i * TWO_QUBIT_DIM + j]
    }

    /// Sets element $\rho_{i, j}$.
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, val: Complex64) {
        self.data[i * TWO_QUBIT_DIM + j] = val;
    }

    /// Initializes in unentangled state $|eg\rangle$ (Qubit 1 excited, Qubit 2 ground).
    pub fn new_eg() -> Self {
        let mut dm = Self::default();
        let idx = Self::state_index(1, 0); // index 2
        dm.set(idx, idx, Complex64::ONE);
        dm
    }

    /// Enforces Hermiticity $\rho \leftarrow \frac{1}{2}(\rho + \rho^\dagger)$ and unit trace.
    pub fn hermitize_and_normalize(&mut self) {
        for i in 0..TWO_QUBIT_DIM {
            for j in i..TWO_QUBIT_DIM {
                let a = self.get(i, j);
                let b = self.get(j, i);
                let avg_re = 0.5 * (a.re + b.re);
                let avg_im = 0.5 * (a.im - b.im);
                if i == j {
                    self.set(i, i, Complex64::new(avg_re, 0.0));
                } else {
                    self.set(i, j, Complex64::new(avg_re, avg_im));
                    self.set(j, i, Complex64::new(avg_re, -avg_im));
                }
            }
        }

        let mut tr = 0.0;
        for i in 0..TWO_QUBIT_DIM {
            tr += self.get(i, i).re;
        }

        if tr > 1e-12 {
            let inv_tr = 1.0 / tr;
            for elem in self.data.iter_mut() {
                *elem = elem.scale(inv_tr);
            }
        }
    }

    /// Fidelity relative to the target entangled Bell state $|\Psi^+\rangle = \frac{|eg\rangle - i |ge\rangle}{\sqrt{2}}$.
    pub fn bell_state_fidelity(&self) -> f64 {
        let p_eg = self.get(2, 2).re;
        let p_ge = self.get(1, 1).re;
        let coh_ge_eg = self.get(1, 2);
        // <Psi+|rho|Psi+> = 0.5*(rho_eg_eg + rho_ge_ge) - Im(rho_ge_eg)
        let f = 0.5 * (p_eg + p_ge) - coh_ge_eg.im;
        f.clamp(0.0, 1.0)
    }

    /// Quantum Concurrence $\mathcal{C}(\rho) \in [0, 1]$ evaluating entanglement of formation.
    pub fn concurrence(&self) -> f64 {
        let p_gg = self.get(0, 0).re.max(0.0);
        let p_ee = self.get(3, 3).re.max(0.0);
        let coh = self.get(1, 2).norm();
        let geom = (p_gg * p_ee).sqrt();
        (2.0 * (coh - geom)).clamp(0.0, 1.0)
    }
}

/// Remote Two-Qubit Entanglement Solver.
#[derive(Debug, Clone, Copy)]
pub struct PhononEntanglementSolver {
    /// Virtual phonon bus parameters.
    pub bus: VirtualPhononBus,
}

impl PhononEntanglementSolver {
    /// Creates a new entanglement solver.
    pub fn new(bus: VirtualPhononBus) -> Self {
        Self { bus }
    }

    /// Evaluates the Lindbladian derivative for the two-qubit density matrix:
    ///
    /// $$\frac{d\rho}{dt} = -i [J_{eff} (\sigma_1^+ \sigma_2^- + \sigma_1^- \sigma_2^+), \rho] + \sum_k \mathcal{D}[C_k]\rho$$
    pub fn lindblad_derivative(&self, rho: &TwoQubitDensityMatrix) -> TwoQubitDensityMatrix {
        let mut d_rho = TwoQubitDensityMatrix::default();
        let j_eff = self.bus.effective_exchange_coupling_rad_s();

        // 1. Commutator: H = hbar * J_eff * (|ge><eg| + |eg><ge|) = hbar * J_eff * (|1><2| + |2><1|)
        // H * rho:
        // row 0: 0
        // row 1: J_eff * rho[2, :]
        // row 2: J_eff * rho[1, :]
        // row 3: 0
        let mut h_rho = [Complex64::ZERO; TWO_QUBIT_DIM * TWO_QUBIT_DIM];
        for col in 0..TWO_QUBIT_DIM {
            h_rho[TWO_QUBIT_DIM + col] = rho.get(2, col).scale(j_eff);
            h_rho[2 * TWO_QUBIT_DIM + col] = rho.get(1, col).scale(j_eff);
        }

        for i in 0..TWO_QUBIT_DIM {
            for j in 0..TWO_QUBIT_DIM {
                let hr = h_rho[i * TWO_QUBIT_DIM + j];
                let rh = h_rho[j * TWO_QUBIT_DIM + i].conj();
                let comm = hr.sub(rh);
                // -i * comm = (comm.im, -comm.re)
                d_rho.set(i, j, Complex64::new(comm.im, -comm.re));
            }
        }

        // 2. Decoherence from Qubit 1:
        let gamma1_q1 = self.bus.coupling1.qubit.relaxation_rate();
        let gphi_q1 = self.bus.coupling1.qubit.pure_dephasing_rate();

        // Relaxation: |eg>(2) -> |gg>(0), |ee>(3) -> |ge>(1)
        if gamma1_q1 > 0.0 {
            let cur_0 = d_rho.get(0, 0);
            d_rho.set(0, 0, cur_0.add(rho.get(2, 2).scale(gamma1_q1)));
            let cur_2 = d_rho.get(2, 2);
            d_rho.set(2, 2, cur_2.sub(rho.get(2, 2).scale(gamma1_q1)));

            let cur_1 = d_rho.get(1, 1);
            d_rho.set(1, 1, cur_1.add(rho.get(3, 3).scale(gamma1_q1)));
            let cur_3 = d_rho.get(3, 3);
            d_rho.set(3, 3, cur_3.sub(rho.get(3, 3).scale(gamma1_q1)));

            // Damping of coherences involving qubit 1
            let c12 = d_rho.get(1, 2);
            d_rho.set(1, 2, c12.sub(rho.get(1, 2).scale(0.5 * gamma1_q1)));
            let c21 = d_rho.get(2, 1);
            d_rho.set(2, 1, c21.sub(rho.get(2, 1).scale(0.5 * gamma1_q1)));
        }

        if gphi_q1 > 0.0 {
            let c12 = d_rho.get(1, 2);
            d_rho.set(1, 2, c12.sub(rho.get(1, 2).scale(gphi_q1)));
            let c21 = d_rho.get(2, 1);
            d_rho.set(2, 1, c21.sub(rho.get(2, 1).scale(gphi_q1)));
        }

        // 3. Decoherence from Qubit 2:
        let gamma1_q2 = self.bus.coupling2.qubit.relaxation_rate();
        let gphi_q2 = self.bus.coupling2.qubit.pure_dephasing_rate();

        // Relaxation: |ge>(1) -> |gg>(0), |ee>(3) -> |eg>(2)
        if gamma1_q2 > 0.0 {
            let cur_0 = d_rho.get(0, 0);
            d_rho.set(0, 0, cur_0.add(rho.get(1, 1).scale(gamma1_q2)));
            let cur_1 = d_rho.get(1, 1);
            d_rho.set(1, 1, cur_1.sub(rho.get(1, 1).scale(gamma1_q2)));

            let cur_2 = d_rho.get(2, 2);
            d_rho.set(2, 2, cur_2.add(rho.get(3, 3).scale(gamma1_q2)));
            let cur_3 = d_rho.get(3, 3);
            d_rho.set(3, 3, cur_3.sub(rho.get(3, 3).scale(gamma1_q2)));

            let c12 = d_rho.get(1, 2);
            d_rho.set(1, 2, c12.sub(rho.get(1, 2).scale(0.5 * gamma1_q2)));
            let c21 = d_rho.get(2, 1);
            d_rho.set(2, 1, c21.sub(rho.get(2, 1).scale(0.5 * gamma1_q2)));
        }

        if gphi_q2 > 0.0 {
            let c12 = d_rho.get(1, 2);
            d_rho.set(1, 2, c12.sub(rho.get(1, 2).scale(gphi_q2)));
            let c21 = d_rho.get(2, 1);
            d_rho.set(2, 1, c21.sub(rho.get(2, 1).scale(gphi_q2)));
        }

        d_rho
    }

    /// Single 4th-order Runge-Kutta step for two-qubit dynamics.
    pub fn rk4_step(&self, rho: &TwoQubitDensityMatrix, dt: f64) -> TwoQubitDensityMatrix {
        let k1 = self.lindblad_derivative(rho);

        let mut r2 = *rho;
        for i in 0..TWO_QUBIT_DIM * TWO_QUBIT_DIM {
            r2.data[i] = r2.data[i].add(k1.data[i].scale(0.5 * dt));
        }
        let k2 = self.lindblad_derivative(&r2);

        let mut r3 = *rho;
        for i in 0..TWO_QUBIT_DIM * TWO_QUBIT_DIM {
            r3.data[i] = r3.data[i].add(k2.data[i].scale(0.5 * dt));
        }
        let k3 = self.lindblad_derivative(&r3);

        let mut r4 = *rho;
        for i in 0..TWO_QUBIT_DIM * TWO_QUBIT_DIM {
            r4.data[i] = r4.data[i].add(k3.data[i].scale(dt));
        }
        let k4 = self.lindblad_derivative(&r4);

        let mut next = *rho;
        let c = dt / 6.0;
        for i in 0..TWO_QUBIT_DIM * TWO_QUBIT_DIM {
            let sum = k1.data[i]
                .add(k2.data[i].scale(2.0))
                .add(k3.data[i].scale(2.0))
                .add(k4.data[i]);
            next.data[i] = next.data[i].add(sum.scale(c));
        }

        next.hermitize_and_normalize();
        next
    }

    /// Simulates entanglement generation starting from $|eg\rangle$ to time $t_{bell} = \frac{\pi}{4 |J_{eff}|}$.
    pub fn simulate_bell_state_generation(
        &self,
        num_steps: usize,
    ) -> (f64, f64, TwoQubitDensityMatrix) {
        let t_bell = self.bus.bell_state_duration_seconds();
        let dt = t_bell / (num_steps.max(1) as f64);

        let mut rho = TwoQubitDensityMatrix::new_eg();
        for _ in 0..num_steps {
            rho = self.rk4_step(&rho, dt);
        }

        let fidelity = rho.bell_state_fidelity();
        let concurrence = rho.concurrence();
        (fidelity, concurrence, rho)
    }

    /// Evaluates the analytical Bell state generation fidelity:
    ///
    /// $$F_{bell} \approx \frac{1 + \exp\left(-\frac{(\gamma_2^{(1)} + \gamma_2^{(2)}) \pi}{4 |J_{eff}|}\right)}{2}$$
    pub fn analytical_bell_fidelity(&self) -> f64 {
        let j = self.bus.effective_exchange_coupling_rad_s().abs();
        if j <= 0.0 {
            return 0.5;
        }
        let g2_1 = self.bus.coupling1.qubit.total_dephasing_rate();
        let g2_2 = self.bus.coupling2.qubit.total_dephasing_rate();
        let exponent = (g2_1 + g2_2) * PI / (4.0 * j);
        let coh = (-exponent).exp();
        (0.5 * (1.0 + coh)).clamp(0.0, 1.0)
    }
}

/// Hong-Ou-Mandel Two-Phonon Interference and Routing Report.
#[derive(Debug, Clone, Copy)]
pub struct HomRoutingReport {
    /// Power transmission fraction $T$.
    pub transmission: f64,
    /// Power reflection fraction $R$.
    pub reflection: f64,
    /// Two-phonon coincidence probability $P_{coinc} = (T - R)^2$.
    pub coincidence_probability: f64,
    /// Hong-Ou-Mandel visibility $V_{HOM} = 1 - P_{coinc} = 4 T R / (T + R)^2$.
    pub hom_visibility: f64,
    /// Two-phonon bunching state amplitude $|2, 0\rangle$ or $|0, 2\rangle$.
    pub bunched_amplitude: f64,
}

/// Evaluates phononic routing and two-phonon Hong-Ou-Mandel interference through a SAW beam splitter.
pub fn evaluate_saw_beam_splitter(beam_splitter: &SawBeamSplitter) -> HomRoutingReport {
    let t = beam_splitter.power_transmission();
    let r = beam_splitter.power_reflection();
    let diff = t - r;
    let p_coinc = diff * diff;
    let v_hom = beam_splitter.hong_ou_mandel_visibility();
    let bunched_amp = (2.0 * t * r).sqrt();

    HomRoutingReport {
        transmission: t,
        reflection: r,
        coincidence_probability: p_coinc,
        hom_visibility: v_hom,
        bunched_amplitude: bunched_amp,
    }
}
