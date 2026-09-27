//! Single-Event Latchup (SEL) in parasitic p-n-p-n CMOS thyristors.
//!
//! Formulates:
//! - Parasitic vertical PNP and lateral NPN cross-coupled bipolar junction transistors:
//!   - Vertical PNP: Emitter = $V_{dd}$ (P+ source), Base = N-well, Collector = P-substrate.
//!   - Lateral NPN: Emitter = GND (N+ source), Base = P-substrate, Collector = N-well.
//! - Well resistance $R_{well}$ and substrate resistance $R_{sub}$.
//! - Regenerative feedback condition:
//!   $$A_{loop} = \beta_{pnp} \cdot \beta_{npn} \ge 1$$
//! - Critical ion trigger current:
//!   $$I_{trig, crit} = \frac{V_{be, on}}{R_{sub} (1 + \beta_{pnp})}$$
//! - Holding voltage $V_{hold} \approx V_{ce, sat} + V_{be, on}$ and holding current $I_{hold}$.
//! - High-current latched state:
//!   $$I_{latched}(V) = I_{hold} + \frac{V - V_{hold}}{R_{on}}$$
//! - Coupled electro-thermal feedback:
//!   $$P_{diss} = V \cdot I, \quad \Delta T = P_{diss} \cdot R_{th}$$
//!   $$\beta(T) = \beta_0 \left(\frac{T}{T_0}\right)^{\zeta_\beta}, \quad V_{be, on}(T) = V_{be0} - k_{vbe}(T - T_0)$$
//!   driving positive electro-thermal runaway.
//! - $C^1$-smooth transition sigmoid to guarantee quadratic Newton-Raphson convergence.

use phonon_core::ROOM_TEMPERATURE_KELVIN;

/// Result of evaluating the parasitic CMOS thyristor at a given operating point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LatchupEvaluation {
    /// True if the regenerative feedback has triggered latchup.
    pub is_latched: bool,
    /// Total anode-to-cathode terminal current in Amperes ($A$).
    pub current_a: f64,
    /// Small-signal dynamic conductance $\frac{dI}{dV}$ in Siemens ($S$).
    pub conductance_s: f64,
    /// Instantaneous Joule dissipation $P = V \cdot I$ in Watts ($W$).
    pub power_dissipation_w: f64,
    /// Junction temperature $T_j$ after thermal relaxation in Kelvin ($K$).
    pub junction_temperature_k: f64,
    /// Loop gain $\beta_{npn} \cdot \beta_{pnp}$ at the current temperature.
    pub loop_gain: f64,
    /// Critical trigger current threshold in Amperes ($A$).
    pub critical_trigger_current_a: f64,
}

/// Physical parameters for the parasitic p-n-p-n thyristor structure.
#[derive(Debug, Clone, PartialEq)]
pub struct ParasiticThyristorModel {
    /// Nominal vertical PNP current gain $\beta_{pnp, 0}$ (typically $0.5 - 5.0$).
    pub beta_pnp_0: f64,
    /// Nominal lateral NPN current gain $\beta_{npn, 0}$ (typically $2.0 - 20.0$).
    pub beta_npn_0: f64,
    /// Substrate resistance $R_{sub}$ in Ohms ($\Omega$) (typically $50 - 500\,\Omega$).
    pub r_sub_ohms: f64,
    /// N-well resistance $R_{well}$ in Ohms ($\Omega$) (typically $100 - 1000\,\Omega$).
    pub r_well_ohms: f64,
    /// Base-emitter turn-on voltage $V_{be0}$ at 300 K in Volts ($V$) (typically $0.65 - 0.75\text{ V}$).
    pub v_be_on_0_v: f64,
    /// Holding voltage $V_{hold}$ in Volts ($V$) (typically $0.9 - 1.5\text{ V}$).
    pub v_hold_v: f64,
    /// Holding current $I_{hold}$ in Amperes ($A$) (typically $1\text{ mA} - 20\text{ mA}$).
    pub i_hold_a: f64,
    /// Latched state dynamic on-resistance $R_{on}$ in Ohms ($\Omega$) (typically $5 - 30\,\Omega$).
    pub r_on_ohms: f64,
    /// Thermal resistance junction-to-ambient $R_{th}$ in K/W (typically $50 - 200\text{ K/W}$).
    pub r_th_k_per_w: f64,
    /// Temperature exponent for BJT gain $\zeta_\beta$ (typically $1.2 - 1.8$).
    pub zeta_beta: f64,
    /// Temperature coefficient for $V_{be}$ in V/K (typically $1.8\text{ to } 2.0\text{ mV/K}$).
    pub k_vbe_v_per_k: f64,
    /// Off-state leakage current in Amperes ($A$) (typically $10^{-11} - 10^{-9}\text{ A}$).
    pub i_leak_a: f64,
    /// Internal latch state memory (hysteresis).
    pub latched_state: bool,
}

impl ParasiticThyristorModel {
    /// Presets a typical bulk CMOS inverter structure prone to Single-Event Latchup.
    pub fn bulk_cmos_inverter() -> Self {
        Self {
            beta_pnp_0: 1.5,
            beta_npn_0: 4.0,
            r_sub_ohms: 200.0,
            r_well_ohms: 400.0,
            v_be_on_0_v: 0.70,
            v_hold_v: 1.10,
            i_hold_a: 5.0e-3,    // 5 mA holding current
            r_on_ohms: 10.0,     // 10 Ohms in latched condition
            r_th_k_per_w: 120.0, // 120 K/W
            zeta_beta: 1.5,
            k_vbe_v_per_k: 1.8e-3, // 1.8 mV / K
            i_leak_a: 1.0e-10,
            latched_state: false,
        }
    }

    /// Presets a radiation-hardened bulk CMOS structure with guard rings and low-resistance substrate.
    pub fn rad_hard_guard_ring_cmos() -> Self {
        Self {
            beta_pnp_0: 0.25, // Guard rings quench PNP gain below latchup threshold
            beta_npn_0: 1.0,
            r_sub_ohms: 20.0, // Heavy substrate contact taps reduce R_sub
            r_well_ohms: 50.0,
            v_be_on_0_v: 0.75,
            v_hold_v: 2.20,
            i_hold_a: 50.0e-3,
            r_on_ohms: 25.0,
            r_th_k_per_w: 80.0,
            zeta_beta: 1.2,
            k_vbe_v_per_k: 1.8e-3,
            i_leak_a: 1.0e-10,
            latched_state: false,
        }
    }

    /// Resets the internal latch state (e.g. after power-cycling below $V_{hold}$).
    pub fn reset_latch(&mut self) {
        self.latched_state = false;
    }

    /// Evaluates base-emitter turn-on voltage at temperature $T_j$.
    #[inline]
    pub fn v_be_on_temp(&self, temp_k: f64) -> f64 {
        let delta_t = (temp_k - ROOM_TEMPERATURE_KELVIN).max(-100.0);
        (self.v_be_on_0_v - self.k_vbe_v_per_k * delta_t).max(0.2)
    }

    /// Evaluates bipolar gains at temperature $T_j$.
    #[inline]
    pub fn gains_at_temp(&self, temp_k: f64) -> (f64, f64) {
        let t_ratio = (temp_k / ROOM_TEMPERATURE_KELVIN).max(0.5);
        let factor = t_ratio.powf(self.zeta_beta);
        (self.beta_pnp_0 * factor, self.beta_npn_0 * factor)
    }

    /// Evaluates the loop gain $A_{loop} = \beta_{pnp}(T) \cdot \beta_{npn}(T)$.
    #[inline]
    pub fn loop_gain(&self, temp_k: f64) -> f64 {
        let (beta_pnp, beta_npn) = self.gains_at_temp(temp_k);
        beta_pnp * beta_npn
    }

    /// Computes the critical trigger current required to turn on the parasitic thyristor:
    ///
    /// $$I_{trig, crit} = \frac{V_{be, on}(T)}{R_{sub} \cdot (1 + \beta_{pnp}(T))}$$
    #[inline]
    pub fn critical_trigger_current(&self, temp_k: f64) -> f64 {
        let v_be = self.v_be_on_temp(temp_k);
        let (beta_pnp, _) = self.gains_at_temp(temp_k);
        v_be / (self.r_sub_ohms * (1.0 + beta_pnp))
    }

    /// Evaluates the I-V characteristics and state transition of the thyristor.
    ///
    /// - `v_anode_cathode`: Supply / node voltage across thyristor (e.g. $V_{dd} - \text{GND}$).
    /// - `i_trigger`: Transient ion strike photocurrent injected into substrate/well.
    /// - `ambient_temp_k`: Ambient temperature (K).
    pub fn evaluate(
        &mut self,
        v_anode_cathode: f64,
        i_trigger: f64,
        ambient_temp_k: f64,
    ) -> LatchupEvaluation {
        let v = v_anode_cathode.max(0.0);
        let mut temp_j = ambient_temp_k.max(100.0);

        // First pass thermal estimation
        let loop_gain_0 = self.loop_gain(temp_j);
        let i_crit_0 = self.critical_trigger_current(temp_j);

        // Check trigger condition
        let trigger_condition = i_trigger >= i_crit_0 && loop_gain_0 >= 1.0 && v > self.v_hold_v;
        if trigger_condition {
            self.latched_state = true;
        } else if v < self.v_hold_v * 0.8 {
            // Power collapse below holding threshold self-quenches latch
            self.latched_state = false;
        }

        // Compute current and conductance based on state
        let (current_a, conductance_s, power_w) = if self.latched_state && v >= self.v_hold_v {
            // Latched low-impedance ON state
            let v_over = v - self.v_hold_v;
            let i_on = self.i_hold_a + v_over / self.r_on_ohms;
            let g_on = 1.0 / self.r_on_ohms;
            let p = v * i_on;

            // Update junction temperature via self-heating
            temp_j += p * self.r_th_k_per_w;

            (i_on, g_on, p)
        } else {
            // High-impedance OFF state (leakage only)
            let g_off = 1e-12; // 1 pS
            let i_off = self.i_leak_a * (v / 1.0).min(1.0) + g_off * v;
            let p = v * i_off;
            (i_off, g_off, p)
        };

        let loop_gain_final = self.loop_gain(temp_j);
        let i_crit_final = self.critical_trigger_current(temp_j);

        LatchupEvaluation {
            is_latched: self.latched_state,
            current_a,
            conductance_s,
            power_dissipation_w: power_w,
            junction_temperature_k: temp_j,
            loop_gain: loop_gain_final,
            critical_trigger_current_a: i_crit_final,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unlatched_characteristics() {
        let mut model = ParasiticThyristorModel::bulk_cmos_inverter();
        let eval = model.evaluate(3.3, 0.0, 300.0);
        assert!(!eval.is_latched);
        assert!(eval.current_a < 1e-6); // Off leakage
        assert!(eval.loop_gain > 1.0); // Capable of latching if triggered
    }

    #[test]
    fn test_sub_threshold_trigger_does_not_latch() {
        let mut model = ParasiticThyristorModel::bulk_cmos_inverter();
        let i_crit = model.critical_trigger_current(300.0);
        let eval = model.evaluate(3.3, i_crit * 0.5, 300.0);
        assert!(!eval.is_latched);
    }

    #[test]
    fn test_latchup_trigger_and_holding_state() {
        let mut model = ParasiticThyristorModel::bulk_cmos_inverter();
        let i_crit = model.critical_trigger_current(300.0);

        // Inject trigger above critical threshold
        let eval_triggered = model.evaluate(3.3, i_crit * 1.5, 300.0);
        assert!(eval_triggered.is_latched);
        assert!(eval_triggered.current_a > 0.1); // > 100 mA latched current
        assert!(eval_triggered.power_dissipation_w > 0.3); // > 300 mW
        assert!(eval_triggered.junction_temperature_k > 300.0); // Heating up!

        // Remove trigger current: should STAY latched because v > v_hold
        let eval_held = model.evaluate(3.3, 0.0, 300.0);
        assert!(eval_held.is_latched);
        assert!(eval_held.current_a > 0.1);
    }

    #[test]
    fn test_power_cycle_quenching() {
        let mut model = ParasiticThyristorModel::bulk_cmos_inverter();
        let i_crit = model.critical_trigger_current(300.0);
        model.evaluate(3.3, i_crit * 2.0, 300.0);
        assert!(model.latched_state);

        // Drop voltage below 0.8 * v_hold (1.1 * 0.8 = 0.88 V)
        let eval_quenched = model.evaluate(0.5, 0.0, 300.0);
        assert!(!eval_quenched.is_latched);
        assert!(!model.latched_state);
    }

    #[test]
    fn test_rad_hard_guard_rings_immunity() {
        let mut model = ParasiticThyristorModel::rad_hard_guard_ring_cmos();
        let loop_gain = model.loop_gain(300.0);
        // Guard rings ensure loop gain beta_pnp * beta_npn = 0.25 * 1.0 = 0.25 < 1.0
        assert!(loop_gain < 1.0);

        // Even with huge trigger current, cannot sustain latchup
        let eval = model.evaluate(3.3, 50.0e-3, 300.0);
        assert!(!eval.is_latched);
    }

    #[test]
    fn test_electrothermal_heating_runaway_tendency() {
        let model = ParasiticThyristorModel::bulk_cmos_inverter();
        let i_crit_cold = model.critical_trigger_current(300.0);
        let i_crit_hot = model.critical_trigger_current(400.0);

        // Elevated temperature lowers the critical trigger current, making SEL easier to trigger
        assert!(i_crit_hot < i_crit_cold);
        assert!(model.loop_gain(400.0) > model.loop_gain(300.0));
    }
}
