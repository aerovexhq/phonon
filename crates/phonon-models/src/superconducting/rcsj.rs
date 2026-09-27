//! Resistively and Capacitively Shunted Junction (RCSJ) macro-model.
//!
//! Models the microscopic Josephson effect:
//! - Supercurrent: $I_s = I_c \sin(\phi)$
//! - Phase-voltage evolution: $\frac{d\phi}{dt} = \frac{2\pi}{\Phi_0} V(t)$
//! - Stewart-McCumber damping parameter: $\beta_c = \frac{2\pi I_c R_n^2 C_j}{\Phi_0}$
//! - Josephson plasma frequency: $\omega_p = \sqrt{\frac{2\pi I_c}{\Phi_0 C_j}}$
//! - Dynamic companion model for numerical MNA stamping.

use super::material::SuperconductorMaterial;
use phonon_core::FLUX_QUANTUM;

/// RCSJ companion model stamp for MNA solver iterations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RcsjCompanionStamp {
    /// Equivalent companion conductance $G_{eq}$ in Siemens ($S$).
    pub g_eq: f64,
    /// Equivalent companion current source $I_{eq}$ in Amperes ($A$).
    pub i_eq: f64,
    /// Total instantaneous junction current $I_{total}$ in Amperes ($A$).
    pub i_total: f64,
    /// Updated quantum phase $\phi$ in radians.
    pub phase_next: f64,
}

/// Resistively and Capacitively Shunted Junction (RCSJ) macro-model.
#[derive(Debug, Clone, PartialEq)]
pub struct JosephsonRcsjModel {
    /// Critical current $I_c$ in Amperes ($A$).
    pub ic: f64,
    /// Normal state shunt resistance $R_n$ in Ohms ($\Omega$).
    pub rn: f64,
    /// Intrinsic junction capacitance $C_j$ in Farads ($F$).
    pub cap: f64,
    /// Current quantum phase difference $\phi(t)$ across the junction in radians.
    pub phase: f64,
    /// Current junction voltage $V(t)$ in Volts ($V$).
    pub voltage: f64,
    /// Subgap resistance $R_{sg}$ (quasi-particle resistance below $2\Delta / e$, typically $5 - 10 \times R_n$).
    pub r_subgap: f64,
    /// Superconducting material preset (optional).
    pub material: Option<SuperconductorMaterial>,
}

impl Default for JosephsonRcsjModel {
    fn default() -> Self {
        Self {
            ic: 100e-6,   // 100 uA
            rn: 10.0,     // 10 Ohms -> Ic * Rn = 1.0 mV
            cap: 0.1e-12, // 100 fF
            phase: 0.0,
            voltage: 0.0,
            r_subgap: 100.0, // 10 * Rn
            material: Some(SuperconductorMaterial::niobium()),
        }
    }
}

impl JosephsonRcsjModel {
    /// Creates a new RCSJ model with given $I_c$, $R_n$, and $C_j$.
    pub fn new(ic: f64, rn: f64, cap: f64, phase_init: f64) -> Self {
        Self {
            ic: ic.max(1e-15),
            rn: rn.max(1e-6),
            cap: cap.max(1e-18),
            phase: phase_init,
            voltage: 0.0,
            r_subgap: 10.0 * rn.max(1e-6),
            material: None,
        }
    }

    /// Creates an overdamped junction suitable for RSFQ logic ($\beta_c \approx 1.0$).
    pub fn overdamped_rsfq(ic: f64, rn: f64) -> Self {
        // beta_c = 2 * pi * Ic * Rn^2 * C / Phi_0 = 1.0
        // C = Phi_0 / (2 * pi * Ic * Rn^2)
        let cap = FLUX_QUANTUM / (2.0 * std::f64::consts::PI * ic * rn * rn);
        Self::new(ic, rn, cap, 0.0)
    }

    /// Characteristic Josephson voltage $V_c = I_c R_n$ in Volts ($V$).
    #[inline]
    pub fn characteristic_voltage(&self) -> f64 {
        self.ic * self.rn
    }

    /// Stewart-McCumber non-dimensional damping parameter $\beta_c$:
    /// $$\beta_c = \frac{2\pi I_c R_n^2 C_j}{\Phi_0}$$
    /// - $\beta_c \le 1$: Overdamped (non-hysteretic, single-valued I-V, ideal for SFQ).
    /// - $\beta_c > 1$: Underdamped (hysteretic, bifurcated switching).
    #[inline]
    pub fn stewart_mccumber_beta_c(&self) -> f64 {
        (2.0 * std::f64::consts::PI * self.ic * self.rn * self.rn * self.cap) / FLUX_QUANTUM
    }

    /// Josephson plasma frequency $\omega_p$ in radians per second:
    /// $$\omega_p = \sqrt{\frac{2\pi I_c}{\Phi_0 C_j}}$$
    #[inline]
    pub fn plasma_frequency_rad_per_s(&self) -> f64 {
        ((2.0 * std::f64::consts::PI * self.ic) / (FLUX_QUANTUM * self.cap)).sqrt()
    }

    /// Josephson plasma frequency in Hertz: $f_p = \frac{\omega_p}{2\pi}$.
    #[inline]
    pub fn plasma_frequency_hz(&self) -> f64 {
        self.plasma_frequency_rad_per_s() / (2.0 * std::f64::consts::PI)
    }

    /// Zero-bias Josephson inductance $L_{J0} = \frac{\Phi_0}{2\pi I_c}$ in Henries ($H$).
    #[inline]
    pub fn zero_bias_inductance_h(&self) -> f64 {
        FLUX_QUANTUM / (2.0 * std::f64::consts::PI * self.ic)
    }

    /// Josephson coupling energy $E_J = \frac{\Phi_0 I_c}{2\pi}$ in Joules ($J$).
    #[inline]
    pub fn josephson_energy_joules(&self) -> f64 {
        (FLUX_QUANTUM * self.ic) / (2.0 * std::f64::consts::PI)
    }

    /// Josephson coupling energy in electron-volts ($eV$).
    #[inline]
    pub fn josephson_energy_ev(&self) -> f64 {
        self.josephson_energy_joules() / phonon_core::ELEMENTARY_CHARGE
    }

    /// Computes companion conductance $G_{eq}$ and current $I_{eq}$ for Trapezoidal transient integration.
    ///
    /// Discretization equations:
    /// $$\phi_{n+1} = \phi_n + \frac{\pi \Delta t}{\Phi_0} (V_{n+1} + V_n)$$
    /// $$I_{total} = I_c \sin(\phi_{n+1}) + \frac{V_{n+1}}{R_n} + \frac{2 C_j}{\Delta t} (V_{n+1} - V_n)$$
    ///
    /// Newton-Raphson linearization around $V_{n+1}^{(k)}$:
    /// $$G_s = \frac{\partial I_s}{\partial V_{n+1}} = I_c \cos(\phi_{n+1}) \cdot \frac{\pi \Delta t}{\Phi_0}$$
    /// $$G_{eq} = G_s + \frac{1}{R_n} + \frac{2 C_j}{\Delta t}$$
    /// $$I_{eq} = I_{total} - G_{eq} V_{n+1}^{(k)}$$
    pub fn companion_stamp_trapezoidal(&self, dt: f64, v_guess: f64) -> RcsjCompanionStamp {
        let h = dt.max(1e-18);
        let alpha = (std::f64::consts::PI * h) / FLUX_QUANTUM;
        let phi_next = self.phase + alpha * (v_guess + self.voltage);

        // Supercurrent and Josephson small-signal conductance
        let i_s = self.ic * phi_next.sin();
        let g_s = self.ic * phi_next.cos() * alpha;

        // Shunt resistance branch
        let g_n = 1.0 / self.rn;
        let i_r = g_n * v_guess;

        // Capacitance branch (trapezoidal)
        let g_c = (2.0 * self.cap) / h;
        let i_c = g_c * (v_guess - self.voltage);

        let i_total = i_s + i_r + i_c;
        let g_eq = (g_s + g_n + g_c).max(1e-12);
        let i_eq = g_eq * v_guess - i_total;

        RcsjCompanionStamp {
            g_eq,
            i_eq,
            i_total,
            phase_next: phi_next,
        }
    }

    /// Evaluates DC small-signal or quasi-static branch conductance.
    /// In DC MNA, a non-switching junction behaves as a dynamic non-linear element:
    /// $I = I_c \sin(\phi) + V / R_n$.
    pub fn dc_stamp(&self, v_dc: f64) -> (f64, f64) {
        let g_n = 1.0 / self.rn;
        let g_sc = 1e3; // Low-impedance superconducting state at DC
        let g_eq = g_n + g_sc;
        let i_s = self.ic * self.phase.sin();
        let i_total = i_s + g_eq * v_dc;
        let i_eq = g_eq * v_dc - i_total;
        (g_eq, i_eq)
    }

    /// Updates the internal time-step state upon convergence of $V_{n+1}$.
    pub fn advance_time_step(&mut self, dt: f64, v_converged: f64) {
        let h = dt.max(1e-18);
        let alpha = (std::f64::consts::PI * h) / FLUX_QUANTUM;
        self.phase += alpha * (v_converged + self.voltage);
        self.voltage = v_converged;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stewart_mccumber_damping() {
        let overdamped = JosephsonRcsjModel::overdamped_rsfq(100e-6, 5.0);
        let beta_c = overdamped.stewart_mccumber_beta_c();
        // Designed for beta_c = 1.0
        assert!((beta_c - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_plasma_frequency_and_energy() {
        let jj = JosephsonRcsjModel::default();
        let fp = jj.plasma_frequency_hz();
        let ej_ev = jj.josephson_energy_ev();

        // Standard 100 uA junction has fp in tens to hundreds of GHz
        assert!(fp > 10e9 && fp < 500e9);
        // Ej is in the ~ 0.1 - 1.0 eV range
        assert!(ej_ev > 0.05 && ej_ev < 1.0);
    }

    #[test]
    fn test_companion_stamping() {
        let mut jj = JosephsonRcsjModel::default();
        let dt = 1e-13; // 0.1 ps
        let stamp = jj.companion_stamp_trapezoidal(dt, 0.0);

        assert!(stamp.g_eq > 0.0);
        assert!(stamp.g_eq.is_finite());
        assert!(stamp.i_eq.is_finite());

        // Advance step
        jj.advance_time_step(dt, 0.001); // 1 mV
        assert!(jj.voltage == 0.001);
        assert!(jj.phase > 0.0);
    }
}
