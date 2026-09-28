//! Comparative Benchmark Engine: Diamond NV Quantum Sensors vs SQUID vs Hall Effect
//!
//! Evaluates magnetometry sensor technologies across:
//! 1. Spatial resolution ($< 10\text{ nm}$ for NV vs $\sim 5\,\mu\text{m}$ for SQUID vs $\sim 1\,\mu\text{m}$ for Hall).
//! 2. Operating temperature range ($0\text{ K}$ to $> 600\text{ K}$ for NV vs cryogenic $4.2\text{ K}$ for low-$T_c$ SQUID).
//! 3. Microwave / RF sensing bandwidth (DC to GHz for NV vs $\sim 10\text{ MHz}$ for SQUID vs $\sim 1\text{ MHz}$ for Hall).
//! 4. Standby power ($0.0\text{ W}$ passive diamond head vs $> 500\text{ W}$ cryocooler for SQUID vs $\sim 5\text{ mW}$ for Hall).
//! 5. Multi-threaded Rayon execution over large-scale parameter sweeps.

use rayon::prelude::*;
use std::time::Instant;

/// Sensor technology type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MagnetometerTechnology {
    /// Diamond Nitrogen-Vacancy (NV) Center quantum sensor.
    DiamondNvCenter,
    /// Superconducting Quantum Interference Device (SQUID).
    SquidMagnetometer,
    /// Semiconductor Hall-effect sensor.
    HallSensor,
}

/// Physical specifications and operating boundaries of a magnetometer technology.
#[derive(Debug, Clone, PartialEq)]
pub struct MagnetometerSpecs {
    /// Technology type.
    pub technology: MagnetometerTechnology,
    /// Physical spatial resolution in meters.
    pub spatial_resolution_m: f64,
    /// Minimum operating temperature in Kelvin.
    pub min_temperature_k: f64,
    /// Maximum operating temperature in Kelvin.
    pub max_temperature_k: f64,
    /// Maximum sensing frequency / microwave bandwidth in Hertz.
    pub bandwidth_hz: f64,
    /// Standby electrical power consumption in Watts.
    pub standby_power_w: f64,
    /// Magnetic field sensitivity in $\text{T}/\sqrt{\text{Hz}}$.
    pub sensitivity_t_per_rt_hz: f64,
}

impl MagnetometerSpecs {
    /// Diamond Nitrogen-Vacancy quantum sensor specifications:
    /// - Spatial resolution: $< 10\text{ nm}$ ($10.0\text{ nm}$).
    /// - Temperature: $0\text{ K}$ to $650\text{ K}$ (ambient/high-temp diamond stability).
    /// - Bandwidth: DC to $12\text{ GHz}$ (via microwave ODMR).
    /// - Standby power: $0.0\text{ W}$ (passive crystal head).
    /// - Sensitivity: $\approx 1.0\text{ pT}/\sqrt{\text{Hz}}$ (ensemble) / $50\text{ nT}/\sqrt{\text{Hz}}$ (single NV).
    pub fn diamond_nv() -> Self {
        Self {
            technology: MagnetometerTechnology::DiamondNvCenter,
            spatial_resolution_m: 10.0e-9,
            min_temperature_k: 0.1,
            max_temperature_k: 650.0,
            bandwidth_hz: 12.0e9,
            standby_power_w: 0.0,
            sensitivity_t_per_rt_hz: 1.0e-12,
        }
    }

    /// Superconducting Quantum Interference Device (SQUID) specifications:
    /// - Spatial resolution: $\approx 5\,\mu\text{m}$ ($5000\text{ nm}$, pickup loop limited).
    /// - Temperature: $0.01\text{ K}$ to $4.2\text{ K}$ (cryogenic liquid Helium required).
    /// - Bandwidth: DC to $10\text{ MHz}$ (flux lock loop limited).
    /// - Standby power: $750.0\text{ W}$ (Carnot-limited closed-cycle cryocooler overhead).
    /// - Sensitivity: $\approx 5.0\text{ fT}/\sqrt{\text{Hz}}$ ($5.0\times 10^{-15}\text{ T}/\sqrt{\text{Hz}}$).
    pub fn squid() -> Self {
        Self {
            technology: MagnetometerTechnology::SquidMagnetometer,
            spatial_resolution_m: 5.0e-6,
            min_temperature_k: 0.01,
            max_temperature_k: 4.2,
            bandwidth_hz: 10.0e6,
            standby_power_w: 750.0,
            sensitivity_t_per_rt_hz: 5.0e-15,
        }
    }

    /// Semiconductor Hall-effect sensor specifications:
    /// - Spatial resolution: $\approx 1\,\mu\text{m}$ ($1000\text{ nm}$, lithography and 2DEG depletion limited).
    /// - Temperature: $200.0\text{ K}$ to $420.0\text{ K}$ (carrier freeze-out at cryogenic, thermal noise at high temp).
    /// - Bandwidth: DC to $1.0\text{ MHz}$.
    /// - Standby power: $5.0\text{ mW}$ ($0.005\text{ W}$ constant bias current).
    /// - Sensitivity: $\approx 100.0\text{ nT}/\sqrt{\text{Hz}}$ ($1.0\times 10^{-7}\text{ T}/\sqrt{\text{Hz}}$).
    pub fn hall() -> Self {
        Self {
            technology: MagnetometerTechnology::HallSensor,
            spatial_resolution_m: 1.0e-6,
            min_temperature_k: 200.0,
            max_temperature_k: 420.0,
            bandwidth_hz: 1.0e6,
            standby_power_w: 0.005,
            sensitivity_t_per_rt_hz: 1.0e-7,
        }
    }

    /// Checks if the sensor is physically operable under the given ambient temperature and RF frequency.
    pub fn is_operable(&self, temp_k: f64, freq_hz: f64) -> bool {
        temp_k >= self.min_temperature_k
            && temp_k <= self.max_temperature_k
            && freq_hz <= self.bandwidth_hz
    }

    /// Checks if the sensor can spatially resolve the target feature size.
    pub fn can_resolve(&self, feature_size_m: f64) -> bool {
        self.spatial_resolution_m <= feature_size_m
    }

    /// Scores the sensor for a specific application scenario (0.0 to 100.0).
    pub fn score_scenario(&self, scenario: &BenchmarkScenario) -> f64 {
        if !self.is_operable(scenario.temperature_k, scenario.required_bandwidth_hz) {
            return 0.0;
        }

        let mut score: f64 = 50.0;

        // Resolution score
        if self.can_resolve(scenario.target_feature_size_m) {
            score += 25.0;
            // Bonus for sub-10 nm resolving capability
            if self.spatial_resolution_m <= 15.0e-9 {
                score += 10.0;
            }
        } else {
            score -= 30.0;
        }

        // Power constraint score
        if scenario.power_constrained {
            if self.standby_power_w == 0.0 {
                score += 15.0;
            } else if self.standby_power_w > 10.0 {
                score -= 35.0;
            }
        }

        score.clamp(0.0, 100.0)
    }
}

/// Test scenario for comparative sensor benchmarking.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkScenario {
    /// Ambient operating temperature in Kelvin.
    pub temperature_k: f64,
    /// Required measurement frequency in Hertz.
    pub required_bandwidth_hz: f64,
    /// Target spatial feature size to resolve in meters.
    pub target_feature_size_m: f64,
    /// Whether the scenario has strict power limits (e.g. mobile/uncooled probe head).
    pub power_constrained: bool,
}

/// Comprehensive comparative benchmark report.
#[derive(Debug, Clone, PartialEq)]
pub struct NvBenchmarkReport {
    /// Total number of scenarios evaluated in parallel.
    pub num_scenarios_evaluated: usize,
    /// Percentage of scenarios where Diamond NV was viable (> 0 score).
    pub nv_success_rate: f64,
    /// Percentage of scenarios where SQUID was viable.
    pub squid_success_rate: f64,
    /// Percentage of scenarios where Hall sensor was viable.
    pub hall_success_rate: f64,
    /// Average score of Diamond NV across all scenarios.
    pub nv_average_score: f64,
    /// Average score of SQUID across all scenarios.
    pub squid_average_score: f64,
    /// Average score of Hall sensor across all scenarios.
    pub hall_average_score: f64,
    /// Throughput in scenarios per second.
    pub throughput_scenarios_per_sec: f64,
    /// Total benchmark execution elapsed time in seconds.
    pub elapsed_duration_s: f64,
    /// Formatted markdown summary report.
    pub summary_markdown: String,
}

/// Runner for multi-threaded comparative magnetometer benchmarks.
pub struct NvBenchmarkRunner;

impl NvBenchmarkRunner {
    /// Generates a synthetic testbed of $N$ benchmark scenarios spanning wide temperature,
    /// bandwidth, and feature size domains.
    pub fn generate_testbed(num_scenarios: usize) -> Vec<BenchmarkScenario> {
        let mut scenarios = Vec::with_capacity(num_scenarios);
        for i in 0..num_scenarios {
            // Sweep temperatures: from cryogenic 1K to room 300K to hot 550K
            let temp_k = match i % 5 {
                0 => 2.0,   // Cryogenic Liquid He
                1 => 77.0,  // Liquid N2
                2 => 300.0, // Room temperature
                3 => 373.0, // Boiling water / hot die
                _ => 500.0, // High temperature IC environment
            };

            // Sweep feature sizes: from nanoscale 8nm to micron 2um to macro 20um
            let feature_size_m = match (i / 5) % 4 {
                0 => 8.0e-9,  // 8 nm IC wire
                1 => 25.0e-9, // 25 nm fin pitch
                2 => 1.5e-6,  // 1.5 um metal trace
                _ => 25.0e-6, // 25 um PCB trace
            };

            // Sweep bandwidth: DC to GHz
            let bandwidth_hz = match (i / 20) % 4 {
                0 => 100.0,   // 100 Hz DC
                1 => 500.0e3, // 500 kHz
                2 => 5.0e6,   // 5 MHz
                _ => 2.5e9,   // 2.5 GHz RF microwave
            };

            let power_constrained = (i % 2) == 0;

            scenarios.push(BenchmarkScenario {
                temperature_k: temp_k,
                required_bandwidth_hz: bandwidth_hz,
                target_feature_size_m: feature_size_m,
                power_constrained,
            });
        }
        scenarios
    }

    /// Executes multi-threaded Rayon comparative benchmark across all scenarios.
    pub fn run_benchmark(scenarios: &[BenchmarkScenario]) -> NvBenchmarkReport {
        let nv = MagnetometerSpecs::diamond_nv();
        let squid = MagnetometerSpecs::squid();
        let hall = MagnetometerSpecs::hall();

        let start_time = Instant::now();

        let scores: Vec<(f64, f64, f64)> = scenarios
            .par_iter()
            .map(|sc| {
                let s_nv = nv.score_scenario(sc);
                let s_squid = squid.score_scenario(sc);
                let s_hall = hall.score_scenario(sc);
                (s_nv, s_squid, s_hall)
            })
            .collect();

        let elapsed = start_time.elapsed().as_secs_f64().max(1e-9);
        let n = scenarios.len() as f64;

        let mut nv_successes = 0.0;
        let mut squid_successes = 0.0;
        let mut hall_successes = 0.0;

        let mut nv_sum = 0.0;
        let mut squid_sum = 0.0;
        let mut hall_sum = 0.0;

        for &(s_nv, s_squid, s_hall) in &scores {
            if s_nv > 0.0 {
                nv_successes += 1.0;
            }
            if s_squid > 0.0 {
                squid_successes += 1.0;
            }
            if s_hall > 0.0 {
                hall_successes += 1.0;
            }
            nv_sum += s_nv;
            squid_sum += s_squid;
            hall_sum += s_hall;
        }

        let nv_success_rate = (nv_successes / n) * 100.0;
        let squid_success_rate = (squid_successes / n) * 100.0;
        let hall_success_rate = (hall_successes / n) * 100.0;

        let nv_avg = nv_sum / n;
        let squid_avg = squid_sum / n;
        let hall_avg = hall_sum / n;

        let throughput = scenarios.len() as f64 / elapsed;

        let summary = format!(
            "| Magnetometer Technology | Spatial Resolution | Operating Temp | Bandwidth | Standby Power | Viability Rate | Average Score |\n\
             | :--- | :--- | :--- | :--- | :--- | :--- | :--- |\n\
             | **Diamond NV Center** | **< 10 nm** | **0.1 K - 650 K** | **DC - 12 GHz** | **0.0 W** | **{:.1}%** | **{:.1} / 100** |\n\
             | SQUID | ~ 5 um | 0.01 K - 4.2 K | DC - 10 MHz | 750.0 W | {:.1}% | {:.1} / 100 |\n\
             | Hall Sensor | ~ 1 um | 200 K - 420 K | DC - 1 MHz | 5.0 mW | {:.1}% | {:.1} / 100 |",
            nv_success_rate, nv_avg, squid_success_rate, squid_avg, hall_success_rate, hall_avg
        );

        NvBenchmarkReport {
            num_scenarios_evaluated: scenarios.len(),
            nv_success_rate,
            squid_success_rate,
            hall_success_rate,
            nv_average_score: nv_avg,
            squid_average_score: squid_avg,
            hall_average_score: hall_avg,
            throughput_scenarios_per_sec: throughput,
            elapsed_duration_s: elapsed,
            summary_markdown: summary,
        }
    }
}
