//! Multi-Threaded Parallel Benchmark comparing Molecular Quantum Interference Logic against 3nm GAA CMOS.
//!
//! Evaluates:
//! 1. Switching energy per operation: Molecular (\(< 100\text{ meV}\) / \(< 0.02\text{ aJ}\)) vs 3nm CMOS (\(\sim 0.4\text{ fJ}\) = \(400\text{ aJ}\)).
//! 2. Active and standby power scaling.
//! 3. Volumetric and areal integration density (\(> 10^{13}\text{ gates/cm}^2\) vs \(\sim 3\times 10^8\text{ gates/cm}^2\)).
//! 4. Rayon-parallel multi-core arithmetic throughput across 64-bit adder workloads.

use phonon_core::ELEMENTARY_CHARGE;
use phonon_models::molecular::{MolecularGateMetrics, MultiBitMolecularAdder};
use rayon::prelude::*;
use std::time::Instant;

/// Reference physical parameters for industry-standard 3nm Gate-All-Around (GAA) CMOS logic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Cmos3nmBaseline {
    pub supply_voltage_v: f64,
    pub gate_capacitance_f: f64,
    pub switching_energy_joules: f64,
    pub switching_energy_mev: f64,
    pub active_power_per_ghz_w: f64,
    pub standby_leakage_w_per_gate: f64,
    pub gate_footprint_nm2: f64,
    pub integration_density_gates_cm2: f64,
    pub stage_delay_ps: f64,
}

impl Default for Cmos3nmBaseline {
    fn default() -> Self {
        let vdd = 0.70;
        let c_eff = 0.8e-15; // 0.8 fF effective load capacitance in 3nm fanout-of-3
        let e_joules = c_eff * vdd * vdd; // ~3.92e-16 J = 0.392 fJ
        let e_ev = e_joules / ELEMENTARY_CHARGE;
        let e_mev = e_ev * 1000.0;

        Self {
            supply_voltage_v: vdd,
            gate_capacitance_f: c_eff,
            switching_energy_joules: e_joules,
            switching_energy_mev: e_mev,
            active_power_per_ghz_w: e_joules * 1.0e9, // ~0.392 uW per GHz
            standby_leakage_w_per_gate: 1.5e-9,       // 1.5 nW per gate
            gate_footprint_nm2: 25_000.0,             // 0.025 um^2 standard cell
            integration_density_gates_cm2: 3.5e8,     // ~350M gates/cm^2
            stage_delay_ps: 2.5,                      // 2.5 ps FO4 inverter delay
        }
    }
}

/// Comprehensive comparative report between Molecular Logic and 3nm CMOS.
#[derive(Debug, Clone, PartialEq)]
pub struct MolecularComparisonReport {
    pub energy_reduction_factor: f64,
    pub standby_power_reduction_factor: f64,
    pub density_advantage_factor: f64,
    pub molecular_energy_per_gate_mev: f64,
    pub cmos_energy_per_gate_mev: f64,
    pub molecular_density_gates_cm2: f64,
    pub cmos_density_gates_cm2: f64,
    pub batch_operations_completed: usize,
    pub throughput_ops_per_sec: f64,
    pub total_simulated_energy_savings_joules: f64,
    pub molecular_metrics: MolecularGateMetrics,
    pub cmos_baseline: Cmos3nmBaseline,
}

/// Benchmark runner evaluating parallel multi-bit arithmetic workloads.
#[derive(Debug, Clone)]
pub struct MolecularBenchmarkRunner {
    pub v_supply: f64,
    pub cmos_baseline: Cmos3nmBaseline,
}

impl Default for MolecularBenchmarkRunner {
    fn default() -> Self {
        Self {
            v_supply: 0.35,
            cmos_baseline: Cmos3nmBaseline::default(),
        }
    }
}

impl MolecularBenchmarkRunner {
    pub fn new(v_supply: f64) -> Self {
        Self {
            v_supply,
            cmos_baseline: Cmos3nmBaseline::default(),
        }
    }

    /// Runs a parallel arithmetic benchmark computing N 64-bit additions using Rayon.
    pub fn run_parallel_arithmetic_benchmark(
        &self,
        num_operations: usize,
    ) -> MolecularComparisonReport {
        let adder64 = MultiBitMolecularAdder::new(64, self.v_supply);
        let cell_metrics = adder64.cells[0].compute_metrics();

        // Generate deterministic arithmetic test pairs (A, B)
        let test_pairs: Vec<(u64, u64)> = (0..num_operations)
            .map(|i| {
                let a = (i as u64).wrapping_mul(0x9E3779B97F4A7C15);
                let b = (i as u64).wrapping_mul(0xBF58476D1CE4E5B9);
                (a, b)
            })
            .collect();

        let start_time = Instant::now();

        // Execute parallel 64-bit additions across all available CPU cores with Rayon
        let results: Vec<(u64, bool)> = test_pairs
            .par_iter()
            .map(|&(a, b)| {
                // Thread-local arithmetic cell
                adder64.add(a, b, false)
            })
            .collect();

        let elapsed = start_time.elapsed();
        let elapsed_secs = elapsed.as_secs_f64().max(1.0e-9);
        let throughput = (num_operations as f64) / elapsed_secs;

        // Verify correct arithmetic execution
        for (i, &(a, b)) in test_pairs.iter().enumerate() {
            let (expected_sum, expected_carry) = a.overflowing_add(b);
            let (actual_sum, actual_carry) = results[i];
            assert_eq!(
                actual_sum, expected_sum,
                "Arithmetic sum mismatch at index {}",
                i
            );
            assert_eq!(
                actual_carry, expected_carry,
                "Carry mismatch at index {}",
                i
            );
        }

        // Comparative analysis against 3nm CMOS
        // Per-gate energy is full adder cell energy divided by 4 equivalent logic gates (2 XOR + NAND + NOR)
        let mol_energy_mev = cell_metrics.switching_energy_mev / 4.0;
        let cmos_energy_mev = self.cmos_baseline.switching_energy_mev;
        let energy_reduction = cmos_energy_mev / mol_energy_mev.max(1.0e-6);

        let mol_leakage = cell_metrics.standby_leakage_w;
        let cmos_leakage = self.cmos_baseline.standby_leakage_w_per_gate;
        let standby_reduction = cmos_leakage / mol_leakage.max(1.0e-18);

        // Molecular gate density: 1 cm^2 = 1e14 nm^2. With ~2 nm^2 per gate -> 5e13 gates/cm^2
        let mol_area_nm2 = cell_metrics.physical_area_nm2.max(0.5);
        let mol_density = 1.0e14 / mol_area_nm2;
        let density_adv = mol_density / self.cmos_baseline.integration_density_gates_cm2;

        let energy_saved_per_gate_j = (self.cmos_baseline.switching_energy_joules
            - cell_metrics.switching_energy_joules)
            .max(0.0);
        let total_energy_savings = energy_saved_per_gate_j * (num_operations as f64) * 64.0;

        MolecularComparisonReport {
            energy_reduction_factor: energy_reduction,
            standby_power_reduction_factor: standby_reduction,
            density_advantage_factor: density_adv,
            molecular_energy_per_gate_mev: mol_energy_mev,
            cmos_energy_per_gate_mev: cmos_energy_mev,
            molecular_density_gates_cm2: mol_density,
            cmos_density_gates_cm2: self.cmos_baseline.integration_density_gates_cm2,
            batch_operations_completed: num_operations,
            throughput_ops_per_sec: throughput,
            total_simulated_energy_savings_joules: total_energy_savings,
            molecular_metrics: cell_metrics,
            cmos_baseline: self.cmos_baseline,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_molecular_vs_cmos_benchmark() {
        let runner = MolecularBenchmarkRunner::default();
        let report = runner.run_parallel_arithmetic_benchmark(100);

        assert_eq!(report.batch_operations_completed, 100);
        assert!(
            report.molecular_energy_per_gate_mev < 100.0,
            "Molecular gate must achieve sub-100 meV switching, got {:.2} meV",
            report.molecular_energy_per_gate_mev
        );
        assert!(
            report.energy_reduction_factor > 1000.0,
            "Energy reduction factor must exceed 1,000x over 3nm CMOS, got {:.1}x",
            report.energy_reduction_factor
        );
        assert!(
            report.density_advantage_factor > 10_000.0,
            "Density advantage factor must exceed 10,000x over 3nm CMOS, got {:.1}x",
            report.density_advantage_factor
        );
        assert!(report.throughput_ops_per_sec > 0.0);
    }
}
