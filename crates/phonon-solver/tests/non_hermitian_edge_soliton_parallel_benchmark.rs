#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Hermitian topological acoustic edge solitons and dissipationless phononic
//! shockwave routers across multi-threaded Rayon workers.

use phonon_solver::non_hermitian_edge_soliton::EdgeSolitonBenchmarkRunner;

#[test]
fn test_10k_non_hermitian_edge_soliton_parallel_sweep() {
    let cycles = 10_000;
    let result = EdgeSolitonBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 135 Non-Hermitian Edge Solitons & Shockwave Routers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Soliton Transmission Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_soliton_transmission_fidelity,
        result.min_soliton_transmission_fidelity,
        result.max_soliton_transmission_fidelity
    );
    println!(
        "Harmonic Distortion (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_harmonic_distortion_db,
        result.min_harmonic_distortion_db,
        result.max_harmonic_distortion_db
    );
    println!(
        "Backscattering Immunity (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_backscattering_immunity_db,
        result.min_backscattering_immunity_db,
        result.max_backscattering_immunity_db
    );
    println!(
        "Soliton Pulse Width (ns): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_soliton_pulse_width_ns,
        result.min_soliton_pulse_width_ns,
        result.max_soliton_pulse_width_ns
    );
    println!(
        "Lyapunov Stability Exponent: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_lyapunov_stability_exponent,
        result.min_lyapunov_stability_exponent,
        result.max_lyapunov_stability_exponent
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

    // Assert soliton transmission fidelity >= 0.9920
    assert!(
        result.mean_soliton_transmission_fidelity >= 0.9920,
        "Mean transmission fidelity must be >= 0.9920, got {:.6}",
        result.mean_soliton_transmission_fidelity
    );
    assert!(
        result.min_soliton_transmission_fidelity >= 0.9920,
        "Min transmission fidelity must be >= 0.9920, got {:.6}",
        result.min_soliton_transmission_fidelity
    );

    // Assert harmonic distortion <= -45.0 dB
    assert!(
        result.mean_harmonic_distortion_db <= -45.0,
        "Mean harmonic distortion must be <= -45.0 dB, got {:.4}",
        result.mean_harmonic_distortion_db
    );
    assert!(
        result.max_harmonic_distortion_db <= -45.0,
        "Max harmonic distortion must be <= -45.0 dB, got {:.4}",
        result.max_harmonic_distortion_db
    );

    // Assert backscattering immunity >= 35.0 dB
    assert!(
        result.mean_backscattering_immunity_db >= 35.0,
        "Mean backscattering immunity must be >= 35.0 dB, got {:.4}",
        result.mean_backscattering_immunity_db
    );
    assert!(
        result.min_backscattering_immunity_db >= 35.0,
        "Min backscattering immunity must be >= 35.0 dB, got {:.4}",
        result.min_backscattering_immunity_db
    );

    // Assert soliton pulse width <= 15.0 ns
    assert!(
        result.mean_soliton_pulse_width_ns <= 15.0,
        "Mean soliton pulse width must be <= 15.0 ns, got {:.4}",
        result.mean_soliton_pulse_width_ns
    );
    assert!(
        result.max_soliton_pulse_width_ns <= 15.0,
        "Max soliton pulse width must be <= 15.0 ns, got {:.4}",
        result.max_soliton_pulse_width_ns
    );

    // Assert Lyapunov stability exponent <= 0.050
    assert!(
        result.mean_lyapunov_stability_exponent <= 0.050,
        "Mean Lyapunov stability exponent must be <= 0.050, got {:.6}",
        result.mean_lyapunov_stability_exponent
    );
    assert!(
        result.max_lyapunov_stability_exponent <= 0.050,
        "Max Lyapunov stability exponent must be <= 0.050, got {:.6}",
        result.max_lyapunov_stability_exponent
    );
}
