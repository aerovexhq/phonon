//! Spike-Timing-Dependent Plasticity (STDP) learning rules and synaptic weight adaptation.
//!
//! Formulates:
//! - Biologically realistic asymmetric Hebbian plasticity:
//!   $$\Delta w = \begin{cases} A_+ \exp\left(-\frac{\Delta t}{\tau_+}\right) & \text{if } \Delta t = t_{post} - t_{pre} > 0 \quad (\text{LTP}) \\ -A_- \exp\left(\frac{\Delta t}{\tau_-}\right) & \text{if } \Delta t < 0 \quad (\text{LTD}) \end{cases}$$
//! - Hard and soft conductance saturation boundaries ($w_{min}, w_{max}$).
//! - Non-linear pulse-number-dependent potentiation and depression.
//! - Continuous analog synaptic conductance tuning for neuromorphic memristive arrays.

/// Physical parameters governing Spike-Timing-Dependent Plasticity (STDP).
#[derive(Debug, Clone, PartialEq)]
pub struct SpikeTimingPlasticityModel {
    /// Maximum potentiation amplitude $A_+$ (LTP).
    pub a_plus: f64,
    /// Maximum depression amplitude $A_-$ (LTD).
    pub a_minus: f64,
    /// Potentiation decay time constant $\tau_+$ in seconds ($s$) (typically $10 - 20\text{ ms}$).
    pub tau_plus_s: f64,
    /// Depression decay time constant $\tau_-$ in seconds ($s$) (typically $20 - 40\text{ ms}$).
    pub tau_minus_s: f64,
    /// Minimum allowed synaptic conductance $w_{min}$ in Siemens ($S$).
    pub w_min_siemens: f64,
    /// Maximum allowed synaptic conductance $w_{max}$ in Siemens ($S$).
    pub w_max_siemens: f64,
    /// Non-linearity exponent $\gamma_p$ for potentiation boundary saturation.
    pub gamma_p: f64,
    /// Non-linearity exponent $\gamma_d$ for depression boundary saturation.
    pub gamma_d: f64,
}

impl SpikeTimingPlasticityModel {
    /// Standard biological cortical STDP preset ($\tau_+ = 20\text{ ms}, \tau_- = 20\text{ ms}$).
    pub fn cortical_stdp(g_min: f64, g_max: f64) -> Self {
        Self {
            a_plus: 0.10 * (g_max - g_min),  // 10% step at delta t = 0
            a_minus: 0.12 * (g_max - g_min), // 12% step (depression slightly dominates for stability)
            tau_plus_s: 20.0e-3,             // 20 ms
            tau_minus_s: 20.0e-3,            // 20 ms
            w_min_siemens: g_min.max(1e-12),
            w_max_siemens: g_max.max(g_min * 2.0),
            gamma_p: 1.0,
            gamma_d: 1.0,
        }
    }

    /// Computes the raw infinitesimal conductance update $\Delta w$ from relative spike timing
    /// $\Delta t = t_{post} - t_{pre}$:
    pub fn delta_weight_raw(&self, delta_t_s: f64) -> f64 {
        if delta_t_s > 0.0 {
            // Pre-before-post: Long-Term Potentiation (LTP)
            self.a_plus * (-delta_t_s / self.tau_plus_s.max(1e-6)).exp()
        } else if delta_t_s < 0.0 {
            // Post-before-pre: Long-Term Depression (LTD)
            -self.a_minus * (delta_t_s / self.tau_minus_s.max(1e-6)).exp()
        } else {
            0.0
        }
    }

    /// Evaluates the bounded synaptic weight update taking into account soft saturation boundaries:
    pub fn update_weight(&self, current_weight: f64, delta_t_s: f64) -> f64 {
        let w = current_weight.clamp(self.w_min_siemens, self.w_max_siemens);
        let dw_raw = self.delta_weight_raw(delta_t_s);

        let delta_w = if dw_raw > 0.0 {
            // Soft boundary near w_max:
            let headroom =
                (self.w_max_siemens - w) / (self.w_max_siemens - self.w_min_siemens).max(1e-12);
            dw_raw * headroom.powf(self.gamma_p)
        } else if dw_raw < 0.0 {
            // Soft boundary near w_min:
            let floor =
                (w - self.w_min_siemens) / (self.w_max_siemens - self.w_min_siemens).max(1e-12);
            dw_raw * floor.powf(self.gamma_d)
        } else {
            0.0
        };

        (w + delta_w).clamp(self.w_min_siemens, self.w_max_siemens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stdp_asymmetric_ltp_and_ltd() {
        let stdp = SpikeTimingPlasticityModel::cortical_stdp(1e-6, 1e-4);

        // Pre fires at t=0, post fires at t=+10 ms -> LTP (Delta t = +10 ms > 0)
        let dw_ltp = stdp.delta_weight_raw(10.0e-3);
        assert!(dw_ltp > 0.0, "Pre-before-post must yield potentiation");

        // Post fires at t=0, pre fires at t=+10 ms -> LTD (Delta t = -10 ms < 0)
        let dw_ltd = stdp.delta_weight_raw(-10.0e-3);
        assert!(dw_ltd < 0.0, "Post-before-pre must yield depression");

        // Exponent decay verification: at dt = 20 ms (tau), value should be 1/e of max
        let dw_tau = stdp.delta_weight_raw(20.0e-3);
        assert!((dw_tau - stdp.a_plus / std::f64::consts::E).abs() < 1e-7);
    }

    #[test]
    fn test_stdp_bounded_weight_saturation() {
        let stdp = SpikeTimingPlasticityModel::cortical_stdp(1e-6, 10.0e-6);

        // Repeated potentiation pulses: weight should asymptotically approach w_max without exceeding it
        let mut w = 2.0e-6;
        for _ in 0..100 {
            w = stdp.update_weight(w, 5.0e-3);
        }

        assert!(w <= stdp.w_max_siemens);
        assert!(w > 0.95 * stdp.w_max_siemens);
    }
}
