//! Physical Shockley diode model with temperature scaling, reverse breakdown, and dynamic charge.

use crate::common::safe_exp;
use phonon_core::{thermal_voltage, T_REF};

/// Parameters defining a physical semiconductor diode junction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiodeModel {
    /// Saturation current $I_S$ at reference temperature in Amperes ($A$).
    pub is: f64,
    /// Emission coefficient / ideality factor $N$.
    pub n: f64,
    /// Parasitic bulk series resistance $R_S$ in Ohms ($\Omega$).
    pub rs: f64,
    /// Zero-bias junction capacitance $C_{J0}$ in Farads ($F$).
    pub cj0: f64,
    /// Built-in junction contact potential $V_J$ in Volts ($V$).
    pub vj: f64,
    /// Junction grading coefficient $M$ (typically $0.33$ to $0.5$).
    pub m: f64,
    /// Transit time $\tau_T$ for minority carrier diffusion charge in seconds ($s$).
    pub tt: f64,
    /// Reverse breakdown voltage $V_{BV}$ in Volts ($V$). Positive magnitude.
    pub bv: f64,
    /// Current at reverse breakdown in Amperes ($A$).
    pub ibv: f64,
    /// Saturation current temperature exponent $XTI$.
    pub xti: f64,
    /// Silicon bandgap in electron-volts ($eV$).
    pub eg: f64,
    /// Forward bias junction capacitance coefficient $FC$ (typically $0.5$).
    pub fc: f64,
}

impl Default for DiodeModel {
    fn default() -> Self {
        Self {
            is: 1e-14,
            n: 1.0,
            rs: 0.0,
            cj0: 1e-12,
            vj: 0.7,
            m: 0.5,
            tt: 1e-9,
            bv: 100.0,
            ibv: 1e-3,
            xti: 3.0,
            eg: 1.11,
            fc: 0.5,
        }
    }
}

/// Evaluated diode operating state: currents, conductances, charges, and capacitances.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DiodeEvaluation {
    /// Total junction current $I_D$ in Amperes ($A$).
    pub i_d: f64,
    /// Small-signal dynamic conductance $g_d = \frac{d I_D}{d V_D}$ in Siemens ($S$).
    pub g_d: f64,
    /// Total stored dynamic charge $Q_D = Q_{dep} + Q_{diff}$ in Coulombs ($C$).
    pub q_d: f64,
    /// Small-signal dynamic capacitance $C_d = \frac{d Q_D}{d V_D}$ in Farads ($F$).
    pub c_d: f64,
}

impl DiodeModel {
    /// Evaluates the physical diode at terminal voltage $V_D = V_{anode} - V_{cathode}$
    /// and junction temperature $T$ in Kelvin.
    pub fn evaluate(&self, vd: f64, temp_kelvin: f64) -> DiodeEvaluation {
        let vt = self.n * thermal_voltage(temp_kelvin);

        // Temperature scaling of saturation current I_S(T)
        let t_ratio = (temp_kelvin / T_REF).max(0.1);
        let eg_ev = self.eg;
        let is_t = self.is
            * t_ratio.powf(self.xti / self.n)
            * (-eg_ev / (self.n * 8.617_333_262e-5 * temp_kelvin) * (1.0 - t_ratio)).exp();

        // 1. Forward conduction via Shockley equation
        let arg = vd / vt;
        let (exp_val, exp_deriv) = safe_exp(arg);
        let mut i_d = is_t * (exp_val - 1.0);
        let mut g_d = (is_t / vt) * exp_deriv;

        // 2. Reverse breakdown (Avalanche / Zener)
        if vd < -self.bv {
            let arg_bv = -(vd + self.bv) / vt;
            let (bv_exp, bv_deriv) = safe_exp(arg_bv);
            i_d -= self.ibv * (bv_exp - 1.0);
            g_d += (self.ibv / vt) * bv_deriv;
        }

        // 3. Dynamic Charge Storage: Depletion + Diffusion
        // Diffusion charge: Q_diff = tt * I_D
        let q_diff = self.tt * i_d.max(0.0);
        let c_diff = self.tt * g_d.max(0.0);

        // Depletion charge: Q_dep(V_D)
        let (q_dep, c_dep) = self.evaluate_depletion(vd);

        let q_total = q_diff + q_dep;
        let c_total = c_diff + c_dep;

        DiodeEvaluation {
            i_d,
            g_d,
            q_d: q_total,
            c_d: c_total,
        }
    }

    #[inline]
    fn evaluate_depletion(&self, vd: f64) -> (f64, f64) {
        let fc = self.fc;
        let vj = self.vj;
        let m = self.m;
        let cj0 = self.cj0;
        let v_limit = fc * vj;

        if vd < v_limit {
            let s = 1.0 - vd / vj;
            let s_pow = s.powf(1.0 - m);
            let q_dep = (cj0 * vj / (1.0 - m)) * (1.0 - s_pow);
            let c_dep = cj0 * s.powf(-m);
            (q_dep, c_dep)
        } else {
            // Linearized extrapolation above FC * VJ to guarantee C^1 continuity without singularity
            let s_fc = 1.0 - fc;
            let q_base = (cj0 * vj / (1.0 - m)) * (1.0 - s_fc.powf(1.0 - m));
            let c_base = cj0 * s_fc.powf(-m);
            let dv = vd - v_limit;
            let q_dep = q_base + c_base * dv + 0.5 * (m * c_base / (vj * s_fc)) * dv * dv;
            let c_dep = c_base + (m * c_base / (vj * s_fc)) * dv;
            (q_dep, c_dep)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diode_forward_bias() {
        let model = DiodeModel::default();
        let eval = model.evaluate(0.7, 300.15);

        // At 0.7 V, a silicon diode with I_s = 1e-14 should conduct tens of milliamps
        assert!(
            eval.i_d > 1e-3,
            "Diode current must be in mA range at 0.7V, got {}",
            eval.i_d
        );
        assert!(
            eval.g_d > 0.01,
            "Conductance must be positive and non-trivial"
        );
        assert!(
            eval.c_d > model.cj0,
            "Capacitance must increase under forward bias"
        );
    }

    #[test]
    fn test_diode_reverse_bias() {
        let model = DiodeModel::default();
        let eval = model.evaluate(-5.0, 300.15);

        // In reverse bias, current should be near -I_s
        assert!((eval.i_d - (-model.is)).abs() < 1e-13);
        assert!(eval.g_d >= 0.0);
    }
}
