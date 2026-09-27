//! Integration tests for heterogeneous RISC-V CPU comparative benchmark and Rayon parallel co-optimization.

use phonon_models::hetero::{HeteroMaterialType, ProcessorBlockType};
use phonon_solver::{HeteroCpuBenchmarkRunner, HeteroCpuOptimizer};

#[test]
fn test_hetero_riscv_cpu_benchmark_performance_and_energy() {
    let runner = HeteroCpuBenchmarkRunner::new(0.8);
    let result = runner.run_benchmark(None);

    // 1. Clock frequency F_max speedup must be at least 25%
    assert!(
        result.f_max_speedup_percent >= 25.0,
        "F_max speedup was only {:.2}%, expected >= 25%",
        result.f_max_speedup_percent
    );
    assert!(
        result.hetero_f_max_ghz > result.baseline_f_max_ghz * 1.25,
        "Hetero F_max: {:.2} GHz, Baseline: {:.2} GHz",
        result.hetero_f_max_ghz,
        result.baseline_f_max_ghz
    );

    // 2. Cache static subthreshold leakage power reduction must exceed 60% (IGZO vs Silicon GAA)
    assert!(
        result.cache_leakage_reduction_percent >= 60.0,
        "Cache leakage reduction was only {:.2}%, expected >= 60%",
        result.cache_leakage_reduction_percent
    );
    assert!(
        result.hetero_cache_static_power_mw < result.baseline_cache_static_power_mw * 0.40,
        "Hetero cache static power: {:.4} mW, Baseline: {:.4} mW",
        result.hetero_cache_static_power_mw,
        result.baseline_cache_static_power_mw
    );

    // 3. Peak die junction temperature must remain safe (< 100 °C)
    assert!(
        result.hetero_peak_temp_c < 100.0,
        "Hetero peak temperature {:.2} °C exceeds 100 °C threshold",
        result.hetero_peak_temp_c
    );

    // 4. Energy-Delay-Area Product (EDAP) efficiency improvement
    assert!(
        result.edap_improvement_ratio > 1.0,
        "EDAP improvement ratio {:.2} should be > 1.0",
        result.edap_improvement_ratio
    );

    // 5. Electromigration lifetime of clock spine with CNT bundle
    assert!(
        result.hetero_clock_mttf_years > result.baseline_clock_mttf_years * 10.0,
        "CNT clock MTTF {:.1} yrs should be > 10x Cu MTTF {:.1} yrs",
        result.hetero_clock_mttf_years,
        result.baseline_clock_mttf_years
    );

    // 6. Thermo-mechanical stress check
    assert!(
        result.hetero_max_stress_mpa < 800.0,
        "Hetero max stress {:.1} MPa must be below critical fracture threshold 800 MPa",
        result.hetero_max_stress_mpa
    );
}

#[test]
fn test_rayon_parallel_co_optimizer_multi_objective() {
    let optimizer = HeteroCpuOptimizer::new(0.8);
    let candidates = optimizer.optimize_parallel();

    assert!(
        !candidates.is_empty(),
        "Optimizer should evaluate design space candidates"
    );

    // Highest-merit candidate must be viable
    let best = &candidates[0];
    assert!(
        best.is_viable,
        "Best candidate must satisfy all physical viability constraints"
    );
    assert!(best.merit_score > 0.0);

    // Best candidate should allocate InGaAs or strained Ge to ALU for maximum F_max
    let alu_mat = best.allocation.get(ProcessorBlockType::ExecutionAlu);
    assert!(
        alu_mat == HeteroMaterialType::InGaAsNmos || alu_mat == HeteroMaterialType::StrainedGePmos,
        "Best candidate allocated unexpected ALU material: {:?}",
        alu_mat
    );

    // Best candidate should allocate IGZO to L1 cache to eliminate static subthreshold leakage
    let cache_mat = best.allocation.get(ProcessorBlockType::L1Cache);
    assert_eq!(
        cache_mat,
        HeteroMaterialType::IgzoBeol,
        "Best candidate should allocate IGZO to L1 cache"
    );

    // Best candidate should allocate CNT bundles to clock distribution for electromigration resilience
    let clock_mat = best.allocation.get(ProcessorBlockType::ClockDistribution);
    assert_eq!(
        clock_mat,
        HeteroMaterialType::CntBundleInterconnect,
        "Best candidate should allocate CNT bundles to clock distribution"
    );

    // Verify candidates are sorted descending by merit score
    for i in 1..candidates.len() {
        assert!(
            candidates[i - 1].merit_score >= candidates[i].merit_score,
            "Candidates must be sorted descending by merit score"
        );
    }
}
