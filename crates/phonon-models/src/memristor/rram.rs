//! Filamentary Resistive RAM (RRAM / ReRAM) physical device model.
//!
//! Formulates:
//! - Metal-Oxide-Metal ($\text{TiN} / \text{HfO}_x / \text{Pt}$) oxygen vacancy migration dynamics.
//! - State variable $w \in [0, 1]$ representing normalized conductive filament protrusion.
//! - Non-linear field- and temperature-driven ionic drift rate equation:
//!   $$\frac{dw}{dt} = v_0 \exp\left(-\frac{E_a - q \alpha |V|}{k_B T}\right) \sinh\left(\frac{q \beta V}{k_B T}\right)$$
//! - Ohmic conduction in LRS, Poole-Frenkel / Trap-Assisted Tunneling in HRS.
//! - Current compliance limiting $I_{comp}$ preventing permanent dielectric breakdown.
//! - Pinched hysteresis loop with $I = 0$ at $V = 0$.

use phonon_core::{MemristorState, BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE};

/// Physical parameters for a filamentary transition metal oxide RRAM cell (e.g. $\text{HfO}_x$).
#[derive(Debug, Clone, PartialEq)]
pub struct FilamentaryRramModel {
    /// Nominal low-resistance state (LRS) resistance in Ohms ($\Omega$) (e.g. $1\text{ k}\Omega$).
    pub r_on: f64,
    /// Nominal high-resistance state (HRS) resistance in Ohms ($\Omega$) (e.g. $100\text{ k}\Omega$ to $1\text{ M}\Omega$).
    pub r_off: f64,
    /// Activation energy for oxygen vacancy migration $E_a$ in electron-Volts ($eV$) (typically $\approx 0.8 - 1.2\text{ eV}$).
    pub activation_energy_ev: f64,
    /// Ionic hopping velocity prefactor $v_0$ in $s^{-1}$ (typically $\approx 10^7 - 10^9\text{ s}^{-1}$).
    pub velocity_prefactor: f64,
    /// Forward field lowering coefficient $\alpha$ (SET process).
    pub alpha_set: f64,
    /// Reverse field lowering coefficient $\beta$ (RESET process).
    pub beta_reset: f64,
    /// SET threshold voltage magnitude in Volts ($V$) (e.g. $+1.0\text{ V}$).
    pub v_set: f64,
    /// RESET threshold voltage magnitude in Volts ($V$) (e.g. $-0.8\text{ V}$).
    pub v_reset: f64,
    /// Compliance current $I_{comp}$ in Amperes ($A$) (e.g. $100\,\mu\text{A}$).
    pub compliance_current_a: f64,
    /// Non-linearity coefficient $\kappa$ for HRS Poole-Frenkel / tunneling conduction.
    pub non_linear_factor: f64,
}

impl FilamentaryRramModel {
    /// Standard $\text{TiN} / \text{HfO}_2 / \text{Pt}$ 1T1R neuromorphic synaptic RRAM preset.
    pub fn hfo2_synaptic() -> Self {
        Self {
            r_on: 2.0e3,                // 2 kOhm LRS
            r_off: 200.0e3,             // 200 kOhm HRS (100x window)
            activation_energy_ev: 0.95, // 0.95 eV
            velocity_prefactor: 1.0e8,  // 1e8 s^-1
            alpha_set: 0.45,
            beta_reset: 0.50,
            v_set: 1.1,                   // +1.1 V SET
            v_reset: -0.75,               // -0.75 V RESET
            compliance_current_a: 5.0e-4, // 500 uA
            non_linear_factor: 1.8,
        }
    }

    /// Evaluates the rate of change of filament protrusion $\frac{dw}{dt}$ given voltage $V$ and temperature $T$:
    pub fn state_derivative(&self, w: f64, v_volts: f64, temp_k: f64) -> f64 {
        let vt = (BOLTZMANN_CONSTANT * temp_k) / ELEMENTARY_CHARGE;
        let ea_joules = self.activation_energy_ev * ELEMENTARY_CHARGE;

        if v_volts > 0.0 {
            // SET: filament growth towards w = 1
            if w >= 0.999 && v_volts < self.v_set {
                return 0.0;
            }
            let eff_barrier =
                (ea_joules - ELEMENTARY_CHARGE * self.alpha_set * v_volts).max(0.05 * ea_joules);
            let arrhenius = (-eff_barrier / (BOLTZMANN_CONSTANT * temp_k)).exp();
            let sinh_term = ((v_volts / (2.0 * vt)).min(15.0)).sinh();
            let dw = self.velocity_prefactor * arrhenius * sinh_term * (1.0 - w);
            dw.clamp(-1e9, 1e9)
        } else if v_volts < 0.0 {
            // RESET: filament dissolution towards w = 0
            if w <= 0.001 && v_volts > self.v_reset {
                return 0.0;
            }
            let eff_barrier =
                (ea_joules + ELEMENTARY_CHARGE * self.beta_reset * v_volts).max(0.05 * ea_joules);
            let arrhenius = (-eff_barrier / (BOLTZMANN_CONSTANT * temp_k)).exp();
            let sinh_term = ((v_volts / (2.0 * vt)).max(-15.0)).sinh();
            let dw = self.velocity_prefactor * arrhenius * sinh_term * w;
            dw.clamp(-1e9, 1e9)
        } else {
            0.0
        }
    }

    /// Advances the memristor internal state $w$ forward by time $\Delta t$ using 4th-order Runge-Kutta (RK4).
    pub fn step_rk4(
        &self,
        state: MemristorState,
        v_volts: f64,
        temp_k: f64,
        dt_s: f64,
    ) -> MemristorState {
        let w = state.internal_state_w.clamp(0.0, 1.0);

        let k1 = self.state_derivative(w, v_volts, temp_k);
        let k2 = self.state_derivative((w + 0.5 * dt_s * k1).clamp(0.0, 1.0), v_volts, temp_k);
        let k3 = self.state_derivative((w + 0.5 * dt_s * k2).clamp(0.0, 1.0), v_volts, temp_k);
        let k4 = self.state_derivative((w + dt_s * k3).clamp(0.0, 1.0), v_volts, temp_k);

        let next_w = (w + (dt_s / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4)).clamp(0.0, 1.0);
        let g_eff = self.effective_conductance(next_w, v_volts);

        MemristorState {
            conductance_s: g_eff,
            internal_state_w: next_w,
        }
    }

    /// Effective chord conductance $G(w, V)$ taking into account LRS Ohmic and HRS non-linear conduction:
    pub fn effective_conductance(&self, w: f64, v_volts: f64) -> f64 {
        let g_on = 1.0 / self.r_on;
        let g_off = 1.0 / self.r_off;

        // HRS exhibits non-linear Poole-Frenkel field enhancement:
        let non_lin = (self.non_linear_factor * v_volts.abs()).min(10.0).cosh();
        let g_hrs = g_off * non_lin;

        let g_lin = w * g_on + (1.0 - w) * g_hrs;
        g_lin.clamp(1.0 / self.r_off, 1.0 / (0.5 * self.r_on))
    }

    /// Evaluates device current $I(V, w)$ and small-signal dynamic conductance $g_d = \frac{dI}{dV}$.
    /// Strictly guarantees $I = 0$ when $V = 0$ (pinched hysteresis law).
    pub fn evaluate_current_and_conductance(&self, w: f64, v_volts: f64) -> (f64, f64) {
        if v_volts.abs() < 1e-12 {
            let g = self.effective_conductance(w, 0.0);
            return (0.0, g);
        }

        let g_eff = self.effective_conductance(w, v_volts);
        let mut i_raw = g_eff * v_volts;

        // Apply compliance current limit in positive bias
        if i_raw > self.compliance_current_a {
            i_raw = self.compliance_current_a;
        }

        // Small perturbation for dynamic conductance
        let delta_v = 1e-4;
        let g_eff_plus = self.effective_conductance(w, v_volts + delta_v);
        let mut i_plus = g_eff_plus * (v_volts + delta_v);
        if i_plus > self.compliance_current_a {
            i_plus = self.compliance_current_a;
        }

        let gd = ((i_plus - i_raw) / delta_v).max(1e-12);
        (i_raw, gd)
    }

    /// Generates MNA Newton-Raphson companion stamp: `(conductance_gd, rhs_current_ieq)`.
    pub fn companion_stamp(&self, w: f64, v_volts: f64) -> (f64, f64) {
        let (i_d, g_d) = self.evaluate_current_and_conductance(w, v_volts);
        let i_eq = g_d * v_volts - i_d;
        (g_d, i_eq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phonon_core::T_REF;

    #[test]
    fn test_pinched_hysteresis_zero_crossing() {
        let rram = FilamentaryRramModel::hfo2_synaptic();
        let (i_zero, gd_zero) = rram.evaluate_current_and_conductance(0.5, 0.0);
        assert_eq!(
            i_zero, 0.0,
            "Memristor current must be exactly zero at zero voltage"
        );
        assert!(gd_zero > 0.0);
    }

    #[test]
    fn test_set_and_reset_threshold_dynamics() {
        let rram = FilamentaryRramModel::hfo2_synaptic();
        let state_hrs = MemristorState::new(1.0 / rram.r_off, 0.0);

        // At low reading voltage (+0.1 V), dw/dt should be virtually zero (non-destructive read)
        let dw_read = rram.state_derivative(0.0, 0.1, T_REF);
        assert!(dw_read.abs() < 1.0);

        // At SET voltage (+1.5 V), dw/dt must be strongly positive
        let dw_set = rram.state_derivative(0.0, 1.5, T_REF);
        assert!(dw_set > 100.0);

        // Advance state with 10 ns SET pulse
        let dt = 1e-9;
        let mut state = state_hrs;
        for _ in 0..20 {
            state = rram.step_rk4(state, 1.5, T_REF, dt);
        }
        // State w must have grown towards LRS
        assert!(state.internal_state_w > 0.5);
        assert!(state.conductance_s > 10.0 / rram.r_off);
    }

    #[test]
    fn test_compliance_current_limiting() {
        let rram = FilamentaryRramModel::hfo2_synaptic();
        // Fully formed filament (w = 1.0) under large voltage (3.0 V)
        // Without compliance: I = 3.0 / 2000 = 1.5 mA
        // With compliance: clamped to 500 uA
        let (i_meas, _) = rram.evaluate_current_and_conductance(1.0, 3.0);
        assert!((i_meas - 5.0e-4).abs() < 1e-6);
    }
}
