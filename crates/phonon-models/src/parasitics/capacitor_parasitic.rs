//! Real capacitor model with physical parasitics (ESR, ESL, parallel leakage).

use phonon_core::{CircuitGraph, CoreError};
use std::f64::consts::PI;

/// Physical real-world capacitor model incorporating parasitics:
/// - Equivalent Series Resistance (ESR) due to leads, foils, and dielectric dissipation.
/// - Equivalent Series Inductance (ESL) due to physical geometry and lead length.
/// - Parallel insulation leakage resistance ($R_p$).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RealCapacitorModel {
    /// Nominal capacitance in Farads ($C$).
    pub capacitance: f64,
    /// Equivalent Series Resistance in Ohms (ESR).
    pub esr: f64,
    /// Equivalent Series Inductance in Henrys (ESL).
    pub esl: f64,
    /// Parallel insulation / dielectric leakage resistance in Ohms ($R_p$). Default $10^{10}\ \Omega$.
    pub parallel_resistance: f64,
}

impl Default for RealCapacitorModel {
    fn default() -> Self {
        Self {
            capacitance: 100.0e-9,       // 100 nF
            esr: 0.020,                  // 20 mOhm
            esl: 1.5e-9,                 // 1.5 nH
            parallel_resistance: 1.0e10, // 10 GOhm
        }
    }
}

impl RealCapacitorModel {
    pub fn new(capacitance: f64, esr: f64, esl: f64) -> Self {
        Self {
            capacitance,
            esr,
            esl,
            parallel_resistance: 1.0e10,
        }
    }

    pub fn with_parallel_resistance(mut self, rp: f64) -> Self {
        self.parallel_resistance = rp;
        self
    }

    /// Computes the Self-Resonant Frequency (SRF) in Hertz:
    /// $$f_{\text{SRF}} = \frac{1}{2\pi \sqrt{C \cdot \text{ESL}}}$$
    #[inline]
    pub fn self_resonant_frequency(&self) -> f64 {
        if self.capacitance <= 0.0 || self.esl <= 0.0 {
            0.0
        } else {
            1.0 / (2.0 * PI * (self.capacitance * self.esl).sqrt())
        }
    }

    /// Computes the complex impedance $Z(f) = R(f) + j X(f)$ at frequency $f$ (Hz):
    /// $$Z(f) = \text{ESR} + j 2\pi f \text{ESL} + \frac{1}{j 2\pi f C + 1/R_p}$$
    pub fn complex_impedance(&self, freq_hz: f64) -> (f64, f64) {
        let f = freq_hz.max(1e-6);
        let omega = 2.0 * PI * f;

        // Admittance of parallel C || Rp: Y_p = 1/Rp + j*omega*C
        let g_p = 1.0 / self.parallel_resistance.max(1.0);
        let b_p = omega * self.capacitance;
        let denom = g_p * g_p + b_p * b_p;

        // Invert to get Z_p:
        let r_p = g_p / denom;
        let x_p = -b_p / denom;

        let real = self.esr + r_p;
        let imag = omega * self.esl + x_p;

        (real, imag)
    }

    /// Computes the absolute magnitude of impedance $|Z(f)|$ in Ohms at frequency $f$ (Hz).
    #[inline]
    pub fn impedance_magnitude(&self, freq_hz: f64) -> f64 {
        let (r, x) = self.complex_impedance(freq_hz);
        (r * r + x * x).sqrt()
    }

    /// Computes the Quality Factor $Q(f) = \frac{|X(f)|}{R(f)}$ at frequency $f$ (Hz).
    #[inline]
    pub fn quality_factor(&self, freq_hz: f64) -> f64 {
        let (r, x) = self.complex_impedance(freq_hz);
        if r.abs() < 1e-12 {
            1e6
        } else {
            x.abs() / r
        }
    }

    /// Synthesizes the physical parasitic equivalent subcircuit into a `CircuitGraph`:
    /// Connects: `node_a` -> (ESR) -> `node_int1` -> (ESL) -> `node_int2` -> (C) -> `node_b`
    /// with parallel leakage resistor $R_p$ across `node_int2` and `node_b`.
    pub fn synthesize_subcircuit(
        &self,
        graph: &mut CircuitGraph,
        base_name: &str,
        node_a: &str,
        node_b: &str,
    ) -> Result<(), CoreError> {
        let int1 = format!("{base_name}_int1");
        let int2 = format!("{base_name}_int2");

        // 1. ESR Resistor
        graph.add_resistor(
            &format!("{base_name}_ESR"),
            node_a,
            &int1,
            self.esr.max(1e-6),
        )?;
        // 2. ESL Inductor
        graph.add_inductor(
            &format!("{base_name}_ESL"),
            &int1,
            &int2,
            self.esl.max(1e-15),
            Some(0.0),
        )?;
        // 3. Ideal Capacitor
        graph.add_capacitor(
            &format!("{base_name}_C"),
            &int2,
            node_b,
            self.capacitance,
            None,
        )?;
        // 4. Parallel Leakage Resistance
        if self.parallel_resistance < 1.0e12 {
            graph.add_resistor(
                &format!("{base_name}_RP"),
                &int2,
                node_b,
                self.parallel_resistance,
            )?;
        }

        Ok(())
    }
}
