//! Non-Equilibrium Green's Function (NEGF) transport solver and Landauer-Büttiker transmission.
//!
//! Formulates:
//! 1. Complex matrix inversion computing retarded Green's function \(G^R(E)\).
//! 2. Fisher-Lee relation evaluating quantum transmission \(\mathcal{T}(E) = \Gamma_L \Gamma_R |G^R_{LR}(E)|^2\).
//! 3. Identification of constructive interference peaks and destructive anti-resonances.
//! 4. Landauer-Büttiker current integral across bias window \([f_L(E) - f_R(E)]\).

#![allow(clippy::needless_range_loop)]

use crate::molecular::molecular_junction::MolecularJunction;
use crate::quantum::Complex;
use phonon_core::{ELEMENTARY_CHARGE, PLANCK_CONSTANT};

pub const CONDUCTANCE_QUANTUM_G0: f64 =
    (2.0 * ELEMENTARY_CHARGE * ELEMENTARY_CHARGE) / PLANCK_CONSTANT; // ~77.48 uS

/// Solves a general dense NxN complex linear system A * X = B via Gauss-Jordan elimination with partial pivoting.
pub fn solve_complex_linear_system(
    mut a: Vec<Vec<Complex>>,
    mut b: Vec<Vec<Complex>>,
) -> Option<Vec<Vec<Complex>>> {
    let n = a.len();
    assert_eq!(b.len(), n);
    let m = b[0].len();

    for col in 0..n {
        // Find pivot with maximum norm
        let mut max_norm = 0.0;
        let mut pivot_row = col;
        for row in col..n {
            let norm = a[row][col].norm_sq();
            if norm > max_norm {
                max_norm = norm;
                pivot_row = row;
            }
        }

        if max_norm < 1.0e-30 {
            return None; // Singular matrix
        }

        // Swap pivot row
        if pivot_row != col {
            a.swap(col, pivot_row);
            b.swap(col, pivot_row);
        }

        // Normalize pivot row
        let pivot = a[col][col];
        for j in 0..n {
            a[col][j] = a[col][j].div(pivot);
        }
        for j in 0..m {
            b[col][j] = b[col][j].div(pivot);
        }

        // Eliminate column entries in other rows
        for row in 0..n {
            if row != col {
                let factor = a[row][col];
                if factor.norm_sq() > 1.0e-30 {
                    for j in 0..n {
                        let sub_val = factor.mul(a[col][j]);
                        a[row][j] = a[row][j].sub(sub_val);
                    }
                    for j in 0..m {
                        let sub_val = factor.mul(b[col][j]);
                        b[row][j] = b[row][j].sub(sub_val);
                    }
                }
            }
        }
    }

    Some(b)
}

/// Computes the exact matrix inverse of an NxN complex matrix.
pub fn invert_complex_matrix(a: Vec<Vec<Complex>>) -> Option<Vec<Vec<Complex>>> {
    let n = a.len();
    let mut identity = vec![vec![Complex::ZERO; n]; n];
    for i in 0..n {
        identity[i][i] = Complex::ONE;
    }
    solve_complex_linear_system(a, identity)
}

/// NEGF quantum transport solver evaluating transmission spectra and Landauer currents.
#[derive(Debug, Clone)]
pub struct NegfTransportSolver {
    pub temperature_k: f64,
    pub fermi_level_ev: f64,
}

impl Default for NegfTransportSolver {
    fn default() -> Self {
        Self {
            temperature_k: 300.0,
            fermi_level_ev: 0.0,
        }
    }
}

impl NegfTransportSolver {
    /// Creates a new NEGF solver with specified temperature and Fermi level.
    pub fn new(temperature_k: f64, fermi_level_ev: f64) -> Self {
        Self {
            temperature_k: temperature_k.max(1.0),
            fermi_level_ev,
        }
    }

    /// Evaluates the retarded Green's function G^R(E) for a molecular junction.
    pub fn evaluate_greens_function(
        &self,
        junction: &MolecularJunction,
        energy_ev: f64,
        v_gate_v: f64,
    ) -> Option<Vec<Vec<Complex>>> {
        let a_mat = junction.assemble_green_matrix(energy_ev, v_gate_v);
        invert_complex_matrix(a_mat)
    }

    /// Evaluates quantum transmission function \(\mathcal{T}(E)\) at specified energy E.
    ///
    /// Uses the Fisher-Lee relation:
    /// \[\mathcal{T}(E) = \gamma_L \cdot \gamma_R \cdot |G^R_{LR}(E)|^2\]
    pub fn evaluate_transmission(
        &self,
        junction: &MolecularJunction,
        energy_ev: f64,
        v_gate_v: f64,
    ) -> f64 {
        if let Some(g_mat) = self.evaluate_greens_function(junction, energy_ev, v_gate_v) {
            let g_lr = g_mat[junction.left_lead_site][junction.right_lead_site];
            let t_val = junction.gamma_left_ev * junction.gamma_right_ev * g_lr.norm_sq();

            // Include vibronic inelastic satellite contribution if coupled
            if junction.vibronic_coupling_ev > 0.0 {
                let vib_shift = energy_ev - junction.phonon_energy_ev;
                if let Some(g_vib) = self.evaluate_greens_function(junction, vib_shift, v_gate_v) {
                    let g_vib_lr = g_vib[junction.left_lead_site][junction.right_lead_site];
                    let t_inel = 0.05 * junction.vibronic_coupling_ev * g_vib_lr.norm_sq();
                    return (t_val + t_inel).clamp(0.0, 1.0);
                }
            }

            t_val.clamp(0.0, 1.0)
        } else {
            0.0
        }
    }

    /// Computes the transmission spectrum \(\mathcal{T}(E)\) across an energy range.
    pub fn compute_transmission_spectrum(
        &self,
        junction: &MolecularJunction,
        e_min_ev: f64,
        e_max_ev: f64,
        num_points: usize,
        v_gate_v: f64,
    ) -> Vec<(f64, f64)> {
        let de = (e_max_ev - e_min_ev) / (num_points as f64).max(1.0);
        let mut spectrum = Vec::with_capacity(num_points);

        for i in 0..num_points {
            let e = e_min_ev + (i as f64) * de;
            let t = self.evaluate_transmission(junction, e, v_gate_v);
            spectrum.push((e, t));
        }

        spectrum
    }

    /// Fermi-Dirac distribution function f(E, mu).
    pub fn fermi_dirac(&self, energy_ev: f64, mu_ev: f64) -> f64 {
        let kb_t_ev = 8.617_333_262e-5 * self.temperature_k;
        let x = (energy_ev - mu_ev) / kb_t_ev;
        if x > 40.0 {
            0.0
        } else if x < -40.0 {
            1.0
        } else {
            1.0 / (1.0 + x.exp())
        }
    }

    /// Evaluates Landauer-Büttiker current I [A] at bias voltage V_bias [V].
    ///
    /// \[I(V) = \frac{2 e}{h} \int \mathcal{T}(E) [f_L(E) - f_R(E)] dE\]
    pub fn evaluate_current_landauer(
        &self,
        junction: &MolecularJunction,
        v_bias_v: f64,
        v_gate_v: f64,
    ) -> f64 {
        let mu_l = self.fermi_level_ev + (v_bias_v * 0.5);
        let mu_r = self.fermi_level_ev - (v_bias_v * 0.5);

        let kb_t_ev = 8.617_333_262e-5 * self.temperature_k;
        let e_window = v_bias_v.abs() + (10.0 * kb_t_ev);
        let e_min = self.fermi_level_ev - e_window;
        let e_max = self.fermi_level_ev + e_window;

        let num_steps = 100;
        let de = (e_max - e_min) / (num_steps as f64);
        let mut integral = 0.0;

        for i in 0..num_steps {
            let e = e_min + ((i as f64) + 0.5) * de;
            let f_l = self.fermi_dirac(e, mu_l);
            let f_r = self.fermi_dirac(e, mu_r);
            let f_diff = f_l - f_r;

            if f_diff.abs() > 1.0e-9 {
                let t_e = self.evaluate_transmission(junction, e, v_gate_v);
                integral += t_e * f_diff * de;
            }
        }

        // Conductance factor: 2e/h * integral [eV] -> 2e^2/h * integral [V] = G_0 * integral
        CONDUCTANCE_QUANTUM_G0 * integral
    }

    /// Linear-response zero-bias conductance G [Siemens].
    pub fn evaluate_zero_bias_conductance(
        &self,
        junction: &MolecularJunction,
        v_gate_v: f64,
    ) -> f64 {
        let t_ef = self.evaluate_transmission(junction, self.fermi_level_ev, v_gate_v);
        CONDUCTANCE_QUANTUM_G0 * t_ef
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_para_vs_meta_benzene_quantum_interference() {
        let solver = NegfTransportSolver::default();
        let para = MolecularJunction::para_benzene(0.6);
        let meta = MolecularJunction::meta_benzene(0.6);

        // Transmission at Fermi level E = 0.0 eV
        let t_para = solver.evaluate_transmission(&para, 0.0, 0.0);
        let t_meta = solver.evaluate_transmission(&meta, 0.0, 0.0);

        // Para-benzene exhibits non-zero mid-gap tunneling (T > 0.01)
        assert!(
            t_para > 0.01,
            "Para transmission was {:.4}, expected > 0.01",
            t_para
        );

        // Meta-benzene exhibits destructive quantum interference with sharp anti-resonance (T -> 0)
        assert!(
            t_meta < 0.0001,
            "Meta transmission was {:.6}, expected < 0.0001",
            t_meta
        );

        // Intrinsic on/off ratio between constructive and destructive topologies (> 1000x)
        let qi_ratio = t_para / t_meta.max(1e-12);
        assert!(qi_ratio > 1000.0, "QI on/off ratio was {:.1}x", qi_ratio);

        // Near resonance, para transmission approaches unity
        let spectrum = solver.compute_transmission_spectrum(&para, -6.0, 6.0, 120, 0.0);
        let max_t = spectrum.iter().cloned().fold(
            (0.0, 0.0),
            |acc, (e, t)| if t > acc.1 { (e, t) } else { acc },
        );
        assert!(max_t.1 > 0.40, "Para peak transmission was {:.4}", max_t.1);
    }

    #[test]
    fn test_landauer_current_and_gate_switching() {
        let solver = NegfTransportSolver::default();
        let junction = MolecularJunction::cross_conjugated(0.6);

        // At 0V gate, anti-resonance suppresses current (Off state)
        let i_off = solver.evaluate_current_landauer(&junction, 0.20, 0.0).abs();

        // At 0.8V gate, orbital shift brings transmission resonance into bias window (On state)
        let i_on = solver
            .evaluate_current_landauer(&junction, 0.20, 0.80)
            .abs();

        assert!(
            i_on > i_off * 10.0,
            "Expected >10x gate modulation, I_on = {:.2e}, I_off = {:.2e}",
            i_on,
            i_off
        );
    }
}
