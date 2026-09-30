#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Abelian quantum acoustic anyonic braiding in moire skyrmion crystals
//! and chiral topological spin-Peierls transducers across multi-threaded Rayon workers.

use phonon_solver::moire_skyrmion_anyon_braiding::MoireSkyrmionBenchmarkRunner;

#[test]
fn test_10k_moire_skyrmion_anyon_braiding_parallel_sweep() {
    let cycles = 10_000;
    let result = MoireSkyrmionBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 165 Non-Abelian Quantum Acoustic Anyonic Braiding in Moire Skyrmion Crystals & Chiral Topological Spin-Peierls Transducers Benchmark Results ---");
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
        "Topological Protection Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_protection_gap_mhz,
        result.min_topological_protection_gap_mhz,
        result.max_topological_protection_gap_mhz
    );
    println!(
        "Skyrmion Topological Stability Fraction: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_skyrmion_topological_stability_fraction,
        result.min_skyrmion_topological_stability_fraction,
        result.max_skyrmion_topological_stability_fraction
    );
    println!(
        "Inter-Skyrmion Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_skyrmion_crosstalk_isolation_db,
        result.min_inter_skyrmion_crosstalk_isolation_db,
        result.max_inter_skyrmion_crosstalk_isolation_db
    );
    println!(
        "Topological Mode Dephasing Rate (Hz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_mode_dephasing_rate_hz,
        result.min_topological_mode_dephasing_rate_hz,
        result.max_topological_mode_dephasing_rate_hz
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
        result.min_topological_protection_gap_mhz >= 42.0,
        "Minimum topological protection gap must be >= 42.0 MHz, got {:.4} MHz",
        result.min_topological_protection_gap_mhz
    );
    assert!(
        result.min_skyrmion_topological_stability_fraction >= 0.9970,
        "Minimum skyrmion topological stability fraction must be >= 0.9970, got {:.6}",
        result.min_skyrmion_topological_stability_fraction
    );
    assert!(
        result.min_inter_skyrmion_crosstalk_isolation_db >= 53.0,
        "Minimum inter-skyrmion crosstalk isolation must be >= 53.0 dB, got {:.4} dB",
        result.min_inter_skyrmion_crosstalk_isolation_db
    );
    assert!(
        result.max_topological_mode_dephasing_rate_hz <= 16.0,
        "Maximum topological mode dephasing rate must be <= 16.0 Hz, got {:.4} Hz",
        result.max_topological_mode_dephasing_rate_hz
    );
}
