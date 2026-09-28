use phonon_solver::diamond_nv::{
    NvMagnetometryBenchmarkConfig, NvMagnetometryBenchmarkReport, NvMagnetometryBenchmarkRunner,
};

#[test]
fn test_diamond_nv_10k_parallel_benchmark() {
    let runner = NvMagnetometryBenchmarkRunner::new();
    let config = NvMagnetometryBenchmarkConfig {
        total_cycles: 10_000,
        ..NvMagnetometryBenchmarkConfig::default()
    };

    let report: NvMagnetometryBenchmarkReport = runner.run(&config);

    assert_eq!(report.total_cycles, 10_000);
    assert!(report.elapsed_ms > 0.0);
    assert!(
        report.cycles_per_sec > 20_000.0,
        "Cycles per second was {}, expected > 20,000",
        report.cycles_per_sec
    );

    // 3D vector reconstruction error must be < 0.1 uT (1e-7 T)
    assert!(
        report.mean_reconstruction_error_t < 1.0e-7,
        "Mean reconstruction error was {} T, expected < 1e-7 T",
        report.mean_reconstruction_error_t
    );
    assert!(
        report.max_reconstruction_error_t < 1.0e-6,
        "Max reconstruction error was {} T, expected < 1e-6 T",
        report.max_reconstruction_error_t
    );

    // AC shot-noise-limited magnetic sensitivity must be < 10 pT / sqrt(Hz) (1e-11 T / sqrt(Hz))
    assert!(
        report.mean_ac_sensitivity_t_per_rt_hz < 1.0e-11,
        "Mean AC sensitivity was {} T/rtHz, expected < 1e-11 T/rtHz",
        report.mean_ac_sensitivity_t_per_rt_hz
    );

    // Nanoscale NMR resonance dip depth assertion: > 0.15
    assert!(
        report.mean_nmr_dip_depth > 0.15,
        "Mean NMR dip depth was {}, expected > 0.15",
        report.mean_nmr_dip_depth
    );
}
