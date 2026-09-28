//! Integration Tests for Multi-Tier Acoustic Solvers & 10,000-Scenario Rayon Benchmark

use phonon_models::acoustic::{
    AcousticMedium, AcousticSource, CondenserMicrophone, MicrophonePolarPattern,
};
use phonon_models::em::Vector3D;
use phonon_solver::acoustic::{
    AcousticBenchmarkRunner, AcousticLinkSimulator, AcousticRealismTier, AcousticRoom,
};

#[test]
fn test_fdtd_2d_wave_equation_standing_waves() {
    let air = AcousticMedium::standard_air();
    let wall = AcousticMedium::concrete();
    let room = AcousticRoom::new_box(4.0, 4.0, 2.5, air, wall);

    let src = AcousticSource::new(Vector3D::new(1.0, 2.0, 1.2), Vector3D::ZERO, 500.0, 0.1);
    let mic = CondenserMicrophone::new_studio_capsule(
        Vector3D::new(3.0, 2.0, 1.2),
        Vector3D::new(-1.0, 0.0, 0.0),
        MicrophonePolarPattern::Omnidirectional,
    );

    let sim = AcousticLinkSimulator::new(src, mic, room, AcousticRealismTier::Tier0FullWaveFdtd);

    // Run 2D FDTD grid solver on 32x32 grid for 80 steps
    let fdtd = sim.solve_fdtd_2d(32, 32, 0.125, 80);
    assert_eq!(fdtd.grid_shape, (32, 32));
    assert_eq!(fdtd.total_steps, 80);
    assert!(
        fdtd.peak_pressure_pa > 0.0,
        "Expected non-zero acoustic pressure wave"
    );
    assert_eq!(fdtd.observer_pressure_history.len(), 80);
}

#[test]
fn test_sabine_and_eyring_reverberation_time() {
    let air = AcousticMedium::standard_air();

    // Highly reflective room (concrete, alpha = 0.02):
    let room_concrete =
        AcousticRoom::new_box(6.0, 4.0, 3.0, air.clone(), AcousticMedium::concrete());
    // V = 72 m^3, S = 2*(24 + 12 + 18) = 108 m^2
    assert_eq!(room_concrete.volume(), 72.0);
    assert_eq!(room_concrete.total_surface_area(), 108.0);

    let t60_sabine_conc = room_concrete.sabine_t60();
    // T60 = 0.161 * 72 / (108 * 0.02) = 11.592 / 2.16 ~ 5.37 seconds
    assert!(
        (t60_sabine_conc - 5.37).abs() < 0.2,
        "Expected ~5.37s T60, got {:.2}",
        t60_sabine_conc
    );

    // Moderately absorbing room (wood, alpha = 0.10):
    let room_wood = AcousticRoom::new_box(6.0, 4.0, 3.0, air, AcousticMedium::wood());
    let t60_wood = room_wood.sabine_t60();
    // T60 = 0.161 * 72 / (108 * 0.10) = 11.592 / 10.8 ~ 1.07 seconds
    assert!(
        (t60_wood - 1.07).abs() < 0.1,
        "Expected ~1.07s T60, got {:.2}",
        t60_wood
    );
    assert!(t60_wood < t60_sabine_conc);

    // Eyring vs Sabine: Eyring is slightly lower or equal
    let t60_eyring = room_wood.eyring_t60();
    assert!(t60_eyring <= t60_wood && t60_eyring > 0.8);
}

#[test]
fn test_multi_tier_cross_consistency() {
    let air = AcousticMedium::standard_air();
    let concrete = AcousticMedium::concrete();
    let room = AcousticRoom::new_box(5.0, 4.0, 2.8, air, concrete);

    let src = AcousticSource::new(Vector3D::new(1.0, 1.0, 1.5), Vector3D::ZERO, 1000.0, 0.05);
    let mic = CondenserMicrophone::new_studio_capsule(
        Vector3D::new(3.5, 2.5, 1.5),
        Vector3D::new(-1.0, -1.0, 0.0),
        MicrophonePolarPattern::Cardioid,
    );

    let mut sim = AcousticLinkSimulator::new(
        src,
        mic,
        room,
        AcousticRealismTier::Tier2AcceleratedPathLoss,
    );
    let res_tier2 = sim.step(0.01);

    sim.set_tier(AcousticRealismTier::Tier1RaytracedMultipath);
    let res_tier1 = sim.step(0.01);

    sim.set_tier(AcousticRealismTier::Tier0FullWaveFdtd);
    let res_tier0 = sim.step(0.01);

    // All tiers should evaluate close SPL values within +/- 4 dB:
    assert!(
        (res_tier1.spl_db - res_tier2.spl_db).abs() < 4.0,
        "Tier 1 vs Tier 2 SPL mismatch: {:.2} vs {:.2}",
        res_tier1.spl_db,
        res_tier2.spl_db
    );
    assert!(
        (res_tier0.spl_db - res_tier2.spl_db).abs() < 4.0,
        "Tier 0 vs Tier 2 SPL mismatch: {:.2} vs {:.2}",
        res_tier0.spl_db,
        res_tier2.spl_db
    );
}

#[test]
fn test_parallel_rayon_acoustic_benchmark_10000_scenarios() {
    let report = AcousticBenchmarkRunner::run_benchmark(10_000);

    println!("=== 10,000-Scenario Multi-Medium Acoustic Propagation Benchmark ===");
    println!("Total Scenarios:       {}", report.total_scenarios);
    println!("Elapsed Time:          {:.2} ms", report.elapsed_ms);
    println!(
        "Throughput:            {:.0} scenarios/sec",
        report.scenarios_per_second
    );
    println!("Average SPL:           {:.2} dB SPL", report.average_spl_db);
    println!("Average Sabine T60:    {:.2} s", report.average_t60_seconds);
    println!(
        "Average Mic Voltage:   {:.3} mV",
        report.average_mic_voltage_mv
    );
    println!(
        "Max Doppler Shift:     {:.2} Hz",
        report.max_doppler_shift_hz
    );
    println!("Vacuum Silence:        {}", report.vacuum_silence_verified);

    assert_eq!(report.total_scenarios, 10_000);
    assert!(
        report.scenarios_per_second > 15_000.0,
        "Throughput below target: {:.0} scenarios/sec",
        report.scenarios_per_second
    );
    assert!(
        report.vacuum_silence_verified,
        "Vacuum isolation verification failed"
    );
    assert!(report.average_spl_db > 20.0 && report.average_spl_db < 120.0);
    assert!(report.average_t60_seconds > 0.2 && report.average_t60_seconds < 10.0);
}
