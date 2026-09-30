#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological acoustic parafermionic fractional Josephson interconnects
//! and non-Abelian quantum logic across multi-threaded Rayon workers.

use phonon_solver::fractional_josephson_parafermion::FractionalJosephsonBenchmarkRunner;

#[test]
fn test_10k_fractional_josephson_parafermion_parallel_sweep() {
    let cycles = 10_000;
    let result = FractionalJosephsonBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 148 Topological Acoustic Parafermionic Fractional Josephson Interconnects Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Fractional Braiding Phase Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_fractional_braiding_phase_fidelity,
        result.min_fractional_braiding_phase_fidelity,
        result.max_fractional_braiding_phase_fidelity
    );
    println!(
        "Fractional Josephson Coherence (ms): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_fractional_josephson_coherence_ms,
        result.min_fractional_josephson_coherence_ms,
        result.max_fractional_josephson_coherence_ms
    );
    println!(
        "Non-Adiabatic Excitation Leakage: mean = {:.4e}, min = {:.4e}, max = {:.4e}",
        result.mean_non_adiabatic_excitation_leakage,
        result.min_non_adiabatic_excitation_leakage,
        result.max_non_adiabatic_excitation_leakage
    );
    println!(
        "Quasiparticle Parity Poisoning Immunity (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_quasiparticle_parity_poisoning_immunity_db,
        result.min_quasiparticle_parity_poisoning_immunity_db,
        result.max_quasiparticle_parity_poisoning_immunity_db
    );
    println!(
        "Fractional Conductance Quantization Error: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_fractional_conductance_quantization_error,
        result.min_fractional_conductance_quantization_error,
        result.max_fractional_conductance_quantization_error
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
        result.min_fractional_braiding_phase_fidelity >= 0.9970,
        "Minimum fractional braiding phase fidelity must be >= 0.9970, got {:.6}",
        result.min_fractional_braiding_phase_fidelity
    );
    assert!(
        result.min_fractional_josephson_coherence_ms >= 10.0,
        "Minimum fractional Josephson coherence must be >= 10.0 ms, got {:.4} ms",
        result.min_fractional_josephson_coherence_ms
    );
    assert!(
        result.max_non_adiabatic_excitation_leakage <= 1.0e-5,
        "Maximum non-adiabatic excitation leakage must be <= 1.0e-5, got {:.4e}",
        result.max_non_adiabatic_excitation_leakage
    );
    assert!(
        result.min_quasiparticle_parity_poisoning_immunity_db >= 40.0,
        "Minimum quasiparticle poisoning immunity must be >= 40.0 dB, got {:.4} dB",
        result.min_quasiparticle_parity_poisoning_immunity_db
    );
    assert!(
        result.max_fractional_conductance_quantization_error <= 0.0030,
        "Maximum fractional conductance quantization error must be <= 0.0030, got {:.6}",
        result.max_fractional_conductance_quantization_error
    );
}
