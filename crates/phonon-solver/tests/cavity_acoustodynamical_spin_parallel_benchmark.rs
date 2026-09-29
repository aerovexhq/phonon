#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for cavity quantum acoustodynamical (cQAD) spin-phonon interfaces and chiral
//! squeezed vacuum synthesizers across multi-threaded Rayon workers.

use phonon_solver::cavity_acoustodynamical_spin::CavitySpinBenchmarkRunner;

#[test]
fn test_10k_cavity_acoustodynamical_spin_parallel_sweep() {
    let cycles = 10_000;
    let result = CavitySpinBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 141 Cavity Quantum Acoustodynamical Spin-Phonon Interfaces & Chiral Squeezed Vacuum Synthesizers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Acoustic Quadrature Squeezing (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_acoustic_quadrature_squeezing_db,
        result.min_acoustic_quadrature_squeezing_db,
        result.max_acoustic_quadrature_squeezing_db
    );
    println!(
        "Spin-Phonon Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_spin_phonon_fidelity,
        result.min_spin_phonon_fidelity,
        result.max_spin_phonon_fidelity
    );
    println!(
        "Spin Coherence Lifetime (ms): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_spin_coherence_lifetime_ms,
        result.min_spin_coherence_lifetime_ms,
        result.max_spin_coherence_lifetime_ms
    );
    println!(
        "Thermal Phonon Occupancy: mean = {:.6e}, min = {:.6e}, max = {:.6e}",
        result.mean_thermal_phonon_occupancy,
        result.min_thermal_phonon_occupancy,
        result.max_thermal_phonon_occupancy
    );
    println!(
        "Purcell Enhancement Factor: mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_purcell_enhancement_factor,
        result.min_purcell_enhancement_factor,
        result.max_purcell_enhancement_factor
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
        result.min_acoustic_quadrature_squeezing_db >= 12.0,
        "Minimum acoustic quadrature squeezing must be >= 12.0 dB"
    );
    assert!(
        result.min_spin_phonon_fidelity >= 0.9970,
        "Minimum spin-phonon fidelity must be >= 0.9970"
    );
    assert!(
        result.min_spin_coherence_lifetime_ms >= 50.0,
        "Minimum spin coherence lifetime must be >= 50.0 ms"
    );
    assert!(
        result.max_thermal_phonon_occupancy <= 0.05,
        "Maximum thermal phonon occupancy must be <= 0.05"
    );
    assert!(
        result.min_purcell_enhancement_factor >= 25.0,
        "Minimum Purcell enhancement factor must be >= 25.0"
    );
}
