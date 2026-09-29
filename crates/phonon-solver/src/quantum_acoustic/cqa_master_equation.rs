#![deny(unsafe_code)]

//! Quantum Acoustic Circuit QED (cQAD) Master Equation Solver.
//!
//! Simulates the open-system quantum dynamics of a superconducting transmon qubit
//! strongly coupled to a single localized Surface Acoustic Wave (SAW) cavity mode
//! using the Jaynes-Cummings Hamiltonian with Lindblad dissipation.

use phonon_models::quantum_acoustic::SawQubitCoupling;
use std::f64::consts::PI;

/// Maximum number of phononic Fock states tracked ($n \in \{0, 1, 2, 3\}$).
pub const FOCK_DIM: usize = 4;
/// Total composite Hilbert space dimension ($2 \text{ qubit states} \times 4 \text{ Fock states} = 8$).
pub const HILBERT_DIM: usize = 2 * FOCK_DIM;

/// Lightweight double-precision complex number for quantum state representation.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Complex64 {
    /// Real part.
    pub re: f64,
    /// Imaginary part.
    pub im: f64,
}

impl Complex64 {
    /// Zero complex scalar.
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    /// Unit real complex scalar.
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    /// Imaginary unit $i$.
    pub const I: Self = Self { re: 0.0, im: 1.0 };

    /// Creates a new complex number $re + i im$.
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    /// Squared complex magnitude $|z|^2 = \text{re}^2 + \text{im}^2$.
    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// Complex modulus $|z| = \sqrt{\text{re}^2 + \text{im}^2}$.
    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    /// Complex conjugate $z^* = \text{re} - i \text{im}$.
    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    /// Complex addition.
    pub fn add(&self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    /// Complex subtraction.
    pub fn sub(&self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }

    /// Complex multiplication.
    pub fn mul(&self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    /// Scale by real number.
    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }
}

/// Composite Qubit-Phonon Density Matrix $\rho \in \mathbb{C}^{8 \times 8}$.
///
/// Basis ordering: $\text{idx}(q, n) = q \times 4 + n$ where:
/// - $q = 0 \implies |g\rangle$ (qubit ground state)
/// - $q = 1 \implies |e\rangle$ (qubit excited state)
/// - $n \in \{0, 1, 2, 3\}$ (phonon Fock states $|n\rangle$)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QubitPhononDensityMatrix {
    /// Density matrix elements in row-major format ($8 \times 8 = 64$ elements).
    pub data: [Complex64; HILBERT_DIM * HILBERT_DIM],
}

impl Default for QubitPhononDensityMatrix {
    fn default() -> Self {
        Self {
            data: [Complex64::ZERO; HILBERT_DIM * HILBERT_DIM],
        }
    }
}

impl QubitPhononDensityMatrix {
    /// Maps $(q, n)$ to flat index $\in [0, 7]$.
    #[inline]
    pub fn state_index(q: usize, n: usize) -> usize {
        debug_assert!(q < 2 && n < FOCK_DIM);
        q * FOCK_DIM + n
    }

    /// Gets matrix element $\rho_{i, j}$.
    #[inline]
    pub fn get(&self, i: usize, j: usize) -> Complex64 {
        self.data[i * HILBERT_DIM + j]
    }

    /// Sets matrix element $\rho_{i, j}$.
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, val: Complex64) {
        self.data[i * HILBERT_DIM + j] = val;
    }

    /// Initializes in pure excited-qubit, vacuum-phonon state $|e, 0\rangle$.
    pub fn new_excited_vacuum() -> Self {
        let mut dm = Self::default();
        let idx = Self::state_index(1, 0);
        dm.set(idx, idx, Complex64::ONE);
        dm
    }

    /// Initializes in pure ground-qubit, single-phonon Fock state $|g, 1\rangle$.
    pub fn new_ground_single_phonon() -> Self {
        let mut dm = Self::default();
        let idx = Self::state_index(0, 1);
        dm.set(idx, idx, Complex64::ONE);
        dm
    }

    /// Trace of the density matrix $\text{Tr}(\rho) = \sum_i \rho_{ii}$.
    pub fn trace(&self) -> Complex64 {
        let mut tr = Complex64::ZERO;
        for i in 0..HILBERT_DIM {
            tr = tr.add(self.get(i, i));
        }
        tr
    }

    /// Quantum state purity $\mathcal{P} = \text{Tr}(\rho^2) \in [1/8, 1.0]$.
    pub fn purity(&self) -> f64 {
        let mut p = 0.0;
        for i in 0..HILBERT_DIM {
            for j in 0..HILBERT_DIM {
                let elem = self.get(i, j);
                p += elem.norm_sq();
            }
        }
        p.clamp(0.0, 1.0)
    }

    /// Enforces Hermiticity $\rho \leftarrow \frac{1}{2}(\rho + \rho^\dagger)$ and normalizes trace to 1.
    pub fn hermitize_and_normalize(&mut self) {
        for i in 0..HILBERT_DIM {
            for j in i..HILBERT_DIM {
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

        let tr = self.trace().re;
        if tr > 1e-12 {
            let inv_tr = 1.0 / tr;
            for elem in self.data.iter_mut() {
                *elem = elem.scale(inv_tr);
            }
        }
    }

    /// Qubit excited state probability $P_e = \sum_{n} \rho_{(e,n), (e,n)}$.
    pub fn qubit_excited_probability(&self) -> f64 {
        let mut p = 0.0;
        for n in 0..FOCK_DIM {
            let idx = Self::state_index(1, n);
            p += self.get(idx, idx).re;
        }
        p.clamp(0.0, 1.0)
    }

    /// Average phonon occupation number $\langle n \rangle = \text{Tr}(a^\dagger a \rho) = \sum_{q, n} n \rho_{(q,n), (q,n)}$.
    pub fn average_phonon_number(&self) -> f64 {
        let mut n_avg = 0.0;
        for q in 0..2 {
            for n in 0..FOCK_DIM {
                let idx = Self::state_index(q, n);
                n_avg += (n as f64) * self.get(idx, idx).re;
            }
        }
        n_avg.max(0.0)
    }

    /// Fidelity relative to target state $|g, 1\rangle$: $F_{swap} = \text{Re}(\rho_{(g,1), (g,1)})$.
    pub fn fock1_fidelity(&self) -> f64 {
        let idx = Self::state_index(0, 1);
        self.get(idx, idx).re.clamp(0.0, 1.0)
    }
}

/// Quantum Master Equation Solver for Transmon-Cavity (cQAD) System.
#[derive(Debug, Clone, Copy)]
pub struct CqaMasterEquationSolver {
    /// Qubit-cavity coupling system parameters.
    pub coupling: SawQubitCoupling,
    /// Pure dephasing rate of the phononic cavity mode $\kappa_\phi$ in s^-1.
    pub phonon_dephasing_rate: f64,
}

impl CqaMasterEquationSolver {
    /// Creates a new cQAD master equation solver.
    pub fn new(coupling: SawQubitCoupling, phonon_dephasing_rate: f64) -> Self {
        Self {
            coupling,
            phonon_dephasing_rate,
        }
    }

    /// Evaluates the Lindbladian generator $\frac{d\rho}{dt} = -i [H / \hbar, \rho] + \sum_k \mathcal{D}[C_k]\rho$
    /// in the frame rotating at the cavity frequency $\omega_m$.
    pub fn lindblad_derivative(&self, rho: &QubitPhononDensityMatrix) -> QubitPhononDensityMatrix {
        let mut d_rho = QubitPhononDensityMatrix::default();

        let delta = self.coupling.detuning_rad_s();
        let g = self.coupling.coupling_rate_rad_s();

        // 1. Commutator -i [H/hbar, rho] = -i (H/hbar * rho - rho * H/hbar)
        // Matrix elements of H / hbar:
        // H_{(1, n), (1, n)} = delta
        // H_{(1, n), (0, n+1)} = g * sqrt(n + 1)
        // H_{(0, n+1), (1, n)} = g * sqrt(n + 1)

        // Compute H_eff * rho
        let mut h_rho = [Complex64::ZERO; HILBERT_DIM * HILBERT_DIM];
        for n in 0..FOCK_DIM {
            let e_n = QubitPhononDensityMatrix::state_index(1, n);
            let g_n = QubitPhononDensityMatrix::state_index(0, n);

            for col in 0..HILBERT_DIM {
                // e_n row: delta * rho[e_n, col] + (if n+1 < FOCK_DIM: g*sqrt(n+1) * rho[g_(n+1), col])
                let mut sum_e = Complex64::new(delta, 0.0).mul(rho.get(e_n, col));
                if n + 1 < FOCK_DIM {
                    let g_np1 = QubitPhononDensityMatrix::state_index(0, n + 1);
                    let coup = Complex64::new(g * ((n + 1) as f64).sqrt(), 0.0);
                    sum_e = sum_e.add(coup.mul(rho.get(g_np1, col)));
                }
                h_rho[e_n * HILBERT_DIM + col] = sum_e;

                // g_n row: (if n > 0: g*sqrt(n) * rho[e_(n-1), col])
                let mut sum_g = Complex64::ZERO;
                if n > 0 {
                    let e_nm1 = QubitPhononDensityMatrix::state_index(1, n - 1);
                    let coup = Complex64::new(g * (n as f64).sqrt(), 0.0);
                    sum_g = sum_g.add(coup.mul(rho.get(e_nm1, col)));
                }
                h_rho[g_n * HILBERT_DIM + col] = sum_g;
            }
        }

        // -i (H * rho - (H * rho)^dagger) using H = H^dagger
        for i in 0..HILBERT_DIM {
            for j in 0..HILBERT_DIM {
                let hr = h_rho[i * HILBERT_DIM + j];
                let rh = h_rho[j * HILBERT_DIM + i].conj();
                let comm = hr.sub(rh);
                // -i * (comm) = (comm.im, -comm.re)
                let term = Complex64::new(comm.im, -comm.re);
                d_rho.set(i, j, term);
            }
        }

        // 2. Lindblad Dissipator: Cavity Phonon Loss D[sqrt(kappa) a] rho
        // a |q, n> = sqrt(n) |q, n-1>
        let kappa = self.coupling.cavity_decay_rate;
        if kappa > 0.0 {
            for q1 in 0..2 {
                for q2 in 0..2 {
                    for n1 in 0..FOCK_DIM {
                        for n2 in 0..FOCK_DIM {
                            let idx1 = QubitPhononDensityMatrix::state_index(q1, n1);
                            let idx2 = QubitPhononDensityMatrix::state_index(q2, n2);

                            // Recycling term: kappa * a rho a^\dagger => kappa * sqrt((n1+1)(n2+1)) rho_{(q1, n1+1), (q2, n2+1)}
                            let mut val = Complex64::ZERO;
                            if n1 + 1 < FOCK_DIM && n2 + 1 < FOCK_DIM {
                                let src1 = QubitPhononDensityMatrix::state_index(q1, n1 + 1);
                                let src2 = QubitPhononDensityMatrix::state_index(q2, n2 + 1);
                                let factor = kappa * (((n1 + 1) * (n2 + 1)) as f64).sqrt();
                                val = val.add(rho.get(src1, src2).scale(factor));
                            }

                            // Damping term: -0.5 * kappa * (a^\dagger a rho + rho a^\dagger a)
                            // a^\dagger a |n> = n |n>
                            let damp = 0.5 * kappa * ((n1 + n2) as f64);
                            val = val.sub(rho.get(idx1, idx2).scale(damp));

                            let current = d_rho.get(idx1, idx2);
                            d_rho.set(idx1, idx2, current.add(val));
                        }
                    }
                }
            }
        }

        // 3. Lindblad Dissipator: Qubit Energy Relaxation D[sqrt(gamma1) sigma_-] rho
        // sigma_- |e, n> = |g, n>, sigma_- |g, n> = 0
        let gamma1 = self.coupling.qubit.relaxation_rate();
        if gamma1 > 0.0 {
            for n1 in 0..FOCK_DIM {
                for n2 in 0..FOCK_DIM {
                    // Recycling into ground state: gamma1 * rho[(e, n1), (e, n2)] into (g, n1), (g, n2)
                    let g_n1 = QubitPhononDensityMatrix::state_index(0, n1);
                    let g_n2 = QubitPhononDensityMatrix::state_index(0, n2);
                    let e_n1 = QubitPhononDensityMatrix::state_index(1, n1);
                    let e_n2 = QubitPhononDensityMatrix::state_index(1, n2);

                    let recycle = rho.get(e_n1, e_n2).scale(gamma1);
                    let cur_g = d_rho.get(g_n1, g_n2);
                    d_rho.set(g_n1, g_n2, cur_g.add(recycle));

                    // Damping of excited state elements
                    let cur_e = d_rho.get(e_n1, e_n2);
                    d_rho.set(e_n1, e_n2, cur_e.sub(rho.get(e_n1, e_n2).scale(gamma1)));

                    // Damping of coherence elements between e and g: -0.5 * gamma1 * rho
                    let eg = d_rho.get(e_n1, g_n2);
                    d_rho.set(e_n1, g_n2, eg.sub(rho.get(e_n1, g_n2).scale(0.5 * gamma1)));

                    let ge = d_rho.get(g_n1, e_n2);
                    d_rho.set(g_n1, e_n2, ge.sub(rho.get(g_n1, e_n2).scale(0.5 * gamma1)));
                }
            }
        }

        // 4. Lindblad Dissipator: Qubit Pure Dephasing D[sqrt(gamma_phi/2) sigma_z] rho
        // Causes exponential decay of off-diagonal qubit coherences at rate gamma_phi
        let gamma_phi = self.coupling.qubit.pure_dephasing_rate();
        if gamma_phi > 0.0 {
            for n1 in 0..FOCK_DIM {
                for n2 in 0..FOCK_DIM {
                    let e_n1 = QubitPhononDensityMatrix::state_index(1, n1);
                    let g_n2 = QubitPhononDensityMatrix::state_index(0, n2);
                    let g_n1 = QubitPhononDensityMatrix::state_index(0, n1);
                    let e_n2 = QubitPhononDensityMatrix::state_index(1, n2);

                    let eg = d_rho.get(e_n1, g_n2);
                    d_rho.set(e_n1, g_n2, eg.sub(rho.get(e_n1, g_n2).scale(gamma_phi)));

                    let ge = d_rho.get(g_n1, e_n2);
                    d_rho.set(g_n1, e_n2, ge.sub(rho.get(g_n1, e_n2).scale(gamma_phi)));
                }
            }
        }

        d_rho
    }

    /// Single 4th-order Runge-Kutta numerical integration step for $\rho(t) \to \rho(t + \Delta t)$.
    pub fn rk4_step(&self, rho: &QubitPhononDensityMatrix, dt: f64) -> QubitPhononDensityMatrix {
        let k1 = self.lindblad_derivative(rho);

        // rho + 0.5 * dt * k1
        let mut r2 = *rho;
        for i in 0..HILBERT_DIM * HILBERT_DIM {
            r2.data[i] = r2.data[i].add(k1.data[i].scale(0.5 * dt));
        }
        let k2 = self.lindblad_derivative(&r2);

        // rho + 0.5 * dt * k2
        let mut r3 = *rho;
        for i in 0..HILBERT_DIM * HILBERT_DIM {
            r3.data[i] = r3.data[i].add(k2.data[i].scale(0.5 * dt));
        }
        let k3 = self.lindblad_derivative(&r3);

        // rho + dt * k3
        let mut r4 = *rho;
        for i in 0..HILBERT_DIM * HILBERT_DIM {
            r4.data[i] = r4.data[i].add(k3.data[i].scale(dt));
        }
        let k4 = self.lindblad_derivative(&r4);

        let mut next = *rho;
        let c = dt / 6.0;
        for i in 0..HILBERT_DIM * HILBERT_DIM {
            let sum = k1.data[i]
                .add(k2.data[i].scale(2.0))
                .add(k3.data[i].scale(2.0))
                .add(k4.data[i]);
            next.data[i] = next.data[i].add(sum.scale(c));
        }

        next.hermitize_and_normalize();
        next
    }

    /// Simulates the resonant SWAP gate transferring a single qubit excitation $|e, 0\rangle \to |g, 1\rangle$.
    ///
    /// Evaluates the final Fock state fidelity $F_{swap}$ at $t = t_{swap} = \frac{\pi}{2g}$.
    pub fn simulate_swap_gate(&self, num_steps: usize) -> (f64, QubitPhononDensityMatrix) {
        let t_swap = self.coupling.swap_duration_seconds();
        let dt = t_swap / (num_steps.max(1) as f64);

        let mut rho = QubitPhononDensityMatrix::new_excited_vacuum();
        for _ in 0..num_steps {
            rho = self.rk4_step(&rho, dt);
        }

        let f_swap = rho.fock1_fidelity();
        (f_swap, rho)
    }

    /// Evaluates the analytical vacuum Rabi SWAP fidelity incorporating acoustic cavity
    /// decay rate $\kappa$, qubit relaxation $\gamma_1$, and total dephasing $\gamma_2$:
    ///
    /// $$F_{swap} \approx \exp\left(-\frac{(\kappa + \gamma_1) \pi}{4 g}\right)$$
    pub fn analytical_swap_fidelity(&self) -> f64 {
        let g = self.coupling.coupling_rate_rad_s();
        if g <= 0.0 {
            return 0.0;
        }
        let kappa = self.coupling.cavity_decay_rate;
        let gamma1 = self.coupling.qubit.relaxation_rate();
        let exponent = (kappa + gamma1) * PI / (4.0 * g);
        (-exponent).exp().clamp(0.0, 1.0)
    }
}
