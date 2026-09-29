#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Abelian chiral Majorana bound states in topological phononic superconducting junctions
//! across multi-threaded Rayon workers.

use phonon_solver::phononic_superconducting_majorana::PhononicMajoranaBenchmarkRunner;

#[test]
fn test_10k_phononic_superconducting_majorana_parallel_sweep() {
    let cycles = 10_000;
    let result = PhononicMajoranaBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 145 Non-Abelian Chiral Majorana Bound States in Topological Phononic Superconducting Junctions Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Braiding Phase Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_braiding_phase_fidelity,
        result.min_braiding_phase_fidelity,
        result.max_braiding_phase_fidelity
    );
    println!(
        "Topological Protection Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Non-Adiabatic Leakage Probability: mean = {:.4e}, min = {:.4e}, max = {:.4e}",
        result.mean_non_adiabatic_leakage_probability,
        result.min_non_adiabatic_leakage_probability,
        result.max_non_adiabatic_leakage_probability
    );
    println!(
        "Quasiparticle Poisoning Immunity (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_quasiparticle_poisoning_immunity_db,
        result.min_quasiparticle_poisoning_immunity_db,
        result.max_quasiparticle_poisoning_immunity_db
    );
    println!(
        "Zero-Bias Conductance Peak Error (G_0): mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_zero_bias_conductance_error_g0,
        result.min_zero_bias_conductance_error_g0,
        result.max_zero_bias_conductance_error_g0
    );

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Verify roadmap thresholds
    assert!(
        result.min_braiding_phase_fidelity >= 0.9980,
        "Minimum braiding phase fidelity must be >= 0.9980, got {:.6}",
        result.min_braiding_phase_fidelity
    );
    assert!(
        result.min_topological_protection_gap_mhz >= 22.0,
        "Minimum topological protection gap must be >= 22.0 MHz, got {:.4} MHz",
        result.min_topological_protection_gap_mhz
    );
    assert!(
        result.max_non_adiabatic_leakage_probability <= 1.0e-5,
        "Maximum non-adiabatic leakage probability must be <= 1.0e-5, got {:.4e}",
        result.max_non_adiabatic_leakage_probability
    );
    assert!(
        result.min_quasiparticle_poisoning_immunity_db >= 38.0,
        "Minimum quasiparticle poisoning immunity must be >= 38.0 dB, got {:.4} dB",
        result.min_quasiparticle_poisoning_immunity_db
    );
    assert!(
        result.max_zero_bias_conductance_error_g0 <= 0.0020,
        "Maximum zero-bias conductance error must be <= 0.0020 G_0, got {:.6} G_0",
        result.max_zero_bias_conductance_error_g0
    );
}
