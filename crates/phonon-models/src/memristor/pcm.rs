//! Phase-Change Memory (PCM / PCRAM) physical device model.
//!
//! Formulates:
//! - Chalcogenide glass alloy ($\text{Ge}_2\text{Sb}_2\text{Te}_5$ / GST) mushroom cell architecture.
//! - Crystalline volume fraction state variable $u_c \in [0, 1]$.
//! - Thermal melting above $T_{melt} \approx 873\text{ K}$ and fast melt-quench amorphization (RESET).
//! - Johnson-Mehl-Avrami-Kolmogorov (JMAK) isothermal crystallization kinetics (SET).
//! - Ovonic Threshold Switching (OTS) sub-threshold activation field and snapback conduction.
//! - Coupled electro-thermal Joule self-heating dynamics.

use phonon_core::{MemristorState, BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE};

/// Physical parameters for a Phase-Change Memory (PCM) GST mushroom cell.
#[derive(Debug, Clone, PartialEq)]
pub struct PhaseChangeMemoryModel {
    /// Crystalline state low resistance $R_{cryst}$ in Ohms ($\Omega$) (e.g. $5\text{ k}\Omega$).
    pub r_cryst: f64,
    /// Amorphous state high resistance $R_{amorph}$ in Ohms ($\Omega$) (e.g. $1\text{ M}\Omega$).
    pub r_amorph: f64,
    /// Melting temperature $T_{melt}$ in Kelvin ($K$) (typically $873 - 900\text{ K}$).
    pub t_melt_k: f64,
    /// Peak crystallization temperature $T_{cryst}$ in Kelvin ($K$) (typically $\approx 600\text{ K}$).
    pub t_cryst_k: f64,
    /// Thermal resistance of mushroom heater contact $R_{th}$ in $\text{K} / \text{W}$ (typically $\approx 2 \times 10^6\text{ K/W}$).
    pub thermal_resistance_k_per_w: f64,
    /// Thermal relaxation time constant $\tau_{th}$ in seconds ($s$) (typically $\approx 2\text{ ns}$).
    pub thermal_time_constant_s: f64,
    /// Crystallization activation energy $E_{cryst}$ in $eV$ (typically $\approx 2.0\text{ eV}$).
    pub e_cryst_ev: f64,
    /// JMAK kinetic growth rate prefactor $K_0$ in $s^{-1}$.
    pub k0_cryst_prefactor: f64,
    /// Avrami exponent $n$ (typically $2.0 - 3.0$ for GST).
    pub avrami_exponent: f64,
    /// Ovonic threshold switching voltage $V_{th}$ in Volts ($V$) (typically $\approx 1.0\text{ V}$).
    pub v_threshold_ots: f64,
    /// Dynamic ON-state holding resistance after threshold switching in Ohms ($\Omega$).
    pub r_holding_ots: f64,
}

impl PhaseChangeMemoryModel {
    /// Standard $\text{Ge}_2\text{Sb}_2\text{Te}_5$ (GST) nanoscale mushroom cell preset.
    pub fn gst_mushroom_cell() -> Self {
        Self {
            r_cryst: 4.0e3,                    // 4 kOhm (SET)
            r_amorph: 800.0e3,                 // 800 kOhm (RESET, 200x ratio)
            t_melt_k: 873.15,                  // 600 C (873 K)
            t_cryst_k: 600.0,                  // Peak crystallization ~ 600 K
            thermal_resistance_k_per_w: 2.5e6, // 2.5e6 K/W
            thermal_time_constant_s: 2.0e-9,   // 2 ns
            e_cryst_ev: 2.1,                   // 2.1 eV
            k0_cryst_prefactor: 1.0e14,        // 1e14 s^-1
            avrami_exponent: 2.5,
            v_threshold_ots: 1.1, // 1.1 V OTS threshold
            r_holding_ots: 1.5e3, // 1.5 kOhm in dynamic OTS ON-state
        }
    }

    /// Evaluates cell temperature $T$ resulting from instantaneous electrical dissipation:
    /// $$T_{cell} = T_{ambient} + R_{th} \cdot (I \cdot V)$$
    pub fn cell_temperature(&self, v_volts: f64, u_c: f64, ambient_k: f64) -> f64 {
        let (i_cell, _) = self.evaluate_current_and_conductance(u_c, v_volts);
        let p_diss = (i_cell * v_volts).abs();
        ambient_k + self.thermal_resistance_k_per_w * p_diss
    }

    /// Rate of change of crystalline fraction $\frac{du_c}{dt}$ governed by crystallization and melting:
    pub fn state_derivative(&self, u_c: f64, cell_temp_k: f64) -> f64 {
        let u = u_c.clamp(0.0, 1.0);

        if cell_temp_k >= self.t_melt_k {
            // Melting regime: rapid amorphization (liquefaction followed by quench)
            // Quench rate ~ 1 / tau_th ~ 5e8 s^-1
            let melt_rate = 5.0e8 * ((cell_temp_k - self.t_melt_k) / 50.0).min(5.0);
            -melt_rate * u
        } else if cell_temp_k >= 400.0 {
            // Crystallization window (400 K to 850 K)
            let ea_joules = self.e_cryst_ev * ELEMENTARY_CHARGE;
            let arrhenius = (-ea_joules / (BOLTZMANN_CONSTANT * cell_temp_k)).exp();
            let n = self.avrami_exponent;
            // Physical nucleation seed: provides non-zero nucleation rate when u ~ 0
            let u_eff = u.clamp(1e-4, 1.0 - 1e-4);
            let log_term = (-(1.0 - u_eff).ln()).powf(1.0 - 1.0 / n);
            let rate = self.k0_cryst_prefactor * arrhenius * (1.0 - u) * log_term;
            rate.clamp(0.0, 1e9)
        } else {
            // Room temperature retention: negligible spontaneous crystallization
            0.0
        }
    }

    /// Advances PCM crystalline fraction $u_c$ forward by time $\Delta t$ using RK4 integration:
    pub fn step_rk4(
        &self,
        state: MemristorState,
        v_volts: f64,
        ambient_k: f64,
        dt_s: f64,
    ) -> MemristorState {
        let u = state.internal_state_w.clamp(0.0, 1.0);
        let t_cell = self.cell_temperature(v_volts, u, ambient_k);

        let k1 = self.state_derivative(u, t_cell);
        let k2 = self.state_derivative((u + 0.5 * dt_s * k1).clamp(0.0, 1.0), t_cell);
        let k3 = self.state_derivative((u + 0.5 * dt_s * k2).clamp(0.0, 1.0), t_cell);
        let k4 = self.state_derivative((u + dt_s * k3).clamp(0.0, 1.0), t_cell);

        let next_u = (u + (dt_s / 6.0) * (k1 + 2.0 * k2 + 2.0 * k3 + k4)).clamp(0.0, 1.0);
        let (i_d, _) = self.evaluate_current_and_conductance(next_u, v_volts);
        let g_eff = if v_volts.abs() > 1e-6 {
            (i_d / v_volts).abs()
        } else {
            self.small_signal_conductance(next_u)
        };

        MemristorState {
            conductance_s: g_eff,
            internal_state_w: next_u,
        }
    }

    /// Evaluates small-signal low-field conductance from crystalline fraction $u_c$:
    /// $$\log_{10} R = (1 - u_c) \log_{10} R_{amorph} + u_c \log_{10} R_{cryst}$$
    pub fn small_signal_conductance(&self, u_c: f64) -> f64 {
        let u = u_c.clamp(0.0, 1.0);
        let log_r = (1.0 - u) * self.r_amorph.log10() + u * self.r_cryst.log10();
        1.0 / 10.0_f64.powf(log_r)
    }

    /// Evaluates device current $I(V, u_c)$ and differential conductance $g_d = \frac{dI}{dV}$ incorporating OTS in amorphous phase:
    pub fn evaluate_current_and_conductance(&self, u_c: f64, v_volts: f64) -> (f64, f64) {
        let v_abs = v_volts.abs();
        let u = u_c.clamp(0.0, 1.0);

        if v_abs < 1e-12 {
            let g = self.small_signal_conductance(u);
            return (0.0, g);
        }

        let calc_i = |v: f64| -> f64 {
            let va = v.abs();
            let delta_v = 0.05;
            let sigma = 1.0
                / (1.0
                    + (-(va - self.v_threshold_ots) / delta_v)
                        .clamp(-30.0, 30.0)
                        .exp());
            let log_r_high =
                (1.0 - sigma) * self.r_amorph.log10() + sigma * self.r_holding_ots.log10();
            let log_r = (1.0 - u) * log_r_high + u * self.r_cryst.log10();
            let r_eff = 10.0_f64.powf(log_r);
            v / r_eff
        };

        let i_raw = calc_i(v_volts);
        let dv = 1e-4;
        let i_plus = calc_i(v_volts + dv);
        let i_minus = calc_i(v_volts - dv);
        let g_diff = ((i_plus - i_minus) / (2.0 * dv)).max(1e-12);

        (i_raw, g_diff)
    }

    /// Generates MNA companion stamp: `(conductance_gd, rhs_current_ieq)`.
    pub fn companion_stamp(&self, u_c: f64, v_volts: f64) -> (f64, f64) {
        let (i_d, g_d) = self.evaluate_current_and_conductance(u_c, v_volts);
        let i_eq = g_d * v_volts - i_d;
        (g_d, i_eq)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phonon_core::T_REF;

    #[test]
    fn test_pcm_subthreshold_resistance_states() {
        let pcm = PhaseChangeMemoryModel::gst_mushroom_cell();

        // Fully amorphous state (u_c = 0.0)
        let g_amorph = pcm.small_signal_conductance(0.0);
        assert!((1.0 / g_amorph - pcm.r_amorph).abs() < 10.0);

        // Fully crystalline state (u_c = 1.0)
        let g_cryst = pcm.small_signal_conductance(1.0);
        assert!((1.0 / g_cryst - pcm.r_cryst).abs() < 10.0);

        // Dynamic resistance ratio should exceed 100x
        assert!(g_cryst / g_amorph > 100.0);
    }

    #[test]
    fn test_pcm_thermal_melting_and_amorphization() {
        let pcm = PhaseChangeMemoryModel::gst_mushroom_cell();
        let state_cryst = MemristorState::new(1.0 / pcm.r_cryst, 1.0);

        // At high voltage (2.0 V, well above OTS), high current heats cell above melting point
        let t_hot = pcm.cell_temperature(2.0, 1.0, T_REF);
        assert!(
            t_hot > pcm.t_melt_k,
            "Cell must melt above 873 K at 2.0 V, got {} K",
            t_hot
        );

        // Short reset pulse (3 ns) should amorphize cell (u_c drops towards 0)
        let dt = 1e-10;
        let mut state = state_cryst;
        for _ in 0..30 {
            state = pcm.step_rk4(state, 2.0, T_REF, dt);
        }
        assert!(
            state.internal_state_w < 0.2,
            "Crystal fraction must melt towards amorphous"
        );
    }

    #[test]
    fn test_ovonic_threshold_switching_snapback() {
        let pcm = PhaseChangeMemoryModel::gst_mushroom_cell();
        // At 0.5 V (< V_th), amorphous cell draws negligible current
        let (i_sub, _) = pcm.evaluate_current_and_conductance(0.0, 0.5);
        assert!(i_sub < 1e-6);

        // At 1.5 V (> V_th), OTS activates high current
        let (i_supra, _) = pcm.evaluate_current_and_conductance(0.0, 1.5);
        assert!(i_supra > 1e-4);
    }
}
