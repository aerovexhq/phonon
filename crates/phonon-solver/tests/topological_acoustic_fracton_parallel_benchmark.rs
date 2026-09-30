#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for topological acoustic fracton dynamics and sub-system symmetry-protected
//! phononic multipole routers across multi-threaded Rayon workers.

use phonon_solver::topological_acoustic_fracton::TopologicalAcousticFractonBenchmarkRunner;

#[test]
fn test_10k_topological_acoustic_fracton_parallel_sweep() {
    let cycles = 10_000;
    let result = TopologicalAcousticFractonBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 157 Topological Acoustic Fracton Dynamics & Sub-System Symmetry-Protected Phononic Multipole Routers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Fracton Confinement Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_fracton_confinement_fidelity,
        result.min_fracton_confinement_fidelity,
        result.max_fracton_confinement_fidelity
    );
    println!(
        "Sub-Dimensional Edge Channel Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_sub_dimensional_edge_channel_isolation_db,
        result.min_sub_dimensional_edge_channel_isolation_db,
        result.max_sub_dimensional_edge_channel_isolation_db
    );
    println!(
        "Multipole Charge Conservation Error: mean = {:.4e}, min = {:.4e}, max = {:.4e}",
        result.mean_multipole_charge_conservation_error,
        result.min_multipole_charge_conservation_error,
        result.max_multipole_charge_conservation_error
    );
    println!(
        "Fracton Diffusion Dephasing Rate (Hz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_fracton_diffusion_dephasing_rate_hz,
        result.min_fracton_diffusion_dephasing_rate_hz,
        result.max_fracton_diffusion_dephasing_rate_hz
    );
    println!(
        "Sub-System Boundary Mode Purity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_sub_system_boundary_mode_purity,
        result.min_sub_system_boundary_mode_purity,
        result.max_sub_system_boundary_mode_purity
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
        result.min_fracton_confinement_fidelity >= 0.9970,
        "Minimum fracton confinement fidelity must be >= 0.9970, got {:.6}",
        result.min_fracton_confinement_fidelity
    );
    assert!(
        result.min_sub_dimensional_edge_channel_isolation_db >= 50.0,
        "Minimum sub-dimensional edge channel isolation must be >= 50.0 dB, got {:.4}",
        result.min_sub_dimensional_edge_channel_isolation_db
    );
    assert!(
        result.max_multipole_charge_conservation_error <= 1.0e-5,
        "Maximum multipole charge conservation error must be <= 1.0e-5, got {:.4e}",
        result.max_multipole_charge_conservation_error
    );
    assert!(
        result.max_fracton_diffusion_dephasing_rate_hz <= 25.0,
        "Maximum fracton diffusion dephasing rate must be <= 25.0 Hz, got {:.4}",
        result.max_fracton_diffusion_dephasing_rate_hz
    );
    assert!(
        result.min_sub_system_boundary_mode_purity >= 0.990,
        "Minimum sub-system boundary mode purity must be >= 0.990, got {:.6}",
        result.min_sub_system_boundary_mode_purity
    );
}
