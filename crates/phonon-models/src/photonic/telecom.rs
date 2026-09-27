//! Optical communications and high-speed telemetry analysis.
//!
//! Formulates:
//! - Eye diagram generation and foldings across Unit Intervals (UI).
//! - Eye height, eye width, and root-mean-square (RMS) timing jitter.
//! - Quality factor $Q = \frac{\mu_1 - \mu_0}{\sigma_1 + \sigma_0}$.
//! - Bit Error Rate (BER) evaluation: $\text{BER} = \frac{1}{2} \text{erfc}\left( \frac{Q}{\sqrt{2}} \right)$.
//! - Energy efficiency figure-of-merit in femtojoules per bit ($\text{fJ/bit}$).

/// Statistical metrics of an eye diagram.
#[derive(Debug, Clone, PartialEq)]
pub struct EyeMetrics {
    /// Mean level for logical '1' ($\mu_1$).
    pub mean_one: f64,
    /// Standard deviation for logical '1' ($\sigma_1$).
    pub sigma_one: f64,
    /// Mean level for logical '0' ($\mu_0$).
    pub mean_zero: f64,
    /// Standard deviation for logical '0' ($\sigma_0$).
    pub sigma_zero: f64,
    /// Eye height: $(\mu_1 - 3\sigma_1) - (\mu_0 + 3\sigma_0)$.
    pub eye_height: f64,
    /// Quality factor $Q = \frac{\mu_1 - \mu_0}{\sigma_1 + \sigma_0}$.
    pub q_factor: f64,
    /// Estimated Bit Error Rate (BER) assuming Gaussian noise distributions.
    pub ber: f64,
}

/// Point in a folded eye diagram trace.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EyeSample {
    /// Normalized time within eye interval ($0 \le t_{norm} < 2.0$ for 2-UI display).
    pub time_ui: f64,
    /// Signal value (optical power in Watts or voltage in Volts).
    pub amplitude: f64,
}

/// High-speed digital optical link telemetry analyzer.
pub struct TelecomAnalyzer;

impl TelecomAnalyzer {
    /// Computes statistical eye metrics from samples of logical '1's and '0's:
    pub fn analyze_levels(ones_samples: &[f64], zeros_samples: &[f64]) -> EyeMetrics {
        let (mean_1, std_1) = Self::mean_and_std(ones_samples);
        let (mean_0, std_0) = Self::mean_and_std(zeros_samples);

        let eye_height = (mean_1 - 3.0 * std_1) - (mean_0 + 3.0 * std_0);
        let q_factor = if (std_1 + std_0) > 1e-15 {
            (mean_1 - mean_0).max(0.0) / (std_1 + std_0)
        } else {
            30.0 // Near-infinite Q for noiseless signals
        };

        let ber = Self::ber_from_q(q_factor);

        EyeMetrics {
            mean_one: mean_1,
            sigma_one: std_1,
            mean_zero: mean_0,
            sigma_zero: std_0,
            eye_height: eye_height.max(0.0),
            q_factor,
            ber,
        }
    }

    /// Generates folded eye diagram samples across a 2-UI (two bit period) window.
    pub fn generate_eye_diagram(
        signal_trace: &[(f64, f64)], // (time_s, amplitude)
        bit_rate_bps: f64,
    ) -> Vec<EyeSample> {
        let ui_period = 1.0 / bit_rate_bps.max(1.0);
        let two_ui = 2.0 * ui_period;

        let mut samples = Vec::with_capacity(signal_trace.len());
        for &(t, amp) in signal_trace {
            let folded_time = t.rem_euclid(two_ui);
            let time_ui = folded_time / ui_period; // 0.0 to 2.0
            samples.push(EyeSample {
                time_ui,
                amplitude: amp,
            });
        }
        samples
    }

    /// Evaluates complementary error function $\text{erfc}(x)$ using Chebyshev polynomial approximation
    /// with fractional error $< 1.2 \times 10^{-7}$:
    pub fn erfc(x: f64) -> f64 {
        if x < 0.0 {
            return 2.0 - Self::erfc(-x);
        }
        let z = x.abs();
        let t = 1.0 / (1.0 + 0.5 * z);
        let poly = -z * z - 1.26551223
            + t * (1.00002368
                + t * (0.37409196
                    + t * (0.09678418
                        + t * (-0.18628806
                            + t * (0.27886807
                                + t * (-1.13520398
                                    + t * (1.48851587 + t * (-0.82215223 + t * 0.17087277))))))));

        let ans = t * poly.exp();
        ans.clamp(0.0, 1.0)
    }

    /// Evaluates Bit Error Rate (BER) from $Q$-factor:
    /// $$\text{BER} = \frac{1}{2} \text{erfc}\left( \frac{Q}{\sqrt{2}} \right)$$
    pub fn ber_from_q(q: f64) -> f64 {
        if q <= 0.0 {
            0.5
        } else if q > 12.0 {
            1e-35 // Effectively zero bit errors
        } else {
            0.5 * Self::erfc(q / std::f64::consts::SQRT_2)
        }
    }

    /// Calculates energy efficiency in femtojoules per bit ($\text{fJ/bit}$):
    /// $$E_{bit} = \frac{P_{total}}{R_{bit}} \times 10^{15}$$
    pub fn energy_per_bit_fj(total_power_watts: f64, data_rate_bps: f64) -> f64 {
        (total_power_watts / data_rate_bps.max(1.0)) * 1.0e15
    }

    /// Helper computing mean and sample standard deviation of a slice.
    fn mean_and_std(data: &[f64]) -> (f64, f64) {
        if data.is_empty() {
            return (0.0, 0.0);
        }
        let n = data.len() as f64;
        let mean = data.iter().sum::<f64>() / n;
        let variance = data.iter().map(|&x| (x - mean) * (x - mean)).sum::<f64>() / n.max(1.0);
        (mean, variance.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_erfc_and_ber_standard_values() {
        // erfc(0) = 1.0
        assert!((TelecomAnalyzer::erfc(0.0) - 1.0).abs() < 1e-6);

        // Q = 6 corresponds to BER ~ 1e-9 (telecom standard)
        let ber_q6 = TelecomAnalyzer::ber_from_q(6.0);
        assert!((ber_q6 - 9.86e-10).abs() < 1e-10);

        // Q = 7 corresponds to BER ~ 1.28e-12
        let ber_q7 = TelecomAnalyzer::ber_from_q(7.0);
        assert!((ber_q7 - 1.28e-12).abs() < 1e-13);
    }

    #[test]
    fn test_eye_metrics_computation() {
        let ones = vec![1.02, 0.98, 1.01, 0.99, 1.00];
        let zeros = vec![0.01, 0.02, 0.00, -0.01, 0.00];

        let metrics = TelecomAnalyzer::analyze_levels(&ones, &zeros);
        assert!((metrics.mean_one - 1.0).abs() < 0.01);
        assert!((metrics.mean_zero - 0.004).abs() < 0.01);
        assert!(metrics.q_factor > 10.0);
        assert!(metrics.ber < 1e-20);
    }

    #[test]
    fn test_energy_per_bit_calculation() {
        // 10 mW total link power at 100 Gbps (1e11 bps)
        // E = (1e-2 W / 1e11 bps) * 1e15 = 100 fJ/bit
        let e_bit = TelecomAnalyzer::energy_per_bit_fj(10e-3, 100.0e9);
        assert!((e_bit - 100.0).abs() < 1e-6);
    }
}
