//! Coupled microstrip transmission lines and high-speed digital crosstalk dynamics (NEXT/FEXT).

use phonon_core::{CircuitGraph, CoreError};

/// High-speed coupled transmission line pair (aggressor and victim traces):
/// Characterized by modal parameters:
/// - Even-mode characteristic impedance $Z_{0e}$ and propagation delay $\tau_e$.
/// - Odd-mode characteristic impedance $Z_{0o}$ and propagation delay $\tau_o$.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoupledMicrostripLine {
    /// Even-mode characteristic impedance in Ohms ($Z_{0e}$).
    pub z0_even: f64,
    /// Odd-mode characteristic impedance in Ohms ($Z_{0o}$).
    pub z0_odd: f64,
    /// Even-mode propagation delay in seconds ($\tau_e$).
    pub tau_even: f64,
    /// Odd-mode propagation delay in seconds ($\tau_o$).
    pub tau_odd: f64,
    /// Physical trace length in meters ($\ell$).
    pub length_m: f64,
}

impl Default for CoupledMicrostripLine {
    fn default() -> Self {
        Self {
            z0_even: 58.0,       // 58 Ohm
            z0_odd: 43.0,        // 43 Ohm
            tau_even: 650.0e-12, // 650 ps
            tau_odd: 580.0e-12,  // 580 ps
            length_m: 0.10,      // 10 cm
        }
    }
}

impl CoupledMicrostripLine {
    pub fn new(z0_even: f64, z0_odd: f64, tau_even: f64, tau_odd: f64, length_m: f64) -> Self {
        Self {
            z0_even,
            z0_odd,
            tau_even,
            tau_odd,
            length_m,
        }
    }

    /// Single-ended uncoupled characteristic impedance approximation:
    /// $$Z_0 = \sqrt{Z_{0e} Z_{0o}}$$
    #[inline]
    pub fn single_ended_impedance(&self) -> f64 {
        (self.z0_even * self.z0_odd).sqrt()
    }

    /// Backward / Near-End Crosstalk (NEXT) coupling coefficient:
    /// $$K_B = \frac{Z_{0e} - Z_{0o}}{Z_{0e} + Z_{0o}}$$
    #[inline]
    pub fn next_coefficient(&self) -> f64 {
        (self.z0_even - self.z0_odd) / (self.z0_even + self.z0_odd)
    }

    /// Evaluates the peak Near-End Crosstalk (NEXT) voltage induced on the victim line
    /// when the aggressor line is driven by an incident step pulse of amplitude $V_{\text{step}}$:
    /// $$V_{\text{NEXT}} = K_B \cdot V_{\text{step}}$$
    #[inline]
    pub fn peak_next_voltage(&self, v_step: f64) -> f64 {
        self.next_coefficient() * v_step
    }

    /// Forward / Far-End Crosstalk (FEXT) coupling coefficient per meter:
    /// $$K_F = \frac{\tau_e - \tau_o}{2 \ell}$$
    #[inline]
    pub fn fext_coefficient(&self) -> f64 {
        if self.length_m > 0.0 {
            (self.tau_even - self.tau_odd) / (2.0 * self.length_m)
        } else {
            0.0
        }
    }

    /// Evaluates peak Far-End Crosstalk (FEXT) voltage induced on the victim line for a
    /// step pulse of amplitude $V_{\text{step}}$ with rise time $t_r$:
    /// $$V_{\text{FEXT}} = - \frac{\tau_e - \tau_o}{2} \cdot \frac{V_{\text{step}}}{t_r}$$
    #[inline]
    pub fn peak_fext_voltage(&self, v_step: f64, rise_time: f64) -> f64 {
        let tr = rise_time.max(1e-15);
        -0.5 * (self.tau_even - self.tau_odd) * (v_step / tr)
    }

    /// Synthesizes a coupled line pair into a `CircuitGraph` using uncoupled self-inductances
    /// and mutual capacitive/inductive coupling elements:
    #[allow(clippy::too_many_arguments)]
    pub fn synthesize_subcircuit(
        &self,
        graph: &mut CircuitGraph,
        base_name: &str,
        aggressor_in: &str,
        aggressor_out: &str,
        victim_in: &str,
        victim_out: &str,
        ref_node: &str,
    ) -> Result<(), CoreError> {
        let z0 = self.single_ended_impedance();
        let tau_avg = 0.5 * (self.tau_even + self.tau_odd);

        // Line 1: Aggressor line
        graph.add_transmission_line(
            &format!("{base_name}_T1"),
            aggressor_in,
            ref_node,
            aggressor_out,
            ref_node,
            z0,
            tau_avg,
        )?;

        // Line 2: Victim line
        graph.add_transmission_line(
            &format!("{base_name}_T2"),
            victim_in,
            ref_node,
            victim_out,
            ref_node,
            z0,
            tau_avg,
        )?;

        // Inter-line mutual coupling capacitance C_m
        let c_odd = self.tau_odd / self.z0_odd;
        let c_even = self.tau_even / self.z0_even;
        let c_mutual = (0.5 * (c_odd - c_even)).max(1e-15);

        graph.add_capacitor(
            &format!("{base_name}_CM_in"),
            aggressor_in,
            victim_in,
            c_mutual * 0.5,
            None,
        )?;
        graph.add_capacitor(
            &format!("{base_name}_CM_out"),
            aggressor_out,
            victim_out,
            c_mutual * 0.5,
            None,
        )?;

        Ok(())
    }
}
