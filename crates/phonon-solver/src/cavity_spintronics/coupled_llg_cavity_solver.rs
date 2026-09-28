//! Coupled Macrospin-Cavity LLG Integrator & Spin Pumping Solver.
//!
//! Formulates time-domain 4th-order Runge-Kutta (RK4) integration of the coupled
//! Landau-Lifshitz-Gilbert (LLG) macrospin dynamics and microwave cavity mode,
//! extracting dynamic precession cone angles, non-local pure spin currents, and
//! DC Inverse Spin Hall Effect (ISHE) voltages.

use phonon_models::cavity_spintronics::{MagnonCavityCoupling, YigPtInterface};
use std::f64::consts::PI;

/// Coupled LLG-cavity dynamic simulation state.
#[derive(Debug, Clone, PartialEq)]
pub struct LlgCavityState {
    /// Normalized magnetization components $(m_x, m_y, m_z)$ where $\|\mathbf{m}\| = 1$.
    pub m: [f64; 3],
    /// Cavity field mode real and imaginary parts: $a = a_r + i a_i$.
    pub a: [f64; 2],
}

impl Default for LlgCavityState {
    fn default() -> Self {
        Self {
            m: [0.0, 0.0, 1.0],
            a: [0.0, 0.0],
        }
    }
}

/// Results of the coupled time-domain LLG-cavity simulation.
#[derive(Debug, Clone, PartialEq)]
pub struct LlgCavityResult {
    /// Time points in seconds.
    pub time_points: Vec<f64>,
    /// Precession cone angles in degrees: $\theta_c(t) = \arccos(m_z)$.
    pub cone_angles_deg: Vec<f64>,
    /// Intracavity photon number $|a|^2$.
    pub cavity_photons: Vec<f64>,
    /// Dynamic spin current density $J_s(t)$ in $\text{J/m}^2$.
    pub spin_current_density: Vec<f64>,
    /// Transverse ISHE voltage $V_{ISHE}(t)$ in Volts.
    pub ishe_voltage: Vec<f64>,
    /// Steady-state precession cone angle in degrees.
    pub steady_state_cone_angle_deg: f64,
    /// Time-averaged steady-state DC ISHE voltage in Volts.
    pub dc_ishe_voltage_volts: f64,
}

/// Configuration for the coupled LLG-cavity solver.
#[derive(Debug, Clone, PartialEq)]
pub struct CoupledLlgConfig {
    /// Total simulation duration in seconds (e.g. 50 ns).
    pub duration_seconds: f64,
    /// Integration time step $\Delta t$ in seconds (e.g. 0.5 ps).
    pub dt_seconds: f64,
    /// Drive frequency in Hz (typically near cavity resonance).
    pub drive_freq_hz: f64,
    /// Microwave drive magnetic field amplitude $b_{rf,0}$ in Tesla (e.g. 5.0 uT).
    pub drive_amplitude_t: f64,
}

impl Default for CoupledLlgConfig {
    fn default() -> Self {
        Self {
            duration_seconds: 50.0e-9,
            dt_seconds: 1.0e-12,
            drive_freq_hz: 10.0e9,
            drive_amplitude_t: 10.0e-6,
        }
    }
}

/// Time-domain solver for coupled macrospin and cavity mode dynamics.
#[derive(Debug, Default, Clone)]
pub struct CoupledLlgCavitySolver;

impl CoupledLlgCavitySolver {
    /// Creates a new coupled LLG-cavity solver.
    pub fn new() -> Self {
        Self
    }

    /// Evaluates time derivatives $[d\mathbf{m}/dt, da/dt]$ in the rotating frame of the drive frequency $\omega_d$.
    fn evaluate_derivatives(
        &self,
        coupling: &MagnonCavityCoupling,
        interface: &YigPtInterface,
        config: &CoupledLlgConfig,
        state: &LlgCavityState,
    ) -> ([f64; 3], [f64; 2]) {
        let omega_d = 2.0 * PI * config.drive_freq_hz;
        let gamma = coupling.magnon.gyromagnetic_ratio;
        let alpha = interface.total_damping(&coupling.magnon);

        // Effective field in rotating frame around z:
        // B_z_eff = B_0 - omega_d / gamma
        let bz_rot = coupling.bias_field - omega_d / gamma;

        // Transverse drive from external microwave field + intracavity coupling:
        // cavity mode a induces transverse field: b_cav = (2 * g_mp / gamma) * a_r
        let g_rad = coupling.coupling_rate_rad();
        let b_cav = (2.0 * g_rad / gamma) * state.a[0];
        let bx_rot = config.drive_amplitude_t + b_cav;
        let by_rot = (2.0 * g_rad / gamma) * state.a[1];

        let b_eff = [bx_rot, by_rot, bz_rot];

        // LLG cross products:
        // m x B
        let mx_b = [
            state.m[1] * b_eff[2] - state.m[2] * b_eff[1],
            state.m[2] * b_eff[0] - state.m[0] * b_eff[2],
            state.m[0] * b_eff[1] - state.m[1] * b_eff[0],
        ];

        // m x (m x B)
        let m_x_mx_b = [
            state.m[1] * mx_b[2] - state.m[2] * mx_b[1],
            state.m[2] * mx_b[0] - state.m[0] * mx_b[2],
            state.m[0] * mx_b[1] - state.m[1] * mx_b[0],
        ];

        // dm/dt = - gamma / (1 + alpha^2) * [m x B + alpha * m x (m x B)]
        let prefactor = -gamma / (1.0 + alpha * alpha);
        let dm = [
            prefactor * (mx_b[0] + alpha * m_x_mx_b[0]),
            prefactor * (mx_b[1] + alpha * m_x_mx_b[1]),
            prefactor * (mx_b[2] + alpha * m_x_mx_b[2]),
        ];

        // Cavity mode evolution in rotating frame:
        // da/dt = -(i * delta_c + kappa_c / 2) * a - i * (g_mp / 2) * (m_x + i * m_y) + sqrt(kappa_ext) * s_in
        let delta_c = omega_d - coupling.cavity.omega_c();
        let kappa_c = coupling.cavity.total_decay_rate_rad();
        let kappa_ext = coupling.cavity.external_decay_rate_rad();

        // External input drive proportional to config.drive_amplitude_t
        let s_in = (config.drive_amplitude_t * 1e8).sqrt();

        // Mode damping & detuning:
        let da_decay_r = -0.5 * kappa_c * state.a[0] + delta_c * state.a[1];
        let da_decay_i = -delta_c * state.a[0] - 0.5 * kappa_c * state.a[1];

        // Spin backaction coupling:
        let da_spin_r = 0.5 * g_rad * state.m[1];
        let da_spin_i = -0.5 * g_rad * state.m[0];

        let da = [
            da_decay_r + da_spin_r + kappa_ext.sqrt() * s_in,
            da_decay_i + da_spin_i,
        ];

        (dm, da)
    }

    /// Advances the state by a single RK4 time step.
    pub fn step_rk4(
        &self,
        coupling: &MagnonCavityCoupling,
        interface: &YigPtInterface,
        config: &CoupledLlgConfig,
        state: &LlgCavityState,
        dt: f64,
    ) -> LlgCavityState {
        // k1
        let (k1_m, k1_a) = self.evaluate_derivatives(coupling, interface, config, state);

        // state 2
        let mut s2 = LlgCavityState {
            m: [
                state.m[0] + 0.5 * dt * k1_m[0],
                state.m[1] + 0.5 * dt * k1_m[1],
                state.m[2] + 0.5 * dt * k1_m[2],
            ],
            a: [
                state.a[0] + 0.5 * dt * k1_a[0],
                state.a[1] + 0.5 * dt * k1_a[1],
            ],
        };
        normalize_magnetization(&mut s2.m);
        let (k2_m, k2_a) = self.evaluate_derivatives(coupling, interface, config, &s2);

        // state 3
        let mut s3 = LlgCavityState {
            m: [
                state.m[0] + 0.5 * dt * k2_m[0],
                state.m[1] + 0.5 * dt * k2_m[1],
                state.m[2] + 0.5 * dt * k2_m[2],
            ],
            a: [
                state.a[0] + 0.5 * dt * k2_a[0],
                state.a[1] + 0.5 * dt * k2_a[1],
            ],
        };
        normalize_magnetization(&mut s3.m);
        let (k3_m, k3_a) = self.evaluate_derivatives(coupling, interface, config, &s3);

        // state 4
        let mut s4 = LlgCavityState {
            m: [
                state.m[0] + dt * k3_m[0],
                state.m[1] + dt * k3_m[1],
                state.m[2] + dt * k3_m[2],
            ],
            a: [state.a[0] + dt * k3_a[0], state.a[1] + dt * k3_a[1]],
        };
        normalize_magnetization(&mut s4.m);
        let (k4_m, k4_a) = self.evaluate_derivatives(coupling, interface, config, &s4);

        let mut next_state = LlgCavityState {
            m: [
                state.m[0] + (dt / 6.0) * (k1_m[0] + 2.0 * k2_m[0] + 2.0 * k3_m[0] + k4_m[0]),
                state.m[1] + (dt / 6.0) * (k1_m[1] + 2.0 * k2_m[1] + 2.0 * k3_m[1] + k4_m[1]),
                state.m[2] + (dt / 6.0) * (k1_m[2] + 2.0 * k2_m[2] + 2.0 * k3_m[2] + k4_m[2]),
            ],
            a: [
                state.a[0] + (dt / 6.0) * (k1_a[0] + 2.0 * k2_a[0] + 2.0 * k3_a[0] + k4_a[0]),
                state.a[1] + (dt / 6.0) * (k1_a[1] + 2.0 * k2_a[1] + 2.0 * k3_a[1] + k4_a[1]),
            ],
        };
        normalize_magnetization(&mut next_state.m);
        next_state
    }

    /// Solves the full time-domain coupled dynamics.
    pub fn solve(
        &self,
        coupling: &MagnonCavityCoupling,
        interface: &YigPtInterface,
        config: &CoupledLlgConfig,
    ) -> LlgCavityResult {
        let n_steps = (config.duration_seconds / config.dt_seconds).max(10.0) as usize;
        let omega_d = 2.0 * PI * config.drive_freq_hz;

        let mut time_points = Vec::with_capacity(n_steps);
        let mut cone_angles_deg = Vec::with_capacity(n_steps);
        let mut cavity_photons = Vec::with_capacity(n_steps);
        let mut spin_current_density = Vec::with_capacity(n_steps);
        let mut ishe_voltage = Vec::with_capacity(n_steps);

        let mut current_state = LlgCavityState::default();
        let mut current_time = 0.0;

        for _ in 0..n_steps {
            time_points.push(current_time);

            // Cone angle theta = arccos(m_z)
            let mz_clamped = current_state.m[2].clamp(-1.0, 1.0);
            let theta_rad = mz_clamped.acos();
            let theta_deg = theta_rad * 180.0 / PI;
            cone_angles_deg.push(theta_deg);

            // Intracavity photon density
            let photons =
                current_state.a[0] * current_state.a[0] + current_state.a[1] * current_state.a[1];
            cavity_photons.push(photons);

            // Spin current & ISHE voltage
            let js = interface.spin_current_density_dc(omega_d, theta_rad);
            let v_ishe = interface.ishe_voltage_volts(omega_d, theta_rad);

            spin_current_density.push(js);
            ishe_voltage.push(v_ishe);

            // Advance state
            current_state = self.step_rk4(
                coupling,
                interface,
                config,
                &current_state,
                config.dt_seconds,
            );
            current_time += config.dt_seconds;
        }

        // Compute steady state averages over the last 25% of the trajectory
        let tail_start = (n_steps * 3) / 4;
        let tail_angles = &cone_angles_deg[tail_start..];
        let tail_voltages = &ishe_voltage[tail_start..];

        let steady_cone = tail_angles.iter().sum::<f64>() / tail_angles.len() as f64;
        let dc_ishe = tail_voltages.iter().sum::<f64>() / tail_voltages.len() as f64;

        LlgCavityResult {
            time_points,
            cone_angles_deg,
            cavity_photons,
            spin_current_density,
            ishe_voltage,
            steady_state_cone_angle_deg: steady_cone,
            dc_ishe_voltage_volts: dc_ishe,
        }
    }
}

/// Normalizes the 3D magnetization vector to unit length $\|\mathbf{m}\| = 1$.
#[inline(always)]
fn normalize_magnetization(m: &mut [f64; 3]) {
    let norm_sq = m[0] * m[0] + m[1] * m[1] + m[2] * m[2];
    if norm_sq > 0.0 {
        let inv_norm = 1.0 / norm_sq.sqrt();
        m[0] *= inv_norm;
        m[1] *= inv_norm;
        m[2] *= inv_norm;
    } else {
        m[0] = 0.0;
        m[1] = 0.0;
        m[2] = 1.0;
    }
}
