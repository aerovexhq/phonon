//! Topological Majorana zero mode (MZM) physics, Bogoliubov-de Gennes (BdG) parameters,
//! non-local fermion parity, and non-Abelian braiding transformations.

use std::f64::consts::PI;

/// Quantum parity of a topological fermionic mode: $P = i \gamma_1 \gamma_2 = \pm 1$.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FermionParity {
    /// Even parity mode ($P = +1$, empty fermion state $|0\rangle$).
    Even,
    /// Odd parity mode ($P = -1$, occupied fermion state $|1\rangle$).
    Odd,
}

impl FermionParity {
    /// Numerical sign value $+1.0$ or $-1.0$.
    pub fn sign_value(&self) -> f64 {
        match self {
            Self::Even => 1.0,
            Self::Odd => -1.0,
        }
    }

    /// Inverts the fermion parity upon single quasi-particle poisoning event.
    pub fn flip(&self) -> Self {
        match self {
            Self::Even => Self::Odd,
            Self::Odd => Self::Even,
        }
    }
}

/// Physical parameters of a semiconductor-superconductor topological nanowire (InAs/Al or InSb/Al).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalNanowireParams {
    /// Wire length $L$ in nanometers (typically $1000 - 3000\text{ nm}$).
    pub length_nm: f64,
    /// Induced superconducting pairing gap $\Delta_0$ in meV (typically $0.2 - 0.5\text{ meV}$).
    pub induced_gap_mev: f64,
    /// Rashba spin-orbit coupling energy $E_{SO} = m^* \alpha_R^2 / (2 \hbar^2)$ in meV.
    pub spin_orbit_energy_mev: f64,
    /// Zeeman energy $E_Z = \frac{1}{2} g^* \mu_B B$ in meV.
    pub zeeman_energy_mev: f64,
    /// Chemical potential $\mu$ relative to bottom of band in meV.
    pub chemical_potential_mev: f64,
    /// Quasi-particle poisoning rate $\Gamma_{qp}$ in $s^{-1}$ (Hz).
    pub poisoning_rate_hz: f64,
}

impl TopologicalNanowireParams {
    /// Standard InAs nanowire with epitaxial Aluminum shell parameters.
    pub fn inas_al_standard() -> Self {
        Self {
            length_nm: 2000.0,
            induced_gap_mev: 0.25,
            spin_orbit_energy_mev: 0.20,
            zeeman_energy_mev: 0.75, // B ~ 0.5-1.0 T with g* ~ 10-15
            chemical_potential_mev: 0.0,
            poisoning_rate_hz: 1000.0, // 1 ms poisoning lifetime
        }
    }

    /// Evaluates the topological criterion:
    /// $$E_Z > \sqrt{\Delta_0^2 + \mu^2}$$
    pub fn is_topological(&self) -> bool {
        let threshold = (self.induced_gap_mev.powi(2) + self.chemical_potential_mev.powi(2)).sqrt();
        self.zeeman_energy_mev > threshold
    }

    /// Effective topological gap $\Delta_{top}$ protecting the Majorana zero modes:
    /// $$\Delta_{top} \approx \Delta_0 \frac{E_{SO}}{\sqrt{E_Z^2 + E_{SO}^2}}$$
    pub fn effective_topological_gap_mev(&self) -> f64 {
        if !self.is_topological() {
            0.0
        } else {
            let denom =
                (self.zeeman_energy_mev.powi(2) + self.spin_orbit_energy_mev.powi(2)).sqrt();
            (self.induced_gap_mev * self.spin_orbit_energy_mev / denom.max(1e-6))
                .min(self.induced_gap_mev)
        }
    }

    /// Majorana localization length $\xi_M$ in nanometers:
    /// $$\xi_M \approx \frac{\hbar v_F}{\Delta_{top}}$$
    pub fn majorana_coherence_length_nm(&self) -> f64 {
        let gap = self.effective_topological_gap_mev();
        if gap <= 1e-6 {
            f64::INFINITY
        } else {
            // Typically ~ 100 - 250 nm for InAs/Al
            (15.0 / gap).clamp(50.0, 500.0)
        }
    }

    /// Majorana hybridization splitting energy $\delta E$ in meV due to finite wire length:
    /// $$\delta E \approx \Delta_{top} \exp(-L / \xi_M)$$
    pub fn hybridization_energy_mev(&self) -> f64 {
        let gap = self.effective_topological_gap_mev();
        let xi = self.majorana_coherence_length_nm();
        if xi.is_infinite() {
            0.0
        } else {
            gap * (-self.length_nm / xi).exp()
        }
    }

    /// Parity lifetime $\tau_P = 1 / \Gamma_{qp}$ in seconds.
    pub fn parity_lifetime_seconds(&self) -> f64 {
        1.0 / self.poisoning_rate_hz.max(1e-3)
    }
}

/// 4-Majorana topological qubit encoded in four Majorana zero modes $(\gamma_1, \gamma_2, \gamma_3, \gamma_4)$
/// with conserved total even fermion parity $P_{tot} = (i \gamma_1 \gamma_2)(i \gamma_3 \gamma_4) = +1$:
/// - Logical $|0_L\rangle$: $P_{12} = +1, P_{34} = +1$
/// - Logical $|1_L\rangle$: $P_{12} = -1, P_{34} = -1$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FourMajoranaQubit {
    /// Wire parameters.
    pub params: TopologicalNanowireParams,
    /// Complex probability amplitude for $|0_L\rangle$: $(\text{Re}, \text{Im})$.
    pub alpha: (f64, f64),
    /// Complex probability amplitude for $|1_L\rangle$: $(\text{Re}, \text{Im})$.
    pub beta: (f64, f64),
}

impl FourMajoranaQubit {
    /// Initializes qubit in logical $|0_L\rangle$ state.
    pub fn new_zero(params: TopologicalNanowireParams) -> Self {
        Self {
            params,
            alpha: (1.0, 0.0),
            beta: (0.0, 0.0),
        }
    }

    /// Initializes qubit in logical $|1_L\rangle$ state.
    pub fn new_one(params: TopologicalNanowireParams) -> Self {
        Self {
            params,
            alpha: (0.0, 0.0),
            beta: (1.0, 0.0),
        }
    }

    /// Normalizes state vector $|\psi\rangle = \alpha |0_L\rangle + \beta |1_L\rangle$.
    pub fn normalize(&mut self) {
        let norm_sq =
            self.alpha.0.powi(2) + self.alpha.1.powi(2) + self.beta.0.powi(2) + self.beta.1.powi(2);
        let norm = norm_sq.sqrt().max(1e-12);
        self.alpha.0 /= norm;
        self.alpha.1 /= norm;
        self.beta.0 /= norm;
        self.beta.1 /= norm;
    }

    /// Applies elementary braid operator $B_{12} = \exp(\frac{\pi}{4} \gamma_1 \gamma_2) = \frac{1}{\sqrt{2}}(1 + \gamma_1 \gamma_2)$.
    ///
    /// Acts as topological Phase gate $S = \begin{pmatrix} 1 & 0 \\ 0 & i \end{pmatrix}$ up to a global phase $e^{-i\pi/4}$.
    pub fn apply_braid_12(&mut self) {
        // Phase shift: alpha -> alpha, beta -> i * beta
        let new_beta_re = -self.beta.1;
        let new_beta_im = self.beta.0;
        self.beta = (new_beta_re, new_beta_im);
        self.normalize();
    }

    /// Applies elementary braid operator $B_{23} = \exp(\frac{\pi}{4} \gamma_2 \gamma_3) = \frac{1}{\sqrt{2}}(1 + \gamma_2 \gamma_3)$.
    ///
    /// Acts as topological Hadamard-like gate mixing parity sectors:
    /// $$\alpha' = \frac{1}{\sqrt{2}}(\alpha - i \beta), \quad \beta' = \frac{1}{\sqrt{2}}(-i \alpha + \beta)$$
    pub fn apply_braid_23(&mut self) {
        let inv_sqrt2 = 1.0 / std::f64::consts::SQRT_2;
        // alpha' = (alpha_re + beta_im, alpha_im - beta_re) / sqrt(2)
        let a_re = inv_sqrt2 * (self.alpha.0 + self.beta.1);
        let a_im = inv_sqrt2 * (self.alpha.1 - self.beta.0);
        // beta' = (beta_re + alpha_im, beta_im - alpha_re) / sqrt(2)
        let b_re = inv_sqrt2 * (self.beta.0 + self.alpha.1);
        let b_im = inv_sqrt2 * (self.beta.1 - self.alpha.0);

        self.alpha = (a_re, a_im);
        self.beta = (b_re, b_im);
        self.normalize();
    }

    /// Gate fidelity relative to target state $|\psi_{tgt}\rangle = (a_t, b_t)$:
    /// $$F = |\langle \psi_{tgt} | \psi \rangle|^2$$
    pub fn fidelity_to(&self, target_alpha: (f64, f64), target_beta: (f64, f64)) -> f64 {
        let inner_re = self.alpha.0 * target_alpha.0
            + self.alpha.1 * target_alpha.1
            + self.beta.0 * target_beta.0
            + self.beta.1 * target_beta.1;
        let inner_im = self.alpha.0 * (-target_alpha.1)
            + self.alpha.1 * target_alpha.0
            + self.beta.0 * (-target_beta.1)
            + self.beta.1 * target_beta.0;
        inner_re.powi(2) + inner_im.powi(2)
    }

    /// Diabatic error probability $\epsilon_{diab}$ during finite-duration braid time $\tau_{braid}$:
    /// $$\epsilon_{diab} \approx \exp\left( - \frac{2\pi \Delta_{top} \tau_{braid}}{\hbar} \right)$$
    pub fn diabatic_error(&self, braid_time_seconds: f64) -> f64 {
        let hbar_ev = 6.582_119_569e-16; // eV * s
        let gap_ev = self.params.effective_topological_gap_mev() * 1.0e-3;
        let exponent = (2.0 * PI * gap_ev * braid_time_seconds) / hbar_ev;
        (-exponent.min(100.0)).exp()
    }
}
