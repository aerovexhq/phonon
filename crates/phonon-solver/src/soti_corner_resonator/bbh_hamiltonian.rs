#![deny(unsafe_code)]

//! Benalcazar-Bernevig-Hughes (BBH) quadrupole tight-binding Hamiltonian
//! and bulk topological band structure engine for 2D acoustic metamaterials.
//!
//! Implements the 2D quadrupole topological insulator with quantized quadrupole
//! moment q_xy = 1/2 (mod 1), corner-localized acoustic states, and pi-flux per plaquette.

use std::f64::consts::PI;

/// Physical parameters for the acoustic quadrupole metamaterial unit cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadrupoleParams {
    /// Intracell acoustic hopping coupling gamma in MHz (default: 2.0 MHz).
    pub gamma: f64,
    /// Intercell acoustic hopping coupling lambda in MHz (default: 10.0 MHz).
    pub lambda: f64,
    /// Bare acoustic cavity resonance frequency omega_0 in GHz (default: 1.0 GHz).
    pub omega_0: f64,
    /// Unit cell lattice constant a_mm in mm (default: 5.0 mm).
    pub a_mm: f64,
}

impl Default for QuadrupoleParams {
    fn default() -> Self {
        Self {
            gamma: 2.0,
            lambda: 10.0,
            omega_0: 1.0,
            a_mm: 5.0,
        }
    }
}

impl QuadrupoleParams {
    /// Creates a new QuadrupoleParams configuration.
    pub fn new(gamma: f64, lambda: f64, omega_0: f64, a_mm: f64) -> Self {
        Self {
            gamma,
            lambda,
            omega_0,
            a_mm,
        }
    }

    /// Default preset for Topological Second-Order Topological Insulator (SOTI) phase.
    pub fn topological() -> Self {
        Self {
            gamma: 2.0,
            lambda: 10.0,
            omega_0: 1.0,
            a_mm: 5.0,
        }
    }

    /// Preset for Trivial Insulator phase (gamma > lambda).
    pub fn trivial() -> Self {
        Self {
            gamma: 10.0,
            lambda: 2.0,
            omega_0: 1.0,
            a_mm: 5.0,
        }
    }

    /// Returns the coupling ratio gamma / lambda.
    pub fn coupling_ratio(&self) -> f64 {
        if self.lambda.abs() > 1e-12 {
            self.gamma / self.lambda
        } else {
            f64::INFINITY
        }
    }
}

/// High-symmetry points in the square Brillouin Zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighSymmetryPoint {
    pub label: &'static str,
    pub kx: f64,
    pub ky: f64,
}

/// Standard high-symmetry points along the path Gamma -> X -> M -> Y -> Gamma.
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

/// Sample point along the Brillouin Zone band dispersion path.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BandDispersionPoint {
    /// Cumulative parameter/distance along the k-path [0, 4*pi].
    pub path_distance: f64,
    /// Wavevector component kx in rad.
    pub kx: f64,
    /// Wavevector component ky in rad.
    pub ky: f64,
    /// 4 band eigenenergies in ascending order [E1, E2, E3, E4] in MHz.
    pub eigenvalues: [f64; 4],
    /// High-symmetry label if this point coincides with a vertex.
    pub symmetry_label: Option<&'static str>,
}

/// Complex number representation for 4x4 matrix elements.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BbhComplex {
    pub re: f64,
    pub im: f64,
}

impl BbhComplex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn zero() -> Self {
        Self { re: 0.0, im: 0.0 }
    }

    pub fn norm_sq(&self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

/// Benalcazar-Bernevig-Hughes (BBH) tight-binding model Hamiltonian engine.
#[derive(Debug, Clone, PartialEq)]
pub struct BbhHamiltonian {
    pub params: QuadrupoleParams,
}

impl Default for BbhHamiltonian {
    fn default() -> Self {
        Self::new(QuadrupoleParams::default())
    }
}

impl BbhHamiltonian {
    /// Creates a new BBH Hamiltonian with specified quadrupole parameters.
    pub fn new(params: QuadrupoleParams) -> Self {
        Self { params }
    }

    /// Evaluates the 4x4 momentum-space Hamiltonian matrix H(kx, ky).
    ///
    /// H(kx, ky) = (gamma_x + lambda_x*cos(kx))*Gamma_4 + lambda_x*sin(kx)*Gamma_3
    ///           + (gamma_y + lambda_y*cos(ky))*Gamma_2 + lambda_y*sin(ky)*Gamma_1
    ///
    /// where:
    /// Gamma_4 = tau_3 (x) sigma_0
    /// Gamma_3 = -tau_2 (x) sigma_0
    /// Gamma_2 = -tau_1 (x) sigma_1
    /// Gamma_1 = -tau_1 (x) sigma_2
    pub fn hamiltonian_matrix(&self, kx: f64, ky: f64) -> [[BbhComplex; 4]; 4] {
        let ux = self.params.gamma + self.params.lambda * kx.cos();
        let vx = self.params.lambda * kx.sin();
        let uy = self.params.gamma + self.params.lambda * ky.cos();
        let vy = self.params.lambda * ky.sin();

        let mut h = [[BbhComplex::zero(); 4]; 4];

        // Row 0
        h[0][0] = BbhComplex::new(ux, 0.0);
        h[0][1] = BbhComplex::zero();
        h[0][2] = BbhComplex::new(0.0, vx);
        h[0][3] = BbhComplex::new(-uy, vy);

        // Row 1
        h[1][0] = BbhComplex::zero();
        h[1][1] = BbhComplex::new(ux, 0.0);
        h[1][2] = BbhComplex::new(-uy, -vy);
        h[1][3] = BbhComplex::new(0.0, vx);

        // Row 2
        h[2][0] = BbhComplex::new(0.0, -vx);
        h[2][1] = BbhComplex::new(-uy, vy);
        h[2][2] = BbhComplex::new(-ux, 0.0);
        h[2][3] = BbhComplex::zero();

        // Row 3
        h[3][0] = BbhComplex::new(-uy, -vy);
        h[3][1] = BbhComplex::new(0.0, -vx);
        h[3][2] = BbhComplex::zero();
        h[3][3] = BbhComplex::new(-ux, 0.0);

        h
    }

    /// Computes the 4 eigenvalues of H(kx, ky) in ascending order.
    ///
    /// Due to the anticommutation relations {Gamma_i, Gamma_j} = 2*delta_ij,
    /// H^2 = (ux^2 + vx^2 + uy^2 + vy^2) * I_4.
    /// Thus the eigenvalues are doubly-degenerate pairs: -E_0, -E_0, +E_0, +E_0.
    pub fn eigenvalues(&self, kx: f64, ky: f64) -> [f64; 4] {
        let ux = self.params.gamma + self.params.lambda * kx.cos();
        let vx = self.params.lambda * kx.sin();
        let uy = self.params.gamma + self.params.lambda * ky.cos();
        let vy = self.params.lambda * ky.sin();

        let energy_sq = ux * ux + vx * vx + uy * uy + vy * vy;
        let e0 = energy_sq.max(0.0).sqrt();

        [-e0, -e0, e0, e0]
    }

    /// Bulk bandgap calculation: Delta_bulk = 2 * |lambda - gamma|.
    pub fn bulk_bandgap(&self) -> f64 {
        2.0 * (self.params.lambda - self.params.gamma).abs()
    }

    /// Quantized quadrupole bulk moment q_xy.
    ///
    /// Returns 0.5 when gamma < lambda (topological SOTI), 0.0 when gamma > lambda (trivial).
    pub fn quadrupole_moment(&self) -> f64 {
        if self.params.gamma < self.params.lambda {
            0.5
        } else {
            0.0
        }
    }

    /// Edge dipole polarization moments (p_x^edge, p_y^edge).
    ///
    /// In topological phase: (0.5, 0.5). In trivial phase: (0.0, 0.0).
    pub fn edge_dipole_moments(&self) -> (f64, f64) {
        if self.is_topological() {
            (0.5, 0.5)
        } else {
            (0.0, 0.0)
        }
    }

    /// Returns true if the system is in the topological higher-order corner state phase.
    pub fn is_topological(&self) -> bool {
        self.params.gamma < self.params.lambda
    }

    /// Localization decay length xi in mm: xi = a / ln(lambda / gamma).
    ///
    /// Returns infinity if gamma >= lambda (trivial phase with no localized corner modes).
    pub fn localization_decay_length(&self) -> f64 {
        if self.is_topological() && self.params.gamma > 1e-12 {
            let ratio = self.params.lambda / self.params.gamma;
            self.params.a_mm / ratio.ln()
        } else {
            f64::INFINITY
        }
    }

    /// Computes band dispersion along the high-symmetry path Gamma -> X -> M -> Y -> Gamma.
    ///
    /// `points_per_segment` specifies the number of evaluation points along each of the 4 segments.
    pub fn compute_band_dispersion(&self, points_per_segment: usize) -> Vec<BandDispersionPoint> {
        let n_seg = points_per_segment.max(2);
        let mut result = Vec::with_capacity(n_seg * 4 + 1);

        let vertices = &HIGH_SYMMETRY_PATH;
        let mut cumulative_dist = 0.0;

        for s in 0..4 {
            let p_start = &vertices[s];
            let p_end = &vertices[s + 1];

            let dx = p_end.kx - p_start.kx;
            let dy = p_end.ky - p_start.ky;
            let seg_len = (dx * dx + dy * dy).sqrt();

            for i in 0..n_seg {
                // Avoid duplicating the vertex at segment boundary
                if s > 0 && i == 0 {
                    continue;
                }

                let frac = (i as f64) / (n_seg as f64);
                let kx = p_start.kx + frac * dx;
                let ky = p_start.ky + frac * dy;
                let path_dist = cumulative_dist + frac * seg_len;

                let symmetry_label = if i == 0 {
                    Some(p_start.label)
                } else {
                    None
                };

                let eigenvalues = self.eigenvalues(kx, ky);

                result.push(BandDispersionPoint {
                    path_distance: path_dist,
                    kx,
                    ky,
                    eigenvalues,
                    symmetry_label,
                });
            }

            cumulative_dist += seg_len;
        }

        // Final point at Gamma
        let last_vertex = &vertices[4];
        let last_eigenvalues = self.eigenvalues(last_vertex.kx, last_vertex.ky);
        result.push(BandDispersionPoint {
            path_distance: cumulative_dist,
            kx: last_vertex.kx,
            ky: last_vertex.ky,
            eigenvalues: last_eigenvalues,
            symmetry_label: Some(last_vertex.label),
        });

        result
    }
}
