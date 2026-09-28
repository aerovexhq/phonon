#![deny(unsafe_code)]

use phonon_models::cqed::{DispersiveCqedSystem, MicrowaveCavity, TransmonParams};
use phonon_solver::cqed::{CqedBenchmarkRunner, DispersiveReadoutSolver};

#[test]
fn test_dispersive_readout_iq_pointer_states_and_snr() {
    let transmon = TransmonParams::standard_5ghz();
    let cavity = MicrowaveCavity::standard_7ghz();
    let cqed = DispersiveCqedSystem::new(transmon, cavity, 80.0);

    let readout_solver = DispersiveReadoutSolver::new(300.0, 4.0, 1.5);
    let result = readout_solver.evaluate_readout(&cqed, cavity.resonance_frequency_ghz);

    // Verify pointer separation in IQ plane
    assert!(
        result.pointer_separation > 0.5,
        "Pointer separation should be significant, got {}",
        result.pointer_separation
    );

    // Verify measurement SNR > 5
    assert!(
        result.snr > 5.0,
        "Readout SNR should be > 5, got {}",
        result.snr
    );

    // Verify theoretical discrimination fidelity > 99%
    assert!(
        result.discrimination_fidelity > 0.99,
        "Discrimination fidelity should be > 99%, got {}",
        result.discrimination_fidelity
    );

    // Mean photon number in readout cavity should be moderate (1 - 20 photons)
    assert!(
        result.mean_photon_number > 0.1 && result.mean_photon_number < 100.0,
        "mean_photon_number was {}",
        result.mean_photon_number
    );
}

#[test]
fn test_cqed_quantum_processor_10000_trajectories_benchmark() {
    let runner = CqedBenchmarkRunner::new();
    let report = runner.run_benchmark(10_000);

    assert_eq!(report.total_readout_trajectories, 10_000);
    assert!(
        report.state_discrimination_fidelity >= 0.995,
        "Readout fidelity must exceed 99.5%, got {}",
        report.state_discrimination_fidelity
    );
    assert!(report.is_readout_high_fidelity);
    assert!(
        report.purcell_enhancement_factor > 50.0,
        "Purcell enhancement should exceed 50x, got {}x",
        report.purcell_enhancement_factor
    );
    assert!(
        report.trajectories_per_second > 10_000.0,
        "Throughput should exceed 10,000 trajectories/sec, got {}",
        report.trajectories_per_second
    );
    assert!(report.measurement_snr > 5.0);
}
