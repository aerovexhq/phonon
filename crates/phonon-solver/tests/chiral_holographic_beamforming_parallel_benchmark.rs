#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum acoustic metasurface holography and chiral phonon beamforming arrays
//! across multi-threaded Rayon workers.

use phonon_solver::chiral_holographic_beamforming::HolographicBeamformingBenchmarkRunner;

#[test]
fn test_10k_chiral_holographic_beamforming_parallel_sweep() {
    let cycles = 10_000;
    let result = HolographicBeamformingBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 144 Quantum Acoustic Metasurface Holography & Chiral Phonon Beamforming Arrays Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Holographic Reconstruction Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_holographic_reconstruction_fidelity,
        result.min_holographic_reconstruction_fidelity,
        result.max_holographic_reconstruction_fidelity
    );
    println!(
        "Acoustic Beam Directivity (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_acoustic_beam_directivity_db,
        result.min_acoustic_beam_directivity_db,
        result.max_acoustic_beam_directivity_db
    );
    println!(
        "Beam Steering Angular Resolution (deg): mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_beam_steering_angular_resolution_deg,
        result.min_beam_steering_angular_resolution_deg,
        result.max_beam_steering_angular_resolution_deg
    );
    println!(
        "Side-Lobe Suppression Ratio (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_side_lobe_suppression_ratio_db,
        result.min_side_lobe_suppression_ratio_db,
        result.max_side_lobe_suppression_ratio_db
    );
    println!(
        "Acoustic Mode Insertion Loss (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_acoustic_mode_insertion_loss_db,
        result.min_acoustic_mode_insertion_loss_db,
        result.max_acoustic_mode_insertion_loss_db
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
        result.min_holographic_reconstruction_fidelity >= 0.9960,
        "Minimum holographic reconstruction fidelity must be >= 0.9960"
    );
    assert!(
        result.min_acoustic_beam_directivity_db >= 32.0,
        "Minimum acoustic beam directivity must be >= 32.0 dB"
    );
    assert!(
        result.max_beam_steering_angular_resolution_deg <= 0.050,
        "Maximum beam steering angular resolution must be <= 0.050 deg"
    );
    assert!(
        result.min_side_lobe_suppression_ratio_db >= 28.0,
        "Minimum side-lobe suppression ratio must be >= 28.0 dB"
    );
    assert!(
        result.max_acoustic_mode_insertion_loss_db <= 1.20,
        "Maximum acoustic mode insertion loss must be <= 1.20 dB"
    );
}
