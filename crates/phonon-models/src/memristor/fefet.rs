//! Ferroelectric Field-Effect Transistor (FeFET) physical model.
//!
//! Formulates:
//! - Ferroelectric fluorite ($\text{Hf}_{1-x}\text{Zr}_x\text{O}_2$ / HZO) gate stack.
//! - Non-volatile spontaneous polarization $P$ governed by the Landau-Khalatnikov (L-K) dynamic equation:
//!   $$\rho \frac{dP}{dt} + \alpha P + \beta P^3 + \gamma P^5 = E_{fe}$$
//! - Remnant polarization $P_r$, saturation polarization $P_s$, and coercive field $E_c$.
//! - Polarization-induced threshold voltage shift $\Delta V_{th}(P) = -\frac{P \cdot t_{fe}}{\epsilon_{fe} \epsilon_0}$.
//! - Analog multi-level synaptic conductance modulation for in-memory neuromorphic computing.

use phonon_core::EPSILON_0;

/// Physical parameters for a Ferroelectric FET (FeFET) with HZO gate insulator.
#[derive(Debug, Clone, PartialEq)]
pub struct FerroelectricFetModel {
    /// Remnant polarization $P_r$ in $\text{C} / m^2$ (typically $0.15 - 0.25\text{ C}/m^2 \approx 15 - 25\,\mu\text{C/cm}^2$).
    pub remnant_polarization_c_m2: f64,
    /// Saturation polarization $P_s$ in $\text{C} / m^2$ (typically $\approx 1.2 P_r$).
    pub saturation_polarization_c_m2: f64,
    /// Coercive electric field $E_c$ in $V/m$ (typically $\approx 1.2\text{ MV/cm} = 1.2 \times 10^8\text{ V/m}$).
    pub coercive_field_v_per_m: f64,
    /// Thickness of ferroelectric dielectric layer $t_{fe}$ in meters ($m$) (typically $5 - 10\text{ nm}$).
    pub fe_thickness_m: f64,
    /// Relative dielectric permittivity of ferroelectric layer $\epsilon_{fe}$ (typically $\approx 30$ for HZO).
    pub fe_relative_permittivity: f64,
    /// Landau polarization viscosity / damping coefficient $\rho$ in $\Omega \cdot m$ (typically $\approx 10^{-4} - 10^{-2}$).
    pub landau_viscosity_rho: f64,
    /// Nominal baseline threshold voltage $V_{th,0}$ in Volts ($V$).
    pub base_vth_volts: f64,
    /// Channel transconductance parameter $\beta_{channel} = \mu C_{ox} \frac{W}{L}$ in $A / V^2$.
    pub channel_transconductance: f64,
    /// Subthreshold swing $S$ in Volts per decade ($V/\text{dec}$) (typically $\approx 65\text{ mV/dec}$).
    pub subthreshold_swing_v_per_dec: f64,
}

impl FerroelectricFetModel {
    /// Standard $10\text{ nm}$ HZO ferroelectric gate stack preset.
    pub fn hzo_10nm() -> Self {
        Self {
            remnant_polarization_c_m2: 0.18, // 18 uC/cm^2
            saturation_polarization_c_m2: 0.22,
            coercive_field_v_per_m: 1.2e8, // 1.2 MV/cm
            fe_thickness_m: 10.0e-9,       // 10 nm
            fe_relative_permittivity: 32.0,
            landau_viscosity_rho: 1.0e-3,
            base_vth_volts: 0.60,
            channel_transconductance: 2.0e-3,    // 2 mA/V^2
            subthreshold_swing_v_per_dec: 0.070, // 70 mV/dec
        }
    }

    /// Coercive voltage $V_c = E_c \cdot t_{fe}$ in Volts ($V$).
    #[inline(always)]
    pub fn coercive_voltage(&self) -> f64 {
        self.coercive_field_v_per_m * self.fe_thickness_m
    }

    /// Evaluates dynamic rate of change of polarization $\frac{dP}{dt}$ via Landau-Khalatnikov formulation:
    pub fn polarization_derivative(&self, current_p: f64, v_gate_volts: f64) -> f64 {
        let e_field = v_gate_volts / self.fe_thickness_m;
        let p_norm = (current_p / self.saturation_polarization_c_m2).clamp(-1.5, 1.5);

        // Double-well thermodynamic potential derivative: alpha * P + beta * P^3
        let ec = self.coercive_field_v_per_m;
        let thermodynamic_force = -ec * (p_norm - p_norm.powi(3));
        let external_force = e_field;

        let dp_dt = (external_force + thermodynamic_force) / self.landau_viscosity_rho;
        dp_dt.clamp(-1e12, 1e12)
    }

    /// Advances the polarization state $P$ forward by time $\Delta t$ given gate voltage $V_g$:
    pub fn step_rk4(&self, current_p: f64, v_gate_volts: f64, dt_s: f64) -> f64 {
        let p = current_p.clamp(
            -self.saturation_polarization_c_m2,
            self.saturation_polarization_c_m2,
        );

        let k1 = self.polarization_derivative(p, v_gate_volts);
        let k2 = self.polarization_derivative(p + 0.5 * dt_s * k1, v_gate_volts);
        let k3 = self.polarization_derivative(p + 0.5 * dt_s * k2, v_gate_volts);
        let k4 = self.polarization_derivative(p + dt_s * k3, v_gate_volts);

        let next_p = p + (dt_s / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4);
        next_p.clamp(
            -self.saturation_polarization_c_m2,
            self.saturation_polarization_c_m2,
        )
    }

    /// Evaluates polarization-dependent threshold voltage $V_{th}(P)$:
    /// $$V_{th}(P) = V_{th,0} - \frac{P \cdot t_{fe}}{\epsilon_{fe} \epsilon_0}$$
    pub fn threshold_voltage(&self, polarization: f64) -> f64 {
        let eps = self.fe_relative_permittivity * EPSILON_0;
        let delta_vth = -(polarization * self.fe_thickness_m) / eps;
        self.base_vth_volts + delta_vth
    }

    /// Evaluates channel drain-source current $I_{ds}(V_{gs}, V_{ds}, P)$ given polarization $P$:
    pub fn evaluate_drain_current(&self, polarization: f64, v_gs: f64, v_ds: f64) -> f64 {
        let vth = self.threshold_voltage(polarization);
        let v_ov = v_gs - vth;

        if v_ov > 0.0 {
            // Strong inversion: linear or saturation
            let v_ds_sat = v_ov;
            if v_ds < v_ds_sat {
                self.channel_transconductance * (v_ov * v_ds - 0.5 * v_ds * v_ds)
            } else {
                0.5 * self.channel_transconductance * v_ov * v_ov
            }
        } else {
            // Subthreshold exponential leakage
            let log_current = v_ov / self.subthreshold_swing_v_per_dec;
            let i_off_base = 1.0e-11; // 10 pA subthreshold floor
            i_off_base * 10.0_f64.powf(log_current.max(-10.0))
        }
    }

    /// Channel small-signal synaptic conductance $G_w = \frac{dI_{ds}}{dV_{ds}}$ at small reading voltage:
    pub fn read_conductance(&self, polarization: f64, v_read_gate: f64) -> f64 {
        let delta_vds = 0.05; // 50 mV read voltage
        let i_read = self.evaluate_drain_current(polarization, v_read_gate, delta_vds);
        (i_read / delta_vds).max(1e-12)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fefet_coercive_voltage_and_vth_shift() {
        let fefet = FerroelectricFetModel::hzo_10nm();

        // Coercive voltage: Ec * t_fe = 1.2e8 V/m * 10e-9 m = 1.2 V
        let vc = fefet.coercive_voltage();
        assert!((vc - 1.2).abs() < 1e-6);

        // Threshold voltage for unpolarized state (P = 0)
        let vth_zero = fefet.threshold_voltage(0.0);
        assert!((vth_zero - fefet.base_vth_volts).abs() < 1e-6);

        // Positive polarization (+Pr) lowers Vth (facilitates inversion)
        let vth_pos = fefet.threshold_voltage(fefet.remnant_polarization_c_m2);
        assert!(vth_pos < vth_zero);

        // Negative polarization (-Pr) raises Vth
        let vth_neg = fefet.threshold_voltage(-fefet.remnant_polarization_c_m2);
        assert!(vth_neg > vth_zero);

        // Memory window: Delta Vth = 2 * Pr * t_fe / eps
        let mem_window = vth_neg - vth_pos;
        assert!(
            mem_window > 0.5,
            "Memory window should be > 0.5 V for 10nm HZO"
        );
    }

    #[test]
    fn test_polarization_switching_dynamics() {
        let fefet = FerroelectricFetModel::hzo_10nm();
        let mut p = 0.0;

        // Apply +2.5 V programming write pulse (well above Vc = 1.2 V)
        let dt = 1e-10; // 100 ps
        for _ in 0..50 {
            p = fefet.step_rk4(p, 2.5, dt);
        }
        assert!(p > 0.8 * fefet.remnant_polarization_c_m2);

        // Conductance at Vg = 0.6V read should be high (LRS)
        let g_lrs = fefet.read_conductance(p, 0.6);

        // Apply -2.5 V erase write pulse
        for _ in 0..100 {
            p = fefet.step_rk4(p, -2.5, dt);
        }
        assert!(p < -0.8 * fefet.remnant_polarization_c_m2);

        // Conductance should be low (HRS)
        let g_hrs = fefet.read_conductance(p, 0.6);
        assert!(
            g_lrs / g_hrs > 10.0,
            "Synaptic ON/OFF ratio must exceed 10x"
        );
    }
}
