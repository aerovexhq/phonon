#![deny(unsafe_code)]

//! Multi-rate co-simulation engine partitioning dynamics into fast micro-rate (e.g. 1-10 MHz switching)
//! and slow macro-rate (e.g. 48 kHz acoustic/mechanical) subsystems with passive Hermite interface interpolation.

use phonon_models::symplectic_multirate::{SymplecticMultirateParams, SymplecticMultirateMetrics};
use crate::symplectic_multirate::integrator::{SymplecticIntegrator, MAX_STATE_DIM};

/// Cubic Hermite interpolator for passive boundary interface variables across a macro-step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HermiteInterpolator {
    /// Value at the start of macro-interval t_n.
    pub u0: f64,
    /// Value at the end of macro-interval t_{n+1}.
    pub u1: f64,
    /// Derivative du/dt at t_n.
    pub du0: f64,
    /// Derivative du/dt at t_{n+1}.
    pub du1: f64,
    /// Macro-step interval length H = dt_slow.
    pub h: f64,
}

impl HermiteInterpolator {
    /// Constructs a cubic Hermite interpolator across macro-interval [0, h].
    pub fn new(u0: f64, u1: f64, du0: f64, du1: f64, h: f64) -> Self {
        Self {
            u0,
            u1,
            du0,
            du1,
            h: h.max(1.0e-12),
        }
    }

    /// Evaluates the boundary value at normalized progress tau in [0, 1].
    #[inline]
    pub fn evaluate(&self, tau: f64) -> f64 {
        let t = tau.clamp(0.0, 1.0);
        let t2 = t * t;
        let t3 = t2 * t;

        // Basis polynomials:
        // H0(t) = 1 - 3*t^2 + 2*t^3
        // H1(t) = 3*t^2 - 2*t^3
        // H2(t) = t - 2*t^2 + t^3
        // H3(t) = -t^2 + t^3
        let h0 = 1.0 - 3.0 * t2 + 2.0 * t3;
        let h1 = 3.0 * t2 - 2.0 * t3;
        let h2 = t - 2.0 * t2 + t3;
        let h3 = -t2 + t3;

        h0 * self.u0 + h1 * self.u1 + self.h * (h2 * self.du0 + h3 * self.du1)
    }
}

/// Conservative coupled two-mass oscillator system for multi-rate passivity benchmarking.
///
/// Slow subsystem: Mass M_s, spring K_s, state (q_s, p_s).
/// Fast subsystem: Mass M_f, spring K_f, state (q_f, p_f).
/// Coupling spring: K_c between q_s and q_f.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoupledPassiveOscillators {
    pub mass_slow: f64,
    pub stiff_slow: f64,
    pub q_slow: f64,
    pub p_slow: f64,

    pub mass_fast: f64,
    pub stiff_fast: f64,
    pub q_fast: f64,
    pub p_fast: f64,

    pub stiff_coupling: f64,
}

impl Default for CoupledPassiveOscillators {
    fn default() -> Self {
        Self {
            mass_slow: 1.0,
            stiff_slow: 10.0,
            q_slow: 1.0,
            p_slow: 0.0,

            mass_fast: 0.01,
            stiff_fast: 1000.0,
            q_fast: 0.0,
            p_fast: 0.0,

            stiff_coupling: 5.0,
        }
    }
}

impl CoupledPassiveOscillators {
    /// Computes total Hamiltonian energy of the coupled system.
    pub fn total_energy(&self) -> f64 {
        let e_slow = 0.5 * self.p_slow * self.p_slow / self.mass_slow
            + 0.5 * self.stiff_slow * self.q_slow * self.q_slow;
        let e_fast = 0.5 * self.p_fast * self.p_fast / self.mass_fast
            + 0.5 * self.stiff_fast * self.q_fast * self.q_fast;
        let diff = self.q_slow - self.q_fast;
        let e_coup = 0.5 * self.stiff_coupling * diff * diff;
        e_slow + e_fast + e_coup
    }
}

/// Coupled 1 MHz PWM power electronic circuit and 48 kHz acoustic resonator co-simulation.
///
/// Fast subsystem: 1 MHz switching half-bridge driving LC lowpass filter (L = 22 uH, C = 1 uF).
/// Slow subsystem: Loudspeaker voice coil & acoustic horn (m = 2 g, k = 800 N/m, b = 0.8 Ns/m).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PwmAcousticCoSimSystem {
    // Fast electrical states: inductor current i_L and capacitor voltage v_C
    pub i_inductor: f64,
    pub v_capacitor: f64,
    pub inductance_h: f64,
    pub capacitance_f: f64,
    pub filter_resistance_ohm: f64,
    pub supply_voltage_v: f64,
    pub pwm_frequency_hz: f64,

    // Slow acoustic states: voice coil displacement x and velocity v
    pub voice_coil_pos_m: f64,
    pub voice_coil_vel_m_s: f64,
    pub acoustic_mass_kg: f64,
    pub acoustic_stiffness_n_m: f64,
    pub acoustic_damping_n_s_m: f64,
    pub force_factor_bl: f64,
    pub voice_coil_resistance_ohm: f64,
}

impl Default for PwmAcousticCoSimSystem {
    fn default() -> Self {
        Self {
            i_inductor: 0.0,
            v_capacitor: 0.0,
            inductance_h: 22.0e-6,
            capacitance_f: 1.0e-6,
            filter_resistance_ohm: 0.05,
            supply_voltage_v: 24.0,
            pwm_frequency_hz: 1.0e6,

            voice_coil_pos_m: 0.0,
            voice_coil_vel_m_s: 0.0,
            acoustic_mass_kg: 0.002,
            acoustic_stiffness_n_m: 800.0,
            acoustic_damping_n_s_m: 0.8,
            force_factor_bl: 5.0,
            voice_coil_resistance_ohm: 4.0,
        }
    }
}

impl PwmAcousticCoSimSystem {
    /// Evaluates PWM square-wave voltage at time t with sinusoidal modulation.
    pub fn pwm_voltage_at(&self, t: f64) -> f64 {
        let period = 1.0 / self.pwm_frequency_hz;
        let phase = (t % period) / period;
        // Audio modulating signal at 440 Hz
        let duty = 0.5 + 0.35 * (2.0 * std::f64::consts::PI * 440.0 * t).sin();
        if phase < duty {
            self.supply_voltage_v
        } else {
            -self.supply_voltage_v
        }
    }
}

/// High-throughput multi-rate co-simulation orchestrator.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiRateCoSimulator {
    /// Parameter settings.
    pub params: SymplecticMultirateParams,
    /// Evaluated metrics.
    pub metrics: SymplecticMultirateMetrics,
    /// Numerical integrator.
    pub integrator: SymplecticIntegrator,
    /// Simulated continuous time in seconds.
    pub simulated_time: f64,
    /// Coupled passive benchmark instance.
    pub passive_system: CoupledPassiveOscillators,
    /// Extreme switching acoustic benchmark instance.
    pub pwm_system: PwmAcousticCoSimSystem,
    /// Baseline Hamiltonian energy at initialization.
    pub initial_hamiltonian: f64,
}

impl MultiRateCoSimulator {
    /// Constructs a new MultiRateCoSimulator initialized with specified parameters.
    pub fn new(params: SymplecticMultirateParams) -> Self {
        let passive = CoupledPassiveOscillators::default();
        let pwm = PwmAcousticCoSimSystem::default();
        let initial_h = passive.total_energy();
        let mut sim = Self {
            params,
            metrics: SymplecticMultirateMetrics::default(),
            integrator: SymplecticIntegrator::new(),
            simulated_time: 0.0,
            passive_system: passive,
            pwm_system: pwm,
            initial_hamiltonian: initial_h,
        };
        sim.metrics.is_symplectic_invariant_preserved = true;
        sim
    }

    /// Advances the coupled passive oscillator benchmark by one macro-step slow_dt_s.
    ///
    /// Executes M = subcycling_ratio micro-steps on the fast partition with
    /// Hermite interpolation of the slow boundary effort, preserving passivity.
    pub fn step_passive_macro(&mut self) {
        let slow_dt = self.params.slow_dt_s;
        let fast_dt = self.params.fast_dt_s;
        let m = self.params.subcycling_ratio();

        let t_start = self.simulated_time;
        let q_s0 = self.passive_system.q_slow;
        let v_s0 = self.passive_system.p_slow / self.passive_system.mass_slow;

        // Predict slow position at end of macro-interval via Taylor predictor
        let f_s0 = -self.passive_system.stiff_slow * q_s0
            - self.passive_system.stiff_coupling * (q_s0 - self.passive_system.q_fast);
        let a_s0 = f_s0 / self.passive_system.mass_slow;
        let q_s1 = q_s0 + slow_dt * v_s0 + 0.5 * slow_dt * slow_dt * a_s0;
        let v_s1 = v_s0 + slow_dt * a_s0;

        let interpolator = HermiteInterpolator::new(q_s0, q_s1, v_s0, v_s1, slow_dt);

        let mut avg_q_fast = 0.0;
        let inv_m_fast = 1.0 / self.passive_system.mass_fast;

        // Micro-step fast subsystem
        for step_idx in 0..m {
            let tau = (step_idx as f64 + 0.5) / (m as f64);
            let q_s_interp = interpolator.evaluate(tau);

            // Symplectic separable step on fast subsystem:
            // Fast potential: V_f(q_f) = 0.5 * K_f * q_f^2 + 0.5 * K_c * (q_s_interp - q_f)^2
            // dV_f/dq_f = K_f * q_f - K_c * (q_s_interp - q_f) = (K_f + K_c) * q_f - K_c * q_s_interp
            let stiff_eff = self.passive_system.stiff_fast + self.passive_system.stiff_coupling;
            let k_c = self.passive_system.stiff_coupling;

            let half_dt = 0.5 * fast_dt;

            // Half kick
            let grad_0 = stiff_eff * self.passive_system.q_fast - k_c * q_s_interp;
            self.passive_system.p_fast -= half_dt * grad_0;

            // Full drift
            self.passive_system.q_fast += fast_dt * inv_m_fast * self.passive_system.p_fast;

            // Second half kick
            let grad_1 = stiff_eff * self.passive_system.q_fast - k_c * q_s_interp;
            self.passive_system.p_fast -= half_dt * grad_1;

            avg_q_fast += self.passive_system.q_fast;
            self.metrics.fast_steps_count += 1;
        }

        avg_q_fast /= m as f64;

        // Macro-step slow subsystem using averaged fast coordinate
        let stiff_slow_eff = self.passive_system.stiff_slow + self.passive_system.stiff_coupling;
        let k_c = self.passive_system.stiff_coupling;
        let inv_m_slow = 1.0 / self.passive_system.mass_slow;
        let half_h = 0.5 * slow_dt;

        // Stormer-Verlet on slow subsystem
        let grad_s0 = stiff_slow_eff * self.passive_system.q_slow - k_c * avg_q_fast;
        self.passive_system.p_slow -= half_h * grad_s0;

        self.passive_system.q_slow += slow_dt * inv_m_slow * self.passive_system.p_slow;

        let grad_s1 = stiff_slow_eff * self.passive_system.q_slow - k_c * avg_q_fast;
        self.passive_system.p_slow -= half_h * grad_s1;

        self.metrics.slow_steps_count += 1;
        self.simulated_time = t_start + slow_dt;

        // Update Hamiltonian drift metric
        let current_h = self.passive_system.total_energy();
        let drift = (current_h - self.initial_hamiltonian).abs();
        if drift > self.metrics.hamiltonian_drift_max {
            self.metrics.hamiltonian_drift_max = drift;
        }
        if drift > 1.0e-3 {
            self.metrics.is_symplectic_invariant_preserved = false;
        }
    }

    /// Advances the extreme 1 MHz PWM switching and 48 kHz acoustic resonator co-simulation by one macro-step.
    ///
    /// Micro-steps the 1 MHz LC switching circuit M times per acoustic macro-step.
    pub fn step_pwm_acoustic_macro(&mut self) {
        let slow_dt = self.params.slow_dt_s;
        let fast_dt = self.params.fast_dt_s;
        let m = self.params.subcycling_ratio();

        let t_start = self.simulated_time;

        // Voice coil boundary back-EMF at start of macro step
        let bl = self.pwm_system.force_factor_bl;
        let r_e = self.pwm_system.voice_coil_resistance_ohm;
        let v_emf = bl * self.pwm_system.voice_coil_vel_m_s;

        let mut sum_v_c = 0.0;

        let l_val = self.pwm_system.inductance_h;
        let c_val = self.pwm_system.capacitance_f;
        let r_loss = self.pwm_system.filter_resistance_ohm;

        // Fast micro-steps on LC filter with 1 MHz PWM input
        for step_idx in 0..m {
            let t_micro = t_start + (step_idx as f64) * fast_dt;
            let v_pwm = self.pwm_system.pwm_voltage_at(t_micro);

            // Load current drawn by speaker voice coil
            let i_load = (self.pwm_system.v_capacitor - v_emf) / r_e;

            // Symplectic / implicit midpoint integration on LC state:
            // d i_L / dt = (v_pwm - v_C - r_loss * i_L) / L
            // d v_C / dt = (i_L - i_load) / C
            // Using midpoint state updates:
            let di_l = (v_pwm - self.pwm_system.v_capacitor - r_loss * self.pwm_system.i_inductor) / l_val;
            self.pwm_system.i_inductor += fast_dt * di_l;

            let dv_c = (self.pwm_system.i_inductor - i_load) / c_val;
            self.pwm_system.v_capacitor += fast_dt * dv_c;

            sum_v_c += self.pwm_system.v_capacitor;
            self.metrics.fast_steps_count += 1;
        }

        let avg_v_c = sum_v_c / (m as f64);
        let avg_i_speaker = (avg_v_c - v_emf) / r_e;

        // Slow macro-step on acoustic mechanical oscillator:
        // m * dv/dt = Bl * i_speaker - k * x - b * v
        // dx/dt = v
        let force_drive = bl * avg_i_speaker;
        let k_mech = self.pwm_system.acoustic_stiffness_n_m;
        let b_mech = self.pwm_system.acoustic_damping_n_s_m;
        let m_mech = self.pwm_system.acoustic_mass_kg;

        // Semi-implicit Euler / Stormer-Verlet on mechanical state
        let accel = (force_drive - k_mech * self.pwm_system.voice_coil_pos_m
            - b_mech * self.pwm_system.voice_coil_vel_m_s) / m_mech;
        self.pwm_system.voice_coil_vel_m_s += slow_dt * accel;
        self.pwm_system.voice_coil_pos_m += slow_dt * self.pwm_system.voice_coil_vel_m_s;

        self.metrics.slow_steps_count += 1;
        self.simulated_time = t_start + slow_dt;
    }
}
