//! Integration tests for PWFA beam bunch tracking and parallel Rayon benchmarks.

use phonon_models::wakefield::{
    BetatronRadiation, BubbleRegime, LaserPulseParams, PlasmaChannelParams,
};
use phonon_solver::wakefield::{BeamBunch, PwfaBenchmarkRunner, WakefieldAccelerator};

#[test]
fn test_beam_bunch_statistics_and_emittance() {
    let bunch = BeamBunch::new_gaussian(500, 200.0, 0.02, 1.0e-6, 0.5e-6, 0.0);
    assert_eq!(bunch.len(), 500);

    let mean_e = bunch.mean_energy_mev();
    assert!(
        (mean_e - 200.0).abs() < 5.0,
        "Mean energy should be ~200 MeV, got {}",
        mean_e
    );

    let spread = bunch.energy_spread_rel();
    assert!(
        spread > 0.01 && spread < 0.05,
        "Relative energy spread should be ~2%, got {}",
        spread
    );

    let duration_fs = bunch.bunch_duration_s() * 1.0e15;
    assert!(
        duration_fs > 0.5 && duration_fs < 3.0,
        "Bunch duration should be ~1-2 fs, got {} fs",
        duration_fs
    );

    let emit_x = bunch.normalized_emittance_x_m_rad();
    assert!(
        emit_x > 0.0 && emit_x < 1.0e-5,
        "Normalized emittance should be physical, got {}",
        emit_x
    );
}

#[test]
fn test_wakefield_accelerator_tracking() {
    let laser = LaserPulseParams::standard_tisapphire();
    let plasma = PlasmaChannelParams::standard_underdense();
    let bubble = BubbleRegime::new(laser, plasma);
    let betatron = BetatronRadiation::new(plasma);
    let accelerator = WakefieldAccelerator::new(bubble, betatron, false);

    let mut bunch = BeamBunch::new_gaussian(
        100,
        100.0,
        0.01,
        0.2e-6,
        0.2e-6,
        -0.6 * bubble.bubble_radius_m(),
    );
    let initial_energy = bunch.mean_energy_mev();

    // Track for 100 fs (dt = 0.1 fs, 1000 steps)
    let summary = accelerator.track(&mut bunch, 30.0e-6, 1.0e-16);

    assert!(summary.steps_taken > 0);
    assert!(summary.distance_traveled_m > 0.0);
    assert!(
        summary.final_energy_mev >= initial_energy,
        "Electron bunch in accelerating phase must gain or preserve energy: initial {} MeV, final {} MeV",
        initial_energy, summary.final_energy_mev
    );
}

#[test]
fn test_pwfa_parallel_benchmark() {
    let report = PwfaBenchmarkRunner::run_benchmark(200, 50);

    assert_eq!(report.particle_count, 200);
    assert_eq!(report.steps_per_particle, 50);
    assert_eq!(report.total_particle_steps, 10_000);

    assert!(
        report.energy_conservation_error < 1.0e-6,
        "Drift energy conservation error must be < 1e-6, got {}",
        report.energy_conservation_error
    );

    assert!(
        report.throughput_steps_per_sec > 100_000.0,
        "Throughput must exceed 100,000 steps/sec, got {}",
        report.throughput_steps_per_sec
    );

    assert!(
        report.final_bunch_duration_fs < 5.0,
        "Final bunch duration must be < 5 fs, got {} fs",
        report.final_bunch_duration_fs
    );
}
