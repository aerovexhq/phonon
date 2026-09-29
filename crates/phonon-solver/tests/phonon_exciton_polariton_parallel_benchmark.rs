#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum phonon-exciton polariton condensates and chiral optomechanical
//! polariton transducers across multi-threaded Rayon workers.

use phonon_solver::phonon_exciton_polariton::PolaritonBenchmarkRunner;

#[test]
fn test_10k_phonon_exciton_polariton_parallel_sweep() {
    let cycles = 10_000;
    let result = PolaritonBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 136 Quantum Phonon-Exciton Polaritons & Chiral Transducers Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Quantum State Fidelity: mean = {:.6}, min = {:.6}, max = {:.6}",
        result.mean_quantum_state_fidelity,
        result.min_quantum_state_fidelity,
        result.max_quantum_state_fidelity
    );
    println!(
        "Condensation Threshold Pump (mW): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_condensation_threshold_pump_mw,
        result.min_condensation_threshold_pump_mw,
        result.max_condensation_threshold_pump_mw
    );
    println!(
        "Polariton Coherence Time (ps): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_polariton_coherence_time_ps,
        result.min_polariton_coherence_time_ps,
        result.max_polariton_coherence_time_ps
    );
    println!(
        "Vortex Topological Charge: mean = {:.2}, min = {}, max = {}",
        result.mean_vortex_topological_charge,
        result.min_vortex_topological_charge,
        result.max_vortex_topological_charge
    );
    println!(
        "Optomechanical Coupling Rate (MHz): mean = {:.4}, min = {:.4}, max = {:.4}",
        result.mean_optomechanical_coupling_rate_mhz,
        result.min_optomechanical_coupling_rate_mhz,
        result.max_optomechanical_coupling_rate_mhz
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

    // Assert quantum state fidelity >= 0.9940
    assert!(
        result.mean_quantum_state_fidelity >= 0.9940,
        "Mean state fidelity must be >= 0.9940, got {:.6}",
        result.mean_quantum_state_fidelity
    );
    assert!(
        result.min_quantum_state_fidelity >= 0.9940,
        "Min state fidelity must be >= 0.9940, got {:.6}",
        result.min_quantum_state_fidelity
    );

    // Assert condensation threshold pump <= 1.200 mW
    assert!(
        result.mean_condensation_threshold_pump_mw <= 1.200,
        "Mean condensation threshold must be <= 1.200 mW, got {:.4}",
        result.mean_condensation_threshold_pump_mw
    );
    assert!(
        result.max_condensation_threshold_pump_mw <= 1.200,
        "Max condensation threshold must be <= 1.200 mW, got {:.4}",
        result.max_condensation_threshold_pump_mw
    );

    // Assert polariton coherence time >= 25.0 ps
    assert!(
        result.mean_polariton_coherence_time_ps >= 25.0,
        "Mean coherence time must be >= 25.0 ps, got {:.4}",
        result.mean_polariton_coherence_time_ps
    );
    assert!(
        result.min_polariton_coherence_time_ps >= 25.0,
        "Min coherence time must be >= 25.0 ps, got {:.4}",
        result.min_polariton_coherence_time_ps
    );

    // Assert vortex topological charge == 1
    assert_eq!(
        result.min_vortex_topological_charge, 1,
        "Min vortex topological charge must be 1"
    );
    assert_eq!(
        result.max_vortex_topological_charge, 1,
        "Max vortex topological charge must be 1"
    );

    // Assert optomechanical coupling rate >= 40.0 MHz
    assert!(
        result.mean_optomechanical_coupling_rate_mhz >= 40.0,
        "Mean optomechanical coupling rate must be >= 40.0 MHz, got {:.4}",
        result.mean_optomechanical_coupling_rate_mhz
    );
    assert!(
        result.min_optomechanical_coupling_rate_mhz >= 40.0,
        "Min optomechanical coupling rate must be >= 40.0 MHz, got {:.4}",
        result.min_optomechanical_coupling_rate_mhz
    );
}
