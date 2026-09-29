#![deny(unsafe_code)]

//! Integration Test: 10,000-Sweep Parallel Rayon Benchmark for Quantum Plasmonic Nanocircuits.

use phonon_solver::quantum_plasmonics::run_quantum_plasmonic_benchmark;

#[test]
fn test_quantum_plasmonic_parallel_benchmark_execution() {
    let num_sweeps = 10_000;
    let report = run_quantum_plasmonic_benchmark(num_sweeps);

    assert_eq!(report.total_sweeps, num_sweeps);
    assert!(report.elapsed_seconds > 0.0);
    assert!(
        report.throughput_sweeps_per_sec > 50_000.0,
        "Parallel throughput {} sweeps/sec is below threshold",
        report.throughput_sweeps_per_sec
    );

    // Switching contrast assertions
    assert!(
        report.mean_contrast_db >= 20.0,
        "Mean switching contrast must exceed 20 dB, got {:.2} dB",
        report.mean_contrast_db
    );
    assert!(
        report.high_contrast_compliance_fraction >= 0.99,
        "High contrast compliance fraction must be >= 99%, got {:.2}%",
        report.high_contrast_compliance_fraction * 100.0
    );

    // Sub-diffraction mode volume assertions
    assert!(
        report.mean_normalized_mode_volume < 1.0e-3,
        "Mean normalized mode volume must be sub-diffraction (< 1e-3), got {}",
        report.mean_normalized_mode_volume
    );
    assert_eq!(
        report.subdiffraction_compliance_fraction, 1.0,
        "100% of configurations must be sub-diffraction"
    );

    // Physical coupling metrics
    assert!(
        report.mean_purcell_factor >= 100.0,
        "Mean Purcell enhancement must exceed 100, got {}",
        report.mean_purcell_factor
    );
    assert!(
        report.mean_beta_factor >= 0.90,
        "Mean coupling beta factor must exceed 90%, got {:.4}",
        report.mean_beta_factor
    );
    assert!(
        report.mean_blueshift_ghz > 0.0,
        "Mean non-local blueshift must be positive"
    );
}
