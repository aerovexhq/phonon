#![deny(unsafe_code)]

//! Topological Acoustic Bound States in the Continuum (BIC) lattice solver.
//!
//! Models 2D acoustic metamaterial lattices hosting:
//! - Symmetry-protected BICs at the Gamma-point (k = 0) with vanishing radiation coupling.
//! - Friedrich-Wintgen interference BICs at off-Gamma momentum points.
//! - Topological polarization vortices in momentum space carrying integer topological charge q = +-1, +-2.
//! - Quasi-BIC transition with inverse-quadratic quality factor scaling Q proportional to 1 / alpha^2.

use std::f64::consts::PI;

/// Type and classification of the acoustic Bound State in the Continuum (BIC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BicKind {
    /// Symmetry-protected BIC at the high-symmetry Gamma point (k = 0) with topological charge q = +1.
    SymmetryProtectedGamma,
    /// Off-Gamma Friedrich-Wintgen interference BIC formed by destructive channel interference with q = -1.
    FriedrichWintgenOffGamma,
    /// Higher-order symmetry-protected vortex BIC with topological charge q = +2.
    HigherOrderVortex,
}

/// Parameters for the acoustic BIC metamaterial lattice.
#[derive(Debug, Clone)]
pub struct BicLatticeParams {
    /// Nominal acoustic resonance frequency in Hz (default ~4000.0 Hz).
    pub resonance_freq_hz: f64,
    /// Speed of sound in the acoustic medium in m/s (default ~343.0 m/s).
    pub speed_of_sound_m_s: f64,
    /// Metamaterial unit cell lattice pitch a_0 in mm (default ~40.0 mm).
    pub lattice_pitch_mm: f64,
    /// Acoustic cavity radius in mm (default ~12.0 mm).
    pub cavity_radius_mm: f64,
    /// Classification of the target BIC mode.
    pub bic_kind: BicKind,
    /// Inversion or mirror asymmetry parameter alpha in [0.0, 0.3] (default 0.0 for pure BIC).
    pub asymmetry_parameter: f64,
    /// Non-radiative intrinsic dissipative loss rate in Hz (default ~0.5 Hz).
    pub intrinsic_loss_hz: f64,
    /// Radiative coupling coefficient C_rad in Hz.
    pub radiative_coupling_coeff_hz: f64,
}

impl Default for BicLatticeParams {
    fn default() -> Self {
        Self {
            resonance_freq_hz: 4000.0,
            speed_of_sound_m_s: 343.0,
            lattice_pitch_mm: 40.0,
            cavity_radius_mm: 12.0,
            bic_kind: BicKind::SymmetryProtectedGamma,
            asymmetry_parameter: 0.0,
            intrinsic_loss_hz: 0.5,
            radiative_coupling_coeff_hz: 250.0,
        }
    }
}

/// Far-field acoustic polarization vector and radiation metrics at momentum k.
#[derive(Debug, Clone)]
pub struct FarFieldPolarizationVector {
    /// Normalized wavevector k_x / (pi / a_0) in [-1.0, 1.0].
    pub kx_norm: f64,
    /// Normalized wavevector k_y / (pi / a_0) in [-1.0, 1.0].
    pub ky_norm: f64,
    /// Far-field acoustic velocity projection component v_x.
    pub vx: f64,
    /// Far-field acoustic velocity projection component v_y.
    pub vy: f64,
    /// Polarization orientation angle phi in [-pi, pi].
    pub orientation_angle_rad: f64,
    /// Radiative decay linewidth gamma_rad in Hz.
    pub radiative_linewidth_hz: f64,
    /// Total loaded quality factor Q = omega_0 / (2 * (gamma_rad + gamma_nr)).
    pub quality_factor: f64,
}

/// Solver for topological acoustic BICs and momentum-space polarization vortex fields.
#[derive(Debug, Clone)]
pub struct BicLatticeSolver {
    pub params: BicLatticeParams,
    /// Grid of far-field polarization vectors in momentum space.
    pub polarization_field: Vec<FarFieldPolarizationVector>,
    /// Calculated topological charge q = (1 / 2*pi) oint grad(phi) . dk.
    pub calculated_topological_charge: i32,
    /// Diverging theoretical radiation quality factor at the BIC singularity.
    pub theoretical_q_bic: f64,
    /// Quality factor at current asymmetry parameter.
    pub current_q_factor: f64,
    /// Center BIC singularity momentum position (kx_norm, ky_norm).
    pub bic_singularity_pos: (f64, f64),
}

impl BicLatticeSolver {
    /// Construct a new BIC lattice solver and compute initial momentum fields.
    pub fn new(params: BicLatticeParams) -> Self {
        let mut solver = Self {
            params,
            polarization_field: Vec::new(),
            calculated_topological_charge: 0,
            theoretical_q_bic: 1e9,
            current_q_factor: 1e9,
            bic_singularity_pos: (0.0, 0.0),
        };
        solver.recompute();
        solver
    }

    /// Recompute polarization vector fields, topological charge, and quality factor.
    pub fn recompute(&mut self) {
        let alpha = self.params.asymmetry_parameter.clamp(0.0, 0.5);
        let f0 = self.params.resonance_freq_hz;
        let c_rad = self.params.radiative_coupling_coeff_hz;
        let gamma_nr = self.params.intrinsic_loss_hz.max(1e-6);

        // Identify BIC center position in normalized momentum space
        let (k_bic_x, k_bic_y) = match self.params.bic_kind {
            BicKind::SymmetryProtectedGamma => (0.0, 0.0),
            BicKind::FriedrichWintgenOffGamma => (0.35, 0.0),
            BicKind::HigherOrderVortex => (0.0, 0.0),
        };
        self.bic_singularity_pos = (k_bic_x, k_bic_y);

        // Generate 2D momentum grid in k_norm in [-0.8, 0.8]
        let grid_size = 21;
        let k_extent = 0.8;
        let mut field = Vec::with_capacity(grid_size * grid_size);

        for j in 0..grid_size {
            let ky = -k_extent + 2.0 * k_extent * (j as f64) / (grid_size - 1) as f64;
            for i in 0..grid_size {
                let kx = -k_extent + 2.0 * k_extent * (i as f64) / (grid_size - 1) as f64;

                let dkx = kx - k_bic_x;
                let dky = ky - k_bic_y;
                let k_dist = (dkx * dkx + dky * dky).sqrt();

                // Far-field polarization vector (vx, vy) vortex winding around singularity:
                let (vx, vy) = match self.params.bic_kind {
                    BicKind::SymmetryProtectedGamma => {
                        // Topological charge q = +1: (vx, vy) ~ (-dky, dkx) (counter-clockwise vortex)
                        let raw_vx = -dky;
                        let raw_vy = dkx;
                        let norm = (raw_vx * raw_vx + raw_vy * raw_vy).sqrt().max(1e-9);
                        (raw_vx / norm, raw_vy / norm)
                    }
                    BicKind::FriedrichWintgenOffGamma => {
                        // Topological charge q = -1: (vx, vy) ~ (dky, dkx) (hyperbolic / anti-vortex)
                        let raw_vx = dky;
                        let raw_vy = dkx;
                        let norm = (raw_vx * raw_vx + raw_vy * raw_vy).sqrt().max(1e-9);
                        (raw_vx / norm, raw_vy / norm)
                    }
                    BicKind::HigherOrderVortex => {
                        // Topological charge q = +2: angle winds twice 2*theta
                        let theta = dky.atan2(dkx);
                        let angle2 = 2.0 * theta;
                        (angle2.cos(), angle2.sin())
                    }
                };

                let orientation = vy.atan2(vx);

                // Radiative linewidth:
                // For a pure BIC (alpha = 0), gamma_rad -> 0 at k = k_bic as k_dist^2.
                // With broken symmetry (alpha > 0), a finite leakage floor is opened: gamma_asym ~ c_rad * alpha^2.
                let gamma_rad_sym = c_rad * k_dist * k_dist;
                let gamma_rad_asym = c_rad * alpha * alpha;
                let gamma_rad = gamma_rad_sym + gamma_rad_asym;

                let q_total = f0 / (2.0 * (gamma_rad + gamma_nr));

                field.push(FarFieldPolarizationVector {
                    kx_norm: kx,
                    ky_norm: ky,
                    vx,
                    vy,
                    orientation_angle_rad: orientation,
                    radiative_linewidth_hz: gamma_rad,
                    quality_factor: q_total,
                });
            }
        }
        self.polarization_field = field;

        // Numerical integration of topological charge around small loop enclosing BIC:
        // q = (1 / 2*pi) sum Delta phi
        let num_loop_pts = 32;
        let loop_r = 0.20;
        let mut angle_prev = 0.0;
        let mut total_angle_diff = 0.0;

        for step in 0..=num_loop_pts {
            let t = 2.0 * PI * (step as f64) / (num_loop_pts as f64);
            let kx = k_bic_x + loop_r * t.cos();
            let ky = k_bic_y + loop_r * t.sin();

            let dkx = kx - k_bic_x;
            let dky = ky - k_bic_y;

            let (vx, vy) = match self.params.bic_kind {
                BicKind::SymmetryProtectedGamma => (-dky, dkx),
                BicKind::FriedrichWintgenOffGamma => (dky, dkx),
                BicKind::HigherOrderVortex => {
                    let theta = dky.atan2(dkx);
                    let angle2 = 2.0 * theta;
                    (angle2.cos(), angle2.sin())
                }
            };

            let angle = vy.atan2(vx);
            if step > 0 {
                let mut diff = angle - angle_prev;
                while diff > PI {
                    diff -= 2.0 * PI;
                }
                while diff < -PI {
                    diff += 2.0 * PI;
                }
                total_angle_diff += diff;
            }
            angle_prev = angle;
        }

        let q_computed = (total_angle_diff / (2.0 * PI)).round() as i32;
        self.calculated_topological_charge = q_computed;

        // Diverging theoretical quality factor at k = k_bic:
        // When alpha = 0: gamma_rad = 0, so Q_rad -> infinity, Q_loaded = f0 / (2 * gamma_nr)
        // With alpha > 0: Q(alpha) = f0 / (2 * (c_rad * alpha^2 + gamma_nr))
        self.theoretical_q_bic = f0 / (2.0 * gamma_nr);
        let gamma_current_at_bic = c_rad * alpha * alpha + gamma_nr;
        self.current_q_factor = f0 / (2.0 * gamma_current_at_bic);
    }
}
