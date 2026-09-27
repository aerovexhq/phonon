//! Comprehensive benchmark runner comparing uniform Silicon vs heterogeneous CPU architectures.
//!
//! Evaluates:
//! 1. Maximum clock frequency \(F_{max}\) and speedup ratio.
//! 2. Static standby cache leakage reduction via BEOL oxide semiconductors (IGZO).
//! 3. Total core dynamic and static power dissipation.
//! 4. Energy-Delay-Area Product (EDAP) efficiency figure of merit.
//! 5. 2D finite-difference thermal hotspot relief.
//! 6. Thermo-mechanical stress and Black's electromigration wire reliability.

use crate::hetero::timing_closure::TimingPathAnalyzer;
use phonon_models::hetero::{
    BlackElectromigrationModel, BlockAllocationMap, ProcessorBlockType, RiscVFloorplanBuilder,
    ThermalHotspotSolver, ThermoMechanicalStressModel,
};

/// Comprehensive comparative benchmark report between uniform Silicon and Heterogeneous CPU.
#[derive(Debug, Clone)]
pub struct HeteroCpuBenchmarkResult {
    /// Baseline Silicon maximum clock frequency [GHz]
    pub baseline_f_max_ghz: f64,
    /// Heterogeneous processor maximum clock frequency [GHz]
    pub hetero_f_max_ghz: f64,
    /// Clock frequency speedup ratio (hetero / baseline)
    pub f_max_speedup_ratio: f64,
    /// Clock frequency speedup percentage [%]
    pub f_max_speedup_percent: f64,

    /// Baseline Silicon total power dissipation [mW]
    pub baseline_total_power_mw: f64,
    /// Heterogeneous processor total power dissipation [mW]
    pub hetero_total_power_mw: f64,
    /// Power delta ratio (hetero / baseline)
    pub power_ratio: f64,

    /// Baseline L1 cache static subthreshold leakage [mW]
    pub baseline_cache_static_power_mw: f64,
    /// Heterogeneous L1 cache (IGZO) static subthreshold leakage [mW]
    pub hetero_cache_static_power_mw: f64,
    /// Cache standby leakage power reduction percentage [%]
    pub cache_leakage_reduction_percent: f64,

    /// Baseline peak die hotspot temperature [°C]
    pub baseline_peak_temp_c: f64,
    /// Heterogeneous peak die hotspot temperature [°C]
    pub hetero_peak_temp_c: f64,
    /// Hotspot temperature relief delta [°C]
    pub temp_reduction_c: f64,

    /// Baseline Energy-Delay-Area Product (EDAP) [arbitrary units: mW * ns^2 * mm^2]
    pub baseline_edap: f64,
    /// Heterogeneous Energy-Delay-Area Product (EDAP)
    pub hetero_edap: f64,
    /// EDAP efficiency improvement ratio (baseline / hetero)
    pub edap_improvement_ratio: f64,

    /// Baseline clock spine electromigration MTTF [years]
    pub baseline_clock_mttf_years: f64,
    /// Heterogeneous (CNT bundle) clock spine electromigration MTTF [years]
    pub hetero_clock_mttf_years: f64,

    /// Baseline peak thermo-mechanical bi-axial stress [MPa]
    pub baseline_max_stress_mpa: f64,
    /// Heterogeneous peak thermo-mechanical bi-axial stress [MPa]
    pub hetero_max_stress_mpa: f64,
}

/// Benchmark harness evaluating heterogeneous processor macro-architectures.
#[derive(Debug, Clone)]
pub struct HeteroCpuBenchmarkRunner {
    v_dd: f64,
    timing_analyzer: TimingPathAnalyzer,
    thermal_solver: ThermalHotspotSolver,
}

impl Default for HeteroCpuBenchmarkRunner {
    fn default() -> Self {
        let v_dd = 0.8;
        Self {
            v_dd,
            timing_analyzer: TimingPathAnalyzer::new(v_dd),
            thermal_solver: ThermalHotspotSolver::default(),
        }
    }
}

impl HeteroCpuBenchmarkRunner {
    /// Creates a new benchmark runner for supply voltage Vdd.
    pub fn new(v_dd: f64) -> Self {
        Self {
            v_dd: v_dd.max(0.4),
            timing_analyzer: TimingPathAnalyzer::new(v_dd),
            thermal_solver: ThermalHotspotSolver::default(),
        }
    }

    /// Runs a comparative benchmark between standard uniform Silicon and a heterogeneous allocation.
    pub fn run_benchmark(
        &self,
        hetero_allocation: Option<BlockAllocationMap>,
    ) -> HeteroCpuBenchmarkResult {
        let alloc_si = BlockAllocationMap::uniform_silicon();
        let alloc_hetero =
            hetero_allocation.unwrap_or_else(BlockAllocationMap::synthesized_heterogeneous);

        let fp_si = RiscVFloorplanBuilder::build(&alloc_si);
        let fp_hetero = RiscVFloorplanBuilder::build(&alloc_hetero);

        // 1. Timing closure
        let timing_si = self.timing_analyzer.evaluate_floorplan_timing(&fp_si);
        let timing_hetero = self.timing_analyzer.evaluate_floorplan_timing(&fp_hetero);

        let baseline_f_max_ghz = timing_si.f_max_ghz;
        let hetero_f_max_ghz = timing_hetero.f_max_ghz;
        let f_max_speedup_ratio = hetero_f_max_ghz / baseline_f_max_ghz;
        let f_max_speedup_percent = (f_max_speedup_ratio - 1.0) * 100.0;

        // 2. Power dissipation at each core's F_max
        let baseline_total_power_mw = fp_si.total_power_mw(self.v_dd, baseline_f_max_ghz);
        let hetero_total_power_mw = fp_hetero.total_power_mw(self.v_dd, hetero_f_max_ghz);
        let power_ratio = hetero_total_power_mw / baseline_total_power_mw;

        // 3. Cache static standby leakage
        let si_cache = fp_si.get_block(ProcessorBlockType::L1Cache).unwrap();
        let hetero_cache = fp_hetero.get_block(ProcessorBlockType::L1Cache).unwrap();
        let baseline_cache_static_power_mw = si_cache.static_power_mw(self.v_dd);
        let hetero_cache_static_power_mw = hetero_cache.static_power_mw(self.v_dd);
        let cache_leakage_reduction_percent = (1.0
            - (hetero_cache_static_power_mw / baseline_cache_static_power_mw.max(1e-12)))
            * 100.0;

        // 4. 2D Thermal hotspot simulation
        let thermal_si = self
            .thermal_solver
            .solve(&fp_si, self.v_dd, baseline_f_max_ghz);
        let thermal_hetero = self
            .thermal_solver
            .solve(&fp_hetero, self.v_dd, hetero_f_max_ghz);
        let baseline_peak_temp_c = thermal_si.t_peak_c;
        let hetero_peak_temp_c = thermal_hetero.t_peak_c;
        let temp_reduction_c = baseline_peak_temp_c - hetero_peak_temp_c;

        // 5. EDAP calculation
        // EDAP = Power * Delay^2 * Area = (P / f^2) * Area
        let area_si_mm2 = fp_si.total_area_mm2();
        let area_hetero_mm2 = fp_hetero.total_area_mm2();
        let baseline_edap =
            (baseline_total_power_mw / (baseline_f_max_ghz * baseline_f_max_ghz)) * area_si_mm2;
        let hetero_edap =
            (hetero_total_power_mw / (hetero_f_max_ghz * hetero_f_max_ghz)) * area_hetero_mm2;
        let edap_improvement_ratio = baseline_edap / hetero_edap.max(1e-12);

        // 6. Electromigration on clock spine (current density J = 5 MA/cm^2)
        let j_clock = 5.0e6;
        let si_clock = fp_si
            .get_block(ProcessorBlockType::ClockDistribution)
            .unwrap();
        let hetero_clock = fp_hetero
            .get_block(ProcessorBlockType::ClockDistribution)
            .unwrap();
        let baseline_clock_mttf_years = BlackElectromigrationModel::compute_mttf_years(
            si_clock.material,
            j_clock,
            thermal_si.t_peak_k,
        );
        let hetero_clock_mttf_years = BlackElectromigrationModel::compute_mttf_years(
            hetero_clock.material,
            j_clock,
            thermal_hetero.t_peak_k,
        );

        // 7. Thermo-mechanical stress
        let mut baseline_max_stress_mpa: f64 = 0.0;
        for block in &fp_si.blocks {
            let stress = ThermoMechanicalStressModel::evaluate_block_stress(
                block.material,
                thermal_si.t_peak_k,
                300.0,
            );
            if stress.thermal_stress_mpa.abs() > baseline_max_stress_mpa {
                baseline_max_stress_mpa = stress.thermal_stress_mpa.abs();
            }
        }

        let mut hetero_max_stress_mpa: f64 = 0.0;
        for block in &fp_hetero.blocks {
            let stress = ThermoMechanicalStressModel::evaluate_block_stress(
                block.material,
                thermal_hetero.t_peak_k,
                300.0,
            );
            if stress.thermal_stress_mpa.abs() > hetero_max_stress_mpa {
                hetero_max_stress_mpa = stress.thermal_stress_mpa.abs();
            }
        }

        HeteroCpuBenchmarkResult {
            baseline_f_max_ghz,
            hetero_f_max_ghz,
            f_max_speedup_ratio,
            f_max_speedup_percent,
            baseline_total_power_mw,
            hetero_total_power_mw,
            power_ratio,
            baseline_cache_static_power_mw,
            hetero_cache_static_power_mw,
            cache_leakage_reduction_percent,
            baseline_peak_temp_c,
            hetero_peak_temp_c,
            temp_reduction_c,
            baseline_edap,
            hetero_edap,
            edap_improvement_ratio,
            baseline_clock_mttf_years,
            hetero_clock_mttf_years,
            baseline_max_stress_mpa,
            hetero_max_stress_mpa,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hetero_cpu_benchmark_comparison() {
        let runner = HeteroCpuBenchmarkRunner::default();
        let res = runner.run_benchmark(None);

        // F_max speedup must be at least 25%
        assert!(res.f_max_speedup_percent >= 25.0);
        assert!(res.f_max_speedup_ratio >= 1.25);

        // Cache static standby leakage reduction must exceed 90% (IGZO vs Silicon)
        assert!(res.cache_leakage_reduction_percent >= 90.0);
        assert!(res.hetero_cache_static_power_mw < res.baseline_cache_static_power_mw * 0.01);

        // Ballistic CNT clock distribution must improve electromigration MTTF by over 10x
        assert!(res.hetero_clock_mttf_years > res.baseline_clock_mttf_years * 10.0);

        // Peak temperature hotspot should be mitigated or well within safe limits
        assert!(res.hetero_peak_temp_c < 100.0);

        // EDAP should show efficiency improvement
        assert!(res.edap_improvement_ratio > 1.0);
    }
}
