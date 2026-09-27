//! Lossy transmission lines governed by Telegrapher's differential equations (RLGC model).

use phonon_core::{CircuitGraph, CoreError};
use std::f64::consts::PI;

/// Lossy distributed transmission line characterized by distributed per-unit-length parameters:
/// - $R'$: series resistance ($\Omega/\text{m}$) with optional skin-effect coefficient $R_s'$.
/// - $L'$: series inductance ($\text{H}/\text{m}$).
/// - $G'$: parallel dielectric shunt conductance ($\text{S}/\text{m}$).
/// - $C'$: parallel capacitance ($\text{F}/\text{m}$).
/// - $\ell$: physical length in meters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LossyRlgcLine {
    /// DC series resistance per unit length in $\Omega/\text{m}$ ($R'_0$).
    pub r_per_m: f64,
    /// Series inductance per unit length in $\text{H}/\text{m}$ ($L'$).
    pub l_per_m: f64,
    /// Shunt conductance per unit length in $\text{S}/\text{m}$ ($G'$).
    pub g_per_m: f64,
    /// Shunt capacitance per unit length in $\text{F}/\text{m}$ ($C'$).
    pub c_per_m: f64,
    /// Physical length of the transmission line in meters ($\ell$).
    pub length_m: f64,
    /// Skin-effect resistance coefficient in $\Omega / (\text{m} \cdot \sqrt{\text{Hz}})$.
    pub skin_effect_coeff: f64,
}

impl Default for LossyRlgcLine {
    fn default() -> Self {
        Self {
            r_per_m: 0.20,           // 0.2 Ohm/m
            l_per_m: 250.0e-9,       // 250 nH/m
            g_per_m: 1.0e-6,         // 1 uS/m
            c_per_m: 100.0e-12,      // 100 pF/m -> Z0 ~ 50 Ohm
            length_m: 0.10,          // 10 cm
            skin_effect_coeff: 1e-4, // 0.1 mOhm / (m * sqrt(Hz))
        }
    }
}

impl LossyRlgcLine {
    pub fn new(r_per_m: f64, l_per_m: f64, g_per_m: f64, c_per_m: f64, length_m: f64) -> Self {
        Self {
            r_per_m,
            l_per_m,
            g_per_m,
            c_per_m,
            length_m,
            skin_effect_coeff: 0.0,
        }
    }

    pub fn with_skin_effect(mut self, coeff: f64) -> Self {
        self.skin_effect_coeff = coeff;
        self
    }

    /// Effective series resistance at frequency $f$ (Hz) accounting for skin effect:
    /// $$R'(f) = R'_0 + R'_s \sqrt{f}$$
    #[inline]
    pub fn effective_resistance(&self, freq_hz: f64) -> f64 {
        if self.skin_effect_coeff > 0.0 && freq_hz > 0.0 {
            self.r_per_m + self.skin_effect_coeff * freq_hz.sqrt()
        } else {
            self.r_per_m
        }
    }

    /// Evaluates the complex propagation constant $\gamma(f) = \alpha(f) + j \beta(f)$ in $\text{m}^{-1}$:
    /// $$\gamma(f) = \sqrt{(R'(f) + j\omega L')(G' + j\omega C')}$$
    pub fn propagation_constant(&self, freq_hz: f64) -> (f64, f64) {
        let f = freq_hz.max(1e-6);
        let omega = 2.0 * PI * f;
        let r = self.effective_resistance(f);

        // Z_series = r + j * omega * L
        // Y_shunt = g + j * omega * C
        // Product = (r*g - omega^2*L*C) + j * (r*omega*C + g*omega*L)
        let re_prod = r * self.g_per_m - omega * omega * self.l_per_m * self.c_per_m;
        let im_prod = omega * (r * self.c_per_m + self.g_per_m * self.l_per_m);

        // Complex square root: sqrt(a + j*b)
        let mag = (re_prod * re_prod + im_prod * im_prod).sqrt();
        let alpha = ((mag + re_prod) / 2.0).sqrt();
        let beta = ((mag - re_prod) / 2.0).sqrt() * im_prod.signum();

        (alpha, beta)
    }

    /// Total line attenuation in decibels (dB) at frequency $f$:
    /// $$\text{Attenuation}_{\text{dB}} = 20 \log_{10}(e) \cdot \alpha(f) \cdot \ell \approx 8.68589 \cdot \alpha \cdot \ell$$
    #[inline]
    pub fn attenuation_db(&self, freq_hz: f64) -> f64 {
        let (alpha, _) = self.propagation_constant(freq_hz);
        8.685_889_638 * alpha * self.length_m
    }

    /// Characteristic impedance $Z_0(f)$ magnitude in Ohms:
    /// $$Z_0(f) = \left| \sqrt{\frac{R'(f) + j\omega L'}{G' + j\omega C'}} \right|$$
    pub fn characteristic_impedance_magnitude(&self, freq_hz: f64) -> f64 {
        let f = freq_hz.max(1e-6);
        let omega = 2.0 * PI * f;
        let r = self.effective_resistance(f);

        let z_num = (r * r + omega * omega * self.l_per_m * self.l_per_m).sqrt();
        let y_den =
            (self.g_per_m * self.g_per_m + omega * omega * self.c_per_m * self.c_per_m).sqrt();

        (z_num / y_den).sqrt()
    }

    /// Synthesizes an equivalent high-frequency cascaded lumped $\Pi$-section ladder into a `CircuitGraph`:
    /// Discretizes the line of length $\ell$ into $N$ sections of length $\Delta x = \ell / N$.
    #[allow(clippy::too_many_arguments)]
    pub fn synthesize_lumped_ladder(
        &self,
        graph: &mut CircuitGraph,
        base_name: &str,
        in_pos: &str,
        in_neg: &str,
        out_pos: &str,
        out_neg: &str,
        num_sections: usize,
    ) -> Result<(), CoreError> {
        let n = num_sections.max(1);
        let dx = self.length_m / n as f64;

        let r_sec = self.r_per_m * dx;
        let l_sec = self.l_per_m * dx;
        let c_sec_half = 0.5 * self.c_per_m * dx;
        let g_sec_half = 0.5 * self.g_per_m * dx;

        let mut prev_pos = in_pos.to_string();

        for i in 0..n {
            let is_last = i == n - 1;
            let next_pos = if is_last {
                out_pos.to_string()
            } else {
                format!("{base_name}_node_{}", i + 1)
            };

            // Input shunt C/2 and G/2 at prev_pos
            if c_sec_half > 0.0 {
                graph.add_capacitor(
                    &format!("{base_name}_C_in_{i}"),
                    &prev_pos,
                    in_neg,
                    c_sec_half,
                    None,
                )?;
            }
            if g_sec_half > 1e-12 {
                graph.add_resistor(
                    &format!("{base_name}_G_in_{i}"),
                    &prev_pos,
                    in_neg,
                    1.0 / g_sec_half,
                )?;
            }

            // Series R and L between prev_pos and next_pos
            let int_node = format!("{base_name}_int_{i}");
            if r_sec > 1e-6 {
                graph.add_resistor(&format!("{base_name}_R_{i}"), &prev_pos, &int_node, r_sec)?;
            } else {
                graph.add_resistor(&format!("{base_name}_R_{i}"), &prev_pos, &int_node, 1e-6)?;
            }

            graph.add_inductor(
                &format!("{base_name}_L_{i}"),
                &int_node,
                &next_pos,
                l_sec,
                Some(0.0),
            )?;

            // Output shunt C/2 and G/2 at next_pos
            if c_sec_half > 0.0 {
                graph.add_capacitor(
                    &format!("{base_name}_C_out_{i}"),
                    &next_pos,
                    out_neg,
                    c_sec_half,
                    None,
                )?;
            }
            if g_sec_half > 1e-12 {
                graph.add_resistor(
                    &format!("{base_name}_G_out_{i}"),
                    &next_pos,
                    out_neg,
                    1.0 / g_sec_half,
                )?;
            }

            prev_pos = next_pos;
        }

        Ok(())
    }
}
