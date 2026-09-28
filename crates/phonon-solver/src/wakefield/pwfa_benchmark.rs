//! Multi-particle laser-plasma wakefield acceleration and drift conservation benchmark.

use crate::wakefield::relativistic_boris_pusher::{BorisPusher, RelativisticParticle};
use crate::wakefield::wakefield_accelerator::{BeamBunch, WakefieldAccelerator};
use phonon_models::wakefield::{
    BetatronRadiation, BubbleRegime, LaserPulseParams, PlasmaChannelParams,
};
use std::time::Instant;

/// Benchmark performance and physics report.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PwfaBenchmarkReport {
    /// Number of particles tracked in parallel.
    pub particle_count: usize,
    /// Total number of time steps per particle.
    pub steps_per_particle: usize,
    /// Total simulated steps across all particles.
    pub total_particle_steps: usize,
    /// Elapsed execution wall-clock time in seconds.
    pub elapsed_seconds: f64,
    /// Overall simulation throughput in particle-steps per second.
    pub throughput_steps_per_sec: f64,
    /// Maximum fractional energy drift in unaccelerated magnetic drift (< 1e-6).
    pub energy_conservation_error: f64,
    /// Initial bunch mean energy in MeV.
    pub initial_energy_mev: f64,
    /// Final bunch mean energy in MeV.
    pub final_energy_mev: f64,
    /// Final RMS bunch duration in femtoseconds (< 5.0 fs).
    pub final_bunch_duration_fs: f64,
    /// Final normalized transverse emittance in nm-rad.
    pub final_emittance_nm_rad: f64,
}

/// Runner for high-throughput parallel PWFA benchmarks.
pub struct PwfaBenchmarkRunner;

impl PwfaBenchmarkRunner {
    /// Verifies strict symplectic energy conservation in pure magnetic field (no acceleration).
    pub fn verify_drift_energy_conservation(steps: usize, dt: f64) -> f64 {
        let mut particle = RelativisticParticle::electron_with_energy_mev([0.0, 0.0, 0.0], 100.0);
        let initial_e = particle.kinetic_energy_mev();

        // Constant perpendicular magnetic field: pure cyclotron gyration
        let b = [0.0, 1.0, 0.0];
        let e = [0.0, 0.0, 0.0];

        for _ in 0..steps {
            BorisPusher::step(&mut particle, e, b, dt, false);
        }

        let final_e = particle.kinetic_energy_mev();
        ((final_e - initial_e) / initial_e).abs()
    }

    /// Executes the 10,000-particle-step benchmark suite.
    pub fn run_benchmark(particle_count: usize, steps: usize) -> PwfaBenchmarkReport {
        let drift_error = Self::verify_drift_energy_conservation(10_000, 1.0e-15);

        let laser = LaserPulseParams::standard_tisapphire();
        let plasma = PlasmaChannelParams::standard_underdense();
        let bubble = BubbleRegime::new(laser, plasma);
        let betatron = BetatronRadiation::new(plasma);
        let accelerator = WakefieldAccelerator::new(bubble, betatron, true);

        // Sub-femtosecond bunch: sigma_z = 0.3 um -> ~1 fs duration
        let mut bunch = BeamBunch::new_gaussian(
            particle_count,
            100.0,
            0.01,
            0.5e-6,
            0.3e-6,
            -0.5 * bubble.bubble_radius_m(),
        );

        let dt = 1.0e-16; // 0.1 fs
        let initial_e = bunch.mean_energy_mev();

        let start = Instant::now();
        for step in 0..steps {
            let time = step as f64 * dt;
            accelerator.step_bunch(&mut bunch, time, dt);
        }
        let elapsed = start.elapsed().as_secs_f64().max(1e-9);

        let total_steps = particle_count * steps;
        let throughput = total_steps as f64 / elapsed;

        let final_e = bunch.mean_energy_mev();
        let final_duration_fs = bunch.bunch_duration_s() * 1.0e15;
        let final_emittance_nm = bunch.normalized_emittance_x_m_rad() * 1.0e9;

        PwfaBenchmarkReport {
            particle_count,
            steps_per_particle: steps,
            total_particle_steps: total_steps,
            elapsed_seconds: elapsed,
            throughput_steps_per_sec: throughput,
            energy_conservation_error: drift_error,
            initial_energy_mev: initial_e,
            final_energy_mev: final_e,
            final_bunch_duration_fs: final_duration_fs,
            final_emittance_nm_rad: final_emittance_nm,
        }
    }
}
