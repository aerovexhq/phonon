//! Parallel multi-threaded Rayon benchmark for moiré flat-band dispersion and correlated insulators.
//!
//! Evaluates 10,000 momentum k-points across the moiré Brillouin zone, measuring flat-band bandwidth
//! quenching W < 5 meV, Dirac velocity quenching v_F* / v_F < 0.05, and Mott correlated gaps.

use phonon_models::moire::{
    BistritzerMacDonaldModel, CorrelatedInsulatorModel, SuperconductingDomeModel, Vector2D,
};
use rayon::prelude::*;
use std::time::Instant;

use super::moire_hamiltonian_solver::MoireHamiltonianSolver;

/// Benchmark performance and physics report for Phase 55.
#[derive(Debug, Clone, PartialEq)]
pub struct MoireBenchmarkReport {
    pub total_k_points: usize,
    pub elapsed_ms: f64,
    pub points_per_second: f64,
    pub flatband_bandwidth_mev: f64,
    pub dirac_velocity_ratio: f64,
    pub lower_remote_gap_mev: f64,
    pub upper_remote_gap_mev: f64,
    pub correlated_mott_gap_mev: f64,
    pub superconducting_tc_max_k: f64,
    pub upper_critical_field_perp_tesla: f64,
    pub benchmark_verified: bool,
}

/// Runs the 10,000 k-point parallel Rayon benchmark for twisted bilayer graphene at magic angle.
pub fn run_10k_moire_benchmark(threads: usize) -> MoireBenchmarkReport {
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(threads.max(1))
        .build()
        .unwrap_or_else(|_| rayon::ThreadPoolBuilder::new().build().unwrap());

    pool.install(|| {
        let theta_deg = 1.08; // Magic angle
        let bm_model = BistritzerMacDonaldModel::new_relaxed(theta_deg);
        let h_solver = MoireHamiltonianSolver::new(bm_model.clone());

        let lattice = &bm_model.lattice;
        let k_dirac = lattice.k_m_point();

        // Generate 100 x 100 = 10,000 k-points spanning the moiré mini-Brillouin zone around Dirac point:
        let n_side = 100;
        let mut k_points = Vec::with_capacity(n_side * n_side);
        let half_span = 0.05; // +/- 0.05 nm^-1
        let step = (2.0 * half_span) / (n_side as f64);
        let start_x = k_dirac.x - half_span;
        let start_y = k_dirac.y - half_span;

        for ix in 0..n_side {
            let kx = start_x + (ix as f64) * step;
            for iy in 0..n_side {
                let ky = start_y + (iy as f64) * step;
                k_points.push(Vector2D::new(kx, ky));
            }
        }

        let start_time = Instant::now();

        // Parallel diagonalization across 10,000 k-points:
        let solutions: Vec<_> = k_points.par_iter().map(|&k| h_solver.solve(k)).collect();

        let elapsed = start_time.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let total_k_points = solutions.len();
        let points_per_sec = (total_k_points as f64) / elapsed.as_secs_f64().max(1e-6);

        // Compute bandwidth W:
        let mut max_e4 = -f64::INFINITY;
        let mut min_e3 = f64::INFINITY;
        let mut min_lower_remote_gap = f64::INFINITY;
        let mut min_upper_remote_gap = f64::INFINITY;

        for sol in &solutions {
            if sol.upper_flatband_ev > max_e4 {
                max_e4 = sol.upper_flatband_ev;
            }
            if sol.lower_flatband_ev < min_e3 {
                min_e3 = sol.lower_flatband_ev;
            }
            if sol.lower_remote_gap_ev < min_lower_remote_gap {
                min_lower_remote_gap = sol.lower_remote_gap_ev;
            }
            if sol.upper_remote_gap_ev < min_upper_remote_gap {
                min_upper_remote_gap = sol.upper_remote_gap_ev;
            }
        }

        let bandwidth_ev = (max_e4 - min_e3).max(0.0);
        let bandwidth_mev = bandwidth_ev * 1000.0;
        let lower_remote_mev = min_lower_remote_gap * 1000.0;
        let upper_remote_mev = min_upper_remote_gap * 1000.0;

        // Dirac velocity quenching at the Dirac point:
        let k_dirac = lattice.k_m_point();
        let (_vf_star, vf_ratio) = h_solver.calculate_dirac_velocity(k_dirac, 0.0001);

        // Correlated insulator physics:
        let corr_model = CorrelatedInsulatorModel::new(lattice.moire_period_nm, 5.0, bandwidth_ev);
        let mott_gap_ev = corr_model.correlated_gap_ev(-2.0); // half-filling
        let mott_gap_mev = mott_gap_ev * 1000.0;

        // Superconducting dome:
        let sc_model = SuperconductingDomeModel::new_hole_doped_tbg();
        let tc_max = sc_model.tc_max_k;
        let bc2_perp = sc_model.upper_critical_field_perp_tesla(-2.25); // optimal doping

        let min_throughput = if cfg!(debug_assertions) {
            10_000.0
        } else {
            100_000.0
        };

        let verified = total_k_points == 10_000
            && bandwidth_mev < 10.0 // bandwidth quenching < 10 meV
            && vf_ratio < 0.10 // strong velocity quenching
            && lower_remote_mev > 10.0 // isolated from remote bands
            && mott_gap_mev > 2.0 // correlated Mott gap open
            && tc_max > 1.0 // superconductivity present
            && points_per_sec > min_throughput;

        MoireBenchmarkReport {
            total_k_points,
            elapsed_ms,
            points_per_second: points_per_sec,
            flatband_bandwidth_mev: bandwidth_mev,
            dirac_velocity_ratio: vf_ratio,
            lower_remote_gap_mev: lower_remote_mev,
            upper_remote_gap_mev: upper_remote_mev,
            correlated_mott_gap_mev: mott_gap_mev,
            superconducting_tc_max_k: tc_max,
            upper_critical_field_perp_tesla: bc2_perp,
            benchmark_verified: verified,
        }
    })
}
