#![deny(unsafe_code)]

//! Physical parameters and DSP metrics for the Phonon Interactive
//! Transient Audio DSP Synthesizer & Soundcard Driver.

/// Plosive consonant articulation kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlosiveKind {
    /// No plosive articulation active (standard phonation).
    #[default]
    None,
    /// Bilabial plosive stop /p/ (labial closure, low-frequency burst).
    BilabialP,
    /// Alveolar plosive stop /t/ (tongue-tip to alveolar ridge, high-frequency burst).
    AlveolarT,
    /// Velar plosive stop /k/ (tongue body to velum/soft palate, mid-frequency burst).
    VelarK,
}

/// Operational parameters configuring the interactive audio DSP synthesis engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioDspSynthParams {
    /// Audio digital-to-analog converter sampling rate in Hertz (clamped [16000.0, 192000.0], default 48000.0 Hz).
    pub sampling_rate_hz: f64,
    /// Soundcard driver hardware buffer size in frames (clamped [64, 2048], default 256 frames).
    pub buffer_size_frames: usize,
    /// Cardinal vowel target index (clamped [0, 4]: 0 = /a/, 1 = /i/, 2 = /u/, 3 = /e/, 4 = /o/).
    pub vowel_target_idx: usize,
    /// Vowel vocal-tract articulation interpolation rate in s^-1 (clamped [0.1, 50.0], default 10.0 s^-1).
    pub morph_interpolation_speed: f64,
    /// Active or triggered plosive articulation gesture.
    pub plosive_kind: PlosiveKind,
    /// Normalized transient plosive burst aerodynamic intensity (clamped [0.0, 1.0], default 0.8).
    pub burst_intensity: f64,
    /// Master audio output gain scalar applied before soft limiting (clamped [0.0, 2.0], default 1.0).
    pub audio_gain: f64,
}

impl Default for AudioDspSynthParams {
    fn default() -> Self {
        Self {
            sampling_rate_hz: 48000.0,
            buffer_size_frames: 256,
            vowel_target_idx: 0,
            morph_interpolation_speed: 10.0,
            plosive_kind: PlosiveKind::None,
            burst_intensity: 0.8,
            audio_gain: 1.0,
        }
    }
}

impl AudioDspSynthParams {
    /// Creates a new audio DSP parameter set with boundary clamping.
    pub fn new(
        sampling_rate_hz: f64,
        buffer_size_frames: usize,
        vowel_target_idx: usize,
        morph_interpolation_speed: f64,
        plosive_kind: PlosiveKind,
        burst_intensity: f64,
        audio_gain: f64,
    ) -> Self {
        Self {
            sampling_rate_hz: sampling_rate_hz.clamp(16000.0, 192000.0),
            buffer_size_frames: buffer_size_frames.clamp(64, 2048),
            vowel_target_idx: vowel_target_idx.min(4),
            morph_interpolation_speed: morph_interpolation_speed.clamp(0.1, 50.0),
            plosive_kind,
            burst_intensity: burst_intensity.clamp(0.0, 1.0),
            audio_gain: audio_gain.clamp(0.0, 2.0),
        }
    }

    /// Computes nominal hardware buffer latency in milliseconds.
    pub fn latency_ms(&self) -> f64 {
        (self.buffer_size_frames as f64 / self.sampling_rate_hz) * 1000.0
    }
}

/// Evaluated performance telemetry and DSP metrics from audio synthesis and soundcard streaming.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AudioDspSynthMetrics {
    /// Soundcard driver buffering latency in milliseconds.
    pub latency_ms: f64,
    /// Peak absolute audio sample amplitude across rendered frame block.
    pub peak_amplitude: f64,
    /// Root mean square (RMS) energy amplitude across rendered frame block.
    pub rms_amplitude: f64,
    /// Total cumulative count of ring buffer underruns encountered.
    pub ring_buffer_underruns: u64,
    /// Measured audio rendering throughput in frames per second (fps).
    pub audio_stream_throughput_fps: f64,
    /// Flag indicating whether output amplitude reached the limiting threshold.
    pub is_clipping: bool,
}
