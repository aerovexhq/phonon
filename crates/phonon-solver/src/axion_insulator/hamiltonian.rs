#![deny(unsafe_code)]

//! 3D Higher-Order Axion Insulator tight-binding Hamiltonian, quantized
//! magnetoelectric response, and Clifford algebra representations.
//!
//! Models the 3D topological axion insulator with quantized axion angle theta = pi,
//! bulk topological bandgap, half-quantized surface Hall conductance sigma_xy^surf = pm 1/2 (e^2/h),
//! and quantized magnetoelectric polarizability P_3 = 1/2 (mod 1).

use std::f64::consts::PI;

/// Physical parameters for the 3D higher-order axion insulator metamaterial.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionParams {
    /// Inter-cavity acoustic hopping amplitude t_hop in MHz (default: ~5.0 MHz).
    pub t_hop: f64,
    /// 3D Dirac mass parameter M_0 in MHz (default: ~10.0 MHz, with 0 < M_0 < 6*t for 3D topological regime).
    pub m_0: f64,
    /// Dirac acoustic velocity v_dirac in MHz*mm (default: ~15.0 MHz*mm).
    pub v_dirac: f64,
    /// Axion angle theta in radians (quantized to PI in topological axion insulator, 0.0 in trivial insulator).
    pub theta: f64,
    /// Surface time-reversal symmetry breaking mass Delta_surf in MHz (default: ~2.0 MHz).
    pub delta_surf: f64,
    /// Metamaterial unit cell lattice constant a_mm in mm (default: ~5.0 mm).
    pub a_mm: f64,
    /// Bare acoustic cavity resonance frequency omega_0 in GHz (default: ~1.0 GHz).
    pub omega_0: f64,
}

impl Default for AxionParams {
    fn default() -> Self {
        Self::topological()
    }
}

impl AxionParams {
    /// Creates a new custom AxionParams configuration.
    pub fn new(
        t_hop: f64,
        m_0: f64,
        v_dirac: f64,
        theta: f64,
        delta_surf: f64,
        a_mm: f64,
        omega_0: f64,
    ) -> Self {
        Self {
            t_hop,
            m_0,
            v_dirac,
            theta,
            delta_surf,
            a_mm,
            omega_0,
        }
    }

    /// Preset configuration for the 3D Topological Axion Insulator phase (theta = PI).
    pub fn topological() -> Self {
        Self {
            t_hop: 5.0,
            m_0: 10.0,
            v_dirac: 15.0,
            theta: PI,
            delta_surf: 2.0,
            a_mm: 5.0,
            omega_0: 1.0,
        }
    }

    /// Preset configuration for the Trivial Insulator phase (theta = 0.0).
    pub fn trivial() -> Self {
        Self {
            t_hop: 5.0,
            m_0: 35.0,
            v_dirac: 15.0,
            theta: 0.0,
            delta_surf: 0.0,
            a_mm: 5.0,
            omega_0: 1.0,
        }
    }

    /// Returns the effective Dirac velocity in frequency units (MHz): v_eff = v_dirac / a_mm.
    pub fn effective_velocity(&self) -> f64 {
        if self.a_mm.abs() > 1e-12 {
            self.v_dirac / self.a_mm
        } else {
            0.0
        }
    }

    /// Verifies if the system resides in the non-trivial 3D topological axion regime:
    /// requires theta approx PI (mod 2*pi) and 0 < M_0 < 6*t_hop.
    pub fn is_topological(&self) -> bool {
        let theta_mod = (self.theta % (2.0 * PI) + 2.0 * PI) % (2.0 * PI);
        let theta_is_pi = (theta_mod - PI).abs() < 0.25;
        let mass_in_regime = self.m_0 > 0.0 && self.m_0 < 6.0 * self.t_hop.abs();
        theta_is_pi && mass_in_regime
    }

    /// Quantized magnetoelectric polarizability P_3 (in units of e^2/h):
    /// Returns 0.5 when theta = PI (topological axion insulator) and 0.0 when theta = 0 (trivial).
    pub fn polarizability_p3(&self) -> f64 {
        if self.is_topological() {
            0.5
        } else {
            0.0
        }
    }

    /// Evaluates half-quantized surface Hall conductance sigma_xy^surf = pm 0.5 (in units of e^2/h)
    /// on opposing surfaces (e.g. top/bottom or x/y surfaces).
    pub fn surface_hall_conductance(&self) -> (f64, f64) {
        if self.is_topological() {
            (0.5, -0.5)
        } else {
            (0.0, 0.0)
        }
    }
}

/// Lightweight complex number for 4x4 Clifford algebra and tight-binding Hamiltonians.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxionComplex {
    pub re: f64,
    pub im: f64,
}

impl AxionComplex {
    pub const fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub const fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    pub const fn one() -> Self {
        Self { re: 1.0, im: 0.0 }
    }

    pub const fn i() -> Self {
        Self { re: 0.0, im: 1.0 }
    }

    pub fn conj(self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    pub fn norm_sq(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn abs(self) -> f64 {
        self.norm_sq().sqrt()
    }
}

impl std::ops::Add for AxionComplex {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl std::ops::Sub for AxionComplex {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl std::ops::Mul for AxionComplex {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}

impl std::ops::Mul<f64> for AxionComplex {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl std::ops::Mul<AxionComplex> for f64 {
    type Output = AxionComplex;
    fn mul(self, rhs: AxionComplex) -> AxionComplex {
        AxionComplex {
            re: self * rhs.re,
            im: self * rhs.im,
        }
    }
}

/// 4x4 Clifford algebra Gamma matrices satisfying {Gamma_i, Gamma_j} = 2 * delta_ij * I_4.
///
/// Representation:
/// Gamma_0 = tau_z (x) sigma_0
/// Gamma_1 = tau_x (x) sigma_x
/// Gamma_2 = tau_x (x) sigma_y
/// Gamma_3 = tau_x (x) sigma_z
/// Gamma_4 = tau_y (x) sigma_0 (anticommutes with Gamma_0..3)
pub struct CliffordGamma;

impl CliffordGamma {
    pub const fn gamma_0() -> [[AxionComplex; 4]; 4] {
        [
            [AxionComplex::one(), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero()],
            [AxionComplex::zero(), AxionComplex::one(), AxionComplex::zero(), AxionComplex::zero()],
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::new(-1.0, 0.0), AxionComplex::zero()],
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::new(-1.0, 0.0)],
        ]
    }

    pub const fn gamma_1() -> [[AxionComplex; 4]; 4] {
        [
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::one()],
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::one(), AxionComplex::zero()],
            [AxionComplex::zero(), AxionComplex::one(), AxionComplex::zero(), AxionComplex::zero()],
            [AxionComplex::one(), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero()],
        ]
    }

    pub const fn gamma_2() -> [[AxionComplex; 4]; 4] {
        [
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::new(0.0, -1.0)],
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::new(0.0, 1.0), AxionComplex::zero()],
            [AxionComplex::zero(), AxionComplex::new(0.0, -1.0), AxionComplex::zero(), AxionComplex::zero()],
            [AxionComplex::new(0.0, 1.0), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero()],
        ]
    }

    pub const fn gamma_3() -> [[AxionComplex; 4]; 4] {
        [
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::one(), AxionComplex::zero()],
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::new(-1.0, 0.0)],
            [AxionComplex::one(), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero()],
            [AxionComplex::zero(), AxionComplex::new(-1.0, 0.0), AxionComplex::zero(), AxionComplex::zero()],
        ]
    }

    pub const fn gamma_4() -> [[AxionComplex; 4]; 4] {
        [
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::new(0.0, -1.0), AxionComplex::zero()],
            [AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::new(0.0, -1.0)],
            [AxionComplex::new(0.0, 1.0), AxionComplex::zero(), AxionComplex::zero(), AxionComplex::zero()],
            [AxionComplex::zero(), AxionComplex::new(0.0, 1.0), AxionComplex::zero(), AxionComplex::zero()],
        ]
    }
}

/// 3D momentum-space 4-band Axion Hamiltonian engine.
#[derive(Debug, Clone, PartialEq)]
pub struct AxionHamiltonian {
    pub params: AxionParams,
}

impl AxionHamiltonian {
    /// Creates a new AxionHamiltonian solver instance.
    pub fn new(params: AxionParams) -> Self {
        Self { params }
    }

    /// Assembles the 4x4 Clifford Hamiltonian matrix H(kx, ky, kz) in momentum space:
    /// H(k) = (M_0 - t*(cos kx + cos ky + cos kz)) * Gamma_0 + v*(sin kx * Gamma_1 + sin ky * Gamma_2 + sin kz * Gamma_3)
    pub fn matrix_4x4(&self, kx: f64, ky: f64, kz: f64) -> [[AxionComplex; 4]; 4] {
        let m_k = self.params.m_0 - self.params.t_hop * (kx.cos() + ky.cos() + kz.cos());
        let v = self.params.effective_velocity();

        let g0 = CliffordGamma::gamma_0();
        let g1 = CliffordGamma::gamma_1();
        let g2 = CliffordGamma::gamma_2();
        let g3 = CliffordGamma::gamma_3();

        let s1 = kx.sin() * v;
        let s2 = ky.sin() * v;
        let s3 = kz.sin() * v;

        let mut h = [[AxionComplex::zero(); 4]; 4];
        for r in 0..4 {
            for c in 0..4 {
                h[r][c] = g0[r][c] * m_k + g1[r][c] * s1 + g2[r][c] * s2 + g3[r][c] * s3;
            }
        }
        h
    }

    /// Evaluates the 4 bulk eigenvalues E(kx, ky, kz) in ascending order.
    ///
    /// Due to the Clifford algebra {Gamma_a, Gamma_b} = 2*delta_ab, H(k)^2 is proportional
    /// to the identity matrix, giving exact pairwise degenerate eigenvalues:
    /// E_pm(k) = pm sqrt((M_0 - t*(cos kx + cos ky + cos kz))^2 + v^2*(sin^2 kx + sin^2 ky + sin^2 kz))
    pub fn eigenvalues(&self, kx: f64, ky: f64, kz: f64) -> [f64; 4] {
        let m_k = self.params.m_0 - self.params.t_hop * (kx.cos() + ky.cos() + kz.cos());
        let v = self.params.effective_velocity();
        let sin_sq = kx.sin().powi(2) + ky.sin().powi(2) + kz.sin().powi(2);
        let e = (m_k.powi(2) + v.powi(2) * sin_sq).sqrt();

        [-e, -e, e, e]
    }

    /// Evaluates the 3D bulk bandgap Delta_bulk in MHz:
    /// Delta_bulk = 2 * min_{k} E(k), determined by the minimum gap across the 8 TRIM points.
    pub fn bulk_bandgap(&self) -> f64 {
        let t = self.params.t_hop;
        let m = self.params.m_0;

        // TRIM gaps at Gamma, X, M, R:
        let g_gamma = (m - 3.0 * t).abs();
        let g_x = (m - t).abs();
        let g_m = (m + t).abs();
        let g_r = (m + 3.0 * t).abs();

        let mut min_val = g_gamma.min(g_x).min(g_m).min(g_r);
        if min_val < 0.0 {
            min_val = 0.0;
        }
        2.0 * min_val
    }

    /// Quantized magnetoelectric polarizability P_3:
    /// returns 0.5 for theta = PI (topological) and 0.0 for theta = 0 (trivial).
    pub fn polarizability_p3(&self) -> f64 {
        self.params.polarizability_p3()
    }

    /// Surface Hall conductance sigma_xy^surf:
    /// returns (+0.5, -0.5) for topological and (0.0, 0.0) for trivial.
    pub fn surface_hall_conductance(&self) -> (f64, f64) {
        self.params.surface_hall_conductance()
    }

    /// Computes the 3D bulk band dispersion along the high-symmetry path
    /// Gamma(0,0,0) -> X(pi,0,0) -> M(pi,pi,0) -> Gamma(0,0,0) -> Z(0,0,pi) -> R(pi,pi,pi).
    pub fn compute_bulk_dispersion(&self, steps_per_segment: usize) -> Vec<AxionBandPoint> {
        let n = steps_per_segment.max(5);
        let mut points = Vec::new();
        let mut dist = 0.0;

        for seg in 0..(AXION_HIGH_SYMMETRY_PATH.len() - 1) {
            let p0 = &AXION_HIGH_SYMMETRY_PATH[seg];
            let p1 = &AXION_HIGH_SYMMETRY_PATH[seg + 1];

            let dkx = p1.kx - p0.kx;
            let dky = p1.ky - p0.ky;
            let dkz = p1.kz - p0.kz;
            let seg_len = (dkx * dkx + dky * dky + dkz * dkz).sqrt();

            for s in 0..n {
                let frac = (s as f64) / (n as f64);
                let kx = p0.kx + frac * dkx;
                let ky = p0.ky + frac * dky;
                let kz = p0.kz + frac * dkz;
                let evals = self.eigenvalues(kx, ky, kz);

                let label = if s == 0 { Some(p0.label) } else { None };

                points.push(AxionBandPoint {
                    path_distance: dist + frac * seg_len,
                    point_label: label,
                    kx,
                    ky,
                    kz,
                    eigenvalues: evals,
                });
            }

            dist += seg_len;
        }

        // Add the final point
        if let Some(last) = AXION_HIGH_SYMMETRY_PATH.last() {
            let evals = self.eigenvalues(last.kx, last.ky, last.kz);
            points.push(AxionBandPoint {
                path_distance: dist,
                point_label: Some(last.label),
                kx: last.kx,
                ky: last.ky,
                kz: last.kz,
                eigenvalues: evals,
            });
        }

        points
    }
}

/// A point along the high-symmetry momentum path in the Brillouin Zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighSymmetryPoint {
    pub label: &'static str,
    pub kx: f64,
    pub ky: f64,
    pub kz: f64,
}

/// Standard high-symmetry points along Gamma -> X -> M -> Gamma -> Z -> R in the 3D simple cubic BZ.
pub const AXION_HIGH_SYMMETRY_PATH: [HighSymmetryPoint; 6] = [
    HighSymmetryPoint {
        label: "Gamma",
        kx: 0.0,
        ky: 0.0,
        kz: 0.0,
    },
    HighSymmetryPoint {
        label: "X",
        kx: PI,
        ky: 0.0,
        kz: 0.0,
    },
    HighSymmetryPoint {
        label: "M",
        kx: PI,
        ky: PI,
        kz: 0.0,
    },
    HighSymmetryPoint {
        label: "Gamma",
        kx: 0.0,
        ky: 0.0,
        kz: 0.0,
    },
    HighSymmetryPoint {
        label: "Z",
        kx: 0.0,
        ky: 0.0,
        kz: PI,
    },
    HighSymmetryPoint {
        label: "R",
        kx: PI,
        ky: PI,
        kz: PI,
    },
];

/// Band structure point along a 3D momentum path.
#[derive(Debug, Clone, PartialEq)]
pub struct AxionBandPoint {
    pub path_distance: f64,
    pub point_label: Option<&'static str>,
    pub kx: f64,
    pub ky: f64,
    pub kz: f64,
    pub eigenvalues: [f64; 4],
}
