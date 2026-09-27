//! Single Flux Quantum (SFQ) logic, soliton pulses, JTL stages, and DC-SQUIDs.
//!
//! Enforces exact quantum flux conservation:
//! $$\int_{-\infty}^{\infty} V(t) dt = \Phi_0 = \frac{h}{2e} \approx 2.067833848 \times 10^{-15} \text{ V}\cdot\text{s}$$

use super::rcsj::JosephsonRcsjModel;
use phonon_core::FLUX_QUANTUM;

/// Single Flux Quantum (SFQ) analytical soliton pulse generator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SfqPulse {
    /// Center arrival time $t_0$ in seconds ($s$).
    pub t0: f64,
    /// Characteristic temporal width $\tau$ in seconds ($s$) ($\approx \Phi_0 / (2\pi I_c R_n)$).
    pub tau: f64,
}

impl SfqPulse {
    /// Creates a new SFQ pulse given center time $t_0$ and characteristic duration $\tau$.
    pub fn new(t0: f64, tau: f64) -> Self {
        Self {
            t0,
            tau: tau.max(1e-16),
        }
    }

    /// Creates an SFQ pulse based on the characteristic parameters of an RCSJ junction:
    /// $\tau = \frac{\Phi_0}{2\pi I_c R_n}$.
    pub fn from_junction(jj: &JosephsonRcsjModel, t0: f64) -> Self {
        let vc = jj.characteristic_voltage().max(1e-6);
        let tau = FLUX_QUANTUM / (2.0 * std::f64::consts::PI * vc);
        Self::new(t0, tau)
    }

    /// Evaluates the instantaneous soliton voltage $V_{sfq}(t)$ in Volts ($V$):
    /// $$V(t) = \frac{\Phi_0}{2 \tau} \operatorname{sech}^2\left( \frac{t - t_0}{\tau} \right)$$
    ///
    /// Analytical integral over all time is strictly $\int_{-\infty}^{\infty} V(t) dt = \Phi_0$.
    #[inline]
    pub fn voltage(&self, t: f64) -> f64 {
        let u = ((t - self.t0) / self.tau).clamp(-30.0, 30.0);
        let cosh_u = u.cosh();
        let sech2 = 1.0 / (cosh_u * cosh_u);
        (FLUX_QUANTUM / (2.0 * self.tau)) * sech2
    }

    /// Peak voltage amplitude $V_{peak} = \frac{\Phi_0}{2 \tau}$ in Volts ($V$).
    #[inline]
    pub fn peak_voltage(&self) -> f64 {
        FLUX_QUANTUM / (2.0 * self.tau)
    }
}

/// Numerical trapezoidal integration of an arbitrary discrete voltage profile $\int V(t) dt$.
pub fn integrate_voltage_time(times: &[f64], voltages: &[f64]) -> f64 {
    if times.len() < 2 || times.len() != voltages.len() {
        return 0.0;
    }
    let mut area = 0.0;
    for i in 0..(times.len() - 1) {
        let dt = times[i + 1] - times[i];
        let avg_v = 0.5 * (voltages[i] + voltages[i + 1]);
        area += avg_v * dt;
    }
    area
}

/// Verifies whether the integrated pulse area satisfies flux quantization $\Phi_0$ within relative tolerance.
pub fn verify_flux_quantization(
    times: &[f64],
    voltages: &[f64],
    rel_tol: f64,
) -> Result<f64, String> {
    let flux = integrate_voltage_time(times, voltages);
    let rel_err = (flux - FLUX_QUANTUM).abs() / FLUX_QUANTUM;
    if rel_err <= rel_tol {
        Ok(flux)
    } else {
        Err(format!(
            "Flux quantization violated: integrated flux = {:.6e} Wb, expected Phi_0 = {:.6e} Wb (error = {:.3}%)",
            flux,
            FLUX_QUANTUM,
            rel_err * 100.0
        ))
    }
}

/// Direct-Current Superconducting Quantum Interference Device (DC-SQUID).
#[derive(Debug, Clone, PartialEq)]
pub struct DcSquidModel {
    /// Junction 1 model.
    pub jj1: JosephsonRcsjModel,
    /// Junction 2 model.
    pub jj2: JosephsonRcsjModel,
    /// Loop inductance $L_{loop}$ in Henries ($H$).
    pub loop_inductance: f64,
}

impl DcSquidModel {
    /// Creates a symmetric DC-SQUID with identical junctions.
    pub fn symmetric(ic0: f64, rn: f64, cap: f64, l_loop: f64) -> Self {
        Self {
            jj1: JosephsonRcsjModel::new(ic0, rn, cap, 0.0),
            jj2: JosephsonRcsjModel::new(ic0, rn, cap, 0.0),
            loop_inductance: l_loop.max(1e-15),
        }
    }

    /// Effective critical current $I_{c,squid}(\Phi_{ext})$ modulated by external magnetic flux:
    /// $$I_{c,squid}(\Phi_{ext}) = \sqrt{I_{c1}^2 + I_{c2}^2 + 2 I_{c1} I_{c2} \cos\left( \frac{2\pi \Phi_{ext}}{\Phi_0} \right)}$$
    pub fn critical_current(&self, phi_ext: f64) -> f64 {
        let ic1 = self.jj1.ic;
        let ic2 = self.jj2.ic;
        let phase_shift = (2.0 * std::f64::consts::PI * phi_ext) / FLUX_QUANTUM;
        let sum_sq = ic1 * ic1 + ic2 * ic2;
        let cross = 2.0 * ic1 * ic2 * phase_shift.cos();
        (sum_sq + cross).max(0.0).sqrt()
    }
}

/// Elementary 2-junction Josephson Transmission Line (JTL) stage.
#[derive(Debug, Clone, PartialEq)]
pub struct JtlStage {
    /// Upstream junction J1.
    pub jj1: JosephsonRcsjModel,
    /// Downstream junction J2.
    pub jj2: JosephsonRcsjModel,
    /// Coupling inductance $L$ between junctions in Henries ($H$).
    pub coupling_inductance: f64,
    /// DC bias current $I_{bias}$ per junction ($A$).
    pub bias_current: f64,
}

impl JtlStage {
    /// Creates a new JTL stage.
    pub fn new(ic: f64, rn: f64, l_coupling: f64, bias_fraction: f64) -> Self {
        let b = bias_fraction.clamp(0.0, 0.95);
        let phi_0 = b.asin();
        let mut jj1 = JosephsonRcsjModel::overdamped_rsfq(ic, rn);
        jj1.phase = phi_0;
        let mut jj2 = JosephsonRcsjModel::overdamped_rsfq(ic, rn);
        jj2.phase = phi_0;
        Self {
            jj1,
            jj2,
            coupling_inductance: l_coupling.max(1e-15),
            bias_current: ic * b,
        }
    }

    /// Transient simulation step using Runge-Kutta 4th order (RK4) on the coupled phase equations.
    ///
    /// The coupled RCSJ equations for the JTL stage:
    /// $$\frac{d\phi_1}{dt} = \frac{2\pi}{\Phi_0} V_1$$
    /// $$\frac{d V_1}{dt} = \frac{1}{C_1} \left( I_{bias} + I_{in} - I_c \sin(\phi_1) - \frac{V_1}{R_n} - \frac{\Phi_0}{2\pi L} (\phi_1 - \phi_2) \right)$$
    /// $$\frac{d\phi_2}{dt} = \frac{2\pi}{\Phi_0} V_2$$
    /// $$\frac{d V_2}{dt} = \frac{1}{C_2} \left( I_{bias} - I_c \sin(\phi_2) - \frac{V_2}{R_n} + \frac{\Phi_0}{2\pi L} (\phi_1 - \phi_2) \right)$$
    pub fn step_rk4(&mut self, dt: f64, input_current: f64) -> (f64, f64) {
        // State vector: [phi1, v1, phi2, v2]
        let mut y = [
            self.jj1.phase,
            self.jj1.voltage,
            self.jj2.phase,
            self.jj2.voltage,
        ];

        let derivatives = |s: &[f64; 4], i_in: f64| -> [f64; 4] {
            let p1 = s[0];
            let v1 = s[1];
            let p2 = s[2];
            let v2 = s[3];

            let two_pi_over_phi0 = (2.0 * std::f64::consts::PI) / FLUX_QUANTUM;
            let dphi1_dt = two_pi_over_phi0 * v1;
            let dphi2_dt = two_pi_over_phi0 * v2;

            let i_l = (FLUX_QUANTUM / (2.0 * std::f64::consts::PI * self.coupling_inductance))
                * (p1 - p2);

            let dv1_dt = (1.0 / self.jj1.cap)
                * (self.bias_current + i_in - self.jj1.ic * p1.sin() - v1 / self.jj1.rn - i_l);

            let dv2_dt = (1.0 / self.jj2.cap)
                * (self.bias_current - self.jj2.ic * p2.sin() - v2 / self.jj2.rn + i_l);

            [dphi1_dt, dv1_dt, dphi2_dt, dv2_dt]
        };

        let k1 = derivatives(&y, input_current);
        let mut y2 = [0.0; 4];
        for i in 0..4 {
            y2[i] = y[i] + 0.5 * dt * k1[i];
        }
        let k2 = derivatives(&y2, input_current);

        let mut y3 = [0.0; 4];
        for i in 0..4 {
            y3[i] = y[i] + 0.5 * dt * k2[i];
        }
        let k3 = derivatives(&y3, input_current);

        let mut y4 = [0.0; 4];
        for i in 0..4 {
            y4[i] = y[i] + dt * k3[i];
        }
        let k4 = derivatives(&y4, input_current);

        for i in 0..4 {
            y[i] += (dt / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }

        self.jj1.phase = y[0];
        self.jj1.voltage = y[1];
        self.jj2.phase = y[2];
        self.jj2.voltage = y[3];

        (y[1], y[3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sfq_pulse_quantization_exactness() {
        let pulse = SfqPulse::new(5e-12, 0.5e-12); // t0 = 5 ps, tau = 0.5 ps
        let n_steps = 2000;
        let dt = 1e-14; // 10 fs steps across 20 ps window
        let mut times = Vec::with_capacity(n_steps);
        let mut voltages = Vec::with_capacity(n_steps);

        for i in 0..n_steps {
            let t = (i as f64) * dt;
            times.push(t);
            voltages.push(pulse.voltage(t));
        }

        let result = verify_flux_quantization(&times, &voltages, 1e-4);
        assert!(result.is_ok(), "Quantization check failed: {:?}", result);
        let flux = result.unwrap();
        assert!((flux - FLUX_QUANTUM).abs() / FLUX_QUANTUM < 1e-4);
    }

    #[test]
    fn test_dc_squid_flux_modulation() {
        let squid = DcSquidModel::symmetric(50e-6, 10.0, 1e-13, 2e-12);
        // At zero flux, Ic = 2 * Ic0 = 100 uA
        let ic_0 = squid.critical_current(0.0);
        assert!((ic_0 - 100e-6).abs() < 1e-9);

        // At half flux quantum Phi_0 / 2, Ic = 0
        let ic_half = squid.critical_current(FLUX_QUANTUM * 0.5);
        assert!(ic_half < 1e-9);

        // At integer flux quantum Phi_0, Ic returns to 100 uA
        let ic_full = squid.critical_current(FLUX_QUANTUM);
        assert!((ic_full - 100e-6).abs() < 1e-9);
    }
}
