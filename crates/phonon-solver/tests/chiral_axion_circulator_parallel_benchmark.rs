#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for chiral acoustic axion electrodynamics and dynamic magnetoelectric
//! phonon circulators across multi-threaded Rayon workers.

use phonon_solver::chiral_axion_circulator::ChiralAxionCirculatorBenchmarkRunner;

#[test]
fn test_10k_chiral_axion_circulator_parallel_sweep() {
    let cycles = 10_000;
    let result = ChiralAxionCirculatorBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 161 Chiral Acoustic Axion Electrodynamics & Dynamic Magnetoelectric Phonon Circulators Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Non-Reciprocal Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_non_reciprocal_isolation_db,
        result.min_non_reciprocal_isolation_db,
        result.max_non_reciprocal_isolation_db
    );
    println!(
        "Axion Polariton Transmission Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_axion_polariton_transmission_fidelity,
        result.min_axion_polariton_transmission_fidelity,
        result.max_axion_polariton_transmission_fidelity
    );
    println!(
        "Circulator Insertion Loss (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_circulator_insertion_loss_db,
        result.min_circulator_insertion_loss_db,
        result.max_circulator_insertion_loss_db
    );
    println!(
        "Axionic Phase Stability Error (rad): mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_axionic_phase_stability_error_rad,
        result.min_axionic_phase_stability_error_rad,
        result.max_axionic_phase_stability_error_rad
    );
    println!(
        "Harmonic Distortion Suppression (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_harmonic_distortion_suppression_db,
        result.min_harmonic_distortion_suppression_db,
        result.max_harmonic_distortion_suppression_db
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
        result.min_non_reciprocal_isolation_db >= 52.0,
        "Minimum non-reciprocal isolation must be >= 52.0 dB, got {:.4}",
        result.min_non_reciprocal_isolation_db
    );
    assert!(
        result.min_axion_polariton_transmission_fidelity >= 0.9970,
        "Minimum axion polariton transmission fidelity must be >= 0.9970, got {:.6}",
        result.min_axion_polariton_transmission_fidelity
    );
    assert!(
        result.max_circulator_insertion_loss_db <= 0.35,
        "Maximum circulator insertion loss must be <= 0.35 dB, got {:.4}",
        result.max_circulator_insertion_loss_db
    );
    assert!(
        result.max_axionic_phase_stability_error_rad <= 0.0018,
        "Maximum axionic phase stability error must be <= 0.0018 rad, got {:.6}",
        result.max_axionic_phase_stability_error_rad
    );
    assert!(
        result.min_harmonic_distortion_suppression_db >= 54.0,
        "Minimum harmonic distortion suppression must be >= 54.0 dB, got {:.4}",
        result.min_harmonic_distortion_suppression_db
    );
}
