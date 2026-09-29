//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic metasurface holography and dynamic phonon routing across Rayon workers.

use phonon_solver::acoustic_metasurface_holography::MetasurfaceBenchmarkRunner;

#[test]
fn test_10k_acoustic_metasurface_parallel_sweep() {
    let cycles = 10_000;
    let result = MetasurfaceBenchmarkRunner::run_benchmark(cycles);

    assert_eq!(
        result.total_cycles, cycles,
        "Total benchmark cycles must equal requested 10,000"
    );

    println!(
        "Metasurface Holography 10k Sweep: eff={:.4} (min {:.4}), xtalk={:.2} dB (max {:.2} dB), lat={:.4} ns (max {:.4} ns), IL={:.4} dB (max {:.4} dB), fid={:.4} (min {:.4}), compliance={:.1}%, throughput={:.2} sweeps/sec",
        result.mean_beam_steering_efficiency,
        result.min_beam_steering_efficiency,
        result.mean_inter_channel_crosstalk_db,
        result.max_inter_channel_crosstalk_db,
        result.mean_reconfiguration_latency_ns,
        result.max_reconfiguration_latency_ns,
        result.mean_insertion_loss_db,
        result.max_insertion_loss_db,
        result.mean_routing_channel_fidelity,
        result.min_routing_channel_fidelity,
        result.physical_compliance_fraction * 100.0,
        result.throughput_sweeps_per_sec,
    );

    // Verify 100% physical compliance across all parameter sweep variations
    assert!(
        (result.physical_compliance_fraction - 1.0).abs() < 1.0e-6,
        "Physical compliance fraction must be 100.0%, got {:.2}%",
        result.physical_compliance_fraction * 100.0
    );

    // Assert holographic beam steering efficiency >= 88.0% (0.880)
    assert!(
        result.mean_beam_steering_efficiency >= 0.880,
        "Mean beam steering efficiency must be >= 0.880, got {:.4}",
        result.mean_beam_steering_efficiency
    );
    assert!(
        result.min_beam_steering_efficiency >= 0.880,
        "Worst-case beam steering efficiency must be >= 0.880, got {:.4}",
        result.min_beam_steering_efficiency
    );

    // Assert inter-channel acoustic crosstalk <= -35.0 dB
    assert!(
        result.mean_inter_channel_crosstalk_db <= -35.0,
        "Mean inter-channel crosstalk must be <= -35.0 dB, got {:.2} dB",
        result.mean_inter_channel_crosstalk_db
    );
    assert!(
        result.max_inter_channel_crosstalk_db <= -35.0,
        "Worst-case inter-channel crosstalk must be <= -35.0 dB, got {:.2} dB",
        result.max_inter_channel_crosstalk_db
    );

    // Assert dynamic wavefront reconfiguration latency <= 10.0 ns
    assert!(
        result.mean_reconfiguration_latency_ns <= 10.0,
        "Mean reconfiguration latency must be <= 10.0 ns, got {:.4} ns",
        result.mean_reconfiguration_latency_ns
    );
    assert!(
        result.max_reconfiguration_latency_ns <= 10.0,
        "Worst-case reconfiguration latency must be <= 10.0 ns, got {:.4} ns",
        result.max_reconfiguration_latency_ns
    );

    // Assert acoustic transmission insertion loss <= 1.20 dB
    assert!(
        result.mean_insertion_loss_db <= 1.20,
        "Mean insertion loss must be <= 1.20 dB, got {:.4} dB",
        result.mean_insertion_loss_db
    );
    assert!(
        result.max_insertion_loss_db <= 1.20,
        "Worst-case insertion loss must be <= 1.20 dB, got {:.4} dB",
        result.max_insertion_loss_db
    );

    // Assert dynamic multi-channel routing fidelity >= 0.960
    assert!(
        result.mean_routing_channel_fidelity >= 0.960,
        "Mean routing channel fidelity must be >= 0.960, got {:.4}",
        result.mean_routing_channel_fidelity
    );
    assert!(
        result.min_routing_channel_fidelity >= 0.960,
        "Worst-case routing channel fidelity must be >= 0.960, got {:.4}",
        result.min_routing_channel_fidelity
    );

    assert!(
        result.throughput_sweeps_per_sec > 0.0,
        "Throughput must be positive, got {:.2} sweeps/sec",
        result.throughput_sweeps_per_sec
    );
}
