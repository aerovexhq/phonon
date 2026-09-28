//! Integration test suite for Phase 41:
//! Multi-Threaded Rayon Benchmark Engine: Optomechanical Quantum Transducers vs Bulk EOMs vs Rare-Earth.

use phonon_solver::quantum::{OptomechanicalBenchmarkRunner, TransducerTechnology};

#[test]
fn test_single_point_technology_comparisons() {
    let temp_20mk = 0.020;
    let pump_power = 0.008; // 8 mW
    let fiber_length = 5.0; // 5 km fiber link

    let pt_omt = OptomechanicalBenchmarkRunner::evaluate_state(
        TransducerTechnology::OptomechanicalTransducer,
        temp_20mk,
        pump_power,
        fiber_length,
    );

    let pt_eom = OptomechanicalBenchmarkRunner::evaluate_state(
        TransducerTechnology::BulkLithiumNiobateEom,
        temp_20mk,
        pump_power,
        fiber_length,
    );

    let pt_re = OptomechanicalBenchmarkRunner::evaluate_state(
        TransducerTechnology::RareEarthIonTransducer,
        temp_20mk,
        pump_power,
        fiber_length,
    );

    // 1. Efficiency hierarchy: OMT (> 50%) >> Bulk EOM (~2.5%) >> Rare-Earth (< 0.1%)
    assert!(
        pt_omt.conversion_efficiency > 0.50,
        "OMT efficiency must exceed 50%, got: {:.2}%",
        pt_omt.conversion_efficiency * 100.0
    );
    assert!(
        pt_eom.conversion_efficiency >= 0.01 && pt_eom.conversion_efficiency <= 0.05,
        "Bulk EOM efficiency should be 1-5%, got: {:.2}%",
        pt_eom.conversion_efficiency * 100.0
    );
    assert!(
        pt_re.conversion_efficiency < 0.001,
        "Rare-Earth efficiency should be < 0.1%, got: {:.4}%",
        pt_re.conversion_efficiency * 100.0
    );

    // 2. Added noise at 20 mK: OMT (< 0.5) << Bulk EOM (> 2.0)
    assert!(
        pt_omt.added_noise_quanta < 0.50,
        "OMT added noise must be < 0.5 quanta, got: {}",
        pt_omt.added_noise_quanta
    );
    assert!(
        pt_eom.added_noise_quanta > 2.0,
        "Bulk EOM added noise should be > 2.0 quanta, got: {}",
        pt_eom.added_noise_quanta
    );

    // 3. Dissipated heat load at 20 mK stage: OMT (< 1 uW) vs Bulk EOM (> 1 mW)
    let omt_heat_microwatts = pt_omt.heat_load_watts * 1e6;
    let eom_heat_microwatts = pt_eom.heat_load_watts * 1e6;
    assert!(
        omt_heat_microwatts < 1.0,
        "OMT heat load must be < 1.0 uW, got: {} uW",
        omt_heat_microwatts
    );
    assert!(
        eom_heat_microwatts > 500.0,
        "Bulk EOM heat load must be > 500 uW, got: {} uW",
        eom_heat_microwatts
    );
    assert!(eom_heat_microwatts / omt_heat_microwatts > 1000.0);

    // 4. Interconnect link fidelity across optical fiber:
    assert!(
        pt_omt.link_fidelity > pt_eom.link_fidelity,
        "OMT fidelity ({}) must exceed EOM ({})",
        pt_omt.link_fidelity,
        pt_eom.link_fidelity
    );
}

#[test]
fn test_parallel_rayon_optomechanical_benchmark_10000_evaluations() {
    let total_evals = 10_000;
    let report = OptomechanicalBenchmarkRunner::run_benchmark(total_evals);

    assert_eq!(report.total_evaluations, total_evals);
    assert!(report.elapsed_ms > 0.0);
    assert!(
        report.evaluations_per_second > 50_000.0,
        "Throughput should exceed 50k evals/sec, got: {:.1} evals/sec",
        report.evaluations_per_second
    );

    // Conversion efficiency comparisons:
    assert!(
        report.omt_mean_efficiency > 0.50,
        "OMT mean efficiency must exceed 50%, got: {:.2}%",
        report.omt_mean_efficiency * 100.0
    );
    assert!(
        report.bulk_eom_mean_efficiency < 0.05,
        "Bulk EOM mean efficiency should be < 5%, got: {:.2}%",
        report.bulk_eom_mean_efficiency * 100.0
    );
    assert!(
        report.rare_earth_mean_efficiency < 0.001,
        "Rare-Earth mean efficiency should be < 0.1%, got: {:.4}%",
        report.rare_earth_mean_efficiency * 100.0
    );
    assert!(
        report.omt_efficiency_advantage_vs_eom > 15.0,
        "OMT efficiency advantage over EOM should be > 15x, got: {:.1}x",
        report.omt_efficiency_advantage_vs_eom
    );

    // Cryogenic added noise at 20 mK:
    assert!(
        report.omt_mean_added_noise_20mk < 0.50,
        "OMT mean added noise at 20 mK must be < 0.50 quanta, got: {}",
        report.omt_mean_added_noise_20mk
    );
    assert!(
        report.bulk_eom_mean_added_noise_20mk > 2.0,
        "Bulk EOM added noise should be > 2.0 quanta, got: {}",
        report.bulk_eom_mean_added_noise_20mk
    );

    // Thermal dissipation at 20 mK stage:
    assert!(
        report.omt_cryo_heat_load_microwatts < 1.0,
        "OMT cryogenic heat load must be < 1.0 uW, got: {} uW",
        report.omt_cryo_heat_load_microwatts
    );
    assert!(
        report.bulk_eom_cryo_heat_load_microwatts > 500.0,
        "Bulk EOM cryogenic heat load should exceed 500 uW, got: {} uW",
        report.bulk_eom_cryo_heat_load_microwatts
    );
    assert!(
        report.heat_load_reduction_factor > 1000.0,
        "Cryogenic heat load reduction factor must exceed 1000x, got: {:.1}x",
        report.heat_load_reduction_factor
    );

    // Superconducting transmon interconnect link fidelity:
    assert!(
        report.omt_mean_link_fidelity > 0.60,
        "OMT link fidelity should be > 0.60, got: {}",
        report.omt_mean_link_fidelity
    );
    assert!(
        report.omt_mean_link_fidelity > report.bulk_eom_mean_link_fidelity,
        "OMT link fidelity must exceed bulk EOM"
    );

    assert!(report.target_performance_verified);
}
