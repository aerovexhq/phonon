#![deny(unsafe_code)]

//! Atomic Boundary Step and Potential Barrier Quantum Reflection Engine.
//!
//! Evaluates wavepacket tunneling transmission $T$, boundary reflection $R$,
//! and de Broglie interference fringes during barrier scattering.

use super::schroedinger_stepper::{SchroedingerStepper, ELEMENTARY_CHARGE, HBAR};

/// Geometry shape of the scattering boundary potential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BarrierShape {
    Rectangular,
    Step,
    Delta,
    Triangular,
}

/// Physical parameters for a localized potential barrier or atomic step.
#[derive(Debug, Clone, PartialEq)]
pub struct PotentialBarrier {
    /// Spatial position of the barrier center in nanometers (e.g. 60.0 nm).
    pub barrier_x_nm: f64,
    /// Peak potential barrier height in electron-volts (e.g. 0.20 eV).
    pub barrier_height_ev: f64,
    /// Spatial barrier thickness/width in nanometers (e.g. 2.0 nm).
    pub barrier_width_nm: f64,
    /// Geometric profile shape.
    pub barrier_shape: BarrierShape,
}

impl Default for PotentialBarrier {
    fn default() -> Self {
        Self {
            barrier_x_nm: 60.0,
            barrier_height_ev: 0.20,
            barrier_width_nm: 2.5,
            barrier_shape: BarrierShape::Rectangular,
        }
    }
}

impl PotentialBarrier {
    /// Evaluates the barrier potential V(x) across a grid of spatial coordinates in nanometers.
    pub fn potential_at(&self, x_coords_nm: &[f64]) -> Vec<f64> {
        let x0 = self.barrier_x_nm;
        let w = self.barrier_width_nm;
        let v0 = self.barrier_height_ev;

        let mut v_grid = Vec::with_capacity(x_coords_nm.len());
        for &x in x_coords_nm {
            let pot = match self.barrier_shape {
                BarrierShape::Rectangular => {
                    if (x - x0).abs() <= w * 0.5 {
                        v0
                    } else {
                        0.0
                    }
                }
                BarrierShape::Step => {
                    if x >= x0 {
                        v0
                    } else {
                        0.0
                    }
                }
                BarrierShape::Delta => {
                    let diff = x - x0;
                    let sigma = (w * 0.25).max(0.1);
                    v0 * (-diff * diff / (2.0 * sigma * sigma)).exp()
                }
                BarrierShape::Triangular => {
                    let dist = (x - x0).abs();
                    if dist <= w * 0.5 {
                        v0 * (1.0 - dist / (w * 0.5))
                    } else {
                        0.0
                    }
                }
            };
            v_grid.push(pot);
        }
        v_grid
    }

    /// Evaluates the analytical transmission coefficient T_ana for an incident plane wave.
    pub fn analytical_transmission(
        &self,
        energy_ev: f64,
        effective_mass_ratio: f64,
    ) -> (f64, f64) {
        let v0 = self.barrier_height_ev;
        let m_eff = effective_mass_ratio * super::schroedinger_stepper::ELECTRON_MASS_KG;
        let d_m = self.barrier_width_nm * 1e-9;

        if energy_ev <= 1e-6 {
            return (0.0, 1.0);
        }

        let e_j = energy_ev * ELEMENTARY_CHARGE;
        let v0_j = v0 * ELEMENTARY_CHARGE;

        let t_ana = match self.barrier_shape {
            BarrierShape::Rectangular => {
                if (energy_ev - v0).abs() < 1e-6 {
                    let k0 = (2.0 * m_eff * e_j).sqrt() / HBAR;
                    1.0 / (1.0 + (m_eff * v0_j * d_m / (2.0 * HBAR * HBAR * k0)).powi(2))
                } else if energy_ev < v0 {
                    let kappa = (2.0 * m_eff * (v0_j - e_j)).sqrt() / HBAR;
                    let sinh_val = (kappa * d_m).sinh();
                    let denom = 1.0 + (v0 * v0 * sinh_val * sinh_val) / (4.0 * energy_ev * (v0 - energy_ev));
                    1.0 / denom
                } else {
                    let k_prime = (2.0 * m_eff * (e_j - v0_j)).sqrt() / HBAR;
                    let sin_val = (k_prime * d_m).sin();
                    let denom = 1.0 + (v0 * v0 * sin_val * sin_val) / (4.0 * energy_ev * (energy_ev - v0));
                    1.0 / denom
                }
            }
            BarrierShape::Step => {
                if energy_ev < v0 {
                    0.0
                } else {
                    let k1 = (2.0 * m_eff * e_j).sqrt();
                    let k2 = (2.0 * m_eff * (e_j - v0_j)).sqrt();
                    (4.0 * k1 * k2) / ((k1 + k2) * (k1 + k2))
                }
            }
            BarrierShape::Delta | BarrierShape::Triangular => {
                // WKB tunneling approximation
                if energy_ev < v0 {
                    let kappa = (2.0 * m_eff * (v0_j - e_j).max(0.0)).sqrt() / HBAR;
                    let eff_d = d_m * 0.6;
                    (-2.0 * kappa * eff_d).exp().min(1.0)
                } else {
                    1.0
                }
            }
        };

        let t_clamped = t_ana.clamp(0.0, 1.0);
        let r_clamped = (1.0 - t_clamped).clamp(0.0, 1.0);
        (t_clamped, r_clamped)
    }

    /// Evaluates integrated wavepacket transmission and reflection probabilities from numerical state.
    pub fn evaluate_wavepacket_transmission(
        &self,
        stepper: &SchroedingerStepper,
    ) -> BoundaryTransmissionResult {
        let n = stepper.params.grid_points;
        let x0 = self.barrier_x_nm;
        let dx_m = stepper.dx_m;

        let mut r_prob = 0.0;
        let mut t_prob = 0.0;

        for i in 0..n {
            let x_nm = stepper.x_coords_nm[i];
            let density = stepper.psi[i].norm_sq();
            if x_nm < x0 {
                r_prob += density * dx_m;
            } else {
                t_prob += density * dx_m;
            }
        }

        let total = r_prob + t_prob;
        let norm_r = if total > 1e-30 { r_prob / total } else { 1.0 };
        let norm_t = if total > 1e-30 { t_prob / total } else { 0.0 };

        let (ana_t, ana_r) = self.analytical_transmission(
            stepper.params.initial_energy_ev,
            stepper.params.effective_mass_ratio,
        );

        // de Broglie interference fringe wavelength lambda_e / 2
        let k0 = stepper.k0_rad_per_m.abs();
        let fringe_period_nm = if k0 > 1e-30 {
            (std::f64::consts::PI / k0) * 1e9
        } else {
            10.0
        };

        BoundaryTransmissionResult {
            numerical_transmission: norm_t,
            numerical_reflection: norm_r,
            analytical_transmission: ana_t,
            analytical_reflection: ana_r,
            barrier_center_nm: x0,
            barrier_height_ev: self.barrier_height_ev,
            fringe_period_nm,
            unitarity_residual: (norm_t + norm_r - 1.0).abs(),
        }
    }
}

/// Integrated result of wavepacket interaction with the boundary potential barrier.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundaryTransmissionResult {
    pub numerical_transmission: f64,
    pub numerical_reflection: f64,
    pub analytical_transmission: f64,
    pub analytical_reflection: f64,
    pub barrier_center_nm: f64,
    pub barrier_height_ev: f64,
    pub fringe_period_nm: f64,
    pub unitarity_residual: f64,
}
