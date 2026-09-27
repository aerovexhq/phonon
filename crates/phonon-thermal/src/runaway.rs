//! Thermal runaway stability analysis and early divergence detection.

use phonon_models::diode::DiodeModel;

/// Quantitative assessment of electro-thermal stability and thermal runaway susceptibility.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalStabilityAssessment {
    /// True if the device operating point is stable ($A_{th} < 1.0$).
    pub is_stable: bool,
    /// Electro-thermal loop gain $A_{th} = R_{th} \cdot \frac{\partial P_{diss}}{\partial T}$.
    /// A value $\ge 1.0$ indicates positive feedback leading to thermal runaway.
    pub thermal_loop_gain: f64,
    /// Rate of power increase with temperature $\frac{\partial P_{diss}}{\partial T}$ in $\text{W} / \text{K}$.
    pub dp_dt: f64,
    /// Effective thermal dissipation conductance $G_{th} = 1 / R_{th}$ in $\text{W} / \text{K}$.
    pub g_th: f64,
}

/// Evaluates the thermal stability of a semiconductor diode at operating voltage $V_D$,
/// thermal resistance to ambient $R_{th}$, and junction temperature $T$.
pub fn assess_diode_thermal_stability(
    vd: f64,
    r_th: f64,
    model: &DiodeModel,
    temp_k: f64,
) -> ThermalStabilityAssessment {
    assert!(r_th > 0.0, "Thermal resistance must be strictly positive");
    let g_th = 1.0 / r_th;

    // Evaluate power at current temperature and slightly perturbed temperature
    let delta_t = 0.5; // 0.5 K perturbation
    let eval_0 = model.evaluate(vd, temp_k);
    let eval_1 = model.evaluate(vd, temp_k + delta_t);

    let p0 = vd * eval_0.i_d;
    let p1 = vd * eval_1.i_d;

    let dp_dt = (p1 - p0) / delta_t;
    let thermal_loop_gain = (dp_dt * r_th).max(0.0);
    let is_stable = thermal_loop_gain < 1.0;

    ThermalStabilityAssessment {
        is_stable,
        thermal_loop_gain,
        dp_dt,
        g_th,
    }
}
