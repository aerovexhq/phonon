//! 10,000-Scenario Parallel Rayon Acoustic Benchmark Runner
//!
//! Evaluates multi-medium acoustic wave propagation, Doppler shifts, structural wall
//! transmission loss, Sabine reverberation time ($T_{60}$), and microphone analog audio
//! transduction across 10,000 parallel test scenarios.

use super::acoustic_tier_engine::{AcousticLinkSimulator, AcousticRealismTier, AcousticRoom};
use phonon_models::acoustic::{
    AcousticMedium, AcousticSource, CondenserMicrophone, MicrophonePolarPattern,
};
use phonon_models::em::Vector3D;
use rayon::prelude::*;
use std::time::Instant;

/// Summary report for 10,000-scenario parallel acoustic benchmark.
#[derive(Debug, Clone, PartialEq)]
pub struct AcousticBenchmarkReport {
    /// Total number of scenarios evaluated.
    pub total_scenarios: usize,
    /// Wall-clock execution time in milliseconds.
    pub elapsed_ms: f64,
    /// Throughput in evaluations per second.
    pub scenarios_per_second: f64,
    /// Average Sound Pressure Level (SPL) across all scenarios in dB SPL.
    pub average_spl_db: f64,
    /// Average Sabine $T_{60}$ reverberation time across rooms in seconds.
    pub average_t60_seconds: f64,
    /// Average microphone output voltage magnitude in millivolts.
    pub average_mic_voltage_mv: f64,
    /// Maximum observed kinematic Doppler shift across scenarios in Hz.
    pub max_doppler_shift_hz: f64,
    /// Flag verifying strict zero acoustic transmission in vacuum isolation.
    pub vacuum_silence_verified: bool,
}

/// Parallel Rayon Acoustic Benchmark Runner.
pub struct AcousticBenchmarkRunner;

impl AcousticBenchmarkRunner {
    /// Executes the 10,000-scenario parallel acoustic propagation benchmark.
    pub fn run_benchmark(num_scenarios: usize) -> AcousticBenchmarkReport {
        let start = Instant::now();

        // Parallel Rayon iteration:
        let results: Vec<(f64, f64, f64, f64, bool)> = (0..num_scenarios)
            .into_par_iter()
            .map(|i| {
                // Varying room dimensions:
                let lx = 4.0 + (i % 7) as f64 * 1.5;
                let ly = 3.0 + ((i / 7) % 5) as f64 * 1.2;
                let lz = 2.5 + ((i / 35) % 3) as f64 * 0.5;

                // Varying source kinematics:
                let v_source_x = -25.0 + (i % 51) as f64; // -25 m/s to +25 m/s
                let f_source = 200.0 + (i % 20) as f64 * 150.0; // 200 Hz to 3050 Hz
                let power_watts = 0.01 + (i % 10) as f64 * 0.05; // 10 mW to 460 mW

                let pos_src = Vector3D::new(1.0, 1.0, 1.2);
                let vel_src = Vector3D::new(v_source_x, 0.0, 0.0);
                let source = AcousticSource::new(pos_src, vel_src, f_source, power_watts);

                // Microphone at listener location:
                let pos_mic = Vector3D::new(lx - 1.0, ly - 1.0, 1.5);
                let axis_mic = Vector3D::new(-1.0, -1.0, 0.0);
                let polar = match i % 4 {
                    0 => MicrophonePolarPattern::Omnidirectional,
                    1 => MicrophonePolarPattern::Cardioid,
                    2 => MicrophonePolarPattern::Supercardioid,
                    _ => MicrophonePolarPattern::Figure8,
                };
                let mic = CondenserMicrophone::new_studio_capsule(pos_mic, axis_mic, polar);

                // Medium: Air at 20 C, with 50% RH
                let air = AcousticMedium::standard_air();
                let wall_mat = if i % 2 == 0 {
                    AcousticMedium::concrete()
                } else {
                    AcousticMedium::wood()
                };

                let room = AcousticRoom::new_box(lx, ly, lz, air, wall_mat);
                let t60 = room.sabine_t60();

                let tier = match i % 3 {
                    0 => AcousticRealismTier::Tier2AcceleratedPathLoss,
                    1 => AcousticRealismTier::Tier1RaytracedMultipath,
                    _ => AcousticRealismTier::Tier0FullWaveFdtd,
                };

                let mut sim = AcousticLinkSimulator::new(source, mic, room, tier);
                let step_res = sim.step(0.01);

                let spl = step_res.spl_db;
                let v_mv = step_res.mic_signal.voltage_v.abs() * 1000.0;
                let doppler_shift = (step_res.doppler.observed_frequency_hz - f_source).abs();

                // Test vacuum isolation scenario:
                let vac_source = AcousticSource::new(pos_src, Vector3D::ZERO, 1000.0, 1.0);
                let vac_mic = CondenserMicrophone::new_studio_capsule(
                    pos_mic,
                    axis_mic,
                    MicrophonePolarPattern::Omnidirectional,
                );
                let vac_room = AcousticRoom::new_box(
                    lx,
                    ly,
                    lz,
                    AcousticMedium::vacuum(),
                    AcousticMedium::vacuum(),
                );
                let mut vac_sim = AcousticLinkSimulator::new(
                    vac_source,
                    vac_mic,
                    vac_room,
                    AcousticRealismTier::Tier2AcceleratedPathLoss,
                );
                let vac_res = vac_sim.step(0.01);
                let vac_silent =
                    vac_res.incident_pressure_pa == 0.0 && vac_res.mic_signal.voltage_v == 0.0;

                (spl, t60, v_mv, doppler_shift, vac_silent)
            })
            .collect();

        let elapsed = start.elapsed();
        let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
        let scenarios_per_sec = (num_scenarios as f64) / elapsed.as_secs_f64();

        let mut sum_spl = 0.0;
        let mut sum_t60 = 0.0;
        let mut sum_v = 0.0;
        let mut max_doppler = 0.0f64;
        let mut all_vac_silent = true;

        for (spl, t60, v_mv, doppler, vac_silent) in &results {
            sum_spl += spl;
            sum_t60 += t60;
            sum_v += v_mv;
            max_doppler = max_doppler.max(*doppler);
            if !*vac_silent {
                all_vac_silent = false;
            }
        }

        let n = num_scenarios as f64;
        AcousticBenchmarkReport {
            total_scenarios: num_scenarios,
            elapsed_ms,
            scenarios_per_second: scenarios_per_sec,
            average_spl_db: sum_spl / n,
            average_t60_seconds: sum_t60 / n,
            average_mic_voltage_mv: sum_v / n,
            max_doppler_shift_hz: max_doppler,
            vacuum_silence_verified: all_vac_silent,
        }
    }
}
