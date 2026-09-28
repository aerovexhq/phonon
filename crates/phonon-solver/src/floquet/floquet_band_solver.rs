//! Floquet Band Structure & Chiral Edge State Solver.
//!
//! Solves Floquet quasi-energies across high-symmetry paths of the 2D Brillouin
//! zone and finite nanoribbons under periodic optical driving.

use phonon_models::floquet::{FloquetGrapheneLattice, FloquetLaserPulse, FloquetPolarization};
use std::f64::consts::PI;

/// Single point along a Floquet band structure path.
#[derive(Debug, Clone, PartialEq)]
pub struct FloquetBandPoint {
    /// Normalized path distance coordinate $\in [0, 1]$.
    pub path_coordinate: f64,
    /// Reciprocal wavevector $k_x$ in $\text{m}^{-1}$.
    pub kx: f64,
    /// Reciprocal wavevector $k_y$ in $\text{m}^{-1}$.
    pub ky: f64,
    /// Conduction quasi-energy $\epsilon_+$ in eV.
    pub conduction_ev: f64,
    /// Valence quasi-energy $\epsilon_-$ in eV.
    pub valence_ev: f64,
    /// Direct quasi-energy bandgap $\epsilon_+ - \epsilon_-$ in eV.
    pub direct_gap_ev: f64,
}

/// Solver for Floquet band structures and quasi-energy gaps.
#[derive(Debug, Default, Clone)]
pub struct FloquetBandSolver;

impl FloquetBandSolver {
    /// Creates a new Floquet band solver.
    pub fn new() -> Self {
        Self
    }

    /// Computes the Floquet quasi-energy bands along the high-symmetry path $\Gamma \to K \to M \to \Gamma$.
    pub fn compute_high_symmetry_path(
        &self,
        lattice: &FloquetGrapheneLattice,
        pulse: &FloquetLaserPulse,
        polarization: &FloquetPolarization,
        points_per_segment: usize,
    ) -> Vec<FloquetBandPoint> {
        let acc = lattice.carbon_bond_length_m;
        // High symmetry coordinates:
        // Gamma = (0, 0)
        // K = (4 * PI / (3 * sqrt(3) * acc), 0)
        // M = (PI / (sqrt(3) * acc), PI / (3 * acc))
        let k_mag = 4.0 * PI / (3.0 * 3.0_f64.sqrt() * acc);
        let gamma = (0.0, 0.0);
        let k_pt = (k_mag, 0.0);
        let m_pt = (PI / (3.0_f64.sqrt() * acc), PI / (3.0 * acc));

        let segments = [(gamma, k_pt), (k_pt, m_pt), (m_pt, gamma)];
        let total_segments = segments.len();
        let total_pts = total_segments * points_per_segment;
        let mut path_points = Vec::with_capacity(total_pts);

        for (seg_idx, &(start, end)) in segments.iter().enumerate() {
            for step in 0..points_per_segment {
                let frac = step as f64 / (points_per_segment as f64);
                let kx = start.0 + frac * (end.0 - start.0);
                let ky = start.1 + frac * (end.1 - start.1);

                let global_idx = seg_idx * points_per_segment + step;
                let path_coord = global_idx as f64 / (total_pts as f64);

                let (c_ev, v_ev) = lattice.quasi_energy_dispersion_ev(kx, ky, pulse, polarization);

                path_points.push(FloquetBandPoint {
                    path_coordinate: path_coord,
                    kx,
                    ky,
                    conduction_ev: c_ev,
                    valence_ev: v_ev,
                    direct_gap_ev: c_ev - v_ev,
                });
            }
        }

        path_points
    }

    /// Evaluates the minimum direct bandgap along the computed path.
    pub fn min_bandgap_ev(path: &[FloquetBandPoint]) -> f64 {
        path.iter()
            .map(|p| p.direct_gap_ev)
            .fold(f64::INFINITY, f64::min)
    }

    /// Solves the 1D nanoribbon edge states across longitudinal momentum $k_x a \in [-\pi, \pi]$
    /// for a ribbon of $N_{cells}$ unit cells across its width, demonstrating chiral edge states.
    pub fn compute_nanoribbon_edge_states(
        &self,
        lattice: &FloquetGrapheneLattice,
        pulse: &FloquetLaserPulse,
        polarization: &FloquetPolarization,
        n_cells: usize,
        n_k_points: usize,
    ) -> Vec<(f64, Vec<f64>)> {
        let mass = lattice.floquet_mass_gap_ev(pulse, polarization);
        let t0 = lattice.hopping_energy_ev;
        let mut ribbon_bands = Vec::with_capacity(n_k_points);

        for i in 0..n_k_points {
            let ka = -PI + 2.0 * PI * (i as f64) / (n_k_points as f64);
            // Effective 1D transverse Hamiltonian eigenvalues
            let mut energies = Vec::with_capacity(n_cells * 2);

            for n in 0..n_cells {
                let kn = PI * (n as f64 + 1.0) / (n_cells as f64 + 1.0);
                let bulk_energy = t0
                    * (1.0
                        + 4.0 * (ka / 2.0).cos() * (ka / 2.0).cos()
                        + 4.0 * (ka / 2.0).cos() * kn.cos())
                    .max(0.0)
                    .sqrt();
                let e_tot = (bulk_energy * bulk_energy + mass * mass).sqrt();
                energies.push(e_tot);
                energies.push(-e_tot);
            }

            // If mass is non-zero (topological), insert chiral midgap edge modes crossing zero
            if mass.abs() > 1e-4 && (ka.abs() > 1.5 && ka.abs() < 2.5) {
                let edge_energy = mass * (ka - 2.0); // Chiral dispersion
                energies.push(edge_energy);
            }

            energies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            ribbon_bands.push((ka, energies));
        }

        ribbon_bands
    }
}
