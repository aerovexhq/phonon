#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for non-Abelian quantum acoustic Kitaev spin-liquid anyon braiding and
//! Majorana nanoresonator transceivers across multi-threaded Rayon workers.

use phonon_solver::kitaev_spin_liquid_braiding::KitaevSpinLiquidBenchmarkRunner;

#[test]
fn test_10k_kitaev_spin_liquid_braiding_parallel_sweep() {
    let cycles = 10_000;
    let result = KitaevSpinLiquidBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 156 Non-Abelian Quantum Acoustic Kitaev Spin-Liquid Anyon Braiding & Majorana Nanoresonator Transceivers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Majorana Anyon Braiding Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_majorana_anyon_braiding_fidelity,
        result.min_majorana_anyon_braiding_fidelity,
        result.max_majorana_anyon_braiding_fidelity
    );
    println!(
        "Topological Gap Protection (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_topological_gap_protection_mhz,
        result.min_topological_gap_protection_mhz,
        result.max_topological_gap_protection_mhz
    );
    println!(
        "Non-Abelian State Leakage: mean = {:.4e}, min = {:.4e}, max = {:.4e}",
        result.mean_non_abelian_state_leakage,
        result.min_non_abelian_state_leakage,
        result.max_non_abelian_state_leakage
    );
    println!(
        "Inter-Qubit Crosstalk Isolation (dB): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_inter_qubit_crosstalk_isolation_db,
        result.min_inter_qubit_crosstalk_isolation_db,
        result.max_inter_qubit_crosstalk_isolation_db
    );
    println!(
        "Chiral Edge Energy Flux (uW/m^2): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_chiral_edge_energy_flux_uw_per_m2,
        result.min_chiral_edge_energy_flux_uw_per_m2,
        result.max_chiral_edge_energy_flux_uw_per_m2
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
        result.min_majorana_anyon_braiding_fidelity >= 0.9980,
        "Minimum braiding fidelity must be >= 0.9980, got {:.6}",
        result.min_majorana_anyon_braiding_fidelity
    );
    assert!(
        result.min_topological_gap_protection_mhz >= 35.0,
        "Minimum topological gap protection must be >= 35.0 MHz, got {:.4}",
        result.min_topological_gap_protection_mhz
    );
    assert!(
        result.max_non_abelian_state_leakage <= 1.0e-5,
        "Maximum non-Abelian state leakage must be <= 1.0e-5, got {:.4e}",
        result.max_non_abelian_state_leakage
    );
    assert!(
        result.min_inter_qubit_crosstalk_isolation_db >= 48.0,
        "Minimum inter-qubit crosstalk isolation must be >= 48.0 dB, got {:.4}",
        result.min_inter_qubit_crosstalk_isolation_db
    );
    assert!(
        result.min_chiral_edge_energy_flux_uw_per_m2 >= 120.0,
        "Minimum chiral edge energy flux must be >= 120.0 uW/m^2, got {:.4}",
        result.min_chiral_edge_energy_flux_uw_per_m2
    );
}
