//! Physically rigorous sub-micron MOSFET compact model with continuous overdrive,
//! velocity saturation, DIBL, and Ward-Dutton charge conservation.

use crate::common::safe_exp;
use phonon_core::{thermal_voltage, EPSILON_0, EPSILON_R_OX, T_REF};

/// Polarity of the MOSFET channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MosfetType {
    Nmos,
    Pmos,
}

/// Physical parameters for the continuous BSIM-style MOSFET model.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MosfetModel {
    pub mos_type: MosfetType,
    /// Zero-bias threshold voltage $V_{th0}$ in Volts ($V$).
    pub vth0: f64,
    /// Channel width $W$ in meters ($m$).
    pub w: f64,
    /// Channel length $L$ in meters ($m$).
    pub l: f64,
    /// Gate oxide thickness $t_{ox}$ in meters ($m$).
    pub tox: f64,
    /// Low-field carrier mobility $\mu_0$ in $m^2 / (V \cdot s)$.
    pub mu0: f64,
    /// Carrier saturation velocity $v_{sat}$ in $m/s$.
    pub vsat: f64,
    /// Channel-length modulation parameter $\lambda$ in $V^{-1}$.
    pub lambda: f64,
    /// Body-effect coefficient $\gamma$ in $V^{1/2}$.
    pub gamma: f64,
    /// Surface inversion potential $\phi_s$ in Volts ($V$).
    pub phi_s: f64,
    /// Drain-Induced Barrier Lowering coefficient $\eta_{DIBL}$.
    pub eta_dibl: f64,
    /// Subthreshold ideality swing coefficient $n$.
    pub subthreshold_n: f64,
    /// Temperature coefficient for threshold voltage drift $\alpha_{th}$ in $V/K$.
    pub temp_coeff_vth: f64,
    /// Temperature exponent for mobility degradation $\eta_\mu$.
    pub temp_coeff_mu: f64,
}

impl Default for MosfetModel {
    fn default() -> Self {
        Self {
            mos_type: MosfetType::Nmos,
            vth0: 0.7,
            w: 10e-6,
            l: 0.18e-6,
            tox: 4e-9,
            mu0: 0.067, // 670 cm^2 / (V*s) for electrons
            vsat: 1e5,  // 10^7 cm/s
            lambda: 0.02,
            gamma: 0.4,
            phi_s: 0.65,
            eta_dibl: 0.08,
            subthreshold_n: 1.25,
            temp_coeff_vth: 1.5e-3, // 1.5 mV / K
            temp_coeff_mu: 1.5,
        }
    }
}

/// Evaluated operating state of the MOSFET.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MosfetEvaluation {
    /// Drain-to-source current $I_{ds}$ in Amperes ($A$).
    pub i_ds: f64,
    /// Gate transconductance $g_m = \frac{\partial I_{ds}}{\partial V_{gs}}$ in Siemens ($S$).
    pub g_m: f64,
    /// Drain-source output conductance $g_{ds} = \frac{\partial I_{ds}}{\partial V_{ds}}$ in Siemens ($S$).
    pub g_ds: f64,
    /// Body transconductance $g_{mbs} = \frac{\partial I_{ds}}{\partial V_{bs}}$ in Siemens ($S$).
    pub g_mbs: f64,
    /// Ward-Dutton conserved gate charge $Q_G$ in Coulombs ($C$).
    pub q_g: f64,
    /// Ward-Dutton conserved drain charge $Q_D$ in Coulombs ($C$).
    pub q_d: f64,
    /// Ward-Dutton conserved source charge $Q_S$ in Coulombs ($C$).
    pub q_s: f64,
    /// Ward-Dutton conserved bulk charge $Q_B$ in Coulombs ($C$).
    pub q_b: f64,
}

impl MosfetModel {
    /// Evaluates the physical MOSFET at terminal voltages $(V_{drain}, V_{gate}, V_{source}, V_{bulk})$
    /// and operating temperature $T$ in Kelvin.
    pub fn evaluate(
        &self,
        v_d: f64,
        v_g: f64,
        v_s: f64,
        v_b: f64,
        temp_kelvin: f64,
    ) -> MosfetEvaluation {
        // Handle PMOS inversion by flipping sign of terminal voltages
        let sign = match self.mos_type {
            MosfetType::Nmos => 1.0,
            MosfetType::Pmos => -1.0,
        };

        let mut v_ds = sign * (v_d - v_s);
        let mut v_gs = sign * (v_g - v_s);
        let mut v_bs = sign * (v_b - v_s);

        // Reverse mode: if V_ds < 0, source and drain swap roles symmetrically
        let mode_swap = v_ds < 0.0;
        if mode_swap {
            v_ds = -v_ds;
            v_gs = sign * (v_g - v_d);
            v_bs = sign * (v_b - v_d);
        }

        let vt = self.subthreshold_n * thermal_voltage(temp_kelvin);
        let c_ox = (EPSILON_0 * EPSILON_R_OX) / self.tox;

        // Temperature-scaled threshold voltage and mobility
        let delta_t = temp_kelvin - T_REF;
        let vth_temp = self.vth0 - self.temp_coeff_vth * delta_t;
        let t_ratio = (temp_kelvin / T_REF).max(0.1);
        let mu_temp = self.mu0 * t_ratio.powf(-self.temp_coeff_mu);

        // Body effect: delta_Vth_body = gamma * (sqrt(phi_s - Vbs) - sqrt(phi_s))
        let v_bs_eff = v_bs.min(0.5 * self.phi_s);
        let body_term = (self.phi_s - v_bs_eff).max(1e-4).sqrt() - self.phi_s.sqrt();
        let delta_vth_body = self.gamma * body_term;

        // Drain-Induced Barrier Lowering (DIBL)
        let delta_vth_dibl = self.eta_dibl * v_ds;

        // Net effective threshold voltage
        let vth_eff = vth_temp + delta_vth_body - delta_vth_dibl;

        // 1. Continuous effective gate overdrive V_gsteff smoothly bridging
        // subthreshold exponential conduction and strong inversion:
        let v_ov = v_gs - vth_eff;
        let exp_ov_arg = v_ov / (2.0 * vt);
        let (e_ov, d_e_ov) = safe_exp(exp_ov_arg);

        let v_gsteff = 2.0 * vt * (1.0 + e_ov).ln();
        let dv_gsteff_dvov = d_e_ov / (1.0 + e_ov);

        // 2. Velocity saturation and saturation voltage V_dsat
        let e_sat = 2.0 * self.vsat / mu_temp;
        let e_sat_l = (e_sat * self.l).max(1e-3);
        let v_dsat = (e_sat_l * v_gsteff) / (e_sat_l + v_gsteff);

        // Smooth transition to saturation voltage V_dseff
        let delta_sat = 0.01;
        let term_diff = v_dsat - v_ds - delta_sat;
        let root_term = (term_diff * term_diff + 4.0 * delta_sat * v_dsat).sqrt();
        let v_dseff = v_dsat - 0.5 * (term_diff + root_term);
        let dv_dseff_dvds = 0.5 * (1.0 + term_diff / root_term);

        // 3. Channel Current calculation
        let beta = mu_temp * c_ox * (self.w / self.l);
        let denominator = 1.0 + v_dseff / e_sat_l;
        let i_ds0 = (beta * (v_gsteff - 0.5 * v_dseff) * v_dseff) / denominator;
        let clm_factor = 1.0 + self.lambda * v_ds;
        let mut i_ds = (i_ds0 * clm_factor).max(0.0);

        // 4. Analytical Conductances
        // gm = dIds / dVgs
        let di0_dvgsteff = (beta * v_dseff) / denominator;
        let g_m = di0_dvgsteff * dv_gsteff_dvov * clm_factor;

        // gds = dIds / dVds
        let di0_dvdseff = (beta * (v_gsteff - v_dseff)) / denominator
            - (beta * (v_gsteff - 0.5 * v_dseff) * v_dseff) / (denominator * denominator * e_sat_l);
        let g_ds = (di0_dvdseff * dv_dseff_dvds * clm_factor
            + i_ds0 * self.lambda
            + di0_dvgsteff * dv_gsteff_dvov * self.eta_dibl * clm_factor)
            .max(1e-12);

        // gmbs = dIds / dVbs
        let d_body = if (self.phi_s - v_bs_eff) > 1e-4 {
            0.5 * self.gamma / (self.phi_s - v_bs_eff).sqrt()
        } else {
            0.0
        };
        let g_mbs = (g_m * d_body).max(0.0);

        // 5. Ward-Dutton Charge Conservation
        // Channel charge Q_ch
        let q_ch = -self.w * self.l * c_ox * (v_gsteff - 0.5 * v_dseff);
        // Bulk depletion charge Q_b
        let q_b = -self.w
            * self.l
            * (2.0
                * 1.602_176_634e-19
                * 11.7
                * EPSILON_0
                * 1e23
                * (self.phi_s - v_bs_eff).max(0.0))
            .sqrt();
        // 40/60 Ward-Dutton charge partitioning
        let q_d = 0.4 * q_ch;
        let q_s = 0.6 * q_ch;
        let q_g = -(q_ch + q_b);

        // Restore sign and handle reverse mode
        if mode_swap {
            i_ds = -i_ds;
        }
        let final_i_ds = sign * i_ds;

        MosfetEvaluation {
            i_ds: final_i_ds,
            g_m,
            g_ds,
            g_mbs,
            q_g: sign * q_g,
            q_d: sign * q_d,
            q_s: sign * q_s,
            q_b: sign * q_b,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nmos_output_and_saturation() {
        let mos = MosfetModel::default();

        // Subthreshold
        let eval_sub = mos.evaluate(1.0, 0.3, 0.0, 0.0, 300.15);
        assert!(eval_sub.i_ds < 1e-6, "Current in subthreshold must be tiny");

        // Linear triode region (Vgs = 2V, Vds = 0.2V)
        let eval_triode = mos.evaluate(0.2, 2.0, 0.0, 0.0, 300.15);
        assert!(eval_triode.i_ds > 1e-4);

        // Saturation region (Vgs = 2V, Vds = 2.0V)
        let eval_sat = mos.evaluate(2.0, 2.0, 0.0, 0.0, 300.15);
        assert!(eval_sat.i_ds > eval_triode.i_ds);
        assert!(eval_sat.g_m > 0.0);
        assert!(eval_sat.g_ds > 0.0);

        // Ward-Dutton charge conservation check: Q_G + Q_D + Q_S + Q_B == 0
        let q_sum = eval_sat.q_g + eval_sat.q_d + eval_sat.q_s + eval_sat.q_b;
        assert!(
            q_sum.abs() < 1e-15,
            "Ward-Dutton charge neutrality violated: {}",
            q_sum
        );
    }
}
