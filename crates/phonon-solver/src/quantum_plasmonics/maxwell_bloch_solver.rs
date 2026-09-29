#![deny(unsafe_code)]

//! Time-Dependent Maxwell-Bloch Solver for Quantum Emitter-Plasmon Interactions.
//!
//! Solves the coupled non-linear Maxwell-Bloch equations describing optical field
//! propagation, atomic dipole saturation, single-photon switching transients, and
//! non-linear transmission spectra.

use phonon_models::quantum_plasmonics::SinglePhotonTransistor;

/// Bloch state vector of the two-level quantum emitter: $(u, v, w)$.
///
/// - $u = 2 \text{Re}(\rho_{ge})$: in-phase dispersive dipole
/// - $v = 2 \text{Im}(\rho_{ge})$: in-quadrature absorptive dipole
/// - $w = \rho_{ee} - \rho_{gg}$: atomic population inversion $\in [-1, 1]$
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BlochVector {
    /// In-phase dipole $u$.
    pub u: f64,
    /// In-quadrature dipole $v$.
    pub v: f64,
    /// Population inversion $w$.
    pub w: f64,
}

impl Default for BlochVector {
    fn default() -> Self {
        Self {
            u: 0.0,
            v: 0.0,
            w: -1.0, // Initialized in ground state |g>
        }
    }
}

impl BlochVector {
    /// Initializes emitter in pure ground state $|g\rangle$ ($w = -1.0$).
    pub fn new_ground() -> Self {
        Self::default()
    }

    /// Initializes emitter in fully saturated / excited state $|e\rangle$ ($w = +1.0$).
    pub fn new_excited() -> Self {
        Self {
            u: 0.0,
            v: 0.0,
            w: 1.0,
        }
    }

    /// Excited state population $\rho_{ee} = \frac{1 + w}{2} \in [0, 1]$.
    pub fn excited_population(&self) -> f64 {
        (0.5 * (1.0 + self.w)).clamp(0.0, 1.0)
    }

    /// Quantum state purity $\mathcal{P} = \frac{1 + u^2 + v^2 + w^2}{2} \in [0.5, 1.0]$.
    pub fn purity(&self) -> f64 {
        let norm_sq = self.u * self.u + self.v * self.v + self.w * self.w;
        0.5 * (1.0 + norm_sq.min(1.0))
    }
}

/// Time-Dependent Maxwell-Bloch Numerical Solver.
#[derive(Debug, Clone, Copy)]
pub struct PlasmonicMaxwellBlochSolver {
    /// Single-photon plasmonic transistor parameters.
    pub transistor: SinglePhotonTransistor,
    /// Detuning between probe laser and emitter $\Delta_p = \omega_p - \omega_0$ in rad/s.
    pub detuning_rad_s: f64,
}

impl PlasmonicMaxwellBlochSolver {
    /// Creates a new Maxwell-Bloch solver.
    pub fn new(transistor: SinglePhotonTransistor, detuning_rad_s: f64) -> Self {
        Self {
            transistor,
            detuning_rad_s,
        }
    }

    /// Evaluates time derivatives $(\dot{u}, \dot{v}, \dot{w})$ given current Bloch vector and Rabi drive $\Omega$:
    ///
    /// $$\dot{u} = -\Delta v - \gamma_2 u$$
    /// $$\dot{v} = \Delta u + \Omega w - \gamma_2 v$$
    /// $$\dot{w} = -\Omega v - \Gamma_{tot} (w + 1)$$
    pub fn bloch_derivative(&self, state: &BlochVector, rabi_freq: f64) -> BlochVector {
        let delta = self.detuning_rad_s;
        let g_tot = self.transistor.total_decay_rate();
        let gamma2 = 0.5 * g_tot + self.transistor.emitter.pure_dephasing_rate;

        let du = -delta * state.v - gamma2 * state.u;
        let dv = delta * state.u + rabi_freq * state.w - gamma2 * state.v;
        let dw = -rabi_freq * state.v - g_tot * (state.w + 1.0);

        BlochVector {
            u: du,
            v: dv,
            w: dw,
        }
    }

    /// 4th-order Runge-Kutta numerical integration step for the Bloch equations.
    pub fn rk4_step(&self, state: &BlochVector, rabi_freq: f64, dt: f64) -> BlochVector {
        let k1 = self.bloch_derivative(state, rabi_freq);

        let s2 = BlochVector {
            u: state.u + 0.5 * dt * k1.u,
            v: state.v + 0.5 * dt * k1.v,
            w: state.w + 0.5 * dt * k1.w,
        };
        let k2 = self.bloch_derivative(&s2, rabi_freq);

        let s3 = BlochVector {
            u: state.u + 0.5 * dt * k2.u,
            v: state.v + 0.5 * dt * k2.v,
            w: state.w + 0.5 * dt * k2.w,
        };
        let k3 = self.bloch_derivative(&s3, rabi_freq);

        let s4 = BlochVector {
            u: state.u + dt * k3.u,
            v: state.v + dt * k3.v,
            w: state.w + dt * k3.w,
        };
        let k4 = self.bloch_derivative(&s4, rabi_freq);

        let c = dt / 6.0;
        let new_u = state.u + c * (k1.u + 2.0 * k2.u + 2.0 * k3.u + k4.u);
        let new_v = state.v + c * (k1.v + 2.0 * k2.v + 2.0 * k3.v + k4.v);
        let new_w = (state.w + c * (k1.w + 2.0 * k2.w + 2.0 * k3.w + k4.w)).clamp(-1.0, 1.0);

        BlochVector {
            u: new_u,
            v: new_v,
            w: new_w,
        }
    }

    /// Steady-state probe transmission under continuous excitation at Rabi frequency $\Omega$:
    ///
    /// $$T(\Omega) = \left| 1 - \frac{\beta_{spp}}{1 + s - 2 i \Delta_p / \Gamma_{tot}} \right|^2$$
    /// where $s = \frac{2 |\Omega|^2}{\Gamma_{tot} \gamma_2}$ is the optical saturation parameter.
    pub fn steady_state_transmission(&self, rabi_freq: f64) -> f64 {
        let beta = self.transistor.beta_factor();
        let g_tot = self.transistor.total_decay_rate();
        let gamma2 = 0.5 * g_tot + self.transistor.emitter.pure_dephasing_rate;

        let s = (2.0 * rabi_freq * rabi_freq) / (g_tot * gamma2.max(1e6));
        let delta_norm = (2.0 * self.detuning_rad_s) / g_tot.max(1e6);

        // Denominator: (1 + s) - i delta_norm
        let den_re = 1.0 + s;
        let den_im = -delta_norm;
        let den_sq = den_re * den_re + den_im * den_im;

        let frac_re = (beta * den_re) / den_sq;
        let frac_im = (-beta * den_im) / den_sq;

        // t = 1 - frac
        let t_re = 1.0 - frac_re;
        let t_im = -frac_im;

        (t_re * t_re + t_im * t_im).clamp(1e-6, 1.0)
    }

    /// Evaluates the transient single-photon switching dynamic:
    ///
    /// Simulates arrival of a single-photon gating pulse with duration $\tau_{gate} \sim 1 / \Gamma_{tot}$,
    /// transitioning the transistor from closed ($T_0$) to open ($T_1$) transmission.
    pub fn simulate_switching_transient(
        &self,
        gate_pulse_area: f64,
        num_steps: usize,
    ) -> (f64, f64, BlochVector) {
        let g_tot = self.transistor.total_decay_rate();
        let t_pulse = 2.0 / g_tot.max(1e6);
        let dt = t_pulse / (num_steps.max(1) as f64);

        // Constant Rabi frequency delivering gate_pulse_area = Omega * t_pulse
        let rabi_gate = gate_pulse_area / t_pulse;

        let mut state = BlochVector::new_ground();
        for _ in 0..num_steps {
            state = self.rk4_step(&state, rabi_gate, dt);
        }

        let t0 = self.transistor.unpumped_transmission();
        // After gate pulse, probe transmission scales with saturated excited population
        let t_transient =
            t0 + state.excited_population() * (self.transistor.saturated_transmission() - t0);

        (t0, t_transient, state)
    }
}
