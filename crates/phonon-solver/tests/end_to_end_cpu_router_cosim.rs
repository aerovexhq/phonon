//! Integration Tests for End-to-End CPU-to-Router Co-Simulation across Physical Walls

use phonon_models::em::{
    ChannelRng, DielectricWall, ModulationScheme, RfDielectricMaterial, Vector3D,
};
use phonon_models::net::cpu_node::CpuInstruction;
use phonon_models::net::stack::Ipv4Address;
use phonon_solver::em::RfRealismTier;
use phonon_solver::net::{NetworkBenchmarkRunner, NetworkCoSimulator};

#[test]
fn test_dual_cpu_router_cosim_across_concrete_wall() {
    let pos_a = Vector3D::new(0.0, 0.0, 1.5);
    let pos_router = Vector3D::new(10.0, 0.0, 1.5);
    let pos_b = Vector3D::new(20.0, 0.0, 1.5);

    let mut rng = ChannelRng::new(424242);

    let mut sim = NetworkCoSimulator::new(
        pos_a,
        pos_router,
        pos_b,
        RfRealismTier::Tier1RaytracedMultipath,
        ModulationScheme::Qpsk,
    );

    // Place a 20 cm Concrete Wall between CPU-A and the Router
    let wall = DielectricWall::new(
        Vector3D::new(5.0, 0.0, 1.5),
        Vector3D::new(1.0, 0.0, 0.0), // Normal along X
        0.20,                         // 20 cm thickness
        10.0,
        10.0,
        RfDielectricMaterial::concrete(),
    );
    sim.add_wall(wall);

    let target_ip = Ipv4Address::new(192, 168, 2, 20); // CPU-B IP
    let test_data = b"Ping From CPU-A across Physical Concrete Wall!".to_vec();

    // Load CPU-A with UDP transmission program
    sim.cpu_a.load_program(vec![
        CpuInstruction::SendUdp {
            dst_ip: target_ip,
            src_port: 4000,
            dst_port: 9000,
            payload: test_data.clone(),
        },
        CpuInstruction::Nop,
        CpuInstruction::Nop,
    ]);

    // Step co-simulation loop
    let mut delivered = false;
    for step_idx in 0..10 {
        let report = sim.step(&mut rng);
        if report.payload_received_by_cpu_b {
            delivered = true;
            println!("Delivered on step {}", step_idx);
            break;
        }
    }

    assert!(
        delivered,
        "Packet failed to reach CPU-B through physical obstacle & Router"
    );
    assert_eq!(sim.cpu_b.last_received_payload, Some(test_data));
    assert!(
        sim.cpu_b.irq_count >= 1,
        "CPU-B ISR was not triggered by incoming packet"
    );
    assert!(sim.cpu_a.clock_cycles > 0);
    assert!(sim.cpu_b.clock_cycles > 0);
}

#[test]
fn test_parallel_rayon_network_benchmark_10000_packets() {
    let num_packets = 10_000;
    let report = NetworkBenchmarkRunner::run_benchmark(
        num_packets,
        RfRealismTier::Tier1RaytracedMultipath,
        ModulationScheme::Qpsk,
        true, // Include physical concrete wall
    );

    println!("=== 10,000-Packet End-to-End CPU-to-Router Co-Simulation Benchmark ===");
    println!("Total Packets:         {}", report.total_packets);
    println!("Elapsed Time:          {:.2} ms", report.elapsed_ms);
    println!(
        "Throughput:            {:.0} packets/sec",
        report.packets_per_second
    );
    println!("Successful Deliveries: {}", report.successful_deliveries);
    println!("Packet Loss Rate:      {:.4}", report.packet_loss_rate);
    println!(
        "Avg Cycles / Packet:   {:.1}",
        report.average_cpu_cycles_per_packet
    );

    assert_eq!(report.total_packets, 10_000);
    // Verifying high performance: must execute > 4,000 packets/sec on multicore
    assert!(
        report.packets_per_second > 4000.0,
        "Throughput {:.0} packets/sec too slow",
        report.packets_per_second
    );
    // Over 99% delivery rate across 20m through 15 cm concrete wall
    assert!(report.successful_deliveries > 9500);
}
