//! Integration Test Suite: Diamond NV Dynamical Decoupling, Ramsey, Hahn Echo, CPMG & Sub-pT Sensitivity

use approx::assert_relative_eq;
use phonon_models::sensors::{NitrogenIsotope, NvCenter, NvOrientation};
use phonon_solver::sensors::NvSolver;

#[test]
fn test_ramsey_interferometry_and_t2_star_dephasing() {
    let nv = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14)
        .with_coherence_times(1.5e-6, 400.0e-6); // T2* = 1.5 us
    let solver = NvSolver::new(nv);

    // Apply DC field of 50 uT (approx Earth geomagnetic field)
    let b_dc = 50.0e-6; // in Tesla
    let tau_steps: Vec<f64> = (0..200).map(|i| i as f64 * 2.0e-8).collect(); // 0 to 4 us

    let ramsey = solver.simulate_ramsey(b_dc, &tau_steps);
    assert_eq!(ramsey.populations.len(), 200);

    // Initial population at tau = 0 must be 1.0
    assert_relative_eq!(ramsey.populations[0], 1.0, epsilon = 1e-6);

    // Inhomogeneous dephasing: at large tau >> T2*, oscillation amplitude damps to 0.5 (equal mixture)
    let pop_late = ramsey.populations[199];
    assert!(
        (pop_late - 0.5).abs() < 0.15,
        "Population should damp toward 0.5 under Gaussian T2* dephasing: got {}",
        pop_late
    );
}

#[test]
fn test_hahn_echo_refocusing_and_ac_phase_accumulation() {
    let nv = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14)
        .with_coherence_times(1.0e-6, 600.0e-6); // T2 = 600 us
    let solver = NvSolver::new(nv);

    let tau = 5.0e-6; // 5 us (total echo time 10 us)
    let b_ac = 200.0e-9; // 200 nT AC field amplitude

    let echo = solver.simulate_hahn_echo(b_ac, tau);
    assert_relative_eq!(echo.tau_s, tau, epsilon = 1e-12);
    // Resonant AC frequency f_AC = 1 / (2 * tau) = 100 kHz
    assert_relative_eq!(echo.resonant_ac_freq_hz, 100.0e3, epsilon = 1.0);

    // At 10 us with T2 = 600 us, coherence factor should remain high (> 95%)
    assert!(
        echo.coherence_factor > 0.95,
        "Coherence factor should be high: got {}",
        echo.coherence_factor
    );
    assert!(
        echo.accumulated_phase_rad > 0.0,
        "Accumulated phase must be positive"
    );
}

#[test]
fn test_cpmg_coherence_extension_and_frequency_filtering() {
    let base_t2 = 500.0e-6; // 500 us base coherence time
    let nv = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14)
        .with_coherence_times(1.0e-6, base_t2);
    let solver = NvSolver::new(nv);

    let tau = 2.5e-6; // 2.5 us
    let b_ac = 50.0e-9; // 50 nT

    // Test scaling for N = 1, 8, 64, 256 pulses
    let pulse_counts = [1, 8, 64, 256];
    let results = solver.sweep_cpmg_pulses_parallel(&pulse_counts, tau, b_ac);

    assert_eq!(results.len(), 4);

    // Verify N^(2/3) coherence time extension
    for res in &results {
        let expected_t2 = base_t2 * (res.num_pulses as f64).powf(2.0 / 3.0);
        assert_relative_eq!(res.extended_t2_s, expected_t2, epsilon = 1e-9);
        // Center frequency must be 1 / (4 * tau) = 100 kHz
        assert_relative_eq!(res.center_freq_hz, 100.0e3, epsilon = 1.0);
    }

    // For N = 64, extended T2 should be approx 8 ms (16x base T2)
    assert_relative_eq!(results[2].extended_t2_s, 8.0e-3, epsilon = 1e-6);
    // For N = 256, extended T2 should exceed 20 ms
    assert!(
        results[3].extended_t2_s >= 20.0e-3,
        "Extended T2 for N=256 must exceed 20 ms: got {} s",
        results[3].extended_t2_s
    );
}

#[test]
fn test_dynamic_ac_sensitivity_reaches_sub_picotesla() {
    let nv =
        NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14).with_coherence_times(1.0e-6, 1.0e-3); // T2 = 1.0 ms
    let solver = NvSolver::new(nv).with_optics(2.0e7, 0.18); // 2e7 cps, 18% contrast

    // AC sensitivity for 1.0 ms coherence time with 1-second integration
    let sensitivity = solver.ac_magnetic_sensitivity(1.0e-3, 1.0);

    // Under high collection efficiency and millisecond coherence, sensitivity reaches sub-pT/rtHz (< 1.0e-12 T/rtHz)
    assert!(
        sensitivity < 1.0e-12,
        "AC sensitivity must reach sub-picotesla regime (< 1.0 pT/rtHz): got {:.3e} T/rtHz",
        sensitivity
    );
    assert!(sensitivity > 0.0);
}

#[test]
fn test_parallel_frequency_sweep() {
    let nv = NvCenter::new(NvOrientation::V0, NitrogenIsotope::N14);
    let solver = NvSolver::new(nv);

    let freqs: Vec<f64> = (1..=20).map(|i| i as f64 * 50.0e3).collect(); // 50 kHz to 1 MHz
    let results = solver.sweep_cpmg_frequency_parallel(&freqs, 32, 10.0e-9);

    assert_eq!(results.len(), 20);
    for (i, res) in results.iter().enumerate() {
        assert_relative_eq!(res.center_freq_hz, freqs[i], epsilon = 1.0);
        assert_eq!(res.num_pulses, 32);
    }
}
