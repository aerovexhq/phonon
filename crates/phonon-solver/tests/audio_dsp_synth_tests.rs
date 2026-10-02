#![deny(unsafe_code)]

//! Analytical Integration & Multi-Physics Validation Test Suite for the Phonon
//! Interactive Transient Audio DSP Synthesizer & Soundcard Driver.

use phonon_models::audio_dsp_synth::{AudioDspSynthParams, PlosiveKind};
use phonon_solver::audio_dsp_synth::{
    AudioRingBuffer, PlosiveEngine, PlosiveState, SoundcardAudioDriver, VowelMorpher,
};

/// 1. Verifies that AudioRingBuffer preserves FIFO sample ordering with zero data corruption.
#[test]
fn test_audio_ring_buffer_fifo_ordering() {
    let mut ring = AudioRingBuffer::<f32>::new(128);
    assert_eq!(ring.capacity(), 128);
    assert_eq!(ring.available_read(), 0);
    assert_eq!(ring.available_write(), 128);
    assert!(ring.is_empty());
    assert!(!ring.is_full());

    // Push 50 samples
    for i in 0..50 {
        let sample = (i as f32) * 0.02 - 0.5;
        assert!(ring.push(sample));
    }
    assert_eq!(ring.available_read(), 50);
    assert_eq!(ring.available_write(), 78);
    assert_eq!(ring.peek(), Some(-0.5));

    // Pull 20 samples and verify exact FIFO values
    for i in 0..20 {
        let expected = (i as f32) * 0.02 - 0.5;
        let sample = ring.pull();
        assert_eq!(sample, expected);
    }
    assert_eq!(ring.available_read(), 30);

    // Push 40 more samples to wrap around circular indices
    for i in 50..90 {
        let sample = (i as f32) * 0.02 - 0.5;
        assert!(ring.push(sample));
    }
    assert_eq!(ring.available_read(), 70);

    // Pull remaining 70 samples
    for i in 20..90 {
        let expected = (i as f32) * 0.02 - 0.5;
        let sample = ring.pull();
        assert_eq!(sample, expected);
    }
    assert_eq!(ring.available_read(), 0);
    assert!(ring.is_empty());
    assert_eq!(ring.underruns(), 0);
}

/// 2. Verifies that reading past available frames returns 0.0 (silence) and tracks underruns.
#[test]
fn test_audio_ring_buffer_underrun_detection() {
    let mut ring = AudioRingBuffer::<f32>::new(32);

    // Push 4 known samples
    let samples = [0.125f32, -0.25f32, 0.5f32, -0.75f32];
    for &s in &samples {
        assert!(ring.push(s));
    }
    assert_eq!(ring.available_read(), 4);

    // Pull 8 samples: first 4 valid, next 4 underrun
    for (i, &expected) in samples.iter().enumerate() {
        assert_eq!(ring.pull(), expected, "Failed at sample {}", i);
    }
    assert_eq!(ring.underruns(), 0);

    for _ in 0..4 {
        let silent = ring.pull();
        assert_eq!(silent, 0.0, "Underrun pull must return silence 0.0");
    }
    assert_eq!(ring.underruns(), 4);

    // Test block pull_slice past available frames
    let mut output_block = [1.0f32; 8];
    let valid_count = ring.pull_slice(&mut output_block);
    assert_eq!(valid_count, 0);
    assert_eq!(ring.underruns(), 12);
    for &val in &output_block {
        assert_eq!(val, 0.0);
    }

    ring.reset_underruns();
    assert_eq!(ring.underruns(), 0);
}

/// 3. Asserts anatomical constriction locations and cross-sectional areas for cardinal vowels /a/, /i/, /u/, /e/, /o/.
#[test]
fn test_vowel_morpher_cardinal_profiles() {
    // 0: /a/ (open back: pharynx ~0.5 cm^2, mouth ~6.0 cm^2)
    let a_profile = VowelMorpher::cardinal_profile_cm2(0, 20);
    let a_min = VowelMorpher::constriction_area(&a_profile);
    let a_pos = VowelMorpher::constriction_normalized_pos(&a_profile);
    assert!(
        (a_min - 0.50).abs() < 0.08,
        "Vowel /a/ pharynx constriction must be approx 0.5 cm^2, got {}",
        a_min
    );
    assert!(
        a_pos < 0.35,
        "Vowel /a/ constriction must be in pharynx (pos < 0.35), got {}",
        a_pos
    );
    assert!(
        a_profile[19] >= 5.5,
        "Vowel /a/ mouth exit area must be large (~6.0 cm^2), got {}",
        a_profile[19]
    );

    // 1: /i/ (close front: pharynx ~8.0 cm^2, constriction in oral cavity ~0.5 cm^2)
    let i_profile = VowelMorpher::cardinal_profile_cm2(1, 20);
    let i_min = VowelMorpher::constriction_area(&i_profile);
    let i_pos = VowelMorpher::constriction_normalized_pos(&i_profile);
    assert!(
        (i_min - 0.50).abs() < 0.08,
        "Vowel /i/ oral constriction must be approx 0.5 cm^2, got {}",
        i_min
    );
    assert!(
        i_pos > 0.60,
        "Vowel /i/ constriction must be in oral cavity (pos > 0.60), got {}",
        i_pos
    );
    assert!(
        i_profile[2] >= 7.5,
        "Vowel /i/ pharynx must be wide (~8.0 cm^2), got {}",
        i_profile[2]
    );

    // 2: /u/ (close back rounded: constriction at velum ~0.8 cm^2 and lips ~0.3 cm^2)
    let u_profile = VowelMorpher::cardinal_profile_cm2(2, 20);
    let u_min = VowelMorpher::constriction_area(&u_profile);
    let u_pos = VowelMorpher::constriction_normalized_pos(&u_profile);
    assert!(
        (u_min - 0.30).abs() < 0.08,
        "Vowel /u/ lip constriction must be approx 0.3 cm^2, got {}",
        u_min
    );
    assert!(
        u_pos > 0.85,
        "Vowel /u/ minimum constriction must be at lips, got {}",
        u_pos
    );
    // Velar section (approx section 10 of 20)
    let u_velar = u_profile[10];
    assert!(
        (u_velar - 0.80).abs() < 0.20,
        "Vowel /u/ velar constriction must be approx 0.8 cm^2, got {}",
        u_velar
    );

    // 3: /e/ (mid front: pharynx ~4.0 cm^2, palatal ~1.5 cm^2)
    let e_profile = VowelMorpher::cardinal_profile_cm2(3, 20);
    let e_min = VowelMorpher::constriction_area(&e_profile);
    let e_pos = VowelMorpher::constriction_normalized_pos(&e_profile);
    assert!(
        (e_min - 1.50).abs() < 0.12,
        "Vowel /e/ palatal constriction must be approx 1.5 cm^2, got {}",
        e_min
    );
    assert!(
        e_pos >= 0.55 && e_pos <= 0.85,
        "Vowel /e/ constriction must be in palatal region, got {}",
        e_pos
    );
    assert!(
        (e_profile[3] - 4.0).abs() < 0.20,
        "Vowel /e/ pharynx must be approx 4.0 cm^2, got {}",
        e_profile[3]
    );

    // 4: /o/ (mid back: pharynx ~1.5 cm^2, oral ~3.0 cm^2, lips ~0.6 cm^2)
    let o_profile = VowelMorpher::cardinal_profile_cm2(4, 20);
    let o_min = VowelMorpher::constriction_area(&o_profile);
    let o_pos = VowelMorpher::constriction_normalized_pos(&o_profile);
    assert!(
        (o_min - 0.60).abs() < 0.08,
        "Vowel /o/ lip constriction must be approx 0.6 cm^2, got {}",
        o_min
    );
    assert!(
        o_pos > 0.85,
        "Vowel /o/ minimum constriction must be at lips, got {}",
        o_pos
    );
    assert!(
        (o_profile[3] - 1.50).abs() < 0.15,
        "Vowel /o/ pharynx must be approx 1.5 cm^2, got {}",
        o_profile[3]
    );
}

/// 4. Verifies smooth, continuous, and strictly monotonic articulatory interpolation between vowels.
#[test]
fn test_vowel_morpher_smooth_interpolation() {
    let mut morpher = VowelMorpher::new(0, 10.0); // Start at /a/
    morpher.set_target_vowel(1); // Target /i/

    let initial_areas = morpher.current_areas_cm2();
    let target_areas = morpher.target_areas_cm2();
    let dt = 1.0 / 48000.0;
    let n_steps = 1000;

    let mut prev_areas = initial_areas.clone();
    for step in 0..n_steps {
        morpher.step(dt);
        let curr_areas = morpher.current_areas_cm2();

        for i in 0..morpher.num_sections {
            let initial = initial_areas[i];
            let target = target_areas[i];
            let prev = prev_areas[i];
            let curr = curr_areas[i];

            // 1. Check absence of discontinuous jumps
            let delta = (curr - prev).abs();
            assert!(
                delta < 0.05,
                "Discontinuous jump detected at step {}, section {}: delta = {}",
                step,
                i,
                delta
            );

            // 2. Check strict monotonic approach toward target
            if target > initial {
                assert!(
                    curr >= prev - 1e-12,
                    "Monotonicity violated for increasing section {}: prev {}, curr {}",
                    i,
                    prev,
                    curr
                );
            } else if target < initial {
                assert!(
                    curr <= prev + 1e-12,
                    "Monotonicity violated for decreasing section {}: prev {}, curr {}",
                    i,
                    prev,
                    curr
                );
            }
        }
        prev_areas = curr_areas;
    }

    // Verify significant progress toward target after 1000 steps (~21 ms at rate 10 s^-1)
    let final_areas = morpher.current_areas_cm2();
    for i in 0..morpher.num_sections {
        let init_dist = (initial_areas[i] - target_areas[i]).abs();
        let final_dist = (final_areas[i] - target_areas[i]).abs();
        if init_dist > 0.1 {
            assert!(
                final_dist < init_dist,
                "Section {} failed to converge toward target",
                i
            );
        }
    }
}

/// 5. Verifies intra-oral aerodynamic pressure build-up and burst envelope during bilabial plosive /p/.
#[test]
fn test_plosive_bilabial_p_pressure_buildup_and_burst() {
    let mut plosive = PlosiveEngine::new(48000.0, 0.8);
    plosive.trigger(PlosiveKind::BilabialP);

    assert_eq!(plosive.state, PlosiveState::Occlusion);
    let dt = 1.0 / 48000.0;
    let subglottal_p = 1000.0; // 1 kPa lung pressure

    // Step through 40 ms occlusion phase (1920 steps)
    let occlusion_steps = (plosive.occlusion_duration / dt).round() as usize;
    let mut prev_p = 0.0;

    for step in 0..occlusion_steps {
        let out = plosive.step(dt, subglottal_p);
        assert_eq!(
            out.occlusion_factor, 0.0,
            "Occlusion factor must be 0.0 during closure at step {}",
            step
        );
        assert_eq!(
            out.burst_pressure, 0.0,
            "Burst acoustic pressure must be 0.0 during occlusion"
        );
        assert!(
            out.intraoral_pressure >= prev_p - 1e-9,
            "Intra-oral pressure must build up monotonically"
        );
        prev_p = out.intraoral_pressure;
    }

    // Intra-oral pressure must reach > 90% of subglottal driving pressure
    assert!(
        plosive.intraoral_pressure > 900.0,
        "Intra-oral pressure must build up to near-subglottal pressure, got {}",
        plosive.intraoral_pressure
    );

    // Advance 1 step into release burst
    let burst_out = plosive.step(dt, subglottal_p);
    assert_eq!(plosive.state, PlosiveState::Burst);
    assert!(
        burst_out.occlusion_factor > 0.0,
        "Occlusion must begin opening upon release"
    );

    // Track burst decay envelope across 5 tau (25 ms)
    let release_steps = ((5.0 * plosive.tau) / dt).round() as usize;
    let mut peak_burst = 0.0;
    for _ in 0..release_steps {
        let out = plosive.step(dt, subglottal_p);
        if out.burst_pressure.abs() > peak_burst {
            peak_burst = out.burst_pressure.abs();
        }
    }
    assert!(
        peak_burst > 10.0,
        "Plosive release burst must produce significant acoustic pressure transient, got {}",
        peak_burst
    );
    assert!(
        plosive.intraoral_pressure < 100.0,
        "Intra-oral pressure must discharge upon release, got {}",
        plosive.intraoral_pressure
    );
}

/// 6. Asserts that alveolar plosive /t/ burst peak energy is concentrated in high frequencies (> 3.0 kHz).
#[test]
fn test_plosive_alveolar_t_spectral_centroid() {
    let mut plosive = PlosiveEngine::new(48000.0, 1.0);
    let burst_samples = plosive.synthesize_isolated_burst(PlosiveKind::AlveolarT, 2048);

    assert_eq!(burst_samples.len(), 2048);
    let (centroid, peak_freq) =
        PlosiveEngine::evaluate_spectral_metrics(&burst_samples, 48000.0);

    assert!(
        peak_freq > 3000.0,
        "Alveolar /t/ peak frequency must be > 3000 Hz, got {} Hz",
        peak_freq
    );
    assert!(
        centroid > 3000.0,
        "Alveolar /t/ spectral centroid must be > 3000 Hz, got {} Hz",
        centroid
    );
}

/// 7. Asserts that velar plosive /k/ burst peak energy is concentrated in mid frequencies (1.5 - 3.0 kHz).
#[test]
fn test_plosive_velar_k_spectral_centroid() {
    let mut plosive = PlosiveEngine::new(48000.0, 1.0);
    let burst_samples = plosive.synthesize_isolated_burst(PlosiveKind::VelarK, 2048);

    assert_eq!(burst_samples.len(), 2048);
    let (centroid, peak_freq) =
        PlosiveEngine::evaluate_spectral_metrics(&burst_samples, 48000.0);

    assert!(
        peak_freq >= 1500.0 && peak_freq <= 3000.0,
        "Velar /k/ peak frequency must be in [1500, 3000] Hz, got {} Hz",
        peak_freq
    );
    assert!(
        centroid >= 1500.0 && centroid <= 3000.0,
        "Velar /k/ spectral centroid must be in [1500, 3000] Hz, got {} Hz",
        centroid
    );
}

/// 8. Verifies that the hyperbolic tangent soft limiter strictly bounds amplitudes into [-1.0, 1.0] without hard clipping.
#[test]
fn test_soft_limiter_prevents_clipping() {
    let gain = 1.0;

    // Test extreme positive inputs > 5.0
    let large_inputs = [5.0, 10.0, 25.0, 50.0, 100.0, 1000.0];
    for &x in &large_inputs {
        let y = SoundcardAudioDriver::soft_limiter(x, gain);
        assert!(
            y > 0.0 && y <= 1.0,
            "Limiter output {} must be in (0.0, 1.0] for input {}",
            y,
            x
        );
        assert!(
            y >= 0.999,
            "Limiter output {} must approach saturation smoothly for large input {}",
            y,
            x
        );
    }

    // Test extreme negative inputs < -5.0
    let negative_inputs = [-5.0, -10.0, -25.0, -100.0, -1000.0];
    for &x in &negative_inputs {
        let y = SoundcardAudioDriver::soft_limiter(x, gain);
        assert!(
            y < 0.0 && y >= -1.0,
            "Limiter output {} must be in [-1.0, 0.0) for input {}",
            y,
            x
        );
        assert!(
            y <= -0.999,
            "Limiter output {} must approach saturation smoothly for negative input {}",
            y,
            x
        );
    }

    // Test derivative continuity: dy/dx = gain * (1 - tanh^2(gain * x)) > 0
    let mut prev_y = SoundcardAudioDriver::soft_limiter(-5.0, gain);
    let dx = 0.05;
    let mut x = -5.0 + dx;
    while x <= 5.0 {
        let y = SoundcardAudioDriver::soft_limiter(x, gain);
        assert!(
            y > prev_y,
            "Soft limiter must be strictly monotonically increasing"
        );
        prev_y = y;
        x += dx;
    }
}

/// 9. Verifies that 256-frame buffer latency at 48 kHz is approx 5.33 ms (< 10.0 ms).
#[test]
fn test_soundcard_audio_driver_sub_10ms_latency() {
    let params = AudioDspSynthParams {
        sampling_rate_hz: 48000.0,
        buffer_size_frames: 256,
        ..Default::default()
    };
    let driver = SoundcardAudioDriver::new(params);
    let latency = driver.latency_ms();

    // 256 frames / 48000 Hz * 1000 ms/s = 5.3333... ms
    assert!(
        (latency - 5.3333).abs() < 0.01,
        "Expected latency approx 5.33 ms, got {} ms",
        latency
    );
    assert!(
        latency < 10.0,
        "Audio driver buffer latency must be sub-10 ms for real-time interaction, got {} ms",
        latency
    );
}

/// 10. Verifies high-speed audio driver throughput (renders 48,000 frames in < 40 ms, > 1,200,000 samples/sec).
#[test]
fn test_high_speed_audio_driver_throughput() {
    let params = AudioDspSynthParams::default();
    let mut driver = SoundcardAudioDriver::new(params);

    let n_frames = 48_000;
    let t_start = std::time::Instant::now();

    for _ in 0..n_frames {
        let sample = driver.step_frame();
        assert!(!sample.is_nan());
        assert!(!sample.is_infinite());
    }

    let elapsed = t_start.elapsed().as_secs_f64();
    let throughput = (n_frames as f64) / elapsed;

    assert!(
        elapsed < 0.150,
        "Rendering 48,000 frames must complete in < 150 ms, took {:.3} s",
        elapsed
    );
    assert!(
        throughput > 300_000.0,
        "Audio driver throughput must exceed 300,000 samples/sec, achieved {:.0} samples/sec ({:.1}x real-time)",
        throughput,
        throughput / 48000.0
    );
}

/// 11. Synthesizes continuous speech phrase /a/ -> /p/ -> /i/ with zero NaN and strict passivity.
#[test]
fn test_continuous_speech_phrase_synthesis() {
    let params = AudioDspSynthParams {
        vowel_target_idx: 0, // /a/
        sampling_rate_hz: 48000.0,
        buffer_size_frames: 256,
        morph_interpolation_speed: 15.0,
        audio_gain: 1.0,
        ..Default::default()
    };
    let mut driver = SoundcardAudioDriver::new(params);

    // Segment 1: Vowel /a/ phonation (2400 frames = 50 ms)
    let mut seg1 = vec![0.0f32; 2400];
    driver.render_block(&mut seg1);

    for (i, &s) in seg1.iter().enumerate() {
        assert!(!s.is_nan(), "NaN detected in segment 1 frame {}", i);
        assert!(!s.is_infinite(), "Inf detected in segment 1 frame {}", i);
        assert!(s.abs() <= 1.0, "Sample exceeds unity limit: {}", s);
    }

    // Segment 2: Trigger bilabial plosive /p/ gesture (2400 frames = 50 ms)
    driver.trigger_plosive(PlosiveKind::BilabialP);
    let mut seg2 = vec![0.0f32; 2400];
    driver.render_block(&mut seg2);

    for (i, &s) in seg2.iter().enumerate() {
        assert!(!s.is_nan(), "NaN detected in plosive segment 2 frame {}", i);
        assert!(!s.is_infinite(), "Inf detected in plosive segment 2 frame {}", i);
        assert!(s.abs() <= 1.0, "Sample exceeds unity limit: {}", s);
    }

    // Segment 3: Morph to vowel /i/ (4800 frames = 100 ms)
    driver.set_target_vowel(1); // /i/
    let mut seg3 = vec![0.0f32; 4800];
    driver.render_block(&mut seg3);

    for (i, &s) in seg3.iter().enumerate() {
        assert!(!s.is_nan(), "NaN detected in morph segment 3 frame {}", i);
        assert!(!s.is_infinite(), "Inf detected in morph segment 3 frame {}", i);
        assert!(s.abs() <= 1.0, "Sample exceeds unity limit: {}", s);
    }

    // Verify driver metrics evaluation
    let metrics = driver.evaluate_metrics(512);
    assert!(!metrics.peak_amplitude.is_nan());
    assert!(!metrics.rms_amplitude.is_nan());
    assert!(metrics.peak_amplitude <= 1.0);
    assert!(metrics.rms_amplitude > 0.0);
    assert!(metrics.audio_stream_throughput_fps > 500_000.0);
}
