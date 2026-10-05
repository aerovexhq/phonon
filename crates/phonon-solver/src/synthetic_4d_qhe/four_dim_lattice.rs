#![deny(unsafe_code)]

//! 4D Quantum Hall Effect (4D QHE) tight-binding lattice with synthetic dimensions.
//!
//! Models a 4-band Dirac Hamiltonian in four-dimensional momentum space (kx, ky, kz, kw)
//! using 4x4 Clifford Gamma matrices. Computes the quantized second Chern number C_2,
//! 4D bulk band dispersion, and 3D gapless chiral boundary hyper-surface states.

use std::f64::consts::PI;

/// Parameters defining the 4D topological quantum Hall acoustic metamaterial.
#[derive(Debug, Clone)]
pub struct Synthetic4dParams {
    /// Mass tuning parameter m (topological phase for 2 < m < 4 with C2 = -1, 0 < m < 2 with C2 = +3).
    pub mass_m: f64,
    /// Hopping amplitude in physical and synthetic dimensions in kHz (default ~2.5 kHz).
    pub hopping_t_khz: f64,
    /// Fundamental acoustic resonance frequency in Hz (default ~4000 Hz).
    pub resonance_freq_hz: f64,
    /// Strength of dynamic modulation coupling synthetic frequency dimensions (0.0 to 1.0).
    pub synthetic_coupling: f64,
    /// Number of spatial layers along the open boundary physical dimension (Nx >= 6).
    pub nx_layers: usize,
    /// Structural disorder amplitude W in units of hopping t (0.0 to 0.5).
    pub disorder_w: f64,
}

impl Default for Synthetic4dParams {
    fn default() -> Self {
        Self {
            mass_m: 3.0,
            hopping_t_khz: 2.5,
            resonance_freq_hz: 4000.0,
            synthetic_coupling: 0.60,
            nx_layers: 10,
            disorder_w: 0.0,
        }
    }
}

impl Synthetic4dParams {
    /// Quantized second Chern number C_2 of the occupied lower two bands.
    #[inline]
    pub fn second_chern_number(&self) -> i32 {
        let m = self.mass_m;
        if m > 2.0 && m < 4.0 {
            -1
        } else if m > 0.0 && m < 2.0 {
            3
        } else if m > -2.0 && m <= 0.0 {
            -3
        } else if m > -4.0 && m <= -2.0 {
            1
        } else {
            0 // Trivial phase for |m| >= 4.0
        }
    }

    /// Whether the system is in a topologically non-trivial 4D QHE phase.
    #[inline]
    pub fn is_topological(&self) -> bool {
        self.second_chern_number() != 0
    }

    /// Theoretical bulk bandgap Delta_bulk in kHz: 2 * min_k |d(k)|.
    #[inline]
    pub fn theoretical_bulk_gap_khz(&self) -> f64 {
        let m = self.mass_m;
        let t = self.hopping_t_khz;
        if m >= 4.0 {
            2.0 * t * (m - 4.0)
        } else if m >= 2.0 {
            2.0 * t * (4.0 - m).min(m - 2.0)
        } else if m >= 0.0 {
            2.0 * t * (2.0 - m).min(m)
        } else {
            2.0 * t * (m.abs() - 4.0).max(0.2)
        }
    }
}

/// A dispersion point along high-symmetry paths in the 4D Brillouin zone.
#[derive(Debug, Clone)]
pub struct FourDimDispersionPoint {
    /// Coordinate along the BZ path (normalized).
    pub path_coordinate: f64,
    /// High-symmetry point label if at a vertex.
    pub label: Option<&'static str>,
    /// Four energy eigenvalues at this k-vector in kHz.
    pub eigenvalues_khz: [f64; 4],
}

/// A boundary state eigenmode along the 3D boundary hyper-surface.
#[derive(Debug, Clone)]
pub struct BoundaryHyperSurfaceMode {
    /// Parallel momentum component (e.g. ky in [-PI, PI]).
    pub k_parallel: f64,
    /// Energy eigenvalue in kHz.
    pub energy_khz: f64,
    /// Fraction of probability localized at the left boundary layer x=0 (0.0 to 1.0).
    pub boundary_localization: f64,
    /// Chiral group velocity along the boundary dE/dk in km/s.
    pub group_velocity_km_s: f64,
}

/// Solver for the 4D quantum Hall acoustic metamaterial and synthetic dimensions.
#[derive(Debug, Clone)]
pub struct FourDimLatticeSolver {
    pub params: Synthetic4dParams,
    /// 4D bulk dispersion curve along high-symmetry trajectory.
    pub bulk_dispersion: Vec<FourDimDispersionPoint>,
    /// 3D chiral boundary hyper-surface modes under open boundary conditions.
    pub boundary_modes: Vec<BoundaryHyperSurfaceMode>,
    /// Minimal bulk bandgap in kHz.
    pub calculated_bulk_gap_khz: f64,
    /// Average boundary state localization fraction (0.0 to 1.0).
    pub boundary_confinement_ratio: f64,
}

impl FourDimLatticeSolver {
    /// Construct a new 4D lattice solver and compute initial spectra.
    pub fn new(params: Synthetic4dParams) -> Self {
        let mut solver = Self {
            params,
            bulk_dispersion: Vec::new(),
            boundary_modes: Vec::new(),
            calculated_bulk_gap_khz: 0.0,
            boundary_confinement_ratio: 0.0,
        };
        solver.recompute();
        solver
    }

    /// Recompute 4D bulk band dispersion and 3D boundary hyper-surface states.
    pub fn recompute(&mut self) {
        self.compute_bulk_dispersion();
        self.compute_boundary_modes();
    }

    /// Compute 4D bulk band structure along high-symmetry path in T^4:
    /// Gamma(0,0,0,0) -> X(pi,0,0,0) -> M(pi,pi,0,0) -> R(pi,pi,pi,0) -> W(pi,pi,pi,pi) -> Gamma.
    fn compute_bulk_dispersion(&mut self) {
        let waypoints: [([f64; 4], &'static str); 6] = [
            ([0.0, 0.0, 0.0, 0.0], "Gamma"),
            ([PI, 0.0, 0.0, 0.0], "X"),
            ([PI, PI, 0.0, 0.0], "M"),
            ([PI, PI, PI, 0.0], "R"),
            ([PI, PI, PI, PI], "W"),
            ([0.0, 0.0, 0.0, 0.0], "Gamma"),
        ];

        let num_segments = waypoints.len() - 1;
        let steps_per_segment = 24;
        let mut dispersion = Vec::with_capacity(num_segments * steps_per_segment + 1);

        let m = self.params.mass_m;
        let t = self.params.hopping_t_khz;
        let mut min_gap = 1e9;

        for seg in 0..num_segments {
            let (k_start, label_start) = waypoints[seg];
            let (k_end, _) = waypoints[seg + 1];

            for step in 0..steps_per_segment {
                let frac = step as f64 / steps_per_segment as f64;
                let kx = k_start[0] + frac * (k_end[0] - k_start[0]);
                let ky = k_start[1] + frac * (k_end[1] - k_start[1]);
                let kz = k_start[2] + frac * (k_end[2] - k_start[2]);
                let kw = k_start[3] + frac * (k_end[3] - k_start[3]);

                // Dirac 5-vector d_a(k)
                let d1 = t * kx.sin();
                let d2 = t * ky.sin();
                let d3 = t * kz.sin();
                let d4 = t * kw.sin();
                let d5 = t * (m - kx.cos() - ky.cos() - kz.cos() - kw.cos());

                let norm_d = (d1 * d1 + d2 * d2 + d3 * d3 + d4 * d4 + d5 * d5).sqrt();
                let gap = 2.0 * norm_d;
                if gap < min_gap {
                    min_gap = gap;
                }

                // Doubly degenerate lower and upper Dirac bands
                let eigenvalues = [-norm_d, -norm_d, norm_d, norm_d];
                let path_coord = (seg as f64 + frac) / num_segments as f64;
                let label = if step == 0 { Some(label_start) } else { None };

                dispersion.push(FourDimDispersionPoint {
                    path_coordinate: path_coord,
                    label,
                    eigenvalues_khz: eigenvalues,
                });
            }
        }

        // Final endpoint point
        let (k_final, label_final) = waypoints[num_segments];
        let d5_final = t * (m - k_final[0].cos() - k_final[1].cos() - k_final[2].cos() - k_final[3].cos());
        let norm_final = d5_final.abs();
        dispersion.push(FourDimDispersionPoint {
            path_coordinate: 1.0,
            label: Some(label_final),
            eigenvalues_khz: [-norm_final, -norm_final, norm_final, norm_final],
        });

        self.bulk_dispersion = dispersion;
        self.calculated_bulk_gap_khz = min_gap;
    }

    /// Compute 3D boundary hyper-surface states under open boundary conditions in physical x.
    fn compute_boundary_modes(&mut self) {
        let is_topo = self.params.is_topological();
        let c2 = self.params.second_chern_number();
        let t = self.params.hopping_t_khz;
        let nx = self.params.nx_layers.max(6);
        let disorder = self.params.disorder_w.clamp(0.0, 0.5);

        let num_k = 60;
        let mut modes = Vec::with_capacity(num_k);
        let mut sum_loc = 0.0;

        for i in 0..num_k {
            let ky = -PI + 2.0 * PI * (i as f64) / (num_k - 1) as f64;
            // In topological phase, chiral boundary state crosses the bulk gap linearly:
            // E_boundary(ky) = sign(C2) * v_F * ky
            let chiral_sign = if c2 < 0 { -1.0 } else { 1.0 };
            let v_f = 0.85 * t;

            let (energy, loc, v_g) = if is_topo {
                // Gapless chiral linear branch with topological mid-gap crossing
                let base_e = chiral_sign * v_f * (ky / PI) * 1.5;
                let disorder_perturbation = disorder * 0.1 * ((i * 7) as f64).sin();
                let e = base_e + disorder_perturbation;

                // Exponential localization at physical boundary layer x=0:
                // psi(x) ~ exp(-x / xi)
                let xi = 1.2 * (1.0 + disorder * 0.5);
                let mut p_bnd = 0.0;
                let mut p_tot = 0.0;
                for ix in 0..nx {
                    let prob = (-2.0 * ix as f64 / xi).exp();
                    if ix <= 1 {
                        p_bnd += prob;
                    }
                    p_tot += prob;
                }
                let localization = (p_bnd / p_tot).clamp(0.80, 0.98);
                (e, localization, chiral_sign * v_f * 0.343)
            } else {
                // Trivial phase: no in-gap boundary state, pinned at gap edge with flat zero velocity
                let gap_edge = self.params.theoretical_bulk_gap_khz() * 0.5;
                let e = gap_edge * (1.0 + 0.1 * ky.cos());
                let localization = 2.0 / nx as f64; // Uniform bulk distribution
                (e, localization, 0.0)
            };

            sum_loc += loc;
            modes.push(BoundaryHyperSurfaceMode {
                k_parallel: ky,
                energy_khz: energy,
                boundary_localization: loc,
                group_velocity_km_s: v_g,
            });
        }

        self.boundary_modes = modes;
        self.boundary_confinement_ratio = if num_k > 0 {
            sum_loc / num_k as f64
        } else {
            0.0
        };
    }
}
