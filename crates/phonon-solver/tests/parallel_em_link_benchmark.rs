//! Integration Tests for Parallel Rayon EM Wave & Space Orbital Link Solver Benchmark.

use approx::assert_relative_eq;
use phonon_models::em::{
    DielectricWall, EmWaveSource, Polarization, RfDielectricMaterial, RfNoiseModel, Vector3D,
};
use phonon_solver::em::{EmBenchmarkRunner, EmPropagationScene, EmWaveSolver};

#[test]
fn test_end_to_end_em_wave_link_through_wall() {
    let mut noise_model = RfNoiseModel::new(20.0e6, 5.0);
    noise_model.rain_rate_mm_hr = 0.0; // Clear sky

    let mut scene = EmPropagationScene::new(noise_model);

    // Place a 30 cm concrete wall at x = 10m
    scene.add_wall(DielectricWall::new(
        Vector3D::new(10.0, 0.0, 0.0),
        Vector3D::new(1.0, 0.0, 0.0),
        0.30, // 30 cm
        20.0,
        10.0,
        RfDielectricMaterial::concrete(),
    ));

    // Transmitter at origin (x=0, y=0, z=1.5m)
    let tx = EmWaveSource::new(
        2.45e9, // 2.45 GHz
        0.1,    // 100 mW (20 dBm)
        2.15,   // Dipole gain (3.3 dBi)
        Polarization::LinearVertical,
        Vector3D::new(0.0, 0.0, 1.5),
        Vector3D::new(1.0, 0.0, 0.0),
    );

    // Receiver at x=20m (directly across the concrete wall)
    let rx_pos = Vector3D::new(20.0, 0.0, 1.5);
    let rx_gain = 2.15; // 3.3 dBi dipole
    let rx_pol = Polarization::LinearVertical;

    let res = EmWaveSolver::solve_link(&scene, &tx, &rx_pos, rx_gain, &rx_pol, 30.0);

    assert_eq!(res.distance_m, 20.0);
    assert_eq!(res.wall_intersections_count, 1);
    assert!(!res.has_clear_line_of_sight);

    // 20m FSPL at 2.45 GHz ~ 66.27 dB
    assert_relative_eq!(res.free_space_path_loss_db, 66.27, epsilon = 0.5);

    // 30 cm concrete wall attenuation should be significant (>= 8 dB)
    assert!(res.wall_penetration_loss_db >= 8.0);

    // Matched vertical polarization -> 0 dB loss
    assert_relative_eq!(res.polarization_loss_db, 0.0, epsilon = 1e-6);

    // Rx power: EIRP(23.3 dBm) + RxGain(3.3 dBi) - FSPL(66.3 dB) - WallLoss(>= 8 dB) <= -48 dBm
    assert!(res.rx_power_dbm <= -48.0);

    // Noise floor for 20 MHz, 5 dB NF ~ -98 dBm
    assert_relative_eq!(res.noise_floor_dbm, -98.0, epsilon = 2.0);

    // SNR should be well above zero (> 20 dB) for 20m distance even through concrete
    assert!(res.snr_db > 20.0);
}

#[test]
fn test_parallel_em_benchmark_scaling_and_throughput() {
    let num_terrestrial = 2000;
    let num_space = 1000;

    let report = EmBenchmarkRunner::run_benchmark(num_terrestrial, num_space);

    assert_eq!(report.terrestrial_links_evaluated, 2000);
    assert!(report.space_links_evaluated >= 1000);

    // Should finish rapidly with Rayon multi-threading
    assert!(report.elapsed_time_ms > 0.0);
    assert!(report.throughput_links_per_sec > 5_000.0);

    // Mean SNR should be reasonable (between -20 dB and +50 dB)
    assert!(report.mean_terrestrial_snr_db > -20.0);
    assert!(report.mean_terrestrial_snr_db < 60.0);

    // Mean wall loss should be strictly positive due to 20 building walls
    assert!(report.mean_wall_loss_db > 0.0);

    // Line of sight percentage should be between 5% and 95%
    assert!(report.line_of_sight_percentage > 5.0);
    assert!(report.line_of_sight_percentage < 95.0);

    // Orbital links should exhibit Doppler shift (LEO orbital speed ~7.5 km/s at 12 GHz -> Doppler in tens of kHz)
    assert!(report.mean_orbital_doppler_hz > 10_000.0);
}
