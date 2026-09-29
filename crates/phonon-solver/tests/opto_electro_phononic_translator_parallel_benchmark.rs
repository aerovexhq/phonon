#![deny(unsafe_code)]

//! Automated 10,000-sweep parallel benchmark and physical compliance test
//! for quantum opto-electro-phononic frequency translators and millimeter-wave
//! cavity interfaces across multi-threaded Rayon workers.

use phonon_solver::opto_electro_phononic_translator::TranslatorBenchmarkRunner;

#[test]
fn test_10k_opto_electro_phononic_translator_parallel_sweep() {
    let cycles = 10_000;
    let result = TranslatorBenchmarkRunner::run_benchmark(cycles);

    println!("--- Phase 128 Quantum Opto-Electro-Phononic Frequency Translators Benchmark Results ---");
    println!("Total Cycles: {}", result.total_cycles);
    println!("Elapsed Seconds: {:.6} s", result.elapsed_seconds);
    println!("Throughput: {:.2} sweeps/sec", result.throughput_sweeps_per_sec);
    println!(
        "Physical Compliance Fraction: {:.2}%",
        result.physical_compliance_fraction * 100.0
    );
    println!(
        "Transduction Efficiency: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_transduction_efficiency,
        result.min_transduction_efficiency,
        result.max_transduction_efficiency
    );
    println!(
        "Added Thermal Noise (quanta): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_added_thermal_noise_quanta,
        result.min_added_thermal_noise_quanta,
        result.max_added_thermal_noise_quanta
    );
    println!(
        "Conversion Bandwidth (MHz): mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_conversion_bandwidth_mhz,
        result.min_conversion_bandwidth_mhz,
        result.max_conversion_bandwidth_mhz
    );
    println!(
        "Quantum State Transfer Fidelity: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_quantum_state_transfer_fidelity,
        result.min_quantum_state_transfer_fidelity,
        result.max_quantum_state_transfer_fidelity
    );
    println!(
        "Ground-State Cooling Occupancy: mean = {:.5}, min = {:.5}, max = {:.5}",
        result.mean_ground_state_cooling_occupancy,
        result.min_ground_state_cooling_occupancy,
        result.max_ground_state_cooling_occupancy
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

    // Assert transduction efficiency >= 0.800
    assert!(
        result.mean_transduction_efficiency >= 0.800,
        "Mean transduction efficiency must be >= 0.800, got {:.5}",
        result.mean_transduction_efficiency
    );
    assert!(
        result.min_transduction_efficiency >= 0.800,
        "Min transduction efficiency must be >= 0.800, got {:.5}",
        result.min_transduction_efficiency
    );

    // Assert added thermal noise quanta <= 0.100
    assert!(
        result.mean_added_thermal_noise_quanta <= 0.100,
        "Mean added thermal noise quanta must be <= 0.100, got {:.5}",
        result.mean_added_thermal_noise_quanta
    );
    assert!(
        result.max_added_thermal_noise_quanta <= 0.100,
        "Max added thermal noise quanta must be <= 0.100, got {:.5}",
        result.max_added_thermal_noise_quanta
    );

    // Assert conversion bandwidth >= 5.0 MHz
    assert!(
        result.mean_conversion_bandwidth_mhz >= 5.0,
        "Mean conversion bandwidth must be >= 5.0 MHz, got {:.3} MHz",
        result.mean_conversion_bandwidth_mhz
    );
    assert!(
        result.min_conversion_bandwidth_mhz >= 5.0,
        "Min conversion bandwidth must be >= 5.0 MHz, got {:.3} MHz",
        result.min_conversion_bandwidth_mhz
    );

    // Assert quantum state transfer fidelity >= 0.9850
    assert!(
        result.mean_quantum_state_transfer_fidelity >= 0.9850,
        "Mean quantum state transfer fidelity must be >= 0.9850, got {:.5}",
        result.mean_quantum_state_transfer_fidelity
    );
    assert!(
        result.min_quantum_state_transfer_fidelity >= 0.9850,
        "Min quantum state transfer fidelity must be >= 0.9850, got {:.5}",
        result.min_quantum_state_transfer_fidelity
    );

    // Assert ground-state cooling occupancy <= 0.050
    assert!(
        result.mean_ground_state_cooling_occupancy <= 0.050,
        "Mean ground-state cooling occupancy must be <= 0.050, got {:.5}",
        result.mean_ground_state_cooling_occupancy
    );
    assert!(
        result.max_ground_state_cooling_occupancy <= 0.050,
        "Max ground-state cooling occupancy must be <= 0.050, got {:.5}",
        result.max_ground_state_cooling_occupancy
    );
}
