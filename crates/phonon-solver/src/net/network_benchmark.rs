//! High-Throughput Rayon Parallel Network Co-Simulation Benchmark Runner
//!
//! Benchmarks 10,000+ packets routed between dual CPUs across physical dielectric obstacles,
//! validating packet loss, buffer exhaustion dynamics, and CPU execution overhead.

use crate::em::rf_tier_engine::RfRealismTier;
use crate::net::network_cosim_solver::NetworkCoSimulator;
use phonon_models::em::{
    ChannelRng, DielectricWall, ModulationScheme, RfDielectricMaterial, Vector3D,
};
use phonon_models::net::cpu_node::CpuInstruction;
use phonon_models::net::stack::Ipv4Address;
use rayon::prelude::*;
use std::time::Instant;

/// Comprehensive report produced by the Network Benchmark Runner.
#[derive(Debug, Clone, PartialEq)]
pub struct NetworkBenchmarkReport {
    /// Total number of packets initiated by CPU-A.
    pub total_packets: usize,
    /// Total elapsed wall-clock simulation time in milliseconds.
    pub elapsed_ms: f64,
    /// Parallel co-simulation throughput (packets processed per second).
    pub packets_per_second: f64,
    /// Number of packets successfully delivered to CPU-B and processed in ISR.
    pub successful_deliveries: usize,
    /// End-to-end Packet Loss Rate.
    pub packet_loss_rate: f64,
    /// Average clock cycles expended per packet transmission.
    pub average_cpu_cycles_per_packet: f64,
}

/// Network Co-Simulation Benchmark Runner executing parallel Rayon sweeps.
pub struct NetworkBenchmarkRunner;

impl NetworkBenchmarkRunner {
    /// Benchmarks end-to-end packet co-simulation across 10,000 packets in parallel.
    pub fn run_benchmark(
        num_packets: usize,
        tier: RfRealismTier,
        modulation: ModulationScheme,
        include_wall: bool,
    ) -> NetworkBenchmarkReport {
        let start_time = Instant::now();

        let results: Vec<(bool, u64)> = (0..num_packets)
            .into_par_iter()
            .map(|i| {
                let mut rng = ChannelRng::new(
                    0xFEEDFACECAFEBEEF ^ (i as u64).wrapping_mul(0x5851F42D4C957F2D),
                );
                let pos_a = Vector3D::new(0.0, 0.0, 1.5);
                let pos_router = Vector3D::new(10.0, 0.0, 1.5);
                let pos_b = Vector3D::new(20.0, 0.0, 1.5);

                let mut sim = NetworkCoSimulator::new(pos_a, pos_router, pos_b, tier, modulation);

                if include_wall {
                    // 15 cm concrete wall between CPU-A and Router
                    let wall = DielectricWall::new(
                        Vector3D::new(5.0, 0.0, 1.5),
                        Vector3D::new(1.0, 0.0, 0.0),
                        0.15,
                        10.0,
                        10.0,
                        RfDielectricMaterial::concrete(),
                    );
                    sim.add_wall(wall);
                }

                // Program CPU-A to send a UDP datagram to CPU-B (192.168.2.20:8080)
                let dst_ip = Ipv4Address::new(192, 168, 2, 20);
                let payload = format!("TeleCommand-Seq-{}", i).into_bytes();
                sim.cpu_a.load_program(vec![
                    CpuInstruction::SendUdp {
                        dst_ip,
                        src_port: 5000,
                        dst_port: 8080,
                        payload,
                    },
                    CpuInstruction::Nop,
                ]);

                // Step simulation until packet arrives or cycle limit reached
                let mut success = false;
                for _ in 0..10 {
                    let report = sim.step(&mut rng);
                    if report.payload_received_by_cpu_b {
                        success = true;
                        break;
                    }
                }

                (success, sim.cpu_a.clock_cycles + sim.cpu_b.clock_cycles)
            })
            .collect();

        let elapsed_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let packets_per_sec = (num_packets as f64) / (elapsed_ms * 1e-3).max(1e-6);

        let successful = results.iter().filter(|r| r.0).count();
        let loss_rate = 1.0 - (successful as f64) / (num_packets as f64);
        let avg_cycles = (results.iter().map(|r| r.1).sum::<u64>() as f64) / (num_packets as f64);

        NetworkBenchmarkReport {
            total_packets: num_packets,
            elapsed_ms,
            packets_per_second: packets_per_sec,
            successful_deliveries: successful,
            packet_loss_rate: loss_rate,
            average_cpu_cycles_per_packet: avg_cycles,
        }
    }
}
