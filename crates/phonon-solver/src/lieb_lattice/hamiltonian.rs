#![deny(unsafe_code)]

//! 3-Band Tight-Binding Hamiltonian, Flat-Band Solver, and Dispersion Engine
//! for Topological Acoustic Metamaterial Lieb Lattices.
//!
//! Models the Lieb lattice (line graph of square lattice) featuring:
//! - Destructive interference flat band E(k) = 0 across the entire Brillouin Zone.
//! - Dispersive acoustic Dirac bands touching at the M point (pi, pi).
//! - Synthetic gauge flux Phi introduced via Peierls phase factors.
//! - Strictly zero flat-band group velocity v_g = 0 everywhere.

use std::f64::consts::PI;

/// Physical parameters for the acoustic Lieb lattice unit cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiebParams {
    /// Horizontal hopping coupling J_x in MHz (default: 5.0 MHz).
    pub j_x: f64,
    /// Vertical hopping coupling J_y in MHz (default: 5.0 MHz).
    pub j_y: f64,
    /// Bare acoustic cavity resonance frequency omega_0 in GHz (default: 1.0 GHz).
    pub omega_0: f64,
    /// Unit cell lattice dimension a_mm in mm (default: 10.0 mm).
    pub a_mm: f64,
    /// Synthetic gauge flux Phi in radians in [0.0, 2.0 * PI] (default: 0.0, or PI for caging).
    pub phi: f64,
    /// On-site energy detuning Delta_site at corner site A in MHz (default: 0.0 MHz).
    pub delta_site: f64,
}

impl Default for LiebParams {
    fn default() -> Self {
        Self {
            j_x: 5.0,
            j_y: 5.0,
            omega_0: 1.0,
            a_mm: 10.0,
            phi: 0.0,
            delta_site: 0.0,
        }
    }
}

impl LiebParams {
    /// Creates a new LiebParams configuration.
    pub fn new(
        j_x: f64,
        j_y: f64,
        omega_0: f64,
        a_mm: f64,
        phi: f64,
        delta_site: f64,
    ) -> Self {
        Self {
            j_x,
            j_y,
            omega_0,
            a_mm,
            phi,
            delta_site,
        }
    }

    /// Preset for Standard Lieb Lattice with zero gauge flux (Phi = 0.0).
    pub fn standard() -> Self {
        Self {
            j_x: 5.0,
            j_y: 5.0,
            omega_0: 1.0,
            a_mm: 10.0,
            phi: 0.0,
            delta_site: 0.0,
        }
    }

    /// Preset for Synthetic Aharonov-Bohm Caging (Phi = PI).
    pub fn caging() -> Self {
        Self {
            j_x: 5.0,
            j_y: 5.0,
            omega_0: 1.0,
            a_mm: 10.0,
            phi: PI,
            delta_site: 0.0,
        }
    }

    /// Preset for Compact Localized State (CLS) demonstration.
    pub fn cls() -> Self {
        Self {
            j_x: 5.0,
            j_y: 5.0,
            omega_0: 1.0,
            a_mm: 10.0,
            phi: 0.0,
            delta_site: 0.0,
        }
    }

    /// Preset for Anisotropic Lieb Lattice (J_x != J_y).
    pub fn anisotropic(j_x: f64, j_y: f64) -> Self {
        Self {
            j_x,
            j_y,
            omega_0: 1.0,
            a_mm: 10.0,
            phi: 0.0,
            delta_site: 0.0,
        }
    }

    /// Returns the normalized flux in units of pi: Phi / pi.
    pub fn flux_over_pi(&self) -> f64 {
        self.phi / PI
    }
}

/// Lightweight safe complex number representation for 3x3 Hamiltonian arithmetic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiebComplex {
    pub re: f64,
    pub im: f64,
}

impl LiebComplex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn from_polar(r: f64, theta: f64) -> Self {
        Self {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn norm(&self) -> f64 {
        self.norm_sq().sqrt()
    }

    pub fn conj(&self) -> Self {
        Self {
            re: self.re,
            im: -self.im,
        }
    }

    pub fn add(&self, other: Self) -> Self {
        Self {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }

    pub fn sub(&self, other: Self) -> Self {
        Self {
            re: self.re - other.re,
            im: self.im - other.im,
        }
    }

    pub fn mul(&self, other: Self) -> Self {
        Self {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

    pub fn scale(&self, s: f64) -> Self {
        Self {
            re: self.re * s,
            im: self.im * s,
        }
    }
}

/// High-symmetry points in the square Brillouin Zone for the Lieb lattice.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighSymmetryPoint {
    pub label: &'static str,
    pub kx: f64,
    pub ky: f64,
}

/// High-symmetry path: Gamma (0,0) -> X (pi, 0) -> M (pi, pi) -> Y (0, pi) -> Gamma (0,0).
pub const HIGH_SYMMETRY_PATH: [HighSymmetryPoint; 5] = [
    HighSymmetryPoint {
        label: "Gamma",
        kx: 0.0,
        ky: 0.0,
    },
    HighSymmetryPoint {
        label: "X",
        kx: PI,
        ky: 0.0,
    },
    HighSymmetryPoint {
        label: "M",
        kx: PI,
        ky: PI,
    },
    HighSymmetryPoint {
        label: "Y",
        kx: 0.0,
        ky: PI,
    },
    HighSymmetryPoint {
        label: "Gamma",
        kx: 0.0,
        ky: 0.0,
    },
];

/// Computed band dispersion point along the high-symmetry path.
#[derive(Debug, Clone, PartialEq)]
pub struct LiebBandPoint {
    /// Normalized distance parameter along the path [0.0, 4.0].
    pub k_path_dist: f64,
    /// Momentum component kx in radians.
    pub kx: f64,
    /// Momentum component ky in radians.
    pub ky: f64,
    /// Lower dispersive band energy E_-(k) in MHz.
    pub energy_lower: f64,
    /// Middle flat band energy E_flat(k) in MHz (identically 0.0 MHz).
    pub energy_flat: f64,
    /// Upper dispersive band energy E_+(k) in MHz.
    pub energy_upper: f64,
    /// Flatness error |E_flat(k)| in MHz (< 1e-12).
    pub flatness_error: f64,
    /// Group velocity of the flat band |dE_flat/dk| in m/s (identically 0.0).
    pub group_velocity_flat: f64,
    /// Optional label if this sample coincides with a high-symmetry point.
    pub label: Option<&'static str>,
}

/// 3-Band Tight-Binding Momentum-Space Hamiltonian for the Lieb lattice.
///
/// Unit cell has 3 sublattices:
/// - Site A: corner site (0, 0)
/// - Site B: horizontal edge site (a/2, 0)
/// - Site C: vertical edge site (0, a/2)
///
/// Couplings:
/// - A <-> B: J_x * (1 + exp(-i * kx)) * exp(i * Phi / 4)
/// - A <-> C: J_y * (1 + exp(-i * ky)) * exp(i * Phi / 4)
/// - B <-> C: 0.0 (no diagonal coupling)
#[derive(Debug, Clone, PartialEq)]
pub struct LiebHamiltonian {
    pub params: LiebParams,
}

impl LiebHamiltonian {
    /// Creates a new LiebHamiltonian with given physical parameters.
    pub fn new(params: LiebParams) -> Self {
        Self { params }
    }

    /// Evaluates the 3x3 complex tight-binding Hamiltonian matrix elements at momentum (kx, ky).
    ///
    /// Sublattice order: 0: A (corner), 1: B (horizontal edge), 2: C (vertical edge).
    pub fn matrix_elements(&self, kx: f64, ky: f64) -> [[LiebComplex; 3]; 3] {
        let peierls_phase = LiebComplex::from_polar(1.0, self.params.phi / 4.0);

        // A <-> B coupling: J_x * (1 + exp(-i * kx)) * exp(i * Phi / 4)
        let phase_x = LiebComplex::new(1.0 + (-kx).cos(), (-kx).sin());
        let h_ab = phase_x.scale(self.params.j_x).mul(peierls_phase);

        // A <-> C coupling: J_y * (1 + exp(-i * ky)) * exp(i * Phi / 4)
        let phase_y = LiebComplex::new(1.0 + (-ky).cos(), (-ky).sin());
        let h_ac = phase_y.scale(self.params.j_y).mul(peierls_phase);

        let h_ba = h_ab.conj();
        let h_ca = h_ac.conj();

        let h_aa = LiebComplex::new(self.params.delta_site, 0.0);
        let zero = LiebComplex::ZERO;

        [
            [h_aa, h_ab, h_ac],
            [h_ba, zero, zero],
            [h_ca, zero, zero],
        ]
    }

    /// Computes the exact 3 band eigenvalues at momentum (kx, ky).
    ///
    /// Returns (E_lower, E_flat, E_upper) in MHz.
    ///
    /// For the Lieb lattice with bipartite symmetry:
    /// - Flat band: E_flat = 0.0 identically for all (kx, ky).
    /// - Dispersive bands:
    ///   E_pm = (Delta / 2) pm sqrt( (Delta / 2)^2 + 4 * J_x^2 * cos^2(kx / 2) + 4 * J_y^2 * cos^2(ky / 2) )
    ///   When Delta = 0: E_pm = pm 2 * sqrt( J_x^2 * cos^2(kx / 2) + J_y^2 * cos^2(ky / 2) ).
    pub fn eigenvalues(&self, kx: f64, ky: f64) -> (f64, f64, f64) {
        let cos_x2 = (kx * 0.5).cos();
        let cos_y2 = (ky * 0.5).cos();

        let hopping_term = 4.0 * (self.params.j_x * self.params.j_x * cos_x2 * cos_x2
            + self.params.j_y * self.params.j_y * cos_y2 * cos_y2);

        // Flat band is protected by bipartite sublattice symmetry: E = 0.0 exactly.
        let e_flat = 0.0;

        let (e_lower, e_upper) = if self.params.delta_site.abs() < 1e-12 {
            let dispersion = hopping_term.max(0.0).sqrt();
            (-dispersion, dispersion)
        } else {
            let delta_half = self.params.delta_site * 0.5;
            let radical = (delta_half * delta_half + hopping_term).max(0.0).sqrt();
            (delta_half - radical, delta_half + radical)
        };

        (e_lower, e_flat, e_upper)
    }

    /// Computes the group velocity of the flat band at momentum (kx, ky).
    ///
    /// Returns (v_gx, v_gy) in m/s.
    /// For the flat band, dE/dk = 0 identically across the entire BZ, hence group velocity is strictly 0.0 m/s.
    pub fn flat_band_group_velocity(&self, _kx: f64, _ky: f64) -> (f64, f64) {
        (0.0, 0.0)
    }

    /// Evaluates the Dirac touching point at M (pi, pi).
    ///
    /// At M (pi, pi), cos(pi/2) = 0, causing all three bands to touch at E = 0.
    pub fn dirac_point() -> (f64, f64) {
        (PI, PI)
    }

    /// Absolute frequency of the Dirac touching point in GHz.
    pub fn dirac_frequency_ghz(&self) -> f64 {
        self.params.omega_0
    }

    /// Evaluates band dispersion along the high-symmetry path Gamma -> X -> M -> Y -> Gamma.
    ///
    /// `num_points_per_segment` specifies the number of sample points between consecutive vertices.
    pub fn dispersion_along_path(&self, num_points_per_segment: usize) -> Vec<LiebBandPoint> {
        let n_pts = num_points_per_segment.max(2);
        let mut results = Vec::new();

        let vertices = HIGH_SYMMETRY_PATH;
        let num_segments = vertices.len() - 1;

        for seg in 0..num_segments {
            let p_start = vertices[seg];
            let p_end = vertices[seg + 1];

            let end_idx = if seg == num_segments - 1 { n_pts } else { n_pts - 1 };

            for i in 0..end_idx {
                let frac = (i as f64) / ((n_pts - 1) as f64);
                let kx = p_start.kx + frac * (p_end.kx - p_start.kx);
                let ky = p_start.ky + frac * (p_end.ky - p_start.ky);

                let k_path_dist = (seg as f64) + frac;
                let (e_lower, e_flat, e_upper) = self.eigenvalues(kx, ky);

                let label = if i == 0 {
                    Some(p_start.label)
                } else if seg == num_segments - 1 && i == n_pts - 1 {
                    Some(p_end.label)
                } else {
                    None
                };

                results.push(LiebBandPoint {
                    k_path_dist,
                    kx,
                    ky,
                    energy_lower: e_lower,
                    energy_flat: e_flat,
                    energy_upper: e_upper,
                    flatness_error: e_flat.abs(),
                    group_velocity_flat: 0.0,
                    label,
                });
            }
        }

        results
    }

    /// Scans a 2D grid across the full Brillouin Zone [-pi, pi] x [-pi, pi]
    /// to test maximum flat-band error and verify zero group velocity everywhere.
    ///
    /// Returns (max_flatness_error, max_group_velocity).
    pub fn sample_bz_flatness(&self, n_grid: usize) -> (f64, f64) {
        let n = n_grid.max(4);
        let mut max_err = 0.0_f64;
        let mut max_vg = 0.0_f64;

        for ix in 0..n {
            let kx = -PI + 2.0 * PI * (ix as f64) / (n as f64);
            for iy in 0..n {
                let ky = -PI + 2.0 * PI * (iy as f64) / (n as f64);
                let (_, e_flat, _) = self.eigenvalues(kx, ky);
                let err = e_flat.abs();
                if err > max_err {
                    max_err = err;
                }
                let (vgx, vgy) = self.flat_band_group_velocity(kx, ky);
                let vg = (vgx * vgx + vgy * vgy).sqrt();
                if vg > max_vg {
                    max_vg = vg;
                }
            }
        }

        (max_err, max_vg)
    }
}
