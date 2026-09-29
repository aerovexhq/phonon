#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for coherent quantum phonon-magnon-polariton transducers and chiral spin-acoustic
//! interfaces across multi-threaded Rayon workers.

use phonon_solver::phonon_magnon_polariton::PolaritonBenchmarkRunner;

#[test]
fn test_10k_phonon_magnon_polariton_parallel_sweep() {
    let cycles = 10_000;
    let result = PolaritonBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 125 Phonon-Magnon-Polariton Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Polariton Cooperativity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_polariton_cooperativity,
        result.min_polariton_cooperativity,
        result.max_polariton_cooperativity
    );
    println!(
        "Bidirectional Transduction Efficiency: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_bidirectional_transduction_efficiency,
        result.min_bidirectional_transduction_efficiency,
        result.max_bidirectional_transduction_efficiency
    );
    println!(
        "Spin-Wave Dephasing Rate (MHz): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_spin_wave_dephasing_rate_mhz,
        result.min_spin_wave_dephasing_rate_mhz,
        result.max_spin_wave_dephasing_rate_mhz
    );
    println!(
        "Chiral Isolation (dB): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_chiral_isolation_db,
        result.min_chiral_isolation_db,
        result.max_chiral_isolation_db
    );
    println!(
        "Single-Quantum Conversion Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_single_quantum_conversion_fidelity,
        result.min_single_quantum_conversion_fidelity,
        result.max_single_quantum_conversion_fidelity
    );

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert polariton cooperativity >= 50.0
    assert!(
        result.mean_polariton_cooperativity >= 50.0,
        "Mean polariton cooperativity must be >= 50.0, got {:.5}",
        result.mean_polariton_cooperativity
    );
    assert!(
        result.min_polariton_cooperativity >= 50.0,
        "Min polariton cooperativity must be >= 50.0, got {:.5}",
        result.min_polariton_cooperativity
    );

    // Assert bidirectional transduction efficiency >= 0.850
    assert!(
        result.mean_bidirectional_transduction_efficiency >= 0.850,
        "Mean transduction efficiency must be >= 0.850, got {:.5}",
        result.mean_bidirectional_transduction_efficiency
    );
    assert!(
        result.min_bidirectional_transduction_efficiency >= 0.850,
        "Min transduction efficiency must be >= 0.850, got {:.5}",
        result.min_bidirectional_transduction_efficiency
    );

    // Assert spin-wave dephasing rate <= 1.00 MHz
    assert!(
        result.mean_spin_wave_dephasing_rate_mhz <= 1.00,
        "Mean spin-wave dephasing rate must be <= 1.00 MHz, got {:.5}",
        result.mean_spin_wave_dephasing_rate_mhz
    );
    assert!(
        result.max_spin_wave_dephasing_rate_mhz <= 1.00,
        "Max spin-wave dephasing rate must be <= 1.00 MHz, got {:.5}",
        result.max_spin_wave_dephasing_rate_mhz
    );

    // Assert chiral isolation >= 30.0 dB
    assert!(
        result.mean_chiral_isolation_db >= 30.0,
        "Mean chiral isolation must be >= 30.0 dB, got {:.5}",
        result.mean_chiral_isolation_db
    );
    assert!(
        result.min_chiral_isolation_db >= 30.0,
        "Min chiral isolation must be >= 30.0 dB, got {:.5}",
        result.min_chiral_isolation_db
    );

    // Assert single-quantum conversion fidelity >= 0.990
    assert!(
        result.mean_single_quantum_conversion_fidelity >= 0.990,
        "Mean conversion fidelity must be >= 0.990, got {:.5}",
        result.mean_single_quantum_conversion_fidelity
    );
    assert!(
        result.min_single_quantum_conversion_fidelity >= 0.990,
        "Min conversion fidelity must be >= 0.990, got {:.5}",
        result.min_single_quantum_conversion_fidelity
    );
}
