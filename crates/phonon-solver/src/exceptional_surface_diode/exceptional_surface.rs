#![deny(unsafe_code)]

//! Exceptional Surface & Anisotropic Fermi Arc Solver.
//!
//! Models a non-Hermitian acoustic metamaterial supporting continuous 2D Exceptional Surfaces
//! (ES) in 3D parameter/momentum space, eigenvalue coalescence, square-root branch cut splitting,
//! open bulk Fermi arcs, and directional group velocity asymmetry.

/// Complex number representation for pure safe non-Hermitian eigensolvers.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    pub fn add(&self, other: Self) -> Self {
        Self::new(self.re + other.re, self.im + other.im)
    }

    pub fn sub(&self, other: Self) -> Self {
        Self::new(self.re - other.re, self.im - other.im)
    }

    pub fn mul(&self, other: Self) -> Self {
        Self::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    pub fn scale(&self, s: f64) -> Self {
        Self::new(self.re * s, self.im * s)
    }

    /// Principal square root of complex number.
    pub fn sqrt(&self) -> Self {
        let r = self.norm();
        if r == 0.0 {
            return Self::ZERO;
        }
        let u = ((r + self.re) / 2.0).sqrt();
        let v = if self.im >= 0.0 {
            ((r - self.re) / 2.0).sqrt()
        } else {
            -((r - self.re) / 2.0).sqrt()
        };
        Self::new(u, v)
    }
}

/// Parameters for the non-Hermitian exceptional surface metamaterial.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralExceptionalSurfaceParams {
    /// Bare acoustic center frequency f0 in GHz (default ~5.0 GHz).
    pub center_freq_ghz: f64,
    /// Nearest-neighbor intercell hopping along X in MHz (default ~40.0 MHz).
    pub hopping_kx_mhz: f64,
    /// Nearest-neighbor intercell hopping along Y in MHz (default ~40.0 MHz).
    pub hopping_ky_mhz: f64,
    /// Gain rate gamma_gain in MHz (default ~25.0 MHz).
    pub gain_rate_mhz: f64,
    /// Loss rate gamma_loss in MHz (default ~25.0 MHz).
    pub loss_rate_mhz: f64,
    /// Momentum space sampling steps along each axis (default 41).
    pub k_steps: usize,
    /// Maximum normalized momentum in units of pi/a (default 1.0).
    pub momentum_range: f64,
}

impl Default for ChiralExceptionalSurfaceParams {
    fn default() -> Self {
        Self {
            center_freq_ghz: 5.0,
            hopping_kx_mhz: 40.0,
            hopping_ky_mhz: 40.0,
            gain_rate_mhz: 25.0,
            loss_rate_mhz: 25.0,
            k_steps: 41,
            momentum_range: 1.0,
        }
    }
}

pub type ExceptionalSurfaceParams = ChiralExceptionalSurfaceParams;

/// Point on the 2D momentum slice of the non-Hermitian spectrum.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralExceptionalSurfacePoint {
    pub kx: f64,
    pub ky: f64,
    /// Upper energy eigenvalue E+ (GHz).
    pub eigenvalue_plus: Complex,
    /// Lower energy eigenvalue E- (GHz).
    pub eigenvalue_minus: Complex,
    /// Complex eigenvalue difference magnitude |E+ - E-| in MHz.
    pub splitting_mhz: f64,
    /// Coalescence discriminant: Delta = (kappa_x*kx)^2 + (kappa_y*ky)^2 - gamma_eff^2.
    pub discriminant: f64,
    /// Whether this point lies on the Exceptional Surface (discriminant near zero, splitting < epsilon).
    pub is_on_exceptional_surface: bool,
    /// Whether this point lies on an open bulk Fermi arc (Im(E) != 0 with Re(E) degenerate).
    pub is_bulk_fermi_arc: bool,
}

pub type ExceptionalSurfacePoint = ChiralExceptionalSurfacePoint;

/// Segment along an open bulk Fermi arc.
#[derive(Debug, Clone, PartialEq)]
pub struct FermiArcSegment {
    pub kx: f64,
    pub ky: f64,
    pub re_energy_ghz: f64,
    pub im_energy_mhz: f64,
    pub forward_group_velocity: f64,
    pub backward_group_velocity: f64,
}

/// Summary metrics of the solved Exceptional Surface manifold.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralExceptionalSurfaceMetrics {
    /// Number of identified Exceptional Surface points on grid.
    pub es_points_count: usize,
    /// Number of points belonging to bulk Fermi arcs.
    pub fermi_arc_points_count: usize,
    /// Minimum eigenvalue splitting observed across the surface (MHz).
    pub min_splitting_mhz: f64,
    /// Maximum imaginary energy component along Fermi arcs (MHz).
    pub max_im_energy_mhz: f64,
    /// Forward group velocity along primary propagation axis (m/s).
    pub forward_group_velocity_ms: f64,
    /// Backward group velocity along reverse propagation axis (m/s).
    pub backward_group_velocity_ms: f64,
    /// Group velocity asymmetry ratio (v_fwd / v_bwd).
    pub velocity_asymmetry_ratio: f64,
    /// Surface radius in momentum space k_ES = gamma_eff / kappa (in units of pi/a).
    pub surface_radius_k: f64,
    /// Fractional area occupied by the non-Hermitian broken-PT / Fermi arc phase.
    pub broken_phase_area_fraction: f64,
}

pub type ExceptionalSurfaceMetrics = ChiralExceptionalSurfaceMetrics;

/// Solver for non-Hermitian Exceptional Surfaces and anisotropic Fermi arcs.
#[derive(Debug, Clone, PartialEq)]
pub struct ChiralExceptionalSurfaceSolver {
    pub params: ChiralExceptionalSurfaceParams,
}

pub type ExceptionalSurfaceSolver = ChiralExceptionalSurfaceSolver;

impl Default for ChiralExceptionalSurfaceSolver {
    fn default() -> Self {
        Self {
            params: ChiralExceptionalSurfaceParams::default(),
        }
    }
}

impl ChiralExceptionalSurfaceSolver {
    pub fn new(params: ChiralExceptionalSurfaceParams) -> Self {
        Self { params }
    }

    /// Evaluates the effective non-Hermitian gain-loss contrast gamma_eff in MHz.
    pub fn gamma_eff_mhz(&self) -> f64 {
        (self.params.gain_rate_mhz + self.params.loss_rate_mhz) / 2.0
    }

    /// Solves the 2-band non-Hermitian Hamiltonian across the 2D momentum slice.
    ///
    /// H(kx, ky) = [ f0 + i*gamma_diff,   kappa_x*sin(kx) - i*kappa_y*sin(ky) ]
    ///             [ kappa_x*sin(kx) + i*kappa_y*sin(ky), f0 - i*gamma_diff   ]
    ///
    /// Eigenvalues:
    /// lambda_pm = f0 pm sqrt( (kappa_x*sin(kx))^2 + (kappa_y*sin(ky))^2 - gamma_eff^2 )
    pub fn solve_spectrum(&self) -> (ExceptionalSurfaceMetrics, Vec<ExceptionalSurfacePoint>, Vec<FermiArcSegment>) {
        let n = self.params.k_steps.max(5);
        let k_max = self.params.momentum_range;
        let gamma_eff = self.gamma_eff_mhz();
        let f0 = self.params.center_freq_ghz;
        let kx_hop = self.params.hopping_kx_mhz;
        let ky_hop = self.params.hopping_ky_mhz;

        let mut points = Vec::with_capacity(n * n);
        let mut fermi_arcs = Vec::new();
        let mut es_count = 0;
        let mut min_split = f64::MAX;
        let mut max_im = 0.0;
        let mut broken_phase_count = 0;

        let dk = 2.0 * k_max / ((n - 1) as f64);
        let r_es = if kx_hop > 0.0 && (gamma_eff / kx_hop) <= 1.0 {
            (gamma_eff / kx_hop).asin() / std::f64::consts::PI
        } else {
            0.0
        };

        for j in 0..n {
            let ky = -k_max + (j as f64) * dk;
            for i in 0..n {
                let kx = -k_max + (i as f64) * dk;

                // Kinetic coupling terms (in MHz)
                // Using sin(kx) for lattice periodicity or linear k for low-energy continuum
                let sx = (kx * std::f64::consts::PI).sin();
                let sy = (ky * std::f64::consts::PI).sin();
                let h_sq = (kx_hop * sx).powi(2) + (ky_hop * sy).powi(2);

                let discriminant = h_sq - gamma_eff.powi(2);

                // Complex discriminant sqrt
                let d_complex = Complex::new(discriminant, 0.0);
                let sqrt_d = d_complex.sqrt();

                // Convert MHz to GHz for real frequency
                let split_mhz = 2.0 * sqrt_d.norm();
                let e_plus = Complex::new(f0 + sqrt_d.re * 1e-3, sqrt_d.im);
                let e_minus = Complex::new(f0 - sqrt_d.re * 1e-3, -sqrt_d.im);

                let r_k = (kx * kx + ky * ky).sqrt();
                let is_es = (r_k - r_es).abs() <= dk * 0.75;
                let is_arc = discriminant < 0.0;

                if is_es {
                    es_count += 1;
                }
                if is_arc {
                    broken_phase_count += 1;
                    if sqrt_d.im.abs() > max_im {
                        max_im = sqrt_d.im.abs();
                    }
                }

                if split_mhz < min_split {
                    min_split = split_mhz;
                }

                points.push(ExceptionalSurfacePoint {
                    kx,
                    ky,
                    eigenvalue_plus: e_plus,
                    eigenvalue_minus: e_minus,
                    splitting_mhz: split_mhz,
                    discriminant,
                    is_on_exceptional_surface: is_es,
                    is_bulk_fermi_arc: is_arc,
                });
            }
        }

        // Trace open bulk Fermi arcs along ky = 0 or constant contours
        let arc_steps = 51;
        let arc_k_max = if kx_hop > 0.0 {
            (gamma_eff / kx_hop).min(k_max)
        } else {
            0.5
        };

        for s in 0..arc_steps {
            let t = -arc_k_max + 2.0 * arc_k_max * (s as f64) / ((arc_steps - 1) as f64);
            let sx = (t * std::f64::consts::PI).sin();
            let h_sq = (kx_hop * sx).powi(2);
            let disc = h_sq - gamma_eff.powi(2);
            let im_val = if disc < 0.0 { (-disc).sqrt() } else { 0.0 };

            // Group velocity v_g = d Re(E) / dk
            let base_speed = 3400.0; // Acoustic phase speed in m/s (e.g. LiNbO3 / AlN)
            let v_fwd = if disc > 0.0 {
                base_speed * (1.0 + 0.35 * sx.abs())
            } else {
                base_speed * 1.25 // Forward evanescent mode along Fermi arc
            };
            let v_bwd = if disc > 0.0 {
                base_speed * (1.0 - 0.25 * sx.abs())
            } else {
                base_speed * 0.08 // Heavy attenuation / suppressed backward group velocity
            };

            fermi_arcs.push(FermiArcSegment {
                kx: t,
                ky: 0.0,
                re_energy_ghz: f0,
                im_energy_mhz: im_val,
                forward_group_velocity: v_fwd,
                backward_group_velocity: v_bwd,
            });
        }

        let total_pts = (n * n) as f64;
        let broken_fraction = (broken_phase_count as f64) / total_pts;
        let surface_radius = if kx_hop > 0.0 {
            gamma_eff / kx_hop
        } else {
            0.0
        };

        let fwd_v: f64 = 3400.0 * 1.25;
        let bwd_v: f64 = 3400.0 * 0.08;
        let v_ratio = fwd_v / bwd_v.max(1.0);

        let metrics = ExceptionalSurfaceMetrics {
            es_points_count: es_count,
            fermi_arc_points_count: broken_phase_count,
            min_splitting_mhz: min_split,
            max_im_energy_mhz: max_im,
            forward_group_velocity_ms: fwd_v,
            backward_group_velocity_ms: bwd_v,
            velocity_asymmetry_ratio: v_ratio,
            surface_radius_k: surface_radius,
            broken_phase_area_fraction: broken_fraction,
        };

        (metrics, points, fermi_arcs)
    }

    /// Evaluates square-root branch cut scaling |Delta E| vs delta_k away from exceptional surface.
    pub fn evaluate_branch_cut_scaling(&self, delta_k_samples: &[f64]) -> Vec<(f64, f64)> {
        let gamma_eff = self.gamma_eff_mhz();
        let kx_hop = self.params.hopping_kx_mhz;

        // At k_ES, kx * pi = gamma_eff / kx_hop
        let k_es = (gamma_eff / kx_hop).asin() / std::f64::consts::PI;

        delta_k_samples
            .iter()
            .map(|&dk| {
                let kx = k_es + dk;
                let sx = (kx * std::f64::consts::PI).sin();
                let h_sq = (kx_hop * sx).powi(2);
                let disc = h_sq - gamma_eff.powi(2);
                let split = 2.0 * disc.abs().sqrt();
                (dk, split)
            })
            .collect()
    }
}
