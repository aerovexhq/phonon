#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for chiral phonon-magnon polariton frequency combs and quantum topological
//! acoustomagnonics across multi-threaded Rayon workers.

use phonon_solver::acoustomagnonic_comb::CombBenchmarkRunner;

#[test]
fn test_10k_acoustomagnonic_comb_parallel_sweep() {
    let cycles = 10_000;
    let result = CombBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 133 Chiral Phonon-Magnon Polariton Frequency Combs Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Comb Spectral Span (GHz): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_comb_spectral_span_ghz,
        result.min_comb_spectral_span_ghz,
        result.max_comb_spectral_span_ghz
    );
    println!(
        "Phase Noise at 10 kHz (dBc/Hz): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_phase_noise_at_10khz_dbc,
        result.min_phase_noise_at_10khz_dbc,
        result.max_phase_noise_at_10khz_dbc
    );
    println!(
        "Polariton Conversion Efficiency: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_polariton_conversion_efficiency,
        result.min_polariton_conversion_efficiency,
        result.max_polariton_conversion_efficiency
    );
    println!(
        "Inter-Modal Isolation (dB): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_inter_modal_isolation_db,
        result.min_inter_modal_isolation_db,
        result.max_inter_modal_isolation_db
    );
    println!(
        "Polariton Cooperativity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_polariton_cooperativity,
        result.min_polariton_cooperativity,
        result.max_polariton_cooperativity
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

    // Assert comb spectral span >= 60.0 GHz
    assert!(
        result.mean_comb_spectral_span_ghz >= 60.0,
        "Mean comb spectral span must be >= 60.0 GHz, got {:.5}",
        result.mean_comb_spectral_span_ghz
    );
    assert!(
        result.min_comb_spectral_span_ghz >= 60.0,
        "Min comb spectral span must be >= 60.0 GHz, got {:.5}",
        result.min_comb_spectral_span_ghz
    );

    // Assert phase noise at 10 kHz <= -125.0 dBc/Hz
    assert!(
        result.mean_phase_noise_at_10khz_dbc <= -125.0,
        "Mean phase noise must be <= -125.0 dBc/Hz, got {:.5}",
        result.mean_phase_noise_at_10khz_dbc
    );
    assert!(
        result.max_phase_noise_at_10khz_dbc <= -125.0,
        "Max phase noise must be <= -125.0 dBc/Hz, got {:.5}",
        result.max_phase_noise_at_10khz_dbc
    );

    // Assert polariton conversion efficiency >= 0.880
    assert!(
        result.mean_polariton_conversion_efficiency >= 0.880,
        "Mean polariton conversion efficiency must be >= 0.880, got {:.6}",
        result.mean_polariton_conversion_efficiency
    );
    assert!(
        result.min_polariton_conversion_efficiency >= 0.880,
        "Min polariton conversion efficiency must be >= 0.880, got {:.6}",
        result.min_polariton_conversion_efficiency
    );

    // Assert inter-modal isolation >= 32.0 dB
    assert!(
        result.mean_inter_modal_isolation_db >= 32.0,
        "Mean inter-modal isolation must be >= 32.0 dB, got {:.5}",
        result.mean_inter_modal_isolation_db
    );
    assert!(
        result.min_inter_modal_isolation_db >= 32.0,
        "Min inter-modal isolation must be >= 32.0 dB, got {:.5}",
        result.min_inter_modal_isolation_db
    );

    // Assert polariton cooperativity >= 80.0
    assert!(
        result.mean_polariton_cooperativity >= 80.0,
        "Mean polariton cooperativity must be >= 80.0, got {:.5}",
        result.mean_polariton_cooperativity
    );
    assert!(
        result.min_polariton_cooperativity >= 80.0,
        "Min polariton cooperativity must be >= 80.0, got {:.5}",
        result.min_polariton_cooperativity
    );
}
