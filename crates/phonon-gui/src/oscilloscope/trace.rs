//! Waveform trace storage, min-max decimation, RMS/peak measurements, and FFT spectrum.

use egui::Color32;

/// A single electrical waveform trace (e.g. node voltage V(out) or branch current I(V1)).
#[derive(Debug, Clone, PartialEq)]
pub struct WaveformTrace {
    pub name: String,
    pub color: Color32,
    /// Ordered `[time_seconds, value]` pairs.
    pub samples: Vec<[f64; 2]>,
}

impl WaveformTrace {
    pub fn new(name: &str, color: Color32) -> Self {
        Self {
            name: name.to_string(),
            color,
            samples: Vec::new(),
        }
    }

    pub fn push(&mut self, time: f64, value: f64) {
        self.samples.push([time, value]);
    }

    pub fn clear(&mut self) {
        self.samples.clear();
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Peak-to-peak voltage ($V_{pp} = V_{max} - V_{min}$).
    pub fn v_pp(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for &[_, v] in &self.samples {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }
        max - min
    }

    /// Root-mean-square value ($V_{rms} = \sqrt{\frac{1}{N} \sum V_i^2}$).
    pub fn v_rms(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f64 = self.samples.iter().map(|&[_, v]| v * v).sum();
        (sum_sq / self.samples.len() as f64).sqrt()
    }

    /// Arithmetic mean value ($V_{avg} = \frac{1}{N} \sum V_i$).
    pub fn v_mean(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.samples.iter().map(|&[_, v]| v).sum();
        sum / self.samples.len() as f64
    }

    /// Estimates fundamental frequency via mean-crossing detection.
    pub fn estimate_frequency(&self) -> Option<f64> {
        if self.samples.len() < 10 {
            return None;
        }
        let mean = self.v_mean();
        let mut crossings = Vec::new();

        for i in 0..self.samples.len() - 1 {
            let [t0, v0] = self.samples[i];
            let [t1, v1] = self.samples[i + 1];
            // Rising edge mean crossing
            if v0 <= mean && v1 > mean {
                let frac = (mean - v0) / (v1 - v0).max(1e-18);
                crossings.push(t0 + frac * (t1 - t0));
            }
        }

        if crossings.len() >= 2 {
            let mut periods = Vec::new();
            for i in 0..crossings.len() - 1 {
                periods.push(crossings[i + 1] - crossings[i]);
            }
            let avg_period: f64 = periods.iter().sum::<f64>() / periods.len() as f64;
            if avg_period > 1e-15 {
                return Some(1.0 / avg_period);
            }
        }

        None
    }

    /// High-performance Min-Max decimation for fast GPU rendering.
    /// Preserves all peak extremes while downsampling to `target_points`.
    pub fn min_max_decimate(&self, target_points: usize) -> Vec<[f64; 2]> {
        if self.samples.len() <= target_points || target_points < 4 {
            return self.samples.clone();
        }

        let num_buckets = target_points / 2;
        let bucket_size = self.samples.len() as f64 / num_buckets as f64;
        let mut decimated = Vec::with_capacity(target_points);

        for b in 0..num_buckets {
            let start_idx = (b as f64 * bucket_size).floor() as usize;
            let end_idx = (((b + 1) as f64 * bucket_size).ceil() as usize).min(self.samples.len());

            if start_idx >= end_idx {
                continue;
            }

            let mut min_pt = self.samples[start_idx];
            let mut max_pt = self.samples[start_idx];

            for i in start_idx + 1..end_idx {
                let pt = self.samples[i];
                if pt[1] < min_pt[1] {
                    min_pt = pt;
                }
                if pt[1] > max_pt[1] {
                    max_pt = pt;
                }
            }

            // Order chronologically
            if min_pt[0] <= max_pt[0] {
                decimated.push(min_pt);
                decimated.push(max_pt);
            } else {
                decimated.push(max_pt);
                decimated.push(min_pt);
            }
        }

        decimated
    }

    /// Computes discrete Fourier magnitude spectrum `[frequency_hz, magnitude_db]`.
    pub fn compute_spectrum(&self, n_bins: usize) -> Vec<[f64; 2]> {
        if self.samples.len() < 4 || n_bins < 2 {
            return Vec::new();
        }

        let t_start = self.samples.first().map(|s| s[0]).unwrap_or(0.0);
        let t_end = self.samples.last().map(|s| s[0]).unwrap_or(1.0);
        let t_span = (t_end - t_start).max(1e-15);
        let dt = t_span / (self.samples.len() as f64);
        let f_max = 0.5 / dt; // Nyquist

        let mut spectrum = Vec::with_capacity(n_bins);
        let n_samples = self.samples.len();

        for b in 1..=n_bins {
            let f = (b as f64 / n_bins as f64) * f_max;
            let omega = 2.0 * std::f64::consts::PI * f;

            let mut re = 0.0;
            let mut im = 0.0;

            for &[t, v] in &self.samples {
                let angle = omega * (t - t_start);
                re += v * angle.cos();
                im -= v * angle.sin();
            }

            let mag = (re * re + im * im).sqrt() / (n_samples as f64);
            let mag_db = 20.0 * (mag.max(1e-12)).log10();
            spectrum.push([f, mag_db]);
        }

        spectrum
    }
}
