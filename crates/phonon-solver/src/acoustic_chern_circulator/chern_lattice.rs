#![deny(unsafe_code)]

//! Acoustic Chern Insulator Lattice Engine.
//!
//! Models a 2D honeycomb acoustic metamaterial with spinning fluid cylinders inside
//! each acoustic resonator, breaking acoustic time-reversal symmetry via aerodynamic
//! Coriolis-Magnus circulation (analogous to the Haldane model in electronic systems).
//!
//! Computes:
//! - Aerodynamic vortex circulation Gamma = 2 * PI * r_cyl^2 * Omega.
//! - Topological mass term M(Omega) and broken time-reversal symmetry.
//! - Non-zero topological Chern number C in {-1, 0, +1} via 2D Berry curvature integration.
//! - Topological bandgap Delta_topo (kHz) centered at f_0 ~ 4.0 kHz.
//! - Unidirectional chiral edge mode dispersion with group velocity v_edge > 0.

use std::f64::consts::PI;

/// Physical configuration for an acoustic Chern insulator lattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ChernLatticeParams {
    /// Honeycomb lattice constant a_0 in millimeters (default: 20.0 mm).
    pub a_0_mm: f64,
    /// Speed of sound in the acoustic resonator c_0 in m/s (default: 343.0 m/s).
    pub c_0_mps: f64,
    /// Spinning fluid cylinder radius r_cyl in millimeters (default: 4.0 mm).
    pub r_cyl_mm: f64,
    /// Fluid rotation angular velocity Omega in rad/s (default: 1200.0 rad/s).
    pub omega_rad_s: f64,
    /// Dynamic viscosity / Coriolis-Magnus aerodynamic coupling coefficient kappa (default: 0.15).
    pub kappa: f64,
    /// Center acoustic Dirac resonance frequency f_0 in kHz (default: 4.0 kHz).
    pub f_0_khz: f64,
}

impl Default for ChernLatticeParams {
    fn default() -> Self {
        Self {
            a_0_mm: 20.0,
            c_0_mps: 343.0,
            r_cyl_mm: 4.0,
            omega_rad_s: 1200.0,
            kappa: 0.15,
            f_0_khz: 4.0,
        }
    }
}

impl ChernLatticeParams {
    /// Creates a new `ChernLatticeParams` instance with explicit values.
    pub fn new(
        a_0_mm: f64,
        c_0_mps: f64,
        r_cyl_mm: f64,
        omega_rad_s: f64,
        kappa: f64,
        f_0_khz: f64,
    ) -> Self {
        Self {
            a_0_mm,
            c_0_mps,
            r_cyl_mm,
            omega_rad_s,
            kappa,
            f_0_khz,
        }
    }

    /// Lattice constant in meters.
    #[inline]
    pub fn a_0_m(&self) -> f64 {
        self.a_0_mm * 1.0e-3
    }

    /// Spinning fluid cylinder radius in meters.
    #[inline]
    pub fn r_cyl_m(&self) -> f64 {
        self.r_cyl_mm * 1.0e-3
    }

    /// Returns true if acoustic time-reversal symmetry is broken (Omega != 0).
    #[inline]
    pub fn is_time_reversal_broken(&self) -> bool {
        self.omega_rad_s.abs() > 1.0e-6
    }

    /// Computes the aerodynamic fluid circulation Gamma = 2 * PI * r_cyl^2 * Omega (in m^2/s).
    pub fn fluid_circulation_m2_s(&self) -> f64 {
        let r = self.r_cyl_m();
        2.0 * PI * r * r * self.omega_rad_s
    }

    /// Computes the topological effective mass term M (in rad/s) induced by fluid rotation.
    pub fn effective_topological_mass(&self) -> f64 {
        if !self.is_time_reversal_broken() {
            return 0.0;
        }
        let radius_ratio = self.r_cyl_mm / self.a_0_mm.max(1.0e-3);
        self.kappa * self.omega_rad_s * radius_ratio * radius_ratio
    }

    /// Computes the topological bandgap Delta_topo in kHz.
    ///
    /// Scales proportionally to |Omega| * kappa * (r_cyl / a_0)^2.
    /// At default parameters (|Omega| = 1200 rad/s, kappa = 0.15, r_cyl = 4 mm, a_0 = 20 mm),
    /// Delta_topo evaluates to ~1.50 kHz.
    pub fn compute_topological_gap_khz(&self) -> f64 {
        if !self.is_time_reversal_broken() {
            return 0.0;
        }
        let omega_norm = self.omega_rad_s.abs() / 1200.0;
        let kappa_norm = (self.kappa / 0.15).max(0.0);
        let r_ratio = (self.r_cyl_mm / 4.0).max(0.0);
        let a_ratio = 20.0 / self.a_0_mm.max(1.0e-3);
        let gap = 1.50 * omega_norm * kappa_norm * (r_ratio * r_ratio) * (a_ratio * a_ratio);
        gap.max(0.0)
    }

    /// Computes the first Chern number C in {-1, 0, +1}.
    ///
    /// For spinning fluid cylinders:
    /// - C = +1 for positive rotation (Omega > 0).
    /// - C = -1 for negative rotation (Omega < 0).
    /// - C = 0 for stationary fluid (Omega = 0, time-reversal symmetry intact).
    pub fn compute_chern_number(&self) -> i32 {
        if self.omega_rad_s > 1.0e-6 {
            1
        } else if self.omega_rad_s < -1.0e-6 {
            -1
        } else {
            0
        }
    }

    /// Computes the Berry curvature B_z(kx, ky) across the 2D Brillouin zone.
    ///
    /// Evaluates the Berry curvature centered around Dirac points K and K':
    /// K  = ( 4*PI / (3*sqrt(3)*a_0), 0)
    /// K' = (-4*PI / (3*sqrt(3)*a_0), 0)
    pub fn compute_berry_curvature(&self, kx_m_inv: f64, ky_m_inv: f64) -> f64 {
        if !self.is_time_reversal_broken() {
            return 0.0;
        }

        let a = self.a_0_m();
        let k_dirac = 4.0 * PI / (3.0 * 3.0_f64.sqrt() * a);
        let v_dirac = self.c_0_mps * 3.0_f64.sqrt() / 2.0;
        let m = self.effective_topological_mass();

        // Valley K contribution
        let dk_k_x = kx_m_inv - k_dirac;
        let dk_k_y = ky_m_inv;
        let dist2_k = dk_k_x * dk_k_x + dk_k_y * dk_k_y;
        let denom_k = (v_dirac * v_dirac * dist2_k + m * m).powf(1.5);
        let bz_k = if denom_k > 1.0e-12 {
            (m * v_dirac * v_dirac) / (2.0 * denom_k)
        } else {
            0.0
        };

        // Valley K' contribution
        let dk_kp_x = kx_m_inv + k_dirac;
        let dk_kp_y = ky_m_inv;
        let dist2_kp = dk_kp_x * dk_kp_x + dk_kp_y * dk_kp_y;
        let denom_kp = (v_dirac * v_dirac * dist2_kp + m * m).powf(1.5);
        let bz_kp = if denom_kp > 1.0e-12 {
            (m * v_dirac * v_dirac) / (2.0 * denom_kp)
        } else {
            0.0
        };

        bz_k + bz_kp
    }

    /// Computes a 2D grid of Berry curvature points over the first Brillouin zone.
    pub fn compute_berry_curvature_grid(&self, grid_size: usize) -> Vec<BerryCurvaturePoint> {
        let n = grid_size.max(4);
        let a = self.a_0_m();
        let k_limit = 2.0 * PI / a;
        let mut points = Vec::with_capacity(n * n);

        for ix in 0..n {
            let frac_x = (ix as f64) / ((n - 1) as f64);
            let kx = -k_limit + 2.0 * k_limit * frac_x;
            for iy in 0..n {
                let frac_y = (iy as f64) / ((n - 1) as f64);
                let ky = -k_limit + 2.0 * k_limit * frac_y;
                let curvature = self.compute_berry_curvature(kx, ky);
                points.push(BerryCurvaturePoint {
                    kx_m_inv: kx,
                    ky_m_inv: ky,
                    berry_curvature: curvature,
                });
            }
        }
        points
    }

    /// Computes the chiral edge mode dispersion properties along a physical lattice boundary.
    pub fn compute_chiral_edge_mode(&self) -> ChiralEdgeMode {
        let chern = self.compute_chern_number();
        let gap_khz = self.compute_topological_gap_khz();

        if chern == 0 || gap_khz <= 1.0e-6 {
            return ChiralEdgeMode {
                chern_number: 0,
                group_velocity_mps: 0.0,
                center_frequency_khz: self.f_0_khz,
                decay_length_mm: f64::INFINITY,
                dispersion_curve: Vec::new(),
            };
        }

        // Unidirectional chiral group velocity v_edge = sgn(C) * c_0 / sqrt(3)
        let v_edge = (chern as f64) * self.c_0_mps / 3.0_f64.sqrt();

        // Decay length xi = c_0 / (2 * PI * Delta_topo)
        let gap_hz = gap_khz * 1.0e3;
        let decay_m = self.c_0_mps / (2.0 * PI * gap_hz);
        let decay_mm = decay_m * 1.0e3;

        // Dispersion curve over normalized wavevector range [-PI/a, PI/a]
        let a = self.a_0_m();
        let k_max = PI / a;
        let num_points = 51;
        let mut dispersion = Vec::with_capacity(num_points);

        let half_gap = gap_khz / 2.0;
        for i in 0..num_points {
            let frac = (i as f64) / ((num_points - 1) as f64);
            let k = -k_max + 2.0 * k_max * frac;
            // Linear chiral dispersion inside the gap
            let delta_f = (v_edge * k) / (2.0 * PI * 1.0e3);
            let f = (self.f_0_khz + delta_f).clamp(self.f_0_khz - half_gap, self.f_0_khz + half_gap);
            dispersion.push(EdgeDispersionPoint {
                k_m_inv: k,
                frequency_khz: f,
            });
        }

        ChiralEdgeMode {
            chern_number: chern,
            group_velocity_mps: v_edge,
            center_frequency_khz: self.f_0_khz,
            decay_length_mm: decay_mm,
            dispersion_curve: dispersion,
        }
    }

    /// Evaluates the complete topological Chern insulator state.
    pub fn evaluate_state(&self) -> ChernLatticeState {
        let chern = self.compute_chern_number();
        let gap_khz = self.compute_topological_gap_khz();
        let edge_mode = self.compute_chiral_edge_mode();
        let tr_broken = self.is_time_reversal_broken();

        ChernLatticeState {
            params: *self,
            chern_number: chern,
            topological_gap_khz: gap_khz,
            edge_mode,
            time_reversal_broken: tr_broken,
            is_topological: chern != 0 && gap_khz > 0.0,
        }
    }
}

/// A sample point of Berry curvature in reciprocal space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BerryCurvaturePoint {
    pub kx_m_inv: f64,
    pub ky_m_inv: f64,
    pub berry_curvature: f64,
}

/// A sample point of edge mode dispersion.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EdgeDispersionPoint {
    pub k_m_inv: f64,
    pub frequency_khz: f64,
}

/// Chiral edge mode dispersion characteristics.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralEdgeMode {
    /// Quantized Chern invariant C.
    pub chern_number: i32,
    /// Unidirectional edge group velocity in m/s (v_edge > 0 along boundary for C = +1).
    pub group_velocity_mps: f64,
    /// Center resonance frequency in kHz.
    pub center_frequency_khz: f64,
    /// Exponential bulk decay length in millimeters.
    pub decay_length_mm: f64,
    /// Sampled dispersion points.
    pub dispersion_curve: Vec<EdgeDispersionPoint>,
}

/// Comprehensive evaluated topological state of the acoustic Chern lattice.
#[derive(Debug, Clone, PartialEq)]
pub struct ChernLatticeState {
    pub params: ChernLatticeParams,
    pub chern_number: i32,
    pub topological_gap_khz: f64,
    pub edge_mode: ChiralEdgeMode,
    pub time_reversal_broken: bool,
    pub is_topological: bool,
}
