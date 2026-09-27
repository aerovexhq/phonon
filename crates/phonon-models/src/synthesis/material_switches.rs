//! Direct Material-Level Switching Primitives:
//! Negative Differential Resistance (NDR / RTD) and Metal-Insulator Transition (MIT $\text{VO}_2$).
//!
//! Provides physically rigorous compact representations for two-terminal direct material actions
//! enabling ultra-compact non-transistor logic, monostable-bistable transition logic (MOBILE),
//! and threshold-switched logic gates.

/// Parameters for a Negative Differential Resistance (NDR) device / Resonant Tunneling Diode (RTD).
#[derive(Debug, Clone, PartialEq)]
pub struct NdrParameters {
    /// Peak resonance current $I_p$ in Amperes.
    pub peak_current_a: f64,
    /// Peak resonance voltage $V_p$ in Volts.
    pub peak_voltage_v: f64,
    /// Valley minimum current $I_v$ in Amperes ($I_v < I_p$).
    pub valley_current_a: f64,
    /// Valley voltage $V_v$ in Volts ($V_v > V_p$).
    pub valley_voltage_v: f64,
    /// Thermal voltage scale parameter $V_0$ for excess tunneling in Volts.
    pub excess_voltage_scale_v: f64,
}

impl Default for NdrParameters {
    fn default() -> Self {
        Self {
            peak_current_a: 1.0e-3,       // 1.0 mA peak current
            peak_voltage_v: 0.25,         // 0.25 V peak voltage
            valley_current_a: 1.0e-4,     // 0.1 mA valley current (PVCR = 10)
            valley_voltage_v: 0.50,       // 0.50 V valley voltage
            excess_voltage_scale_v: 0.15, // 0.15 V excess scale
        }
    }
}

/// Evaluation output of an NDR / RTD element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NdrEvaluation {
    /// Total device current $I$ in Amperes.
    pub current_a: f64,
    /// Small-signal dynamic conductance $g = \frac{dI}{dV}$ in Siemens.
    pub conductance_s: f64,
    /// Peak-to-Valley Current Ratio (PVCR).
    pub pvcr: f64,
    /// Returns true if operating in the negative differential conductance region ($g < 0$).
    pub in_ndr_region: bool,
}

/// Negative Differential Resistance (NDR / RTD) device model.
#[derive(Debug, Clone, PartialEq)]
pub struct NdrDeviceModel {
    pub params: NdrParameters,
}

impl NdrDeviceModel {
    pub fn new(params: NdrParameters) -> Self {
        Self { params }
    }

    /// Evaluates current and small-signal conductance using the Schulman-Esaki analytical formulation.
    ///
    /// The I-V curve decomposes into three physical transport components:
    /// 1. Resonant tunneling peak current: $I_{res}(V) = I_p \left(\frac{V}{V_p}\right) \exp\left(1 - \frac{V}{V_p}\right)$
    /// 2. Valley excess current: $I_{valley}(V) = I_v \exp\left(\frac{V - V_v}{V_0}\right)$
    /// 3. Thermal diffusion current for $V > V_v$.
    pub fn evaluate(&self, voltage_v: f64) -> NdrEvaluation {
        let p = &self.params;
        let v = voltage_v;
        let sign = if v >= 0.0 { 1.0 } else { -1.0 };
        let abs_v = v.abs();

        let v_norm = abs_v / p.peak_voltage_v.max(1e-3);
        let exp_peak = (1.0 - v_norm).exp();
        let i_res = p.peak_current_a * v_norm * exp_peak;
        let di_res_dv = (p.peak_current_a / p.peak_voltage_v.max(1e-3)) * (1.0 - v_norm) * exp_peak;

        let delta_v = abs_v - p.valley_voltage_v;
        let exp_valley = (delta_v / p.excess_voltage_scale_v.max(1e-3)).exp();
        let i_valley = p.valley_current_a * exp_valley;
        let di_valley_dv = (p.valley_current_a / p.excess_voltage_scale_v.max(1e-3)) * exp_valley;

        // Blended current
        let (current_mag, cond_mag) = if abs_v <= p.peak_voltage_v {
            (i_res, di_res_dv)
        } else if abs_v <= p.valley_voltage_v {
            // Negative differential resistance transition region
            let w = (abs_v - p.peak_voltage_v) / (p.valley_voltage_v - p.peak_voltage_v).max(1e-4);
            let s_curve = 0.5 * (1.0 + (std::f64::consts::PI * (w - 0.5)).sin());
            let i_trans = (1.0 - s_curve) * p.peak_current_a + s_curve * p.valley_current_a;
            let ds_curve = 0.5
                * (std::f64::consts::PI / (p.valley_voltage_v - p.peak_voltage_v).max(1e-4))
                * (std::f64::consts::PI * (w - 0.5)).cos();
            let di_trans = ds_curve * (p.valley_current_a - p.peak_current_a);
            (i_trans, di_trans)
        } else {
            // Valley and forward conduction
            (
                p.valley_current_a + i_valley
                    - p.valley_current_a * (-delta_v / p.excess_voltage_scale_v).exp(),
                di_valley_dv,
            )
        };

        let current_a = sign * current_mag;
        let conductance_s = cond_mag;
        let pvcr = p.peak_current_a / p.valley_current_a.max(1e-9);
        let in_ndr_region = conductance_s < 0.0;

        NdrEvaluation {
            current_a,
            conductance_s,
            pvcr,
            in_ndr_region,
        }
    }
}

/// Parameters for a Metal-Insulator Transition (MIT) $\text{VO}_2$ material switch.
#[derive(Debug, Clone, PartialEq)]
pub struct MitParameters {
    /// Insulating off-state resistance $R_{off}$ in Ohms.
    pub r_off_ohms: f64,
    /// Metallic on-state resistance $R_{on}$ in Ohms ($R_{on} \ll R_{off}$).
    pub r_on_ohms: f64,
    /// Critical threshold voltage $V_{MIT}$ for triggering metallic phase in Volts.
    pub threshold_voltage_v: f64,
    /// Transition smoothing width $\Delta V$ in Volts.
    pub transition_width_v: f64,
    /// Critical transition temperature in Kelvin (typically ~341 K = 68 °C for VO2).
    pub critical_temperature_k: f64,
    /// Thermal transition smoothing width in Kelvin (typically ~1.5 K for first-order VO2 phase change).
    pub thermal_transition_width_k: f64,
}

impl Default for MitParameters {
    fn default() -> Self {
        Self {
            r_off_ohms: 100_000.0,     // 100 kOhms insulating state
            r_on_ohms: 100.0,          // 100 Ohms metallic state (3 orders of magnitude switch)
            threshold_voltage_v: 0.80, // 0.8 V critical threshold
            transition_width_v: 0.05,  // 50 mV transition sharpness
            critical_temperature_k: 341.0,
            thermal_transition_width_k: 1.5,
        }
    }
}

/// Evaluation output of an MIT $\text{VO}_2$ element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MitEvaluation {
    /// Current through the device in Amperes.
    pub current_a: f64,
    /// Effective resistance in Ohms.
    pub resistance_ohms: f64,
    /// Small-signal conductance $g = \frac{1}{R}$ in Siemens.
    pub conductance_s: f64,
    /// Order of magnitude on/off switching ratio.
    pub switching_ratio: f64,
    /// Returns true if the device is in the low-resistance metallic state.
    pub is_metallic: bool,
}

/// Metal-Insulator Transition (MIT $\text{VO}_2$) device model.
#[derive(Debug, Clone, PartialEq)]
pub struct MitDeviceModel {
    pub params: MitParameters,
}

impl MitDeviceModel {
    pub fn new(params: MitParameters) -> Self {
        Self { params }
    }

    /// Evaluates dynamic resistance and conduction under applied voltage and temperature.
    pub fn evaluate(&self, voltage_v: f64, temperature_k: f64) -> MitEvaluation {
        let p = &self.params;
        let abs_v = voltage_v.abs();

        // Phase order parameter eta in [0, 1] where 0 = fully insulating, 1 = fully metallic
        // Driven by electric field / voltage threshold and thermal heating
        let v_drive = (abs_v - p.threshold_voltage_v) / p.transition_width_v.max(1e-3);
        let t_drive =
            (temperature_k - p.critical_temperature_k) / p.thermal_transition_width_k.max(0.1);
        let net_drive = v_drive.max(t_drive);

        let eta = 1.0 / (1.0 + (-net_drive).exp());

        // Effective resistance logarithmic interpolation
        let log_r_off = p.r_off_ohms.ln();
        let log_r_on = p.r_on_ohms.ln();
        let log_r_eff = log_r_off + eta * (log_r_on - log_r_off);
        let resistance_ohms = log_r_eff.exp().clamp(p.r_on_ohms * 0.9, p.r_off_ohms * 1.1);

        let conductance_s = 1.0 / resistance_ohms.max(1e-3);
        let current_a = voltage_v * conductance_s;
        let switching_ratio = p.r_off_ohms / p.r_on_ohms.max(1.0);
        let is_metallic = eta > 0.5;

        MitEvaluation {
            current_a,
            resistance_ohms,
            conductance_s,
            switching_ratio,
            is_metallic,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ndr_iv_curve_and_negative_conductance() {
        let model = NdrDeviceModel::new(NdrParameters::default());

        // Peak point at 0.25 V
        let eval_peak = model.evaluate(0.25);
        assert!((eval_peak.current_a - 1.0e-3).abs() < 1.0e-4);

        // NDR transition region at 0.35 V
        let eval_ndr = model.evaluate(0.35);
        assert!(
            eval_ndr.in_ndr_region,
            "Conductance must be negative between peak and valley"
        );
        assert!(eval_ndr.conductance_s < 0.0);

        // Valley point at 0.50 V
        let eval_valley = model.evaluate(0.50);
        assert!(eval_valley.current_a < eval_peak.current_a);
        assert_eq!(eval_valley.pvcr, 10.0);
    }

    #[test]
    fn test_mit_vo2_switching_action() {
        let model = MitDeviceModel::new(MitParameters::default());

        // Sub-threshold voltage (0.2 V): should be insulating
        let eval_low = model.evaluate(0.20, 300.0);
        assert!(!eval_low.is_metallic);
        assert!(eval_low.resistance_ohms > 50_000.0);

        // Above threshold voltage (1.0 V): should trigger metallic state
        let eval_high = model.evaluate(1.00, 300.0);
        assert!(eval_high.is_metallic);
        assert!(eval_high.resistance_ohms < 500.0);
        assert!(eval_high.switching_ratio >= 1000.0);

        // Thermal trigger above 341 K even at 0 V
        let eval_thermal = model.evaluate(0.10, 350.0);
        assert!(eval_thermal.is_metallic);
        assert!(eval_thermal.resistance_ohms < 500.0);
    }
}
