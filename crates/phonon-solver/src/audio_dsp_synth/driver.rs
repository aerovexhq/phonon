#![deny(unsafe_code)]

//! Interactive Transient Audio DSP Synthesizer & Soundcard Driver.
//!
//! Monolithically integrates the continuous Port-Hamiltonian fluid-structure-acoustic
//! vocal fold oscillator, Riccati Webster-horn spatial transmission line,
//! continuous cardinal vowel area morpher, plosive transient aerodynamic turbulence engine,
//! and lock-free circular soundcard streaming FIFO buffer.

use phonon_models::audio_dsp_synth::{
    AudioDspSynthMetrics, AudioDspSynthParams, PlosiveKind,
};
use phonon_models::port_hamiltonian::PortHamiltonianAcousticParams;

use crate::port_hamiltonian::audio_engine::PortHamiltonianAudioEngine;
use super::plosive_engine::PlosiveEngine;
use super::ring_buffer::AudioRingBuffer;
use super::vowel_morpher::VowelMorpher;

/// Interactive real-time soundcard audio driver and DSP synthesizer.
#[derive(Debug, Clone, PartialEq)]
pub struct SoundcardAudioDriver {
    /// Synthesis parameters and audio hardware configuration.
    pub params: AudioDspSynthParams,
    /// Low-level continuous Port-Hamiltonian physical acoustic engine.
    pub audio_engine: PortHamiltonianAudioEngine,
    /// Continuous vocal tract cardinal vowel area function morpher.
    pub vowel_morpher: VowelMorpher,
    /// Transient plosive consonant aerodynamic engine (/p/, /t/, /k/).
    pub plosive_engine: PlosiveEngine,
    /// Low-latency circular soundcard streaming ring buffer.
    pub ring_buffer: AudioRingBuffer<f32>,
    /// Discrete integration time step in seconds (1.0 / sampling_rate_hz).
    pub dt: f64,
    /// Total frames synthesized across driver lifetime.
    pub total_rendered_frames: u64,
    /// Peak absolute amplitude observed across synthesis.
    pub peak_amplitude: f64,
    /// Cumulative sum of squared amplitudes for RMS tracking.
    pub sum_sq_amplitude: f64,
    /// Pre-allocated buffer for resampled 24-section horn area profile.
    area_buffer_24: Vec<f64>,
}

impl SoundcardAudioDriver {
    /// Creates and initializes a new soundcard audio driver from synthesis parameters.
    pub fn new(params: AudioDspSynthParams) -> Self {
        let ph_params = PortHamiltonianAcousticParams {
            sampling_rate_hz: params.sampling_rate_hz,
            ..Default::default()
        };
        let mut audio_engine = PortHamiltonianAudioEngine::new(ph_params);
        let vowel_morpher =
            VowelMorpher::new(params.vowel_target_idx, params.morph_interpolation_speed);
        let mut plosive_engine =
            PlosiveEngine::new(params.sampling_rate_hz, params.burst_intensity);

        if params.plosive_kind != PlosiveKind::None {
            plosive_engine.trigger(params.plosive_kind);
        }

        // Initialize horn area profile from target vowel
        let area_buffer_24 = vowel_morpher.interpolate_to_sections(24);
        audio_engine.tract.area_profile.copy_from_slice(&area_buffer_24);

        // Pre-allocate ring buffer with capacity for multiple audio blocks
        let ring_buffer_capacity = (params.buffer_size_frames * 4).max(1024);
        let ring_buffer = AudioRingBuffer::new(ring_buffer_capacity);
        let dt = 1.0 / params.sampling_rate_hz;

        Self {
            params,
            audio_engine,
            vowel_morpher,
            plosive_engine,
            ring_buffer,
            dt,
            total_rendered_frames: 0,
            peak_amplitude: 0.0,
            sum_sq_amplitude: 0.0,
            area_buffer_24,
        }
    }

    /// Evaluates smooth hyperbolic tangent soft limiter:
    /// y = tanh(gain * x)
    ///
    /// Smoothly compresses audio amplitudes into (-1.0, 1.0) without hard clipping harmonics.
    #[inline(always)]
    pub fn soft_limiter(x: f64, gain: f64) -> f32 {
        (gain * x).tanh() as f32
    }

    /// Computes exactly one discrete audio frame at sampling_rate_hz.
    ///
    /// Updates vocal tract morphology, plosive aerodynamics, executes Port-Hamiltonian
    /// acoustic physics, applies soft limiting, and pushes the sample to the ring buffer.
    /// Executes with strictly zero heap allocations.
    #[inline(always)]
    pub fn step_frame(&mut self) -> f32 {
        // 1. Advance vowel morphing interpolation
        self.vowel_morpher.step(self.dt);

        // 2. Resample area profile into 24-section horn without new heap allocations
        let n_curr = self.vowel_morpher.num_sections;
        let curr_areas = self.vowel_morpher.current_areas();
        for j in 0..24 {
            let u = (j as f64 + 0.5) / 24.0;
            let src_idx_f = u * (n_curr as f64) - 0.5;
            let i0 = (src_idx_f.floor() as isize).clamp(0, (n_curr - 1) as isize) as usize;
            let i1 = (i0 + 1).min(n_curr - 1);
            let frac = (src_idx_f - (i0 as f64)).clamp(0.0, 1.0);
            self.area_buffer_24[j] = curr_areas[i0] * (1.0 - frac) + curr_areas[i1] * frac;
        }
        self.audio_engine
            .tract
            .area_profile
            .copy_from_slice(&self.area_buffer_24);

        // 3. Advance plosive aerodynamics and apply occlusion factor
        let subglottal_p = self.audio_engine.subglottal_pressure_pa;
        let plosive_out = self.plosive_engine.step(self.dt, subglottal_p);
        self.audio_engine.set_occlusion_factor(plosive_out.occlusion_factor);

        // 4. Step monolithic Port-Hamiltonian audio-acoustic physics
        let p_lip = self.audio_engine.step();

        // 5. Couple vocal tract lip radiation pressure and plosive release turbulence burst
        // Normalizes nominal lip pressure (approx 50 Pa) to unit audio amplitude
        let raw_audio = (p_lip + plosive_out.burst_pressure) * 0.02;

        // 6. Apply master audio gain and soft tanh limiter
        let sample = Self::soft_limiter(raw_audio, self.params.audio_gain);

        // 7. Push sample to circular audio ring buffer
        self.ring_buffer.push_overwrite(sample);

        // 8. Track cumulative telemetry
        let abs_val = sample.abs() as f64;
        if abs_val > self.peak_amplitude {
            self.peak_amplitude = abs_val;
        }
        self.sum_sq_amplitude += abs_val * abs_val;
        self.total_rendered_frames += 1;

        sample
    }

    /// Fills an audio hardware DMA block buffer.
    ///
    /// Pulls buffered samples from the ring buffer, or generates new frames on-the-fly
    /// if buffer is empty, guaranteeing uninterrupted soundcard streaming.
    pub fn render_block(&mut self, output: &mut [f32]) {
        for sample in output.iter_mut() {
            if self.ring_buffer.available_read() > 0 {
                *sample = self.ring_buffer.pull();
            } else {
                *sample = self.step_frame();
                // Pull the pushed frame to keep write/read pointers aligned
                let _ = self.ring_buffer.pull();
            }
        }
    }

    /// Pulls samples strictly from the ring buffer into an output slice,
    /// triggering underrun tracking if buffer under-fills.
    pub fn pull_from_ring_buffer(&mut self, output: &mut [f32]) -> usize {
        self.ring_buffer.pull_slice(output)
    }

    /// Sets the target vowel index (0: /a/, 1: /i/, 2: /u/, 3: /e/, 4: /o/).
    pub fn set_target_vowel(&mut self, vowel_idx: usize) {
        let v_idx = vowel_idx.min(4);
        self.params.vowel_target_idx = v_idx;
        self.vowel_morpher.set_target_vowel(v_idx);
    }

    /// Triggers a plosive consonant articulation (/p/, /t/, /k/).
    pub fn trigger_plosive(&mut self, plosive: PlosiveKind) {
        self.params.plosive_kind = plosive;
        self.plosive_engine.trigger(plosive);
    }

    /// Returns the soundcard hardware buffer latency in milliseconds.
    pub fn latency_ms(&self) -> f64 {
        self.params.latency_ms()
    }

    /// Evaluates multi-physics synthesis performance, latency, throughput, and underrun metrics.
    pub fn evaluate_metrics(&mut self, frames_to_render: usize) -> AudioDspSynthMetrics {
        let count = frames_to_render.max(64);
        let latency_ms = self.latency_ms();

        let mut peak = 0.0f64;
        let mut sum_sq = 0.0f64;

        let t_start = std::time::Instant::now();
        for _ in 0..count {
            let s = self.step_frame() as f64;
            let abs_s = s.abs();
            if abs_s > peak {
                peak = abs_s;
            }
            sum_sq += abs_s * abs_s;
        }
        let elapsed = t_start.elapsed().as_secs_f64().max(1.0e-9);

        let throughput = (count as f64) / elapsed;
        let rms = (sum_sq / (count as f64)).sqrt();
        let is_clipping = peak >= 0.999;
        let underruns = self.ring_buffer.underruns();

        AudioDspSynthMetrics {
            latency_ms,
            peak_amplitude: peak,
            rms_amplitude: rms,
            ring_buffer_underruns: underruns,
            audio_stream_throughput_fps: throughput,
            is_clipping,
        }
    }
}
