//! Non-Hermitian Su-Schrieffer-Heeger (SSH) and topological photonic lattices.
//!
//! Formulates:
//! - 1D SSH dimer lattice with alternating hoppings $t_1, t_2$ and on-site gain/loss $\pm i\gamma$:
//!   $$H_{SSH}(k) = \begin{pmatrix} i\gamma & t_1 + t_2 e^{-i k a} \\ t_1 + t_2 e^{i k a} & -i\gamma \end{pmatrix}$$
//! - Bulk topological energy bandgap:
//!   $$\Delta_{gap} = 2 |t_2 - t_1|$$
//! - Non-Hermitian winding number:
//!   $$w = \frac{1}{2\pi} \oint_{BZ} \nabla_k \arg(\det H(k)) dk$$
//! - Localized topological zero-energy edge state on lattice boundaries with decay length $\xi = \frac{a}{\ln(t_2 / t_1)}$.

/// Topological phase classification for an SSH lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TopologicalLatticePhase {
    /// Trivial insulator phase ($t_1 > t_2$, winding number $w = 0$, no edge states).
    Trivial,
    /// Topological insulator phase ($t_2 > t_1$, winding number $w = 1$, robust edge states).
    Topological,
    /// Metallic gapless transition point ($t_1 = t_2$).
    GaplessTransition,
}

/// Physical parameters for a 1D non-Hermitian SSH photonic lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct SshLatticeParams {
    /// Number of unit cells $N$ ($2N$ total resonators / sites).
    pub num_unit_cells: usize,
    /// Intracell evanescent coupling rate $t_1 / (2\pi)$ in Hz.
    pub intracell_coupling_t1_hz: f64,
    /// Intercell evanescent coupling rate $t_2 / (2\pi)$ in Hz.
    pub intercell_coupling_t2_hz: f64,
    /// Sublattice gain/loss parameter $\gamma / (2\pi)$ in Hz ($+i\gamma$ on A, $-i\gamma$ on B).
    pub gain_loss_gamma_hz: f64,
    /// Unperturbed cavity resonance frequency $\omega_0 / (2\pi)$ in Hz.
    pub resonance_frequency_hz: f64,
}

impl SshLatticeParams {
    /// Constructs SSH lattice parameters.
    pub fn new(
        num_unit_cells: usize,
        intracell_coupling_t1_hz: f64,
        intercell_coupling_t2_hz: f64,
        gain_loss_gamma_hz: f64,
        resonance_frequency_hz: f64,
    ) -> Self {
        Self {
            num_unit_cells,
            intracell_coupling_t1_hz,
            intercell_coupling_t2_hz,
            gain_loss_gamma_hz,
            resonance_frequency_hz,
        }
    }

    /// Standard topological laser lattice:
    /// $N = 10$ unit cells (20 sites), $t_1 = 10\text{ GHz}, t_2 = 30\text{ GHz}$ (topological regime),
    /// $\gamma = 2.0\text{ GHz}$, $\omega_0 = 193.4\text{ THz}$.
    pub fn standard_topological_laser_lattice() -> Self {
        Self::new(10, 10.0e9, 30.0e9, 2.0e9, 193.4e12)
    }

    /// Determines the topological phase.
    pub fn phase(&self) -> TopologicalLatticePhase {
        let diff = self.intercell_coupling_t2_hz - self.intracell_coupling_t1_hz;
        if diff.abs() < 1e-6 * self.intercell_coupling_t2_hz.max(1.0) {
            TopologicalLatticePhase::GaplessTransition
        } else if self.intercell_coupling_t2_hz > self.intracell_coupling_t1_hz {
            TopologicalLatticePhase::Topological
        } else {
            TopologicalLatticePhase::Trivial
        }
    }

    /// Bulk topological bandgap $\Delta_{gap} = 2 |t_2 - t_1|$ in Hz.
    pub fn topological_bandgap_hz(&self) -> f64 {
        2.0 * (self.intercell_coupling_t2_hz - self.intracell_coupling_t1_hz).abs()
    }

    /// Edge state localization length $\xi / a = \frac{1}{\ln(t_2 / t_1)}$ in unit cells:
    pub fn edge_state_localization_length(&self) -> f64 {
        if self.intercell_coupling_t2_hz <= self.intracell_coupling_t1_hz {
            f64::INFINITY
        } else {
            1.0 / (self.intercell_coupling_t2_hz / self.intracell_coupling_t1_hz).ln()
        }
    }

    /// Assembles the $2N \times 2N$ tridiagonal non-Hermitian Hamiltonian matrix in real space:
    /// Diagonal elements: $\omega_0 + i\gamma$ on A sites, $\omega_0 - i\gamma$ on B sites.
    /// Off-diagonal elements: alternating $t_1$ and $t_2$.
    #[allow(clippy::needless_range_loop)]
    pub fn assemble_real_space_hamiltonian(&self) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let num_sites = 2 * self.num_unit_cells;
        let diag_re = vec![self.resonance_frequency_hz; num_sites];
        let mut diag_im = vec![0.0; num_sites];
        let mut off_diag = vec![0.0; num_sites.saturating_sub(1)];

        for site in 0..num_sites {
            if site % 2 == 0 {
                // Sublattice A (Gain)
                diag_im[site] = self.gain_loss_gamma_hz;
            } else {
                // Sublattice B (Loss)
                diag_im[site] = -self.gain_loss_gamma_hz;
            }
        }

        for i in 0..off_diag.len() {
            if i % 2 == 0 {
                off_diag[i] = self.intracell_coupling_t1_hz;
            } else {
                off_diag[i] = self.intercell_coupling_t2_hz;
            }
        }

        (diag_re, diag_im, off_diag)
    }

    /// Evaluates the theoretical edge mode localization ratio (edge site intensity vs bulk average):
    pub fn theoretical_edge_to_bulk_ratio(&self) -> f64 {
        if self.phase() != TopologicalLatticePhase::Topological {
            return 1.0;
        }
        let ratio = self.intercell_coupling_t2_hz / self.intracell_coupling_t1_hz.max(1.0);
        (ratio * ratio).powi(2).max(10.0)
    }
}
