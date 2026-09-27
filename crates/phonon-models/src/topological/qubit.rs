//! 4-Majorana fault-tolerant topological qubit encoding and Clifford logic.
//!
//! Encodes a logical qubit into four localized Majorana Zero Modes $(\gamma_1, \gamma_2, \gamma_3, \gamma_4)$
//! in the parity-conserving even subspace ($P_{tot} = +1$).
//! Generates exact topologically protected Clifford gates $(S, H, X, Z)$ via non-Abelian braiding.

/// Two-component complex state vector $(\alpha, \beta)$ for a single logical qubit:
/// $|\psi_L\rangle = \alpha |0_L\rangle + \beta |1_L\rangle$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QubitState {
    pub alpha_re: f64,
    pub alpha_im: f64,
    pub beta_re: f64,
    pub beta_im: f64,
}

impl Default for QubitState {
    fn default() -> Self {
        Self {
            alpha_re: 1.0,
            alpha_im: 0.0,
            beta_re: 0.0,
            beta_im: 0.0,
        }
    }
}

impl QubitState {
    /// Constructs a state from complex components and normalizes.
    pub fn new(alpha_re: f64, alpha_im: f64, beta_re: f64, beta_im: f64) -> Self {
        let mut s = Self {
            alpha_re,
            alpha_im,
            beta_re,
            beta_im,
        };
        s.normalize();
        s
    }

    /// Normalizes the state vector so that $|\alpha|^2 + |\beta|^2 = 1.0$.
    pub fn normalize(&mut self) {
        let norm_sq = self.alpha_re * self.alpha_re
            + self.alpha_im * self.alpha_im
            + self.beta_re * self.beta_re
            + self.beta_im * self.beta_im;

        if norm_sq > 1e-18 {
            let inv_norm = 1.0 / norm_sq.sqrt();
            self.alpha_re *= inv_norm;
            self.alpha_im *= inv_norm;
            self.beta_re *= inv_norm;
            self.beta_im *= inv_norm;
        } else {
            self.alpha_re = 1.0;
            self.alpha_im = 0.0;
            self.beta_re = 0.0;
            self.beta_im = 0.0;
        }
    }

    /// Probability of measuring $|0_L\rangle$: $P(0) = |\alpha|^2$.
    #[inline]
    pub fn prob_0(&self) -> f64 {
        self.alpha_re * self.alpha_re + self.alpha_im * self.alpha_im
    }

    /// Probability of measuring $|1_L\rangle$: $P(1) = |\beta|^2$.
    #[inline]
    pub fn prob_1(&self) -> f64 {
        self.beta_re * self.beta_re + self.beta_im * self.beta_im
    }

    /// Quantum state fidelity $|\langle \psi | \phi \rangle|^2$ between two pure states.
    pub fn fidelity(&self, other: &Self) -> f64 {
        // <psi | phi> = alpha_psi^* * alpha_phi + beta_psi^* * beta_phi
        let inner_re = (self.alpha_re * other.alpha_re + self.alpha_im * other.alpha_im)
            + (self.beta_re * other.beta_re + self.beta_im * other.beta_im);
        let inner_im = (self.alpha_re * other.alpha_im - self.alpha_im * other.alpha_re)
            + (self.beta_re * other.beta_im - self.beta_im * other.beta_re);

        inner_re * inner_re + inner_im * inner_im
    }
}

/// Fault-tolerant topological qubit realized by 4 Majorana bound states.
#[derive(Debug, Clone, PartialEq)]
pub struct TopologicalQubit {
    /// Active state vector in computational basis $(|0_L\rangle, |1_L\rangle)$.
    pub state: QubitState,
    /// Total conserved fermion parity $P_{tot} = P_{12} P_{34} \in \{+1, -1\}$.
    pub total_parity: i8,
    /// Cumulative count of completed braiding operations.
    pub braid_count: u64,
}

impl Default for TopologicalQubit {
    fn default() -> Self {
        Self {
            state: QubitState::default(), // |0_L>
            total_parity: 1,              // Even parity
            braid_count: 0,
        }
    }
}

impl TopologicalQubit {
    /// Creates a new topological qubit initialized to $|0_L\rangle$.
    pub fn new() -> Self {
        Self::default()
    }

    /// Applies the non-Abelian braiding operator $B_{12} = \exp(\frac{\pi}{4} \gamma_2 \gamma_1)$.
    /// In the even parity basis, this implements the Phase Gate $S = \text{diag}(1, i)$ (up to global phase $e^{i\pi/4}$).
    pub fn apply_braid_12(&mut self) {
        // B_12 = 1/sqrt(2) * (I + i Z_L)
        // Action on |0>: 1/sqrt(2) * (1 + i) = e^{i pi/4} |0>
        // Action on |1>: 1/sqrt(2) * (1 - i) = e^{-i pi/4} |1>
        // Relative phase between |0> and |1> is +pi/2, which is the Clifford S gate!
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();

        // (alpha_re + i alpha_im) * (1 + i) * inv_sqrt2
        let new_a_re = (self.state.alpha_re - self.state.alpha_im) * inv_sqrt2;
        let new_a_im = (self.state.alpha_re + self.state.alpha_im) * inv_sqrt2;

        // (beta_re + i beta_im) * (1 - i) * inv_sqrt2
        let new_b_re = (self.state.beta_re + self.state.beta_im) * inv_sqrt2;
        let new_b_im = (self.state.beta_im - self.state.beta_re) * inv_sqrt2;

        self.state.alpha_re = new_a_re;
        self.state.alpha_im = new_a_im;
        self.state.beta_re = new_b_re;
        self.state.beta_im = new_b_im;

        self.braid_count += 1;
    }

    /// Applies the non-Abelian braiding operator $B_{23} = \exp(\frac{\pi}{4} \gamma_3 \gamma_2)$.
    /// In the even parity basis, this implements $R_x(\pi/2) = \frac{1}{\sqrt{2}} (I + i X_L)$.
    pub fn apply_braid_23(&mut self) {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();

        // new_alpha = inv_sqrt2 * (alpha + i beta)
        let new_a_re = (self.state.alpha_re - self.state.beta_im) * inv_sqrt2;
        let new_a_im = (self.state.alpha_im + self.state.beta_re) * inv_sqrt2;

        // new_beta = inv_sqrt2 * (i alpha + beta)
        let new_b_re = (self.state.beta_re - self.state.alpha_im) * inv_sqrt2;
        let new_b_im = (self.state.beta_im + self.state.alpha_re) * inv_sqrt2;

        self.state.alpha_re = new_a_re;
        self.state.alpha_im = new_a_im;
        self.state.beta_re = new_b_re;
        self.state.beta_im = new_b_im;

        self.braid_count += 1;
    }

    /// Applies the topologically protected Clifford Hadamard gate $H$:
    /// Synthesized via the braid sequence: $B_{12} B_{23} B_{12}$.
    pub fn apply_hadamard(&mut self) {
        self.apply_braid_12();
        self.apply_braid_23();
        self.apply_braid_12();
    }

    /// Applies Pauli $X_L$ flip (bit flip) via a double braid $B_{23}^2$.
    pub fn apply_pauli_x(&mut self) {
        self.apply_braid_23();
        self.apply_braid_23();
    }

    /// Applies Pauli $Z_L$ flip (phase flip) via a double braid $B_{12}^2$.
    pub fn apply_pauli_z(&mut self) {
        self.apply_braid_12();
        self.apply_braid_12();
    }

    /// Applies Pauli $Y_L = i X_L Z_L$.
    pub fn apply_pauli_y(&mut self) {
        self.apply_pauli_z();
        self.apply_pauli_x();
    }

    /// Checks if total fermion parity has been preserved: $P_{tot} \equiv +1$.
    #[inline]
    pub fn is_parity_preserved(&self) -> bool {
        self.total_parity == 1
    }
}
