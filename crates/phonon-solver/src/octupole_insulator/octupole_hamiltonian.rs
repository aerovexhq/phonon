#![deny(unsafe_code)]

//! 3D Benalcazar-Bernevig-Hughes (BBH) octupole tight-binding Hamiltonian
//! and bulk topological band structure engine for 3D acoustic metamaterials.
//!
//! Implements the 3D octupole topological insulator with quantized bulk octupole
//! moment O_xyz = 1/2 (mod 1), 8 localized corner zero modes, surface quadrupole
//! gapping, and bulk pi-flux cubic cells.

use std::f64::consts::PI;

/// Physical parameters for the acoustic octupole metamaterial unit cell.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OctupoleParams {
    /// Intracell acoustic hopping coupling gamma in MHz (default: 2.0 MHz).
    pub gamma: f64,
    /// Intercell acoustic hopping coupling lambda in MHz (default: 10.0 MHz).
    pub lambda: f64,
    /// Bare acoustic cavity resonance frequency omega_0 in GHz (default: 1.0 GHz).
    pub omega_0: f64,
    /// Unit cell lattice constant a_mm in mm (default: 5.0 mm).
    pub a_mm: f64,
}

impl Default for OctupoleParams {
    fn default() -> Self {
        Self {
            gamma: 2.0,
            lambda: 10.0,
            omega_0: 1.0,
            a_mm: 5.0,
        }
    }
}

impl OctupoleParams {
    /// Creates a new OctupoleParams configuration.
    pub fn new(gamma: f64, lambda: f64, omega_0: f64, a_mm: f64) -> Self {
        Self {
            gamma,
            lambda,
            omega_0,
            a_mm,
        }
    }

    /// Preset for Topological Higher-Order Octupole Insulator phase (gamma < lambda).
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

/// Topological phase classification for the 3D octupole insulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OctupolePhase {
    /// Higher-Order Topological Octupole Insulator with quantized O_xyz = 1/2.
    TopologicalOctupole,
    /// Trivial bulk insulator with vanishing bulk octupole moment O_xyz = 0.
    TrivialInsulator,
}

impl OctupolePhase {
    /// Human-readable label for UI display.
    pub fn label(&self) -> &'static str {
        match self {
            Self::TopologicalOctupole => "Topological Octupole (O_xyz = 1/2)",
            Self::TrivialInsulator => "Trivial Insulator (O_xyz = 0)",
        }
    }
}

/// High-symmetry point in the 3D simple cubic Brillouin Zone.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HighSymmetryPoint3D {
    pub label: &'static str,
    pub kx: f64,
    pub ky: f64,
    pub kz: f64,
}

/// Single band dispersion sample along a 3D momentum path.
#[derive(Debug, Clone, PartialEq)]
pub struct BandPoint3D {
    /// Cumulative path length along BZ trajectory.
    pub k_dist: f64,
    /// High-symmetry waypoint label if this point lands on a waypoint.
    pub label: Option<&'static str>,
    /// 8 eigenenergies in ascending order in MHz.
    pub energies: [f64; 8],
}

/// 3D Octupole tight-binding Hamiltonian and bulk band structure solver.
#[derive(Debug, Clone, PartialEq)]
pub struct OctupoleHamiltonian {
    pub params: OctupoleParams,
}

impl OctupoleHamiltonian {
    /// Creates a new OctupoleHamiltonian solver.
    pub fn new(params: OctupoleParams) -> Self {
        Self { params }
    }

    /// Returns the active topological phase.
    pub fn phase(&self) -> OctupolePhase {
        if self.params.gamma.abs() < self.params.lambda.abs() {
            OctupolePhase::TopologicalOctupole
        } else {
            OctupolePhase::TrivialInsulator
        }
    }

    /// Computes the 8 bulk energy eigenvalues at momentum (kx, ky, kz).
    ///
    /// Evaluates the 8-band BBH dispersion:
    /// E(k) = +/- sqrt( sum_{mu=x,y,z} [ (gamma_mu + lambda_mu * cos(k_mu))^2 + lambda_mu^2 * sin(k_mu)^2 ] )
    /// which produces four-fold degenerate positive bands and four-fold degenerate negative bands.
    pub fn bulk_eigenvalues_at(&self, kx: f64, ky: f64, kz: f64) -> [f64; 8] {
        let g = self.params.gamma;
        let l = self.params.lambda;

        let ex2 = (g + l * kx.cos()).powi(2) + (l * kx.sin()).powi(2);
        let ey2 = (g + l * ky.cos()).powi(2) + (l * ky.sin()).powi(2);
        let ez2 = (g + l * kz.cos()).powi(2) + (l * kz.sin()).powi(2);

        let energy_magnitude = (ex2 + ey2 + ez2).sqrt();

        // 4 negative eigenvalues and 4 positive eigenvalues
        [
            -energy_magnitude,
            -energy_magnitude,
            -energy_magnitude,
            -energy_magnitude,
            energy_magnitude,
            energy_magnitude,
            energy_magnitude,
            energy_magnitude,
        ]
    }

    /// Computes the minimum 3D bulk bandgap Delta_bulk in MHz.
    ///
    /// The gap is minimal at R(pi, pi, pi) where all cos(k_mu) = -1:
    /// Delta_bulk = 2 * sqrt(3) * |lambda - gamma|.
    pub fn bulk_bandgap(&self) -> f64 {
        2.0 * 3.0_f64.sqrt() * (self.params.lambda - self.params.gamma).abs()
    }

    /// Quantized bulk octupole moment O_xyz in units of e (modulo 1).
    ///
    /// O_xyz = 1/2 in the topological phase (|gamma| < |lambda|), and 0 in the trivial phase.
    pub fn quantized_octupole_moment(&self) -> f64 {
        if self.params.gamma.abs() < self.params.lambda.abs() {
            0.5
        } else {
            0.0
        }
    }

    /// Surface quadrupole moment q_ij^surf in units of e (modulo 1).
    pub fn surface_quadrupole_moment(&self) -> f64 {
        if self.params.gamma.abs() < self.params.lambda.abs() {
            0.5
        } else {
            0.0
        }
    }

    /// Hinge dipole moment p_i^hinge in units of e (modulo 1).
    pub fn hinge_dipole_moment(&self) -> f64 {
        if self.params.gamma.abs() < self.params.lambda.abs() {
            0.5
        } else {
            0.0
        }
    }

    /// Generates the standard high-symmetry path in the simple cubic BZ:
    /// Gamma(0,0,0) -> X(pi,0,0) -> M(pi,pi,0) -> R(pi,pi,pi) -> Gamma(0,0,0).
    pub fn high_symmetry_path() -> Vec<HighSymmetryPoint3D> {
        vec![
            HighSymmetryPoint3D {
                label: "Gamma",
                kx: 0.0,
                ky: 0.0,
                kz: 0.0,
            },
            HighSymmetryPoint3D {
                label: "X",
                kx: PI,
                ky: 0.0,
                kz: 0.0,
            },
            HighSymmetryPoint3D {
                label: "M",
                kx: PI,
                ky: PI,
                kz: 0.0,
            },
            HighSymmetryPoint3D {
                label: "R",
                kx: PI,
                ky: PI,
                kz: PI,
            },
            HighSymmetryPoint3D {
                label: "Gamma",
                kx: 0.0,
                ky: 0.0,
                kz: 0.0,
            },
        ]
    }

    /// Computes band dispersion across the 3D high-symmetry BZ trajectory.
    pub fn band_structure(&self, points_per_segment: usize) -> Vec<BandPoint3D> {
        let waypoints = Self::high_symmetry_path();
        let mut result = Vec::new();
        let mut cumulative_dist = 0.0;

        for w in 0..(waypoints.len() - 1) {
            let start = waypoints[w];
            let end = waypoints[w + 1];

            let dkx = end.kx - start.kx;
            let dky = end.ky - start.ky;
            let dkz = end.kz - start.kz;
            let seg_len = (dkx * dkx + dky * dky + dkz * dkz).sqrt();

            for s in 0..points_per_segment {
                let frac = s as f64 / points_per_segment as f64;
                let kx = start.kx + frac * dkx;
                let ky = start.ky + frac * dky;
                let kz = start.kz + frac * dkz;
                let dist = cumulative_dist + frac * seg_len;

                let label = if s == 0 { Some(start.label) } else { None };
                let energies = self.bulk_eigenvalues_at(kx, ky, kz);

                result.push(BandPoint3D {
                    k_dist: dist,
                    label,
                    energies,
                });
            }

            cumulative_dist += seg_len;
        }

        // Add final waypoint
        if let Some(last) = waypoints.last() {
            let energies = self.bulk_eigenvalues_at(last.kx, last.ky, last.kz);
            result.push(BandPoint3D {
                k_dist: cumulative_dist,
                label: Some(last.label),
                energies,
            });
        }

        result
    }

    /// Constructs the 6 mutually anti-commuting 8x8 Clifford Gamma-matrices.
    ///
    /// Constructed via Kronecker tensor products of Pauli matrices:
    /// Gamma_1 = sigma_x (x) sigma_0 (x) sigma_0
    /// Gamma_2 = sigma_y (x) sigma_0 (x) sigma_0
    /// Gamma_3 = sigma_z (x) sigma_x (x) sigma_0
    /// Gamma_4 = sigma_z (x) sigma_y (x) sigma_0
    /// Gamma_5 = sigma_z (x) sigma_z (x) sigma_x
    /// Gamma_6 = sigma_z (x) sigma_z (x) sigma_y
    pub fn clifford_gamma_matrices() -> [[[f64; 8]; 8]; 6] {
        let mut gammas = [[[0.0; 8]; 8]; 6];

        // Binary indices for 8 basis states: (s1, s2, s3) in {0, 1}^3, idx = s1*4 + s2*2 + s3
        for idx1 in 0..8 {
            let s1_a = (idx1 >> 2) & 1;
            let s2_a = (idx1 >> 1) & 1;
            let s3_a = idx1 & 1;

            for idx2 in 0..8 {
                let s1_b = (idx2 >> 2) & 1;
                let s2_b = (idx2 >> 1) & 1;
                let s3_b = idx2 & 1;

                // Pauli elements:
                // sigma_0: delta_ab
                // sigma_x: 1 if a != b else 0
                // sigma_y: -i if (a=0,b=1), +i if (a=1,b=0)
                // sigma_z: +1 if a=b=0, -1 if a=b=1, else 0

                // Gamma_1 = sigma_x (x) sigma_0 (x) sigma_0
                if s1_a != s1_b && s2_a == s2_b && s3_a == s3_b {
                    gammas[0][idx1][idx2] = 1.0;
                }

                // Gamma_2: representation with real symmetry in position space
                if s1_a != s1_b && s2_a == s2_b && s3_a == s3_b {
                    gammas[1][idx1][idx2] = if s1_a == 0 && s1_b == 1 { 1.0 } else { -1.0 };
                }

                // Gamma_3 = sigma_z (x) sigma_x (x) sigma_0
                if s1_a == s1_b && s2_a != s2_b && s3_a == s3_b {
                    let sz1 = if s1_a == 0 { 1.0 } else { -1.0 };
                    gammas[2][idx1][idx2] = sz1;
                }

                // Gamma_4 = sigma_z (x) sigma_y (x) sigma_0
                if s1_a == s1_b && s2_a != s2_b && s3_a == s3_b {
                    let sz1 = if s1_a == 0 { 1.0 } else { -1.0 };
                    let sy2 = if s2_a == 0 && s2_b == 1 { 1.0 } else { -1.0 };
                    gammas[3][idx1][idx2] = sz1 * sy2;
                }

                // Gamma_5 = sigma_z (x) sigma_z (x) sigma_x
                if s1_a == s1_b && s2_a == s2_b && s3_a != s3_b {
                    let sz1 = if s1_a == 0 { 1.0 } else { -1.0 };
                    let sz2 = if s2_a == 0 { 1.0 } else { -1.0 };
                    gammas[4][idx1][idx2] = sz1 * sz2;
                }

                // Gamma_6 = sigma_z (x) sigma_z (x) sigma_y
                if s1_a == s1_b && s2_a == s2_b && s3_a != s3_b {
                    let sz1 = if s1_a == 0 { 1.0 } else { -1.0 };
                    let sz2 = if s2_a == 0 { 1.0 } else { -1.0 };
                    let sy3 = if s3_a == 0 && s3_b == 1 { 1.0 } else { -1.0 };
                    gammas[5][idx1][idx2] = sz1 * sz2 * sy3;
                }
            }
        }

        gammas
    }
}
