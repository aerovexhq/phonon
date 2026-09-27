//! Gummel-Poon Bipolar Junction Transistor (BJT) model with Early effect and high-injection knee rolloff.

use crate::common::safe_exp;
use phonon_core::thermal_voltage;

/// Polarity of the bipolar transistor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BjtType {
    Npn,
    Pnp,
}

/// Physical parameters for the Gummel-Poon BJT compact model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BjtModel {
    pub bjt_type: BjtType,
    /// Transport saturation current $I_S$ in Amperes ($A$).
    pub is: f64,
    /// Ideal maximum forward current gain $\beta_F$.
    pub bf: f64,
    /// Ideal maximum reverse current gain $\beta_R$.
    pub br: f64,
    /// Forward Early voltage $V_{AF}$ in Volts ($V$).
    pub vaf: f64,
    /// Reverse Early voltage $V_{AR}$ in Volts ($V$).
    pub var: f64,
    /// Forward knee current for high-injection rolloff $I_{KF}$ in Amperes ($A$).
    pub ikf: f64,
    /// Reverse knee current for high-injection rolloff $I_{KR}$ in Amperes ($A$).
    pub ikr: f64,
    /// Forward current emission coefficient $N_F$.
    pub nf: f64,
    /// Reverse current emission coefficient $N_R$.
    pub nr: f64,
}

impl Default for BjtModel {
    fn default() -> Self {
        Self {
            bjt_type: BjtType::Npn,
            is: 1e-16,
            bf: 100.0,
            br: 1.0,
            vaf: 100.0,
            var: 20.0,
            ikf: 0.1,
            ikr: 0.02,
            nf: 1.0,
            nr: 1.0,
        }
    }
}

/// Operating state evaluation of the Gummel-Poon BJT.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BjtEvaluation {
    /// Terminal Collector current $I_C$ in Amperes ($A$).
    pub i_c: f64,
    /// Terminal Base current $I_B$ in Amperes ($A$).
    pub i_b: f64,
    /// Terminal Emitter current $I_E = -(I_C + I_B)$ in Amperes ($A$).
    pub i_e: f64,
    /// Transconductance $g_m = \frac{\partial I_C}{\partial V_{be}}$ in Siemens ($S$).
    pub g_m: f64,
    /// Base-emitter dynamic conductance $g_\pi = \frac{\partial I_B}{\partial V_{be}}$ in Siemens ($S$).
    pub g_pi: f64,
    /// Output conductance $g_o = -\frac{\partial I_C}{\partial V_{bc}}$ in Siemens ($S$).
    pub g_o: f64,
    /// Base-collector dynamic conductance $g_\mu = \frac{\partial I_B}{\partial V_{bc}}$ in Siemens ($S$).
    pub g_mu: f64,
}

impl BjtModel {
    /// Evaluates the Gummel-Poon BJT at terminal voltages $(V_{collector}, V_{base}, V_{emitter})$
    /// and operating temperature $T$ in Kelvin.
    pub fn evaluate(&self, v_c: f64, v_b: f64, v_e: f64, temp_kelvin: f64) -> BjtEvaluation {
        let sign = match self.bjt_type {
            BjtType::Npn => 1.0,
            BjtType::Pnp => -1.0,
        };

        let v_be = sign * (v_b - v_e);
        let v_bc = sign * (v_b - v_c);

        let vt_f = self.nf * thermal_voltage(temp_kelvin);
        let vt_r = self.nr * thermal_voltage(temp_kelvin);

        // Forward and reverse ideal diode exponential currents
        let (e_f, de_f) = safe_exp(v_be / vt_f);
        let (e_r, de_r) = safe_exp(v_bc / vt_r);

        let i_f = self.is * (e_f - 1.0);
        let i_r = self.is * (e_r - 1.0);

        let di_f_dvbe = (self.is / vt_f) * de_f;
        let di_r_dvbc = (self.is / vt_r) * de_r;

        // Normalized Base Charge q_b accounting for Early effect and high injection
        // q_1 = 1 + V_bc / V_AF + V_be / V_AR
        let q1 = 1.0 + (v_bc / self.vaf) + (v_be / self.var);
        // q_2 = I_f / I_KF + I_r / I_KR
        let q2 = (i_f / self.ikf).max(0.0) + (i_r / self.ikr).max(0.0);
        let root_qb = (0.25 * q1 * q1 + q2).max(1e-6).sqrt();
        let q_b = 0.5 * q1 + root_qb;

        // Terminal currents
        let i_ct = (i_f - i_r) / q_b;
        let i_b = (i_f / self.bf) + (i_r / self.br);
        let i_c = i_ct - (i_r / self.br);
        let i_e = -(i_c + i_b);

        // Small-signal companion conductances
        let g_m = (di_f_dvbe / q_b).max(1e-12);
        let g_pi = (di_f_dvbe / self.bf).max(1e-12);
        let g_mu = (di_r_dvbc / self.br).max(1e-12);
        let g_o = ((di_r_dvbc / q_b) + (i_ct / self.vaf)).max(1e-12);

        BjtEvaluation {
            i_c: sign * i_c,
            i_b: sign * i_b,
            i_e: sign * i_e,
            g_m,
            g_pi,
            g_o,
            g_mu,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_npn_forward_active_bias() {
        let bjt = BjtModel::default();

        // Forward active: Vbe = 0.7V, Vce = 5V -> Vbc = -4.3V
        let eval = bjt.evaluate(5.0, 0.7, 0.0, 300.15);

        assert!(
            eval.i_c > 1e-5,
            "Collector current should be substantial: {}",
            eval.i_c
        );
        assert!(eval.i_b > 0.0, "Base current must be positive");
        assert!(
            eval.i_c > eval.i_b * 50.0,
            "Current gain beta should exceed 50"
        );
        assert!(
            eval.g_m > 0.001,
            "Transconductance gm should be substantial"
        );
        assert_eq!(eval.i_e, -(eval.i_c + eval.i_b));
    }
}
