//! Multi-core parallel co-optimizer for heterogeneous material and device allocation.
//!
//! Explores the combinatorial space of block-to-material assignments using Rayon across CPU cores,
//! balancing timing closure (F_max), static leakage, thermal hotspots, thermo-mechanical stress,
//! and Black's electromigration wire lifetime.

use crate::hetero::timing_closure::TimingPathAnalyzer;
use phonon_models::hetero::{
    BlackElectromigrationModel, BlockAllocationMap, HeteroMaterialType, ProcessorBlockType,
    RiscVFloorplanBuilder, ThermalHotspotSolver, ThermoMechanicalStressModel,
};
use rayon::prelude::*;

/// Evaluated candidate in the heterogeneous architecture design space.
#[derive(Debug, Clone)]
pub struct HeteroOptimizationCandidate {
    /// Material allocation map for processor blocks
    pub allocation: BlockAllocationMap,
    /// Maximum clock frequency F_max [GHz]
    pub f_max_ghz: f64,
    /// Total processor power dissipation [mW] at F_max
    pub total_power_mw: f64,
    /// Peak junction temperature [°C]
    pub peak_temp_c: f64,
    /// Maximum thermo-mechanical bi-axial stress [MPa]
    pub max_stress_mpa: f64,
    /// Clock distribution wire electromigration MTTF [years]
    pub clock_mttf_years: f64,
    /// Multi-objective merit score (higher is superior)
    pub merit_score: f64,
    /// True if all thermal, mechanical, and reliability constraints are satisfied
    pub is_viable: bool,
}

/// Multi-threaded optimizer discovering optimal heterogeneous CPU allocations.
#[derive(Debug, Clone)]
pub struct HeteroCpuOptimizer {
    v_dd: f64,
    timing_analyzer: TimingPathAnalyzer,
    thermal_solver: ThermalHotspotSolver,
}

impl Default for HeteroCpuOptimizer {
    fn default() -> Self {
        let v_dd = 0.8;
        Self {
            v_dd,
            timing_analyzer: TimingPathAnalyzer::new(v_dd),
            thermal_solver: ThermalHotspotSolver::default(),
        }
    }
}

impl HeteroCpuOptimizer {
    /// Creates a new optimizer for specified operating voltage.
    pub fn new(v_dd: f64) -> Self {
        Self {
            v_dd,
            timing_analyzer: TimingPathAnalyzer::new(v_dd),
            thermal_solver: ThermalHotspotSolver::default(),
        }
    }

    /// Evaluates a single candidate allocation map across all physical domains.
    pub fn evaluate_candidate(
        &self,
        allocation: BlockAllocationMap,
    ) -> HeteroOptimizationCandidate {
        let floorplan = RiscVFloorplanBuilder::build(&allocation);

        // 1. Timing closure evaluation
        let timing = self.timing_analyzer.evaluate_floorplan_timing(&floorplan);
        let f_max = timing.f_max_ghz;

        // 2. Power dissipation at F_max
        let total_power_mw = floorplan.total_power_mw(self.v_dd, f_max);

        // 3. 2D thermal hotspot diffusion
        let thermal = self.thermal_solver.solve(&floorplan, self.v_dd, f_max);

        // 4. Thermo-mechanical stress evaluation
        let mut max_stress_mpa: f64 = 0.0;
        let mut mechanically_sound = true;

        for block in &floorplan.blocks {
            let stress_rep = ThermoMechanicalStressModel::evaluate_block_stress(
                block.material,
                thermal.t_peak_k,
                300.0,
            );
            if stress_rep.thermal_stress_mpa.abs() > max_stress_mpa {
                max_stress_mpa = stress_rep.thermal_stress_mpa.abs();
            }
            if !stress_rep.is_mechanically_sound {
                mechanically_sound = false;
            }
        }

        // 5. Electromigration on clock spine
        let clock_block = floorplan
            .get_block(ProcessorBlockType::ClockDistribution)
            .unwrap();
        let j_clock = 5.0e6; // 5 MA/cm^2
        let clock_mttf_years = BlackElectromigrationModel::compute_mttf_years(
            clock_block.material,
            j_clock,
            thermal.t_peak_k,
        );

        // Constraints:
        // - Temperature < 105 °C
        // - Mechanically sound (< 800 MPa stress)
        // - Electromigration MTTF >= 10 years
        let is_viable = thermal.t_peak_c < 105.0 && mechanically_sound && clock_mttf_years >= 10.0;

        // Multi-objective merit score:
        // Proportional to F_max^2 (Energy-Delay Product figure of merit), inversely proportional to power and peak temperature
        let merit_score = if is_viable {
            (f_max * f_max * 1000.0)
                / (total_power_mw.max(1.0) * (thermal.t_peak_c.max(30.0) / 50.0))
        } else {
            0.0
        };

        HeteroOptimizationCandidate {
            allocation,
            f_max_ghz: f_max,
            total_power_mw,
            peak_temp_c: thermal.t_peak_c,
            max_stress_mpa,
            clock_mttf_years,
            merit_score,
            is_viable,
        }
    }

    /// Explores the discrete combinatorial design space of material-block allocations in parallel.
    ///
    /// Distributes candidate evaluations across all available CPU cores using Rayon.
    pub fn optimize_parallel(&self) -> Vec<HeteroOptimizationCandidate> {
        let alu_options = [
            HeteroMaterialType::SiliconGaa,
            HeteroMaterialType::StrainedGePmos,
            HeteroMaterialType::InGaAsNmos,
        ];
        let cache_options = [HeteroMaterialType::SiliconGaa, HeteroMaterialType::IgzoBeol];
        let fivr_options = [
            HeteroMaterialType::SiliconGaa,
            HeteroMaterialType::WideBandgapGan,
            HeteroMaterialType::SiliconCarbide,
        ];
        let clock_options = [
            HeteroMaterialType::SiliconGaa,
            HeteroMaterialType::CntBundleInterconnect,
        ];

        // Generate combinatorial allocation space
        let mut candidate_maps = Vec::new();

        for &alu in &alu_options {
            for &cache in &cache_options {
                for &fivr in &fivr_options {
                    for &clock in &clock_options {
                        let mut map = BlockAllocationMap::uniform_silicon();
                        map.assign(ProcessorBlockType::ExecutionAlu, alu);
                        map.assign(ProcessorBlockType::L1Cache, cache);
                        map.assign(ProcessorBlockType::PowerDeliveryFivr, fivr);
                        map.assign(ProcessorBlockType::ClockDistribution, clock);

                        // If ALU is InGaAs or Strained Ge, optimize decode and memory pipeline datapath as well
                        if alu != HeteroMaterialType::SiliconGaa {
                            map.assign(ProcessorBlockType::InstructionDecode, alu);
                            map.assign(ProcessorBlockType::MemoryAccess, alu);
                        }

                        candidate_maps.push(map);
                    }
                }
            }
        }

        // Parallel evaluation across CPU cores
        let mut results: Vec<HeteroOptimizationCandidate> = candidate_maps
            .into_par_iter()
            .map(|map| self.evaluate_candidate(map))
            .collect();

        // Sort descending by merit score
        results.sort_by(|a, b| b.merit_score.partial_cmp(&a.merit_score).unwrap());
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parallel_optimization_finds_superior_hetero_design() {
        let optimizer = HeteroCpuOptimizer::default();
        let candidates = optimizer.optimize_parallel();

        assert!(!candidates.is_empty());
        let best = &candidates[0];

        // Best candidate must be physically viable
        assert!(best.is_viable);
        assert!(best.merit_score > 0.0);

        // Best candidate should select InGaAs or Strained Ge for ALU to boost F_max
        let best_alu = best.allocation.get(ProcessorBlockType::ExecutionAlu);
        assert!(
            best_alu == HeteroMaterialType::InGaAsNmos
                || best_alu == HeteroMaterialType::StrainedGePmos
        );

        // Best candidate should select IGZO for L1 cache to eliminate leakage
        assert_eq!(
            best.allocation.get(ProcessorBlockType::L1Cache),
            HeteroMaterialType::IgzoBeol
        );

        // Best candidate should select CNT bundle for clock distribution to achieve long MTTF
        assert_eq!(
            best.allocation.get(ProcessorBlockType::ClockDistribution),
            HeteroMaterialType::CntBundleInterconnect
        );
    }
}
