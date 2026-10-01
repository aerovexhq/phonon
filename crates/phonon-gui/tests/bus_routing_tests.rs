#![deny(unsafe_code)]

//! Verification test suite for high-density vectorized bus routing, bit tap-offs, and throughput benchmarks.

use egui::Pos2;
use phonon_gui::schematic::{BusSignal, SchematicBus, WireSegment};
use std::time::Instant;

#[test]
fn test_bus_creation_with_bit_ranges() {
    let data_bus = BusSignal::new("DATA", 31, 0);
    assert_eq!(data_bus.base_name, "DATA");
    assert_eq!(data_bus.width, 32);
    assert_eq!(data_bus.msb, 31);
    assert_eq!(data_bus.lsb, 0);
    assert_eq!(data_bus.format_label(), "DATA[31:0]");

    let addr_bus = BusSignal::new("ADDR", 15, 0);
    assert_eq!(addr_bus.width, 16);
    assert_eq!(addr_bus.format_label(), "ADDR[15:0]");

    // Reversed bit order (LSB to MSB)
    let rev_bus = BusSignal::new("REV", 0, 7);
    assert_eq!(rev_bus.width, 8);
    assert_eq!(rev_bus.format_label(), "REV[0:7]");
}

#[test]
fn test_bus_tap_off_extraction_and_breakout() {
    let signal = BusSignal::new("DATA", 31, 0);
    let segments = vec![
        WireSegment::new(Pos2::new(10.0, 50.0), Pos2::new(200.0, 50.0)),
        WireSegment::new(Pos2::new(200.0, 50.0), Pos2::new(200.0, 150.0)),
    ];
    let mut bus = SchematicBus::new(1, signal, segments);

    assert_eq!(bus.id, 1);
    assert_eq!(bus.stroke_width, 3.5);
    assert_eq!(bus.tap_offs.len(), 0);

    // Extract tap-off for bit 7
    let tap7 = bus.add_tap_off(7, Pos2::new(50.0, 50.0), 101);
    assert!(tap7.is_ok());
    let tap7 = tap7.unwrap();
    assert_eq!(tap7.bit_index, 7);
    assert_eq!(tap7.breakout_net, "DATA[7]");
    assert_eq!(tap7.breakout_wire_id, 101);
    assert_eq!(tap7.tap_pos, Pos2::new(50.0, 50.0));

    // Extract tap-off for bit 0 and bit 31 (extremities)
    let tap0 = bus.add_tap_off(0, Pos2::new(20.0, 50.0), 102).unwrap();
    assert_eq!(tap0.breakout_net, "DATA[0]");

    let tap31 = bus.add_tap_off(31, Pos2::new(180.0, 50.0), 103).unwrap();
    assert_eq!(tap31.breakout_net, "DATA[31]");

    assert_eq!(bus.tap_offs.len(), 3);
}

#[test]
fn test_boundary_clamping_out_of_range_tap_off_fails_cleanly() {
    let signal = BusSignal::new("DATA", 15, 0);
    let segments = vec![WireSegment::new(Pos2::new(0.0, 0.0), Pos2::new(100.0, 0.0))];
    let mut bus = SchematicBus::new(1, signal, segments);

    // Valid indices within [0..=15]
    assert!(bus.add_tap_off(0, Pos2::ZERO, 1).is_ok());
    assert!(bus.add_tap_off(15, Pos2::ZERO, 2).is_ok());

    // Out-of-bounds index 16
    let res16 = bus.add_tap_off(16, Pos2::ZERO, 3);
    assert!(res16.is_err());
    let err_msg = res16.unwrap_err();
    assert!(err_msg.contains("Bit index 16 out of range for bus DATA[15:0]"));

    // Out-of-bounds index 100
    let res100 = bus.add_tap_off(100, Pos2::ZERO, 4);
    assert!(res100.is_err());
}

#[test]
fn test_bus_contains_point_hit_testing() {
    let signal = BusSignal::new("CTRL", 7, 0);
    let segments = vec![WireSegment::new(
        Pos2::new(10.0, 20.0),
        Pos2::new(100.0, 20.0),
    )];
    let bus = SchematicBus::new(1, signal, segments);

    // Hit testing on bus segment with 4px tolerance
    assert!(bus.contains_point(Pos2::new(50.0, 20.0), 4.0));
    assert!(bus.contains_point(Pos2::new(50.0, 22.0), 4.0));
    assert!(!bus.contains_point(Pos2::new(50.0, 30.0), 4.0));
    assert!(!bus.contains_point(Pos2::new(200.0, 20.0), 4.0));
}

#[test]
fn test_throughput_benchmark_exceeds_250k_bus_operations_per_second() {
    let iterations = 100_000;
    let signal = BusSignal::new("BUS_BENCH", 63, 0);
    let segments = vec![WireSegment::new(
        Pos2::new(0.0, 0.0),
        Pos2::new(1000.0, 0.0),
    )];
    let mut bus = SchematicBus::new(1, signal, segments);

    let start = Instant::now();

    for i in 0..iterations {
        let bit = (i % 64) as u16;
        let pos = Pos2::new((i % 1000) as f32, 0.0);
        let _ = bus.signal.net_at_index(bit);
        if i < 1000 {
            let _ = bus.add_tap_off(bit, pos, i as usize);
        }
    }

    let elapsed = start.elapsed();
    let elapsed_ms = elapsed.as_secs_f64() * 1000.0;
    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();

    println!(
        "Bus operations benchmark: {} ops in {:.3} ms ({:.0} ops/sec)",
        iterations, elapsed_ms, ops_per_sec
    );

    assert!(
        ops_per_sec > 250_000.0,
        "Throughput {:.0} ops/sec below 250,000 threshold",
        ops_per_sec
    );
}
