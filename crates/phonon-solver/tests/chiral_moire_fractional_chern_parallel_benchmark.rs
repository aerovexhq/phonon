#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for chiral acoustic moire fractional Chern insulators and anyonic interferometric
//! braiding networks across multi-threaded Rayon workers.

use phonon_solver::chiral_moire_fractional_chern::MoireFractionalChernBenchmarkRunner;

#[test]
fn test_10k_chiral_moire_fractional_chern_parallel_sweep() {
    let cycles = 10_000;
    let result = MoireFractionalChernBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 151 Chiral Acoustic Moire Fractional Chern Insulators & Anyonic Interferometric Braiding Networks Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Anyonic Braiding Phase Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_anyonic_braiding_phase_fidelity,
        result.min_anyonic_braiding_phase_fidelity,
        result.max_anyonic_braiding_phase_fidelity
    );
    println!(
        "Moire Flatband Coherence (ms): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_moire_flatband_coherence_ms,
        result.min_moire_flatband_coherence_ms,
        result.max_moire_flatband_coherence_ms
    );
    println!(
        "Non-Adiabatic Braiding Leakage: mean = {:.4e}, min = {:.4e}, max = {:.4e}",
        result.mean_non_adiabatic_braiding_leakage,
        result.min_non_adiabatic_braiding_leakage,
        result.max_non_adiabatic_braiding_leakage
    );
    println!(
        "Quasiparticle Parity Poisoning Immunity (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_quasiparticle_parity_poisoning_immunity_db,
        result.min_quasiparticle_parity_poisoning_immunity_db,
        result.max_quasiparticle_parity_poisoning_immunity_db
    );
    println!(
        "Braiding Phase Stability Error (rad): mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_braiding_phase_stability_error_rad,
        result.min_braiding_phase_stability_error_rad,
        result.max_braiding_phase_stability_error_rad
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
        result.min_anyonic_braiding_phase_fidelity >= 0.9980,
        "Minimum anyonic braiding phase fidelity must be >= 0.9980, got {:.6}",
        result.min_anyonic_braiding_phase_fidelity
    );
    assert!(
        result.min_moire_flatband_coherence_ms >= 15.0,
        "Minimum moire flatband coherence must be >= 15.0 ms, got {:.4} ms",
        result.min_moire_flatband_coherence_ms
    );
    assert!(
        result.max_non_adiabatic_braiding_leakage <= 1.0e-5,
        "Maximum non-adiabatic braiding leakage must be <= 1.0e-5, got {:.4e}",
        result.max_non_adiabatic_braiding_leakage
    );
    assert!(
        result.min_quasiparticle_parity_poisoning_immunity_db >= 42.0,
        "Minimum quasiparticle parity poisoning immunity must be >= 42.0 dB, got {:.4} dB",
        result.min_quasiparticle_parity_poisoning_immunity_db
    );
    assert!(
        result.max_braiding_phase_stability_error_rad <= 0.0020,
        "Maximum braiding phase stability error must be <= 0.0020 rad, got {:.6} rad",
        result.max_braiding_phase_stability_error_rad
    );
}
