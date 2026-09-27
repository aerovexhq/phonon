//! Integration tests for oscilloscope trace decimation, FFT spectrum, and signal metrics.

use egui::Color32;
use phonon_gui::oscilloscope::WaveformTrace;
use std::f64::consts::PI;

#[test]
fn test_trace_metrics_sine_wave() {
    let mut trace = WaveformTrace::new("Sine1k", Color32::WHITE);

    let freq = 1000.0;
    let amplitude = 2.0;
    let num_samples = 4000;
    let dt = 0.005 / num_samples as f64; // 5 periods

    for i in 0..num_samples {
        let t = i as f64 * dt;
        let v = amplitude * (2.0 * PI * freq * t).sin();
        trace.push(t, v);
    }

    // Vpp should be ~ 4.0 V
    let v_pp = trace.v_pp();
    assert!(
        (v_pp - 4.0).abs() < 0.05,
        "Vpp should be ~4.0, got {}",
        v_pp
    );

    // Vrms should be ~ 2.0 / sqrt(2) = 1.4142 V
    let v_rms = trace.v_rms();
    let expected_rms = amplitude / 2.0f64.sqrt();
    assert!(
        (v_rms - expected_rms).abs() < 0.05,
        "Vrms should be ~{}, got {}",
        expected_rms,
        v_rms
    );

    // Frequency estimate should be ~1000 Hz
    let estimated_freq = trace
        .estimate_frequency()
        .expect("Frequency should be detectable");
    assert!(
        (estimated_freq - freq).abs() < 50.0,
        "Frequency should be ~1000, got {}",
        estimated_freq
    );
}

#[test]
fn test_min_max_decimation() {
    let mut trace = WaveformTrace::new("DenseData", Color32::RED);

    for i in 0..10_000 {
        let t = i as f64 * 1e-6;
        let v = (i as f64 * 0.1).sin();
        trace.push(t, v);
    }

    assert_eq!(trace.len(), 10_000);

    // Decimate to 100 bins (max 200 points)
    let decimated = trace.min_max_decimate(100);
    assert!(decimated.len() <= 200);
    assert!(decimated.len() >= 100);
}

#[test]
fn test_fft_spectrum_peak() {
    let mut trace = WaveformTrace::new("Tone500Hz", Color32::GREEN);

    let freq = 500.0;
    let num_samples = 2048;
    let sample_rate = 10_000.0; // Nyquist = 5000 Hz
    let dt = 1.0 / sample_rate;

    for i in 0..num_samples {
        let t = i as f64 * dt;
        let v = 3.0 * (2.0 * PI * freq * t).sin();
        trace.push(t, v);
    }

    let spectrum = trace.compute_spectrum(64);
    assert!(!spectrum.is_empty());

    // Find bin with maximum magnitude
    let mut max_mag = f64::NEG_INFINITY;
    let mut peak_freq = 0.0;

    for &[f, mag] in &spectrum {
        if mag > max_mag {
            max_mag = mag;
            peak_freq = f;
        }
    }

    // Peak frequency should be close to 500 Hz
    assert!(
        (peak_freq - freq).abs() < 150.0,
        "Peak freq should be near 500 Hz, got {}",
        peak_freq
    );
}
