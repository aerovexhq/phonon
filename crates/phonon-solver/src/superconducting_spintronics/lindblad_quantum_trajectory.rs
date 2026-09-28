//! Open-system Lindblad Master Equation and quantum trajectory solver for topological
//! Majorana zero mode qubits subjected to quasi-particle poisoning and cryogenic dephasing.

/// Bloch sphere representation of the single topological qubit density matrix:
/// $$\rho = \frac{1}{2} (I + r_x \sigma_x + r_y \sigma_y + r_z \sigma_z)$$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TopologicalQubitDensityMatrix {
    /// Bloch vector $X$ component $r_x = \text{Tr}(\rho \sigma_x)$.
    pub rx: f64,
    /// Bloch vector $Y$ component $r_y = \text{Tr}(\rho \sigma_y)$.
    pub ry: f64,
    /// Bloch vector $Z$ component $r_z = \text{Tr}(\rho \sigma_z)$ (non-local fermion parity expectation).
    pub rz: f64,
}

impl TopologicalQubitDensityMatrix {
    /// Initializes in pure logical $|0_L\rangle$ state ($r_z = +1.0$, even parity).
    pub fn new_zero() -> Self {
        Self {
            rx: 0.0,
            ry: 0.0,
            rz: 1.0,
        }
    }

    /// Initializes in pure logical $|1_L\rangle$ state ($r_z = -1.0$, odd parity).
    pub fn new_one() -> Self {
        Self {
            rx: 0.0,
            ry: 0.0,
            rz: -1.0,
        }
    }

    /// Quantum state purity $\mathcal{P} = \text{Tr}(\rho^2) = \frac{1 + \|\mathbf{r}\|^2}{2} \in [0.5, 1.0]$.
    pub fn purity(&self) -> f64 {
        let r_sq = self.rx.powi(2) + self.ry.powi(2) + self.rz.powi(2);
        0.5 * (1.0 + r_sq.min(1.0))
    }

    /// State fidelity relative to a target pure state $(x_t, y_t, z_t)$:
    /// $$F = \frac{1 + \mathbf{r} \cdot \mathbf{r}_{tgt}}{2}$$
    pub fn fidelity_to_pure(&self, target_rx: f64, target_ry: f64, target_rz: f64) -> f64 {
        let dot = self.rx * target_rx + self.ry * target_ry + self.rz * target_rz;
        (0.5 * (1.0 + dot)).clamp(0.0, 1.0)
    }

    /// Applies unitary rotation around $Z$ by angle $\theta$ (Phase / $S$ gate when $\theta = \pi / 2$).
    pub fn rotate_z(&mut self, theta_rad: f64) {
        let cos_t = theta_rad.cos();
        let sin_t = theta_rad.sin();
        let new_rx = cos_t * self.rx - sin_t * self.ry;
        let new_ry = sin_t * self.rx + cos_t * self.ry;
        self.rx = new_rx;
        self.ry = new_ry;
    }

    /// Applies unitary rotation around $Y$ by angle $\theta$ (Hadamard-like transformation).
    pub fn rotate_y(&mut self, theta_rad: f64) {
        let cos_t = theta_rad.cos();
        let sin_t = theta_rad.sin();
        let new_rx = cos_t * self.rx + sin_t * self.rz;
        let new_rz = -sin_t * self.rx + cos_t * self.rz;
        self.rx = new_rx;
        self.rz = new_rz;
    }

    /// Applies unitary rotation around $X$ (bit-flip / parity operation).
    pub fn rotate_x(&mut self, theta_rad: f64) {
        let cos_t = theta_rad.cos();
        let sin_t = theta_rad.sin();
        let new_ry = cos_t * self.ry - sin_t * self.rz;
        let new_rz = sin_t * self.ry + cos_t * self.rz;
        self.ry = new_ry;
        self.rz = new_rz;
    }
}

/// Lindblad open-quantum-system trajectory propagator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LindbladTrajectorySolver {
    /// Quasi-particle poisoning rate $\Gamma_{qp}$ in $s^{-1}$ (parity flip rate).
    pub gamma_qp: f64,
    /// Cryogenic thermal dephasing rate $\Gamma_\phi$ in $s^{-1}$.
    pub gamma_dephasing: f64,
}

impl LindbladTrajectorySolver {
    /// Creates a Lindblad solver with specified decay channels.
    pub fn new(gamma_qp: f64, gamma_dephasing: f64) -> Self {
        Self {
            gamma_qp,
            gamma_dephasing,
        }
    }

    /// Advances the density matrix over time interval $\Delta t$ under dissipative Lindblad channels:
    /// $$\frac{dr_x}{dt} = -\Gamma_\phi r_x, \quad \frac{dr_y}{dt} = -\Gamma_\phi r_y, \quad \frac{dr_z}{dt} = -2 \Gamma_{qp} r_z$$
    pub fn step_dissipation(&self, state: &mut TopologicalQubitDensityMatrix, dt_seconds: f64) {
        let dephasing_decay = (-self.gamma_dephasing * dt_seconds).exp();
        let poisoning_decay = (-2.0 * self.gamma_qp * dt_seconds).exp();

        state.rx *= dephasing_decay;
        state.ry *= dephasing_decay;
        state.rz *= poisoning_decay;
    }

    /// Simulates a complete adiabatic braid operation $B_{12}$ (Phase $S$ gate) over duration $\tau_{braid}$.
    pub fn execute_braid_12(&self, state: &mut TopologicalQubitDensityMatrix, braid_time_s: f64) {
        // Unitary phase rotation by pi/2
        state.rotate_z(std::f64::consts::FRAC_PI_2);
        // Dissipation during braid
        self.step_dissipation(state, braid_time_s);
    }

    /// Simulates a complete adiabatic braid operation $B_{23}$ (Hadamard-like gate) over duration $\tau_{braid}$.
    pub fn execute_braid_23(&self, state: &mut TopologicalQubitDensityMatrix, braid_time_s: f64) {
        // Unitary rotation around Y by pi/2
        state.rotate_y(std::f64::consts::FRAC_PI_2);
        // Dissipation during braid
        self.step_dissipation(state, braid_time_s);
    }
}
