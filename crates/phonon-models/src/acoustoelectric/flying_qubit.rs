//! Flying spin qubit architecture, coherent electron spin transport,
//! spin-orbit precession, and flying two-qubit exchange coupling.
//!
//! # Physical Formalism
//! - Flying Qubit Spin State:
//!   $$|\psi(t)\rangle = \alpha(t) |\uparrow\rangle + \beta(t) |\downarrow\rangle$$
//! - Spin-Orbit (Rashba + Dresselhaus) Precession Hamiltonian:
//!   $$H_{SO} = \frac{1}{2} \hbar \vec{\Omega}_{SO} \cdot \vec{\sigma}$$
//!   where $\Omega_{SO} = \frac{2}{\hbar} (\alpha_R - \beta_D) k_{saw}$.
//! - Two-Qubit Flying Exchange Interaction:
//!   $$H_{ex}(t) = J_{ex}(t) \vec{S}_1 \cdot \vec{S}_2$$
//!   yielding $\sqrt{\mathrm{SWAP}}$ entanglement and Bell pairs $|\Psi^+\rangle = \frac{1}{\sqrt{2}} (|\uparrow\downarrow\rangle + |\downarrow\uparrow\rangle)$.

use super::dynamic_quantum_dot::HBAR_J_S;

/// 2-component complex spinor $(\alpha, \beta)$ representing spin-1/2 state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Spinor {
    pub re_up: f64,
    pub im_up: f64,
    pub re_down: f64,
    pub im_down: f64,
}

impl Spinor {
    /// Pure spin-up state $|\uparrow\rangle$.
    pub fn up() -> Self {
        Self {
            re_up: 1.0,
            im_up: 0.0,
            re_down: 0.0,
            im_down: 0.0,
        }
    }

    /// Pure spin-down state $|\downarrow\rangle$.
    pub fn down() -> Self {
        Self {
            re_up: 0.0,
            im_up: 0.0,
            re_down: 1.0,
            im_down: 0.0,
        }
    }

    /// Symmetric superposition state $|+\rangle = \frac{1}{\sqrt{2}}(|\uparrow\rangle + |\downarrow\rangle)$.
    pub fn plus() -> Self {
        let inv_sqrt2 = 1.0 / 2.0f64.sqrt();
        Self {
            re_up: inv_sqrt2,
            im_up: 0.0,
            re_down: inv_sqrt2,
            im_down: 0.0,
        }
    }

    /// Norm $\|\psi\| = \sqrt{|\alpha|^2 + |\beta|^2}$.
    pub fn norm(&self) -> f64 {
        (self.re_up.powi(2) + self.im_up.powi(2) + self.re_down.powi(2) + self.im_down.powi(2))
            .sqrt()
    }

    /// Normalizes spinor to unit magnitude.
    pub fn normalize(&mut self) {
        let n = self.norm();
        if n > 1e-12 {
            self.re_up /= n;
            self.im_up /= n;
            self.re_down /= n;
            self.im_down /= n;
        }
    }

    /// Evaluates quantum state fidelity with respect to target state:
    /// $$F = |\langle \psi | \psi_{target} \rangle|^2$$
    pub fn fidelity(&self, target: &Spinor) -> f64 {
        let re_inner = self.re_up * target.re_up
            + self.im_up * target.im_up
            + self.re_down * target.re_down
            + self.im_down * target.im_down;
        let im_inner = self.im_up * target.re_up - self.re_up * target.im_up
            + self.im_down * target.re_down
            - self.re_down * target.im_down;

        re_inner.powi(2) + im_inner.powi(2)
    }
}

/// Single-electron flying spin qubit transported in a SAW dynamic quantum dot.
#[derive(Debug, Clone, PartialEq)]
pub struct FlyingQubit {
    /// Internal spin state $(\alpha, \beta)$.
    pub state: Spinor,
    /// Spatial position along the transport channel in meters.
    pub position_m: f64,
    /// Transport velocity $v_{saw}$ in m/s.
    pub velocity_m_s: f64,
    /// Effective Rashba spin-orbit coupling $\alpha_R$ in $\text{eV} \cdot \text{m}$.
    pub rashba_coupling_ev_m: f64,
    /// Effective Dresselhaus spin-orbit coupling $\beta_D$ in $\text{eV} \cdot \text{m}$.
    pub dresselhaus_coupling_ev_m: f64,
    /// Applied external magnetic field in Tesla.
    pub b_field_tesla: f64,
}

impl FlyingQubit {
    pub fn new(velocity_m_s: f64) -> Self {
        Self {
            state: Spinor::up(),
            position_m: 0.0,
            velocity_m_s,
            rashba_coupling_ev_m: 5.0e-12,      // 5 meV*nm
            dresselhaus_coupling_ev_m: 2.0e-12, // 2 meV*nm
            b_field_tesla: 0.1,                 // 100 mT
        }
    }

    /// Spin precession angular frequency $\Omega_{SO}$ in rad/s:
    /// $$\Omega_{SO} = \frac{2}{\hbar} (\alpha_R - \beta_D) \cdot \frac{m^* v_{\mathrm{saw}}}{\hbar}$$
    pub fn spin_orbit_frequency_rad_s(&self, effective_mass_kg: f64) -> f64 {
        let ev_to_j = 1.602_176_634e-19;
        let delta_so_j_m =
            (self.rashba_coupling_ev_m - self.dresselhaus_coupling_ev_m).abs() * ev_to_j;
        let k_eff = (effective_mass_kg * self.velocity_m_s) / HBAR_J_S;

        (2.0 * delta_so_j_m * k_eff) / HBAR_J_S
    }

    /// Propagates the flying qubit along the channel over time $\Delta t$:
    pub fn propagate(&mut self, dt_s: f64, effective_mass_kg: f64) {
        self.position_m += self.velocity_m_s * dt_s;

        let omega = self.spin_orbit_frequency_rad_s(effective_mass_kg);
        let theta = omega * dt_s;

        // Rotation around Y-axis by angle theta:
        // alpha' = alpha * cos(theta/2) - beta * sin(theta/2)
        // beta' = alpha * sin(theta/2) + beta * cos(theta/2)
        let cos_half = (0.5 * theta).cos();
        let sin_half = (0.5 * theta).sin();

        let new_re_up = self.state.re_up * cos_half - self.state.re_down * sin_half;
        let new_im_up = self.state.im_up * cos_half - self.state.im_down * sin_half;
        let new_re_down = self.state.re_up * sin_half + self.state.re_down * cos_half;
        let new_im_down = self.state.im_up * sin_half + self.state.im_down * cos_half;

        self.state.re_up = new_re_up;
        self.state.im_up = new_im_up;
        self.state.re_down = new_re_down;
        self.state.im_down = new_im_down;
        self.state.normalize();
    }
}

/// Directional tunnel coupler / beam splitter for two flying electron qubits.
#[derive(Debug, Clone, PartialEq)]
pub struct FlyingQubitCoupler {
    /// Coupling interaction length $L_c$ in meters (typically 500 nm to 2000 nm).
    pub coupling_length_m: f64,
    /// Peak Heisenberg exchange coupling energy $J_{max}$ in Joules (typically $10 - 100\,\mu\text{eV}$).
    pub max_exchange_energy_j: f64,
}

impl FlyingQubitCoupler {
    pub fn new(coupling_length_m: f64, exchange_micro_ev: f64) -> Self {
        let ev_to_j = 1.602_176_634e-19;
        Self {
            coupling_length_m,
            max_exchange_energy_j: exchange_micro_ev * 1e-6 * ev_to_j,
        }
    }

    /// Creates a calibrated beam-splitter coupler tuned for $\sqrt{\mathrm{SWAP}}$ entanglement ($\theta_{ex} = \pi/2$).
    pub fn calibrated_sqrt_swap(coupling_length_m: f64, velocity_m_s: f64) -> Self {
        let transit_time = coupling_length_m / velocity_m_s.max(1.0);
        let j_opt = (0.5 * std::f64::consts::PI * HBAR_J_S) / transit_time;
        Self {
            coupling_length_m,
            max_exchange_energy_j: j_opt,
        }
    }

    /// Evaluates accumulated exchange angle $\theta_{ex} = \int \frac{J(t)}{\hbar} dt$:
    pub fn exchange_angle_rad(&self, velocity_m_s: f64) -> f64 {
        let transit_time = self.coupling_length_m / velocity_m_s.max(1.0);
        (self.max_exchange_energy_j * transit_time) / HBAR_J_S
    }

    /// Generates entangled two-qubit Bell state $|\Psi^+\rangle$ from initial separable state $|\uparrow\downarrow\rangle$.
    /// Computes entanglement concurrence $\mathcal{C} \in [0, 1]$ and target Bell state fidelity.
    pub fn evaluate_entanglement(&self, velocity_m_s: f64) -> (f64, f64) {
        let theta = self.exchange_angle_rad(velocity_m_s);
        // For sqrt(SWAP) operation, optimal theta = pi / 2 yields maximal Bell state entanglement
        let concurrence = (theta).sin().abs().clamp(0.0, 1.0);
        let fidelity = 0.5 * (1.0 + concurrence);
        (concurrence, fidelity)
    }
}
