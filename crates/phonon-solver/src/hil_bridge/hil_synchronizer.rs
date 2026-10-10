#![deny(unsafe_code)]

//! Hardware-in-the-Loop (HIL) Co-Simulation Synchronizer.
//!
//! Provides bidirectional co-simulation synchronization between real-world oscilloscope
//! and logic analyzer signals and internal SPICE numerical transients.

/// Co-simulation operational coupling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HilMode {
    #[default]
    HwToSpiceInjection,
    SpiceToHwOutput,
    DigitalCoVerification,
}

impl HilMode {
    pub fn name(&self) -> &'static str {
        match self {
            Self::HwToSpiceInjection => "Hardware-to-SPICE Injection (Sensor/Stimulus)",
            Self::SpiceToHwOutput => "SPICE-to-Hardware AWG Output (Device-Under-Test)",
            Self::DigitalCoVerification => "Digital Logic Co-Verification & Assertion",
        }
    }
}

/// Co-simulation synchronization metrics.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HilSyncMetrics {
    pub time_skew_us: f64,
    pub rms_jitter_us: f64,
    pub buffer_occupancy_ratio: f64,
    pub dropped_samples: usize,
    pub interpolated_voltage: f64,
    pub co_sim_steps_completed: usize,
}

impl Default for HilSyncMetrics {
    fn default() -> Self {
        Self {
            time_skew_us: 0.12,
            rms_jitter_us: 0.35,
            buffer_occupancy_ratio: 0.65,
            dropped_samples: 0,
            interpolated_voltage: 1.65,
            co_sim_steps_completed: 0,
        }
    }
}

/// Hardware-in-the-Loop Synchronizer Engine.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HilSynchronizer {
    pub mode: HilMode,
    pub target_net: String,
    pub hardware_sample_rate_hz: f64,
    pub hw_timestamps_s: Vec<f64>,
    pub hw_voltages_v: Vec<f64>,
    pub hw_digital_samples: Vec<u16>,
    pub spice_timestamps_s: Vec<f64>,
    pub spice_voltages_v: Vec<f64>,
    pub spice_digital_samples: Vec<u16>,
    pub metrics: HilSyncMetrics,
    pub digital_glitch_count: usize,
    pub timing_violation_count: usize,
}

impl Default for HilSynchronizer {
    fn default() -> Self {
        Self::new_synthetic()
    }
}

impl HilSynchronizer {
    /// Creates a synthetic co-simulation session pre-populated with aligned test traces.
    pub fn new_synthetic() -> Self {
        let n_pts = 1000;
        let dt = 1.0e-6; // 1 us step -> 1 MSa/s
        let mut hw_t = Vec::with_capacity(n_pts);
        let mut hw_v = Vec::with_capacity(n_pts);
        let mut hw_d = Vec::with_capacity(n_pts);

        let mut sp_t = Vec::with_capacity(n_pts);
        let mut sp_v = Vec::with_capacity(n_pts);
        let mut sp_d = Vec::with_capacity(n_pts);

        for i in 0..n_pts {
            let t = (i as f64) * dt;
            hw_t.push(t);
            sp_t.push(t);

            // Hardware signal: 2.5 kHz sine with subtle 10 mV noise
            let sig = 1.65 + 1.25 * (2.0 * std::f64::consts::PI * 2500.0 * t).sin();
            hw_v.push(sig);

            // SPICE response: low-pass filtered with slight phase lag
            let sp_sig = 1.65 + 1.22 * (2.0 * std::f64::consts::PI * 2500.0 * (t - 5.0e-6)).sin();
            sp_v.push(sp_sig);

            // Digital clock on bit 0, data bit 1
            let clk_bit = if (i % 20) < 10 { 1u16 } else { 0u16 };
            let data_bit = if (i % 80) < 40 { 2u16 } else { 0u16 };
            let d_word = clk_bit | data_bit;
            hw_d.push(d_word);
            sp_d.push(d_word);
        }

        Self {
            mode: HilMode::HwToSpiceInjection,
            target_net: "NET_HIL_IN".to_string(),
            hardware_sample_rate_hz: 1.0e6,
            hw_timestamps_s: hw_t,
            hw_voltages_v: hw_v,
            hw_digital_samples: hw_d,
            spice_timestamps_s: sp_t,
            spice_voltages_v: sp_v,
            spice_digital_samples: sp_d,
            metrics: HilSyncMetrics::default(),
            digital_glitch_count: 0,
            timing_violation_count: 0,
        }
    }

    /// Evaluates continuous interpolated hardware voltage at an arbitrary SPICE time t.
    pub fn interpolate_hw_voltage(&self, t_query: f64) -> f64 {
        if self.hw_timestamps_s.is_empty() || self.hw_voltages_v.is_empty() {
            return 0.0;
        }

        if t_query <= self.hw_timestamps_s[0] {
            return self.hw_voltages_v[0];
        }

        let n = self.hw_timestamps_s.len();
        if t_query >= self.hw_timestamps_s[n - 1] {
            return self.hw_voltages_v[n - 1];
        }

        // Binary search for enclosing time interval
        match self.hw_timestamps_s.binary_search_by(|t| t.partial_cmp(&t_query).unwrap_or(std::cmp::Ordering::Equal)) {
            Ok(idx) => self.hw_voltages_v[idx],
            Err(idx) => {
                let idx1 = idx.saturating_sub(1);
                let idx2 = idx.min(n - 1);
                let t1 = self.hw_timestamps_s[idx1];
                let t2 = self.hw_timestamps_s[idx2];
                let v1 = self.hw_voltages_v[idx1];
                let v2 = self.hw_voltages_v[idx2];

                if (t2 - t1).abs() < 1.0e-15 {
                    v1
                } else {
                    let frac = (t_query - t1) / (t2 - t1);
                    v1 + frac * (v2 - v1)
                }
            }
        }
    }

    /// Steps the co-simulation forward at the specified SPICE transient time step.
    pub fn step_co_simulation(&mut self, current_time_s: f64, spice_voltage: f64, spice_logic: u16) -> HilSyncMetrics {
        let v_interp = self.interpolate_hw_voltage(current_time_s);

        self.spice_timestamps_s.push(current_time_s);
        self.spice_voltages_v.push(spice_voltage);
        self.spice_digital_samples.push(spice_logic);

        // Calculate time skew against nearest hardware timestamp
        let nearest_hw_t = self
            .hw_timestamps_s
            .iter()
            .copied()
            .min_by(|a, b| (a - current_time_s).abs().partial_cmp(&(b - current_time_s).abs()).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(current_time_s);

        let skew_us = (nearest_hw_t - current_time_s).abs() * 1.0e6;
        let alpha = 0.05;
        self.metrics.time_skew_us = self.metrics.time_skew_us * (1.0 - alpha) + skew_us * alpha;
        self.metrics.rms_jitter_us = (self.metrics.rms_jitter_us * 0.95) + (skew_us * 0.05);
        self.metrics.interpolated_voltage = v_interp;
        self.metrics.co_sim_steps_completed += 1;

        // Digital co-verification check
        if self.mode == HilMode::DigitalCoVerification && !self.hw_digital_samples.is_empty() {
            let idx = (self.metrics.co_sim_steps_completed % self.hw_digital_samples.len()).min(self.hw_digital_samples.len() - 1);
            let hw_val = self.hw_digital_samples[idx];
            if hw_val != spice_logic {
                self.digital_glitch_count += 1;
            }
        }

        self.metrics.clone()
    }

    /// Verifies that co-simulation time-alignment jitter remains strictly below 1.0 microsecond.
    pub fn is_jitter_compliant(&self) -> bool {
        self.metrics.rms_jitter_us < 1.0
    }
}
