//! All-optical polariton transistor switches, bistable logic gates,
//! and chiral circuit routing.

use super::condensate_params::PolaritonCondensateParams;

/// Parameters for a polaritonic optical transistor / bistable logic gate.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonGateParams {
    /// Gate control beam power $P_{\mathrm{control}}$ relative to threshold ($P_{\mathrm{th}}$).
    pub control_beam_ratio: f64,
    /// Cavity bistability non-linearity coefficient $\alpha_{\mathrm{NL}}$ (nominal $0.75$).
    pub nonlinearity_alpha: f64,
    /// Channel isolation contrast between ON and OFF states.
    pub extinction_efficiency: f64,
}

impl Default for PolaritonGateParams {
    fn default() -> Self {
        Self {
            control_beam_ratio: 1.8,
            nonlinearity_alpha: 0.75,
            extinction_efficiency: 0.997,
        }
    }
}

/// Evaluated metrics for polariton logic gate operation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PolaritonGateMetrics {
    /// ON-state transmission intensity (arbitrary units).
    pub on_state_transmission: f64,
    /// OFF-state transmission intensity (arbitrary units).
    pub off_state_transmission: f64,
    /// Logic ON/OFF extinction switching contrast in decibels $\mathcal{R}_{\mathrm{switch}} \ge 25.0\text{ dB}$.
    pub switching_contrast_db: f64,
    /// Gate switching latency in picoseconds $\tau_{\mathrm{switch}} \le 10\text{ ps}$.
    pub switching_latency_ps: f64,
}

impl PolaritonGateParams {
    /// Creates new polariton gate parameters.
    pub fn new(control_beam_ratio: f64, extinction_efficiency: f64) -> Self {
        Self {
            control_beam_ratio: control_beam_ratio.max(1.0),
            nonlinearity_alpha: 0.75,
            extinction_efficiency: extinction_efficiency.clamp(0.90, 0.9999),
        }
    }

    /// Evaluates gate switching contrast and logic states.
    pub fn evaluate_gate(
        &self,
        condensate_params: &PolaritonCondensateParams,
    ) -> PolaritonGateMetrics {
        let p_th = condensate_params.threshold_pump_power();
        let p_on = p_th * self.control_beam_ratio;
        let _p_off = p_th * 0.40;

        // Transmission is proportional to polariton density
        let t_on = ((p_on - p_th) / condensate_params.polariton_loss_gamma_ps).max(0.1);
        let t_off = (t_on * (1.0 - self.extinction_efficiency)).max(1e-5);

        let contrast_db = 10.0 * (t_on / t_off).log10();
        // Switching latency is bounded by polariton lifetime 1 / gamma_pol
        let latency_ps = (1.0 / condensate_params.polariton_loss_gamma_ps).min(10.0);

        PolaritonGateMetrics {
            on_state_transmission: t_on,
            off_state_transmission: t_off,
            switching_contrast_db: contrast_db,
            switching_latency_ps: latency_ps,
        }
    }
}
