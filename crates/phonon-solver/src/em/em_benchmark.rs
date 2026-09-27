//! High-Throughput Parallel Electromagnetic Wave & Orbital Link Benchmark.
//!
//! Evaluates multi-threaded Rayon execution throughput, statistical path loss distributions,
//! wall obstacle penetration, and solar burst link degradation across 10,000 simulated links.

use super::em_wave_solver::{EmLinkResult, EmPropagationScene, EmWaveSolver};
use super::space_link_solver::{
    GroundStation, SatelliteNode, SpaceLinkBudgetResult, SpaceLinkSolver,
};
use phonon_models::em::{
    DielectricWall, EcefCoord, EmWaveSource, GeodeticCoord, Polarization, RfDielectricMaterial,
    Vector3D,
};
use std::time::Instant;

/// Benchmark performance and statistical summary report.
#[derive(Debug, Clone, PartialEq)]
pub struct EmBenchmarkReport {
    /// Total terrestrial links simulated.
    pub terrestrial_links_evaluated: usize,
    /// Total space/orbital links simulated.
    pub space_links_evaluated: usize,
    /// Total execution time in milliseconds.
    pub elapsed_time_ms: f64,
    /// Throughput in evaluated links per second.
    pub throughput_links_per_sec: f64,
    /// Mean received SNR across terrestrial links (dB).
    pub mean_terrestrial_snr_db: f64,
    /// Mean wall penetration loss across obstructed links (dB).
    pub mean_wall_loss_db: f64,
    /// Percentage of terrestrial links with clear line of sight (%).
    pub line_of_sight_percentage: f64,
    /// Mean Doppler frequency shift magnitude across orbital links (Hz).
    pub mean_orbital_doppler_hz: f64,
    /// Number of orbital links experiencing active solar outage.
    pub solar_outage_links_count: usize,
}

/// Electromagnetic and Space Channel Benchmark Engine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EmBenchmarkRunner;

impl EmBenchmarkRunner {
    /// Runs a comprehensive multi-threaded benchmark across 10,000 links.
    pub fn run_benchmark(num_terrestrial: usize, num_space: usize) -> EmBenchmarkReport {
        let start_time = Instant::now();

        // 1. Synthesize Complex Indoor/Outdoor Urban Propagation Scene
        let mut noise_model = phonon_models::em::RfNoiseModel::new(20.0e6, 4.0);
        noise_model.rain_rate_mm_hr = 15.0; // Moderate rain
        noise_model.rain_path_length_km = 0.5;

        let mut scene = EmPropagationScene::new(noise_model);

        // Add 20 Realistic Dielectric Building Walls
        for i in 0..10 {
            let x = (i as f64 - 5.0) * 20.0;
            // North-South concrete walls
            scene.add_wall(DielectricWall::new(
                Vector3D::new(x, 0.0, 5.0),
                Vector3D::new(1.0, 0.0, 0.0),
                0.20, // 20 cm concrete
                30.0,
                10.0,
                RfDielectricMaterial::concrete(),
            ));
            // East-West glass/drywall partitions
            let y = (i as f64 - 5.0) * 15.0;
            scene.add_wall(DielectricWall::new(
                Vector3D::new(0.0, y, 5.0),
                Vector3D::new(0.0, 1.0, 0.0),
                0.015, // 15 mm glass
                25.0,
                8.0,
                RfDielectricMaterial::glass(),
            ));
        }

        // 2. Synthesize Transmitters and Receivers
        let tx = EmWaveSource::new(
            2.45e9, // 2.45 GHz Wi-Fi band
            1.0,    // 1 Watt (30 dBm)
            2.15,   // Dipole gain (3.3 dBi)
            Polarization::LinearVertical,
            Vector3D::new(0.0, 0.0, 15.0), // Base station antenna at 15m
            Vector3D::new(1.0, 0.0, 0.0),
        );

        let rx_pol = Polarization::LinearVertical;
        let rx_gain = 1.5;

        // Generate synthetic grid of receiver positions
        let mut rx_positions = Vec::with_capacity(num_terrestrial);
        for i in 0..num_terrestrial {
            let radius = 10.0 + (i % 500) as f64 * 1.5;
            let angle = (i as f64 * 0.05).sin() * std::f64::consts::PI;
            let z = 1.5 + (i % 5) as f64 * 3.0; // Handheld at 1.5m or floors
            rx_positions.push(Vector3D::new(radius * angle.cos(), radius * angle.sin(), z));
        }

        // Parallel execution of terrestrial EM links
        let terrestrial_results = EmWaveSolver::solve_links_parallel(
            &scene,
            &tx,
            &rx_positions,
            rx_gain,
            &rx_pol,
            15.0, // 15 deg elevation
        );

        // 3. Synthesize Space Orbital Links (LEO Satellite Constellation)
        let num_stations = 10;
        let num_sats = (num_space / num_stations).max(1);

        let mut stations = Vec::with_capacity(num_stations);
        for s in 0..num_stations {
            let lat = -60.0 + s as f64 * 12.0;
            let lon = -120.0 + s as f64 * 24.0;
            stations.push(GroundStation::new(
                format!("GS-{}", s),
                GeodeticCoord::new(lat, lon, 100.0),
                5.0,
                38.0, // 38 dBi parabolic dish
                1.5,  // 1.5 dB cryogenic/LNA noise figure
                2.5,  // 2.5 deg beamwidth
            ));
        }

        let mut satellites = Vec::with_capacity(num_sats);
        for k in 0..num_sats {
            let true_anomaly = (k as f64 / num_sats as f64) * 2.0 * std::f64::consts::PI;
            let r_orbit = 6_371_000.0 + 550_000.0; // 550 km LEO altitude
            let v_orbit = 7560.0; // ~7.56 km/s LEO orbital velocity

            let pos_ecef = EcefCoord::new(
                r_orbit * true_anomaly.cos(),
                r_orbit * true_anomaly.sin() * 0.8,
                r_orbit * true_anomaly.sin() * 0.6,
            );
            let vel = Vector3D::new(
                -v_orbit * true_anomaly.sin(),
                v_orbit * true_anomaly.cos() * 0.8,
                v_orbit * true_anomaly.cos() * 0.6,
            );

            satellites.push(SatelliteNode::new(
                format!("Sat-{}", k),
                pos_ecef,
                vel,
                20.0,   // 20 Watts RF
                12.0e9, // 12 GHz Ku-band downlink
                32.0,   // 32 dBi satellite antenna
                4.0,    // 4.0 deg beamwidth
            ));
        }

        // Sun position at 1 AU (~1.496e11 m) along ECEF X-axis
        let sun_pos = EcefCoord::new(1.496e11, 0.0, 0.0);
        let solar_flux = 250.0; // Active solar radio burst
        let tecu = 45.0; // Elevated ionosphere

        let space_results = SpaceLinkSolver::evaluate_constellation_parallel(
            &stations,
            &satellites,
            Some(&sun_pos),
            solar_flux,
            tecu,
            50.0e6, // 50 MHz wideband carrier
        );

        let elapsed_secs = start_time.elapsed().as_secs_f64();
        let elapsed_ms = elapsed_secs * 1000.0;
        let total_links = terrestrial_results.len() + space_results.len();
        let throughput = if elapsed_secs > 1e-9 {
            total_links as f64 / elapsed_secs
        } else {
            0.0
        };

        // 4. Aggregate Statistical Metrics
        let (snr_sum, wall_loss_sum, los_count) = terrestrial_results.iter().fold(
            (0.0, 0.0, 0usize),
            |(acc_snr, acc_loss, acc_los), res: &EmLinkResult| {
                (
                    acc_snr + res.snr_db,
                    acc_loss + res.wall_penetration_loss_db,
                    acc_los + if res.has_clear_line_of_sight { 1 } else { 0 },
                )
            },
        );

        let mean_snr = snr_sum / terrestrial_results.len().max(1) as f64;
        let mean_wall_loss = wall_loss_sum / terrestrial_results.len().max(1) as f64;
        let los_pct = (los_count as f64 / terrestrial_results.len().max(1) as f64) * 100.0;

        let (doppler_sum, solar_outages) = space_results.iter().fold(
            (0.0, 0usize),
            |(acc_dop, acc_out): (f64, usize), res: &SpaceLinkBudgetResult| {
                (
                    acc_dop + res.doppler_shift_hz.abs(),
                    acc_out + if res.solar_outage_active { 1 } else { 0 },
                )
            },
        );

        let mean_doppler = doppler_sum / space_results.len().max(1) as f64;

        EmBenchmarkReport {
            terrestrial_links_evaluated: terrestrial_results.len(),
            space_links_evaluated: space_results.len(),
            elapsed_time_ms: elapsed_ms,
            throughput_links_per_sec: throughput,
            mean_terrestrial_snr_db: mean_snr,
            mean_wall_loss_db: mean_wall_loss,
            line_of_sight_percentage: los_pct,
            mean_orbital_doppler_hz: mean_doppler,
            solar_outage_links_count: solar_outages,
        }
    }
}
