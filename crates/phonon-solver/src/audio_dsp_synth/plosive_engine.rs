#![deny(unsafe_code)]

//! Aerodynamic Plosive Consonant Synthesizer (/p/, /t/, /k/).
//!
//! Models articulatory occlusion, intra-oral subglottal pressure build-up,
//! sudden release burst transient aerodynamics, and spectral shaping
//! via deterministic zero-allocation bandpass filtered turbulence.

use phonon_models::audio_dsp_synth::PlosiveKind;

/// Articulatory state of the plosive synthesis engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlosiveState {
    /// No plosive active (tract fully open, normal phonation).
    #[default]
    Idle,
    /// Occlusion phase: complete closure, intra-oral pressure buildup.
    Occlusion,
    /// Release burst phase: exponential decay turbulence burst.
    Burst,
}

/// Instantaneous physical output of the plosive engine at each simulation step.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PlosiveOutput {
    /// Occlusion scale factor (1.0 = open tract, 0.0 = complete closure).
    pub occlusion_factor: f64,
    /// Intra-oral aerodynamic pressure in Pascals behind the constriction.
    pub intraoral_pressure: f64,
    /// Radiated transient burst acoustic pressure in Pascals.
    pub burst_pressure: f64,
    /// Flag indicating whether occlusion or burst is currently active.
    pub is_active: bool,
}

/// Deterministic 64-bit Xorshift pseudo-random number generator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    /// Creates a new PRNG with a non-zero seed.
    pub fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x5EED_CAFE_BABE_F00D } else { seed },
        }
    }

    /// Generates next uniform random u64.
    #[inline(always)]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Generates next uniform pseudo-random f64 in [-1.0, 1.0].
    #[inline(always)]
    pub fn next_bipolar(&mut self) -> f64 {
        let val = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        val * 2.0 - 1.0
    }
}

/// Pure safe digital biquad 2-pole bandpass filter (Direct Form II Transposed).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BiquadBandpass {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z1: f64,
    z2: f64,
}

impl BiquadBandpass {
    /// Configures a bandpass filter with center frequency fc_hz and quality factor Q.
    pub fn new(fc_hz: f64, q: f64, fs_hz: f64) -> Self {
        let q_clamped = q.max(0.1);
        let omega = 2.0 * std::f64::consts::PI * fc_hz / fs_hz;
        let sin_w = omega.sin();
        let cos_w = omega.cos();
        let alpha = sin_w / (2.0 * q_clamped);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_w;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Filters one input sample with strictly zero allocations.
    #[inline(always)]
    pub fn step(&mut self, x: f64) -> f64 {
        let y = self.b0 * x + self.z1;
        self.z1 = self.b1 * x - self.a1 * y + self.z2;
        self.z2 = self.b2 * x - self.a2 * y;
        y
    }

    /// Clears internal filter state delay line.
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

/// Aerodynamic transient plosive consonant engine.
#[derive(Debug, Clone, PartialEq)]
pub struct PlosiveEngine {
    /// Active plosive consonant kind.
    pub current_kind: PlosiveKind,
    /// Lifecycle state of the articulation.
    pub state: PlosiveState,
    /// Sampling rate in Hz.
    pub sampling_rate_hz: f64,
    /// Burst intensity scalar in [0.0, 1.0].
    pub burst_intensity: f64,
    /// Time elapsed in occlusion phase in seconds.
    pub occlusion_timer: f64,
    /// Total duration of occlusion phase before release in seconds (default 0.040 s = 40 ms).
    pub occlusion_duration: f64,
    /// Time elapsed in release burst phase in seconds.
    pub burst_timer: f64,
    /// Characteristic burst release decay time constant tau in seconds.
    pub tau: f64,
    /// Characteristic center frequency of the release turbulence in Hz.
    pub center_freq_hz: f64,
    /// Current intra-oral pressure behind occlusion in Pascals.
    pub intraoral_pressure: f64,
    /// Peak intra-oral pressure achieved before release in Pascals.
    pub peak_buildup_pressure: f64,
    /// Deterministic PRNG for turbulence generation.
    pub prng: XorShift64,
    /// Bandpass filter shaping the turbulent release burst.
    pub bandpass_filter: BiquadBandpass,
}

impl PlosiveEngine {
    /// Creates a new plosive engine configured for the designated sampling rate.
    pub fn new(sampling_rate_hz: f64, burst_intensity: f64) -> Self {
        let fs = sampling_rate_hz.clamp(16000.0, 192000.0);
        let intensity = burst_intensity.clamp(0.0, 1.0);
        let prng = XorShift64::new(0x424C_4F53_4956_4500);
        let bandpass_filter = BiquadBandpass::new(1000.0, 1.0, fs);

        Self {
            current_kind: PlosiveKind::None,
            state: PlosiveState::Idle,
            sampling_rate_hz: fs,
            burst_intensity: intensity,
            occlusion_timer: 0.0,
            occlusion_duration: 0.040,
            burst_timer: 0.0,
            tau: 0.005,
            center_freq_hz: 800.0,
            intraoral_pressure: 0.0,
            peak_buildup_pressure: 0.0,
            prng,
            bandpass_filter,
        }
    }

    /// Triggers a plosive articulation gesture.
    pub fn trigger(&mut self, kind: PlosiveKind) {
        self.current_kind = kind;
        if kind == PlosiveKind::None {
            self.state = PlosiveState::Idle;
            self.intraoral_pressure = 0.0;
            self.peak_buildup_pressure = 0.0;
            return;
        }

        self.state = PlosiveState::Occlusion;
        self.occlusion_timer = 0.0;
        self.burst_timer = 0.0;
        self.intraoral_pressure = 0.0;
        self.peak_buildup_pressure = 0.0;

        match kind {
            PlosiveKind::BilabialP => {
                // /p/: broad low-frequency burst centered at 800 Hz, tau = 5 ms
                self.center_freq_hz = 800.0;
                self.tau = 0.005;
                self.bandpass_filter = BiquadBandpass::new(800.0, 1.0, self.sampling_rate_hz);
            }
            PlosiveKind::AlveolarT => {
                // /t/: sharp high-frequency burst centered at 4200 Hz, tau = 8 ms
                self.center_freq_hz = 4200.0;
                self.tau = 0.008;
                self.bandpass_filter = BiquadBandpass::new(4200.0, 3.5, self.sampling_rate_hz);
            }
            PlosiveKind::VelarK => {
                // /k/: mid-frequency compact burst centered at 2200 Hz, tau = 12 ms
                self.center_freq_hz = 2200.0;
                self.tau = 0.012;
                self.bandpass_filter = BiquadBandpass::new(2200.0, 2.5, self.sampling_rate_hz);
            }
            PlosiveKind::None => {}
        }
    }

    /// Advances the plosive simulation by time step dt under driving subglottal pressure.
    ///
    /// Returns the instantaneous occlusion factor, intra-oral pressure, and burst acoustic sound.
    #[inline(always)]
    pub fn step(&mut self, dt: f64, subglottal_pressure_pa: f64) -> PlosiveOutput {
        match self.state {
            PlosiveState::Idle => PlosiveOutput {
                occlusion_factor: 1.0,
                intraoral_pressure: 0.0,
                burst_pressure: 0.0,
                is_active: false,
            },
            PlosiveState::Occlusion => {
                self.occlusion_timer += dt;

                // Intra-oral pressure builds up toward subglottal pressure
                // dP_o / dt = (P_sub - P_o) / tau_buildup
                let tau_buildup = 0.012; // 12 ms buildup time constant
                let alpha = (1.0 - (-dt / tau_buildup).exp()).clamp(0.0, 1.0);
                self.intraoral_pressure += alpha * (subglottal_pressure_pa - self.intraoral_pressure);

                if self.occlusion_timer >= self.occlusion_duration - 1e-9 {
                    // Transition to release burst
                    self.state = PlosiveState::Burst;
                    self.burst_timer = 0.0;
                    self.peak_buildup_pressure = self.intraoral_pressure;
                }

                PlosiveOutput {
                    occlusion_factor: 0.0, // Complete closure
                    intraoral_pressure: self.intraoral_pressure,
                    burst_pressure: 0.0,
                    is_active: true,
                }
            }
            PlosiveState::Burst => {
                self.burst_timer += dt;

                // Occlusion recovers rapidly back to 1.0
                let tau_open = self.tau * 0.5;
                let occ_factor = (1.0 - (-self.burst_timer / tau_open).exp()).clamp(0.0, 1.0);

                // Intra-oral pressure discharges
                self.intraoral_pressure =
                    self.peak_buildup_pressure * (-self.burst_timer / self.tau).exp();

                // Release burst exponential envelope
                let p_sub_nom = subglottal_pressure_pa.max(100.0);
                let p_ratio = (self.peak_buildup_pressure / p_sub_nom).clamp(0.0, 1.5);
                let envelope = self.burst_intensity * p_ratio * (-self.burst_timer / self.tau).exp();

                // Generate pseudo-random white noise and filter through bandpass
                let noise_white = self.prng.next_bipolar();
                let noise_filtered = self.bandpass_filter.step(noise_white);

                // Radiated burst acoustic pressure
                let burst_p = envelope * noise_filtered * 250.0;

                if self.burst_timer >= 6.0 * self.tau {
                    // Burst decay finished, return to idle
                    self.state = PlosiveState::Idle;
                    self.current_kind = PlosiveKind::None;
                    self.intraoral_pressure = 0.0;
                }

                PlosiveOutput {
                    occlusion_factor: occ_factor,
                    intraoral_pressure: self.intraoral_pressure,
                    burst_pressure: burst_p,
                    is_active: true,
                }
            }
        }
    }

    /// Evaluates the power spectral centroid (in Hz) and peak frequency (in Hz) of an audio signal.
    pub fn evaluate_spectral_metrics(samples: &[f64], sampling_rate_hz: f64) -> (f64, f64) {
        if samples.is_empty() {
            return (0.0, 0.0);
        }
        let num_bins = 250;
        let max_freq = (sampling_rate_hz * 0.5).min(8000.0);
        let mut total_power = 0.0;
        let mut weighted_freq_sum = 0.0;
        let mut max_power = 0.0;
        let mut peak_freq = 0.0;

        for b in 1..=num_bins {
            let f = (b as f64) * max_freq / (num_bins as f64);
            let omega = 2.0 * std::f64::consts::PI * f / sampling_rate_hz;
            let mut re = 0.0;
            let mut im = 0.0;
            for (i, &s) in samples.iter().enumerate() {
                let phase = omega * (i as f64);
                re += s * phase.cos();
                im -= s * phase.sin();
            }
            let power = re * re + im * im;
            total_power += power;
            weighted_freq_sum += f * power;
            if power > max_power {
                max_power = power;
                peak_freq = f;
            }
        }

        let centroid = if total_power > 1.0e-12 {
            weighted_freq_sum / total_power
        } else {
            0.0
        };

        (centroid, peak_freq)
    }

    /// Synthesizes isolated release burst samples for analytical spectral validation.
    pub fn synthesize_isolated_burst(&mut self, kind: PlosiveKind, num_samples: usize) -> Vec<f64> {
        self.trigger(kind);
        let dt = 1.0 / self.sampling_rate_hz;
        let subglottal_pressure = 1000.0;

        // Fast-forward occlusion phase to reach full pressure build-up
        let occlusion_steps = (self.occlusion_duration / dt).ceil() as usize;
        for _ in 0..occlusion_steps {
            self.step(dt, subglottal_pressure);
        }

        // Collect release burst samples
        let mut burst_samples = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            let out = self.step(dt, subglottal_pressure);
            burst_samples.push(out.burst_pressure);
        }

        burst_samples
    }
}
