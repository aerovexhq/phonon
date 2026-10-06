#![deny(unsafe_code)]

//! Test suite for Phase 379: Deterministic SpaceWire/SpaceFibre & Avionics AFDX Bus Contention Co-Simulator.

use phonon_solver::space_avionics_bus::{
    AfdxSwitch, AfdxVirtualLink, NoCMeshSimulator, SpFiQoSScheduling,
    SpaceAvionicsBusCoSimulator, SpaceFibreMultiLaneLink, SpaceWireLink,
};
use std::time::Instant;

#[test]
fn test_spacewire_credit_flow_control_and_starvation() {
    let mut link = SpaceWireLink::new(200.0, 1024);
    assert_eq!(link.tx_credit_bytes, 56);
    assert!(!link.is_credit_starved());

    // Transmit 40 bytes -> credit should drop to 16
    let sent1 = link.send_bytes(40);
    assert_eq!(sent1, 40);
    assert_eq!(link.tx_credit_bytes, 16);
    assert_eq!(link.rx_buffer_fill_bytes, 40);

    // Transmit 16 bytes -> credit drops to 0
    let sent2 = link.send_bytes(16);
    assert_eq!(sent2, 16);
    assert_eq!(link.tx_credit_bytes, 0);
    assert!(link.is_credit_starved());

    // Attempting further send while starved should return 0 and increment stalls
    let sent3 = link.send_bytes(20);
    assert_eq!(sent3, 0);
    assert_eq!(link.credit_starvation_events, 1);

    // Drain receiver by 32 bytes -> generates 4 FCTs = 32 bytes credit
    let fct = link.drain_rx(32);
    assert_eq!(fct, 4);
    link.receive_fct(fct);
    assert_eq!(link.tx_credit_bytes, 32);
    assert!(!link.is_credit_starved());

    // Now transmission succeeds again
    let sent4 = link.send_bytes(24);
    assert_eq!(sent4, 24);
    assert_eq!(link.tx_credit_bytes, 8);
}

#[test]
fn test_spacefibre_multi_lane_and_qos_scheduling() {
    let mut spfi = SpaceFibreMultiLaneLink::default();
    // 2 lanes * 3.125 Gbps * 0.80 encoding = 5000 Mbps = 5.0 Gbps
    let agg_rate = spfi.aggregate_payload_rate_mbps();
    assert!((agg_rate - 5000.0).abs() < 1e-3);

    // Test Strict Priority: VC0 (priority 7) vs VC3 (priority 1)
    spfi.scheduling = SpFiQoSScheduling::StrictPriority;
    let incoming = vec![(0, 4000), (3, 4000)];
    let results = spfi.step(10.0, &incoming);

    // VC0 must be serviced first
    let vc0_tx = results.iter().find(|&&(id, _)| id == 0).map(|&(_, b)| b).unwrap_or(0);
    assert!(vc0_tx > 0, "High priority VC0 must transmit");

    // Test burst flood simulation
    let trajectory = spfi.simulate_burst_flood(100.0, 1, 4.0, 50);
    assert_eq!(trajectory.len(), 50);
    assert!(trajectory[49].2 > 0.0, "Link utilization must be positive under burst flood");
}

#[test]
fn test_afdx_bag_policing_and_redundant_deduplication() {
    let mut vl = AfdxVirtualLink::new(101, 8.0, 1024);
    assert_eq!(vl.bag_ms, 8.0);

    // Bandwidth = (1024 * 8) / (0.008 * 1e6) = 1.024 Mbps
    let bw = vl.allocated_bandwidth_mbps();
    assert!((bw - 1.024).abs() < 1e-4);

    // First frame transmission at t = 0.0 ms
    let seq1 = vl.transmit_frame(0.0, 1024).expect("Initial frame must pass");
    assert_eq!(seq1, 1);

    // Sending another frame at t = 2.0 ms (below BAG - Jitter = 8.0 - 0.5 = 7.5 ms) must fail
    let err = vl.transmit_frame(2.0, 1024);
    assert!(err.is_err(), "Transmission within BAG interval must be rejected by policing");
    assert_eq!(vl.dropped_by_policing, 1);

    // Sending at t = 8.0 ms must succeed
    let seq2 = vl.transmit_frame(8.0, 1024).expect("Compliant frame must pass");
    assert_eq!(seq2, 2);

    // Test redundant receiver de-duplication: Network A vs Network B
    let acc1 = vl.receive_frame(1, true); // From Net A
    assert!(acc1, "First frame from Net A must be accepted");

    let acc1_dup = vl.receive_frame(1, false); // From Net B (duplicate of frame 1)
    assert!(!acc1_dup, "Duplicate frame from Net B must be rejected");
    assert_eq!(vl.duplicates_rejected, 1);

    let acc2 = vl.receive_frame(2, false); // From Net B (first arrival of frame 2)
    assert!(acc2, "Frame 2 from Net B must be accepted");
}

#[test]
fn test_afdx_switch_transit_and_worst_case_latency() {
    let switch = AfdxSwitch::default();
    assert_eq!(switch.technical_latency_us, 16.0);
    assert_eq!(switch.port_speed_mbps, 100.0);

    let tx_delay = switch.frame_transmission_delay_us(1024);
    // (1024 * 8) / 100 = 81.92 us
    assert!((tx_delay - 81.92).abs() < 0.1);

    let single_hop = switch.worst_case_switch_transit_us(1024);
    assert!(single_hop > 16.0);

    let e2e_bound = switch.end_to_end_latency_bound_us(3, 1024);
    assert!((e2e_bound - (single_hop * 3.0)).abs() < 1e-4);
    assert!(e2e_bound < 2000.0, "AFDX latency bound must be well within millisecond real-time budget");
}

#[test]
fn test_noc_mesh_thermal_deflection_hotspot_avoidance() {
    let mesh = NoCMeshSimulator::new(4, 4);
    assert_eq!(mesh.tiles.len(), 16);
    assert_eq!(mesh.peak_junction_temperature_c(), 45.0);

    // Evaluate thermal deflection vs XY routing under heavy cross-traffic
    let (peak_xy, peak_deflect, mitigation) = mesh.evaluate_thermal_mitigation_delta();

    assert!(peak_xy > 45.0, "Heavy traffic must heat XY center routers");
    assert!(
        peak_deflect <= peak_xy,
        "Thermal deflection must keep peak junction temp lower than or equal to XY (XY: {}, Deflect: {})",
        peak_xy,
        peak_deflect
    );
    assert!(mitigation >= 0.0);
}

#[test]
fn test_space_avionics_bus_co_simulator_cold_boot_and_telemetry() {
    let start = Instant::now();
    let mut co_sim = SpaceAvionicsBusCoSimulator::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "SpaceAvionicsBusCoSimulator::new_fast() must complete in sub-5ms (took {:?})",
        elapsed
    );

    let report = co_sim.report();
    assert!(report.spacefibre_aggregate_gbps > 0.0);
    assert!(report.afdx_allocated_bw_mbps > 0.0);
    assert!(report.afdx_end_to_end_latency_bound_us > 0.0);

    let updated = co_sim.recompute();
    assert!(updated.spacewire_rate_mbps > 0.0);
    assert!(updated.noc_peak_temp_c >= 45.0);
}
