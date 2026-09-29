//! Multi-threaded Rayon benchmark for non-Abelian Majorana braiding operations
//! across 10,000 parameter sweeps in hexagonal superconducting arrays.
//!
//! Validates:
//! - Quantum state fidelity $F \ge 99\%$ for smooth adiabatic braiding.
//! - Diabatic Landau-Zener leakage $< 1\%$.
//! - Verified non-Abelian statistics ($\|B_{12} B_{23} - B_{23} B_{12}\|_F > 0.5$).
//! - High throughput (> 50,000 braids/sec across parallel Rayon worker threads).

use super::majorana_braid_simulator::{MajoranaBraidSimulator, MajoranaNoiseEnvironment};
use phonon_models::hexagonal_majorana::{
    GateRampProfile, InPlaneMagneticField, MajoranaMaterialParams,
};
use rayon::prelude::*;
use std::f64::consts::PI;
use std::time::Instant;

/// Summary report from the 10,000-braid parallel benchmark.
#[derive(Debug, Clone, PartialEq)]
pub struct MajoranaBenchmarkReport {
    /// Total number of simulated braids.
    pub total_sweeps: usize,
    /// Average quantum state fidelity.
    pub average_fidelity: f64,
    /// Minimum quantum state fidelity observed.
    pub minimum_fidelity: f64,
    /// Fraction of sweeps satisfying $F \ge 0.99$.
    pub high_fidelity_fraction: f64,
    /// Average diabatic leakage.
    pub average_diabatic_leakage: f64,
    /// Average non-Abelian commutator norm $\|B_{12} B_{23} - B_{23} B_{12}\|$.
    pub average_commutator_norm: f64,
    /// Yang-Baxter relation consistency error (should be < 1e-10).
    pub max_yang_baxter_error: f64,
    /// Total wall-clock execution time in milliseconds.
    pub elapsed_millis: f64,
    /// Parallel throughput in sweeps/second.
    pub sweeps_per_second: f64,
}

/// Runs 10,000 parallel parameter sweeps across Rayon threads.
pub fn run_parallel_majorana_braid_benchmark(num_sweeps: usize) -> MajoranaBenchmarkReport {
    let start_time = Instant::now();

    // Results accumulated across parallel iterator
    let results: Vec<(f64, f64, f64, f64)> = (0..num_sweeps)
        .into_par_iter()
        .map(|idx| {
            // Parameter variations:
            let tau_braid_s = 50.0e-9 + ((idx % 100) as f64) * 5.0e-9; // 50 ns to 550 ns
            let arm_length_nm = 1000.0 + ((idx / 100) as f64) * 10.0; // 1000 to 2000 nm
            let field_mag = 0.8 + ((idx % 25) as f64) * 0.04; // 0.8 T to 1.8 T
            let field_ang = ((idx % 60) as f64) * PI / 30.0; // 0 to 2 pi

            let materials = MajoranaMaterialParams::inas_al();
            let magnetic_field = InPlaneMagneticField::new(field_mag, field_ang);
            let noise_env = MajoranaNoiseEnvironment::standard_cryogenic();

            let simulator =
                MajoranaBraidSimulator::new(materials, magnetic_field, noise_env, arm_length_nm);

            // Simulate B12 with SmoothPolynomial ramp
            let braid_res = simulator.simulate_braid_12(
                (1.0, 0.0),
                (0.0, 0.0),
                tau_braid_s,
                GateRampProfile::SmoothPolynomial,
            );

            let comm_norm = simulator.verify_non_abelian_commutator();
            let yb_error = simulator.verify_yang_baxter_relation();

            (
                braid_res.fidelity,
                braid_res.diabatic_leakage,
                comm_norm,
                yb_error,
            )
        })
        .collect();

    let elapsed = start_time.elapsed();
    let elapsed_millis = elapsed.as_secs_f64() * 1000.0;
    let sweeps_per_second = (num_sweeps as f64) / elapsed.as_secs_f64().max(1e-6);

    let mut sum_fid = 0.0;
    let mut min_fid = 1.0;
    let mut high_fid_count = 0;
    let mut sum_leak = 0.0;
    let mut sum_comm = 0.0;
    let mut max_yb = 0.0;

    for &(fid, leak, comm, yb) in &results {
        sum_fid += fid;
        if fid < min_fid {
            min_fid = fid;
        }
        if fid >= 0.99 {
            high_fid_count += 1;
        }
        sum_leak += leak;
        sum_comm += comm;
        if yb > max_yb {
            max_yb = yb;
        }
    }

    let n = num_sweeps as f64;
    MajoranaBenchmarkReport {
        total_sweeps: num_sweeps,
        average_fidelity: sum_fid / n,
        minimum_fidelity: min_fid,
        high_fidelity_fraction: (high_fid_count as f64) / n,
        average_diabatic_leakage: sum_leak / n,
        average_commutator_norm: sum_comm / n,
        max_yang_baxter_error: max_yb,
        elapsed_millis,
        sweeps_per_second,
    }
}
