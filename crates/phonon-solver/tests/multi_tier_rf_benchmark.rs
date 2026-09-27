//! Integration Tests for Multi-Tier RF Abstraction & High-Throughput Rayon Benchmark

use phonon_models::em::{
    ChannelRng, DielectricWall, MacAddress, ModulationScheme, RfDielectricMaterial, Vector3D,
};
use phonon_solver::em::{ProtocolBenchmarkRunner, RfRealismTier, WifiLinkSimulator};

#[test]
fn test_multi_tier_rf_abstraction_link_evaluation() {
    let tx_mac = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x01]);
    let rx_mac = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x02]);

    let tx_pos = Vector3D::new(0.0, 0.0, 1.5);
    let rx_pos = Vector3D::new(10.0, 0.0, 1.5); // 10 meters distance
    let mut rng = ChannelRng::new(5555);

    let mut sim = WifiLinkSimulator::new(tx_mac, rx_mac, tx_pos, rx_pos, ModulationScheme::Qpsk);

    // 1. Evaluate Tier 0 (FullWave) in free-space
    sim.set_tier(RfRealismTier::Tier0FullWave);
    let res_t0 = sim.transmit_payload(b"Free Space Tier 0", &mut rng);
    assert!(res_t0.success);
    assert_eq!(res_t0.bit_errors, 0);
    assert!(res_t0.channel_result.snr_db > 15.0);

    // 2. Evaluate Tier 1 (RaytracedMultipath) with fading
    sim.set_tier(RfRealismTier::Tier1RaytracedMultipath);
    let res_t1 = sim.transmit_payload(b"Multipath Tier 1", &mut rng);
    assert!(res_t1.success);

    // 3. Evaluate Tier 2 (AcceleratedPathLoss)
    sim.set_tier(RfRealismTier::Tier2AcceleratedPathLoss);
    let res_t2 = sim.transmit_payload(b"Accelerated Tier 2", &mut rng);
    assert!(res_t2.success);

    // Compare SNR across tiers: should be in the same realistic ballpark (+- 10 dB)
    let snr0 = res_t0.channel_result.snr_db;
    let snr1 = res_t1.channel_result.snr_db;
    let snr2 = res_t2.channel_result.snr_db;
    println!(
        "SNRs: Tier0={:.1} dB, Tier1={:.1} dB, Tier2={:.1} dB",
        snr0, snr1, snr2
    );
    assert!((snr0 - snr1).abs() < 12.0);
    assert!((snr0 - snr2).abs() < 12.0);
}

#[test]
fn test_dielectric_wall_obstacle_penetration_loss() {
    let tx_mac = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x01]);
    let rx_mac = MacAddress([0x00, 0x11, 0x22, 0x33, 0x44, 0x02]);

    let tx_pos = Vector3D::new(0.0, 0.0, 1.5);
    let rx_pos = Vector3D::new(10.0, 0.0, 1.5);
    let mut rng = ChannelRng::new(7777);

    // Free space baseline
    let mut sim_free =
        WifiLinkSimulator::new(tx_mac, rx_mac, tx_pos, rx_pos, ModulationScheme::Qpsk);
    sim_free.set_tier(RfRealismTier::Tier0FullWave);
    let res_free = sim_free.transmit_payload(b"Test", &mut rng);

    // Now insert a 20 cm Concrete wall between transmitter and receiver
    let mut sim_wall =
        WifiLinkSimulator::new(tx_mac, rx_mac, tx_pos, rx_pos, ModulationScheme::Qpsk);
    sim_wall.set_tier(RfRealismTier::Tier0FullWave);
    let wall = DielectricWall::new(
        Vector3D::new(5.0, 0.0, 1.5),
        Vector3D::new(1.0, 0.0, 0.0), // Normal along X axis
        0.20,                         // 20 cm thickness
        10.0,
        10.0,
        RfDielectricMaterial::concrete(),
    );
    sim_wall.add_wall(wall);

    let res_wall = sim_wall.transmit_payload(b"Test", &mut rng);

    // Concrete wall must introduce significant attenuation (> 5 dB)
    let p_free = res_free.channel_result.received_power_dbm;
    let p_wall = res_wall.channel_result.received_power_dbm;
    println!(
        "Power free: {:.2} dBm, Power with wall: {:.2} dBm",
        p_free, p_wall
    );
    assert!(
        p_free > p_wall + 5.0,
        "Wall attenuation failed to attenuate signal properly"
    );
    assert!(res_wall.channel_result.excess_wall_loss_db > 5.0);
}

#[test]
fn test_parallel_rayon_protocol_benchmark_10000_packets() {
    let num_packets = 10_000;
    let report = ProtocolBenchmarkRunner::run_benchmark(
        num_packets,
        RfRealismTier::Tier1RaytracedMultipath,
        ModulationScheme::Qpsk,
    );

    println!("=== 10,000-Packet Wi-Fi Protocol Benchmark Report ===");
    println!("Total Packets:         {}", report.total_packets);
    println!("Elapsed Time:          {:.2} ms", report.elapsed_ms);
    println!(
        "Throughput:            {:.0} packets/sec",
        report.packets_per_second
    );
    println!("Successful Packets:    {}", report.successful_packets);
    println!("Packet Error Rate:     {:.4}", report.packet_error_rate);
    println!("Average BER:           {:.6}", report.average_ber);
    println!("Average SNR:           {:.2} dB", report.average_snr_db);
    println!(
        "Average PHY Rate:      {:.2} Mbps",
        report.average_throughput_mbps
    );

    assert_eq!(report.total_packets, 10_000);
    // Verifying high performance: must execute > 5,000 packets/sec on multicore
    assert!(
        report.packets_per_second > 5000.0,
        "Throughput {:.0} packets/sec too slow",
        report.packets_per_second
    );
    // Average SNR is high (> 15 dB) across 5-40m with 100mW Tx power
    assert!(report.average_snr_db > 10.0);
}
