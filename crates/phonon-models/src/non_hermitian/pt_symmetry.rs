//! Parity-Time (PT) Symmetry, Exceptional Points, and Non-Hermitian Degeneracies.
//!
//! Formulates:
//! - Balanced two-mode PT-symmetric coupled Hamiltonian:
//!   $$H_{PT} = \begin{pmatrix} \omega_0 + i \gamma & \kappa \\ \kappa & \omega_0 - i \gamma \end{pmatrix}$$
//! - Complex eigenfrequencies:
//!   $$\omega_\pm = \omega_0 \pm \sqrt{\kappa^2 - \gamma^2}$$
//! - Exceptional point (EP) degeneracy:
//!   $$\gamma = \kappa \implies \omega_+ = \omega_- = \omega_0, \quad \mathbf{v}_{EP} \propto \begin{pmatrix} 1 \\ i \end{pmatrix}$$
//! - Petermann excess noise factor:
//!   $$K = \frac{\langle v_L | v_L \rangle \langle v_R | v_R \rangle}{|\langle v_L | v_R \rangle|^2} = \frac{1}{1 - (\gamma / \kappa)^2} \to \infty \quad \text{as } \gamma \to \kappa$$

/// Phase regime of a PT-symmetric system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtPhaseRegime {
    /// Exact PT-symmetric phase: real eigenvalues, balanced neutral gain/loss oscillations ($\gamma < \kappa$).
    Exact,
    /// Exceptional point (EP): non-diagonalizable degeneracy where eigenvalues and eigenvectors coalesce ($\gamma = \kappa$).
    ExceptionalPoint,
    /// Broken PT-symmetric phase: complex conjugate eigenvalues with net exponential growth and decay ($\gamma > \kappa$).
    Broken,
}

/// Parameters for a two-mode PT-symmetric non-Hermitian coupled dimer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PtDimerParams {
    /// Bare optical resonance frequency $\omega_0 / (2\pi)$ in Hz.
    pub bare_frequency_hz: f64,
    /// Inter-cavity evanescent coupling rate $\kappa / (2\pi)$ in Hz.
    pub coupling_kappa_hz: f64,
    /// Gain and loss rate $\gamma / (2\pi)$ in Hz ($+i\gamma$ on cavity 1, $-i\gamma$ on cavity 2).
    pub gain_loss_gamma_hz: f64,
}

impl PtDimerParams {
    /// Constructs PT dimer parameters.
    pub fn new(bare_frequency_hz: f64, coupling_kappa_hz: f64, gain_loss_gamma_hz: f64) -> Self {
        Self {
            bare_frequency_hz,
            coupling_kappa_hz,
            gain_loss_gamma_hz,
        }
    }

    /// Standard microring dimer configured at the threshold of the exceptional point:
    /// $\omega_0 = 193.4\text{ THz}$, $\kappa = 20.0\text{ GHz}$, $\gamma = 20.0\text{ GHz}$.
    pub fn standard_exceptional_point_dimer() -> Self {
        Self::new(193.4e12, 20.0e9, 20.0e9)
    }

    /// Identifies the PT phase regime based on the gain-loss to coupling ratio $\gamma / \kappa$.
    pub fn phase_regime(&self) -> PtPhaseRegime {
        let diff = self.coupling_kappa_hz - self.gain_loss_gamma_hz;
        if diff.abs() < 1e-6 * self.coupling_kappa_hz.max(1.0) {
            PtPhaseRegime::ExceptionalPoint
        } else if self.gain_loss_gamma_hz < self.coupling_kappa_hz {
            PtPhaseRegime::Exact
        } else {
            PtPhaseRegime::Broken
        }
    }

    /// Computes the complex eigenfrequencies $(\text{Re}(\omega_\pm), \text{Im}(\omega_\pm))$ in Hz:
    /// $$\omega_\pm = \omega_0 \pm \sqrt{\kappa^2 - \gamma^2}$$
    pub fn eigenfrequencies_hz(&self) -> ((f64, f64), (f64, f64)) {
        let w0 = self.bare_frequency_hz;
        let kappa = self.coupling_kappa_hz;
        let gamma = self.gain_loss_gamma_hz;

        let delta_sq = kappa * kappa - gamma * gamma;
        if delta_sq >= 0.0 {
            let split = delta_sq.sqrt();
            // Both modes have zero net gain/loss in exact phase
            ((w0 + split, 0.0), (w0 - split, 0.0))
        } else {
            let imag_split = (-delta_sq).sqrt();
            // Split into amplified and decayed modes in broken phase
            ((w0, imag_split), (w0, -imag_split))
        }
    }

    /// Petermann excess noise factor $K = \frac{1}{1 - (\gamma / \kappa)^2}$:
    /// Diverges to infinity at the exceptional point due to eigenvector non-orthogonality.
    pub fn petermann_factor(&self) -> f64 {
        let ratio = (self.gain_loss_gamma_hz / self.coupling_kappa_hz.max(1.0)).min(0.9999);
        let denom = 1.0 - ratio * ratio;
        1.0 / denom.max(1e-6)
    }

    /// Returns the normalized right eigenvectors $[v_{R+}, v_{R-}]$ as complex 2-vectors:
    #[allow(clippy::type_complexity)]
    pub fn right_eigenvectors(&self) -> ([(f64, f64); 2], [(f64, f64); 2]) {
        let kappa = self.coupling_kappa_hz;
        let gamma = self.gain_loss_gamma_hz;
        let delta_sq = kappa * kappa - gamma * gamma;

        if delta_sq >= 0.0 {
            let cos_theta = (delta_sq.sqrt() / kappa.max(1.0)).clamp(-1.0, 1.0);
            let sin_theta = gamma / kappa.max(1.0);
            let norm = 2.0_f64.sqrt();
            (
                [(1.0 / norm, 0.0), (cos_theta / norm, sin_theta / norm)],
                [(1.0 / norm, 0.0), (-cos_theta / norm, sin_theta / norm)],
            )
        } else {
            // At and beyond EP
            let norm = 2.0_f64.sqrt();
            (
                [(1.0 / norm, 0.0), (0.0, 1.0 / norm)],
                [(1.0 / norm, 0.0), (0.0, -1.0 / norm)],
            )
        }
    }
}
