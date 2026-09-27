//! Real inductor model with DC series resistance, inter-winding capacitance, and non-linear magnetic saturation.

use phonon_core::{CircuitGraph, CoreError};
use std::f64::consts::PI;

/// Physical real-world inductor model incorporating:
/// - Series DC winding resistance ($R_{dc}$).
/// - Parallel inter-winding parasitic capacitance ($C_p$).
/// - Non-linear magnetic core saturation: $L(I) = \frac{L_0}{1 + (I / I_{sat})^2}$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RealInductorModel {
    /// Nominal low-current inductance in Henrys ($L_0$).
    pub inductance_zero_bias: f64,
    /// DC winding series resistance in Ohms ($R_{dc}$).
    pub r_dc: f64,
    /// Inter-winding parallel capacitance in Farads ($C_p$).
    pub c_parallel: f64,
    /// Magnetic core saturation current in Amperes ($I_{sat}$).
    pub i_sat: Option<f64>,
}

impl Default for RealInductorModel {
    fn default() -> Self {
        Self {
            inductance_zero_bias: 10.0e-3, // 10 mH
            r_dc: 0.50,                    // 500 mOhm
            c_parallel: 20.0e-12,          // 20 pF
            i_sat: Some(2.0),              // 2.0 A saturation knee
        }
    }
}

impl RealInductorModel {
    pub fn new(inductance: f64, r_dc: f64, c_parallel: f64) -> Self {
        Self {
            inductance_zero_bias: inductance,
            r_dc,
            c_parallel,
            i_sat: None,
        }
    }

    pub fn with_saturation(mut self, i_sat: f64) -> Self {
        self.i_sat = Some(i_sat);
        self
    }

    /// Evaluates the instantaneous differential inductance $L(I)$ at bias current $I$ (Amperes):
    /// $$L(I) = \frac{L_0}{1 + (I / I_{sat})^2}$$
    #[inline]
    pub fn inductance_at_current(&self, current: f64) -> f64 {
        if let Some(i_sat) = self.i_sat {
            if i_sat > 0.0 {
                let ratio = current / i_sat;
                return self.inductance_zero_bias / (1.0 + ratio * ratio);
            }
        }
        self.inductance_zero_bias
    }

    /// Total magnetic flux linkage $\Phi(I) = \int_0^I L(i') di'$ in Weber:
    /// $$\Phi(I) = L_0 I_{sat} \arctan(I / I_{sat})$$
    #[inline]
    pub fn flux_linkage(&self, current: f64) -> f64 {
        if let Some(i_sat) = self.i_sat {
            if i_sat > 0.0 {
                return self.inductance_zero_bias * i_sat * (current / i_sat).atan();
            }
        }
        self.inductance_zero_bias * current
    }

    /// Computes the Self-Resonant Frequency (SRF) in Hertz:
    /// $$f_{\text{SRF}} = \frac{1}{2\pi \sqrt{L_0 \cdot C_p}}$$
    #[inline]
    pub fn self_resonant_frequency(&self) -> f64 {
        if self.inductance_zero_bias <= 0.0 || self.c_parallel <= 0.0 {
            0.0
        } else {
            1.0 / (2.0 * PI * (self.inductance_zero_bias * self.c_parallel).sqrt())
        }
    }

    /// Computes complex small-signal impedance $Z(f) = R(f) + j X(f)$ at frequency $f$ (Hz):
    /// Series $R_{dc} + j\omega L_0$ in parallel with $C_p$.
    pub fn complex_impedance(&self, freq_hz: f64) -> (f64, f64) {
        let f = freq_hz.max(1e-6);
        let omega = 2.0 * PI * f;

        // Series branch: Z_s = R_dc + j*omega*L
        let r_s = self.r_dc;
        let x_s = omega * self.inductance_zero_bias;
        let z_s_sq = r_s * r_s + x_s * x_s;

        // Admittance of series branch: Y_s = (R_dc - j*omega*L) / |Z_s|^2
        let g_s = r_s / z_s_sq;
        let b_s = -x_s / z_s_sq;

        // Total admittance Y_tot = Y_s + j*omega*C_p
        let g_tot = g_s;
        let b_tot = b_s + omega * self.c_parallel;
        let y_tot_sq = g_tot * g_tot + b_tot * b_tot;

        // Invert to get Z_tot
        let real = g_tot / y_tot_sq;
        let imag = -b_tot / y_tot_sq;

        (real, imag)
    }

    /// Computes the absolute magnitude of impedance $|Z(f)|$ in Ohms at frequency $f$ (Hz).
    #[inline]
    pub fn impedance_magnitude(&self, freq_hz: f64) -> f64 {
        let (r, x) = self.complex_impedance(freq_hz);
        (r * r + x * x).sqrt()
    }

    /// Synthesizes the physical parasitic equivalent subcircuit into a `CircuitGraph`:
    /// Connects: `node_a` -> (R_dc) -> `node_int` -> (L_0) -> `node_b`
    /// with parallel inter-winding capacitance $C_p$ directly across `node_a` and `node_b`.
    pub fn synthesize_subcircuit(
        &self,
        graph: &mut CircuitGraph,
        base_name: &str,
        node_a: &str,
        node_b: &str,
    ) -> Result<(), CoreError> {
        let int = format!("{base_name}_int");

        // 1. Series DC resistance
        graph.add_resistor(
            &format!("{base_name}_RDC"),
            node_a,
            &int,
            self.r_dc.max(1e-6),
        )?;
        // 2. Inductor
        graph.add_inductor(
            &format!("{base_name}_L"),
            &int,
            node_b,
            self.inductance_zero_bias,
            Some(0.0),
        )?;
        // 3. Parallel Inter-winding Capacitance
        if self.c_parallel > 0.0 {
            graph.add_capacitor(
                &format!("{base_name}_CP"),
                node_a,
                node_b,
                self.c_parallel,
                None,
            )?;
        }

        Ok(())
    }
}
