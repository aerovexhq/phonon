#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Abelian quantum acoustic twisted bilayer topological superfluidity
//! and chiral Majorana vortex networks across multi-threaded Rayon workers.

use phonon_solver::twisted_bilayer_topological_superfluid::TwistedBilayerSuperfluidBenchmarkRunner;

#[test]
fn test_10k_twisted_bilayer_topological_superfluid_parallel_sweep() {
    let cycles = 10_000;
    let result = TwistedBilayerSuperfluidBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 163 Non-Abelian Quantum Acoustic Twisted Bilayer Topological Superfluidity & Chiral Majorana Vortex Networks Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Vortex State Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_vortex_state_fidelity,
        result.min_vortex_state_fidelity,
        result.max_vortex_state_fidelity
    );
    println!(
        "Topological Vortex Pinning Gap (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_vortex_pinning_gap_mhz,
        result.min_topological_vortex_pinning_gap_mhz,
        result.max_topological_vortex_pinning_gap_mhz
    );
    println!(
        "Inter-Vortex Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_vortex_crosstalk_isolation_db,
        result.min_inter_vortex_crosstalk_isolation_db,
        result.max_inter_vortex_crosstalk_isolation_db
    );
    println!(
        "Topological Vortex Dephasing Rate (Hz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_vortex_dephasing_rate_hz,
        result.min_topological_vortex_dephasing_rate_hz,
        result.max_topological_vortex_dephasing_rate_hz
    );
    println!(
        "Chiral Majorana Mode Purity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_chiral_majorana_mode_purity,
        result.min_chiral_majorana_mode_purity,
        result.max_chiral_majorana_mode_purity
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
        result.min_vortex_state_fidelity >= 0.9980,
        "Minimum vortex state fidelity must be >= 0.9980, got {:.6}",
        result.min_vortex_state_fidelity
    );
    assert!(
        result.min_topological_vortex_pinning_gap_mhz >= 40.0,
        "Minimum topological vortex pinning gap must be >= 40.0 MHz, got {:.4}",
        result.min_topological_vortex_pinning_gap_mhz
    );
    assert!(
        result.min_inter_vortex_crosstalk_isolation_db >= 52.0,
        "Minimum inter-vortex crosstalk isolation must be >= 52.0 dB, got {:.4}",
        result.min_inter_vortex_crosstalk_isolation_db
    );
    assert!(
        result.max_topological_vortex_dephasing_rate_hz <= 18.0,
        "Maximum topological vortex dephasing rate must be <= 18.0 Hz, got {:.4}",
        result.max_topological_vortex_dephasing_rate_hz
    );
    assert!(
        result.min_chiral_majorana_mode_purity >= 0.992,
        "Minimum chiral Majorana mode purity must be >= 0.992, got {:.6}",
        result.min_chiral_majorana_mode_purity
    );
}
