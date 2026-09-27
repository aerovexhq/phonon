//! High-Throughput Rayon Parallel Protocol Benchmark Runner
//!
//! Evaluates end-to-end digital PHY/MAC packet transmission across 10,000+ links,
//! benchmarking multi-tier execution latency and validating Bit Error Rate (BER) curves.

use crate::em::rf_tier_engine::RfRealismTier;
use crate::em::wifi_protocol_solver::WifiLinkSimulator;
use phonon_models::em::{ChannelRng, MacAddress, ModulationScheme, Vector3D};
use rayon::prelude::*;
use std::time::Instant;

/// Performance and statistical report produced by the Protocol Benchmark Runner.
#[derive(Debug, Clone, PartialEq)]
pub struct ProtocolBenchmarkReport {
    /// Total number of packets evaluated across all threads.
    pub total_packets: usize,
    /// Total elapsed wall-clock simulation time in milliseconds.
    pub elapsed_ms: f64,
    /// Parallel packet evaluation throughput (packets per second).
    pub packets_per_second: f64,
    /// Number of successfully received packets passing CRC-32 FCS check.
    pub successful_packets: usize,
    /// Packet Error Rate: $\text{PER} = 1 - \frac{N_{success}}{N_{total}}$.
    pub packet_error_rate: f64,
    /// Average empirical Bit Error Rate across all packets.
    pub average_ber: f64,
    /// Average received SNR in decibels.
    pub average_snr_db: f64,
    /// Average effective throughput in Mbps.
    pub average_throughput_mbps: f64,
}

/// Protocol Benchmark Runner executing parallel Rayon sweeps.
pub struct ProtocolBenchmarkRunner;

impl ProtocolBenchmarkRunner {
    /// Benchmarks packet transmission across a range of SNR/distance conditions in parallel.
    pub fn run_benchmark(
        num_packets: usize,
        tier: RfRealismTier,
        modulation: ModulationScheme,
    ) -> ProtocolBenchmarkReport {
        let start_time = Instant::now();
        let payload = vec![0x55; 128]; // 128-byte test payload

        let results: Vec<(bool, f64, f64, f64)> = (0..num_packets)
            .into_par_iter()
            .map(|i| {
                let mut rng = ChannelRng::new(
                    0x1234567890ABCDEF ^ (i as u64).wrapping_mul(0x9E3779B97F4A7C15),
                );
                let tx_mac = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x01]);
                let rx_mac = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x02]);

                // Vary distance smoothly between 5.0 meters and 40.0 meters
                let dist = 5.0 + ((i % 100) as f64) * 0.35;
                let tx_pos = Vector3D::new(0.0, 0.0, 1.5);
                let rx_pos = Vector3D::new(dist, 0.0, 1.5);

                let mut sim = WifiLinkSimulator::new(tx_mac, rx_mac, tx_pos, rx_pos, modulation);
                sim.set_tier(tier);

                let tx_res = sim.transmit_payload(&payload, &mut rng);
                (
                    tx_res.success,
                    tx_res.empirical_ber,
                    tx_res.channel_result.snr_db,
                    tx_res.effective_throughput_mbps,
                )
            })
            .collect();

        let elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let packets_per_sec = (num_packets as f64) / (elapsed_ms * 1e-3).max(1e-6);

        let successful = results.iter().filter(|r| r.0).count();
        let per = 1.0 - (successful as f64) / (num_packets as f64);
        let avg_ber = results.iter().map(|r| r.1).sum::<f64>() / (num_packets as f64);
        let avg_snr = results.iter().map(|r| r.2).sum::<f64>() / (num_packets as f64);
        let avg_tput = results.iter().map(|r| r.3).sum::<f64>() / (num_packets as f64);

        ProtocolBenchmarkReport {
            total_packets: num_packets,
            elapsed_ms,
            packets_per_second: packets_per_sec,
            successful_packets: successful,
            packet_error_rate: per,
            average_ber: avg_ber,
            average_snr_db: avg_snr,
            average_throughput_mbps: avg_tput,
        }
    }
}
