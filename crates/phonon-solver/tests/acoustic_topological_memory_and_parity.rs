//! Integration tests for topological quantum memory coherence and parity readout.

use phonon_models::quantum_acoustic_anyons::QuantumAcousticAnyonParams;
use phonon_solver::quantum_acoustic_anyons::QuantumAcousticAnyonSolver;

#[test]
fn test_memory_coherence_and_readout() {
    let params = QuantumAcousticAnyonParams::default();
    let solver = QuantumAcousticAnyonSolver::new(params);
    let metrics = solver.solve();

    assert!(
        metrics.parity_readout_snr_db >= 30.0,
        "Readout SNR {} dB must be >= 30.0 dB",
        metrics.parity_readout_snr_db
    );
    assert!(
        metrics.coherence_time_us >= 50.0,
        "Coherence time {} us must be >= 50.0 us",
        metrics.coherence_time_us
    );
    assert!(
        metrics.effective_topological_gap_mhz >= 15.0,
        "Topological gap {} MHz must be >= 15.0 MHz",
        metrics.effective_topological_gap_mhz
    );
}

#[test]
fn test_temperature_scaling() {
    let temperatures = [15.0, 25.0, 40.0, 50.0];
    for &t in &temperatures {
        let params = QuantumAcousticAnyonParams {
            temperature_mk: t,
            ..Default::default()
        };
        let solver = QuantumAcousticAnyonSolver::new(params);
        let t2 = solver.compute_coherence_time_us();
        let gap = solver.compute_effective_topological_gap_mhz();

        assert!(t2 >= 50.0, "T2 at T={} mK was {} < 50.0 us", t, t2);
        assert!(gap >= 15.0, "Gap at T={} mK was {} < 15.0 MHz", t, gap);
    }
}
