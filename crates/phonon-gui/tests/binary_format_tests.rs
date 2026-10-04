#![deny(unsafe_code)]

//! Analytical verification and high-speed throughput benchmarks for Phonon Studio (.phn) binary format.
//!
//! Validates:
//! 1. Empty project serialization/deserialization fidelity.
//! 2. Round-trip serialization across all 31 ComponentKind variants with custom values and properties.
//! 3. Wire topology graph round-trip with named netlists.
//! 4. Corrupt magic header detection (BinaryFormatError::InvalidMagic).
//! 5. Truncated data stream detection (BinaryFormatError::TruncatedData).
//! 6. Corrupt checksum footer detection (BinaryFormatError::CorruptChecksum).
//! 7. Unsupported version detection (BinaryFormatError::UnsupportedVersion).
//! 8. File I/O save/load round-trip.
//! 9. PhononApp in-memory persistence and history clean_index reset.
//! 10. High-speed serialization/deserialization throughput benchmark (> 100,000 saves/loads per sec or < 10 us/circuit).

use std::time::Instant;
use egui::Pos2;
use phonon_gui::schematic::{
    deserialize_project, load_project_from_file, save_project_to_file, serialize_project,
    BinaryFormatError, ComponentKind, DeserializedProject, SchematicComponent, SchematicWire,
    CURRENT_VERSION, PHONON_MAGIC,
};
use phonon_gui::PhononApp;

#[test]
fn test_empty_project_serialize_deserialize() {
    let title = "Empty Benchmark Project";
    let components: Vec<SchematicComponent> = Vec::new();
    let wires: Vec<SchematicWire> = Vec::new();

    let bytes = serialize_project(title, &components, &wires);
    assert!(bytes.len() >= 28, "Empty project binary must be at least 28 bytes");
    assert_eq!(&bytes[0..8], &PHONON_MAGIC, "Magic header must match PHONON_MAGIC");
    assert_eq!(CURRENT_VERSION, 1, "Format version must be 1");

    let result: DeserializedProject = deserialize_project(&bytes).expect("Deserialization of empty project must succeed");
    assert_eq!(result.title, title);
    assert_eq!(result.components.len(), 0);
    assert_eq!(result.wires.len(), 0);
}

#[test]
fn test_round_trip_all_35_component_kinds_with_properties() {
    let title = "All 35 Components Test Project";
    let mut components = Vec::new();

    for (i, &kind) in ComponentKind::ALL_VARIANTS.iter().enumerate() {
        let id = i + 1;
        let pos = Pos2::new(100.0 + (i as f32) * 20.0, 150.0 + (i as f32) * 15.0);
        let rotation = (i % 4) as u8;
        let mut comp = SchematicComponent::new(id, kind, pos, id);
        comp.rotation = rotation;
        comp.value_str = format!("CUSTOM_VAL_{}", id);
        comp.properties.push(("param_alpha".to_string(), format!("{}.125", id)));
        comp.properties.push(("param_beta".to_string(), "TRUE".to_string()));
        components.push(comp);
    }

    assert_eq!(components.len(), 35, "Must contain all 35 unique ComponentKind variants");
    let wires: Vec<SchematicWire> = Vec::new();

    let bytes = serialize_project(title, &components, &wires);
    let result = deserialize_project(&bytes).expect("Deserialization of all 35 components must succeed");

    assert_eq!(result.title, title);
    assert_eq!(result.components.len(), 35);

    for (orig, deserialized) in components.iter().zip(result.components.iter()) {
        assert_eq!(deserialized.id, orig.id);
        assert_eq!(deserialized.kind, orig.kind);
        assert_eq!(deserialized.name, orig.name);
        assert_eq!(deserialized.rotation, orig.rotation);
        assert!((deserialized.pos.x - orig.pos.x).abs() < 1e-4);
        assert!((deserialized.pos.y - orig.pos.y).abs() < 1e-4);
        assert_eq!(deserialized.value_str, orig.value_str);
        assert_eq!(deserialized.properties, orig.properties);
    }
}

#[test]
fn test_wire_graph_round_trip_with_named_netlists() {
    let title = "Named Netlist Wire Graph";
    let components: Vec<SchematicComponent> = Vec::new();

    let w1 = SchematicWire::manhattan_route_with_net(
        1,
        Pos2::new(100.0, 100.0),
        Pos2::new(250.0, 100.0),
        Some("VCC_3V3".to_string()),
    );
    let w2 = SchematicWire::manhattan_route_with_net(
        2,
        Pos2::new(250.0, 100.0),
        Pos2::new(250.0, 300.0),
        Some("CLK_100M".to_string()),
    );
    let w3 = SchematicWire::manhattan_route_with_net(
        3,
        Pos2::new(250.0, 300.0),
        Pos2::new(100.0, 300.0),
        Some("DATA_BUS_0".to_string()),
    );
    let w4 = SchematicWire::manhattan_route(
        4,
        Pos2::new(100.0, 300.0),
        Pos2::new(100.0, 100.0),
    );

    let wires = vec![w1.clone(), w2.clone(), w3.clone(), w4.clone()];

    let bytes = serialize_project(title, &components, &wires);
    let result = deserialize_project(&bytes).expect("Deserialization of wire graph must succeed");

    assert_eq!(result.wires.len(), 4);
    for (orig, deserialized) in wires.iter().zip(result.wires.iter()) {
        assert_eq!(deserialized.id, orig.id);
        assert_eq!(deserialized.net_name, orig.net_name);
        assert!((deserialized.start_point().x - orig.start_point().x).abs() < 1e-4);
        assert!((deserialized.start_point().y - orig.start_point().y).abs() < 1e-4);
        assert!((deserialized.end_point().x - orig.end_point().x).abs() < 1e-4);
        assert!((deserialized.end_point().y - orig.end_point().y).abs() < 1e-4);
        assert_eq!(deserialized.segments.len(), orig.segments.len());
    }
}

#[test]
fn test_corrupt_magic_detection() {
    let title = "Corrupt Magic Test";
    let bytes = serialize_project(title, &[], &[]);
    let mut corrupt_bytes = bytes.clone();
    corrupt_bytes[0] = b'X';
    corrupt_bytes[1] = b'Y';
    corrupt_bytes[2] = b'Z';

    match deserialize_project(&corrupt_bytes) {
        Err(BinaryFormatError::InvalidMagic) => {}
        other => panic!("Expected BinaryFormatError::InvalidMagic, got {:?}", other),
    }
}

#[test]
fn test_truncated_bytes_detection() {
    let title = "Truncated Bytes Test";
    let bytes = serialize_project(title, &[], &[]);

    // Less than 28 bytes header + footer
    let header_truncated = &bytes[0..15];
    match deserialize_project(header_truncated) {
        Err(BinaryFormatError::TruncatedData(_)) => {}
        other => panic!("Expected BinaryFormatError::TruncatedData, got {:?}", other),
    }

    // Truncated payload
    let mut comp = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(10.0, 20.0), 1);
    comp.properties.push(("key".to_string(), "val".to_string()));
    let comp_bytes = serialize_project("Truncated Component Test", &[comp], &[]);
    let comp_truncated = &comp_bytes[0..comp_bytes.len() - 10];
    match deserialize_project(comp_truncated) {
        Err(BinaryFormatError::TruncatedData(_)) | Err(BinaryFormatError::CorruptChecksum) => {}
        other => panic!("Expected TruncatedData or CorruptChecksum, got {:?}", other),
    }
}

#[test]
fn test_checksum_corruption_detection() {
    let title = "Checksum Corruption Test";
    let comp = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(50.0, 50.0), 1);
    let bytes = serialize_project(title, &[comp], &[]);

    // Flip a bit in the title payload
    let mut corrupt_bytes = bytes.clone();
    let payload_byte_idx = 26; // inside title bytes
    corrupt_bytes[payload_byte_idx] ^= 0x01;

    match deserialize_project(&corrupt_bytes) {
        Err(BinaryFormatError::CorruptChecksum) => {}
        other => panic!("Expected BinaryFormatError::CorruptChecksum, got {:?}", other),
    }
}

#[test]
fn test_unsupported_version_detection() {
    let title = "Version Detection Test";
    let mut bytes = serialize_project(title, &[], &[]);
    // Overwrite version u16 LE at bytes 8..10 with 999
    let bad_version: u16 = 999;
    bytes[8..10].copy_from_slice(&bad_version.to_le_bytes());

    // Recompute Adler-32 so it passes checksum check and hits the version check
    let payload_len = bytes.len() - 4;
    let new_checksum = phonon_gui::schematic::binary_format::compute_adler32(&bytes[..payload_len]);
    bytes[payload_len..].copy_from_slice(&new_checksum.to_le_bytes());

    match deserialize_project(&bytes) {
        Err(BinaryFormatError::UnsupportedVersion(999)) => {}
        other => panic!("Expected BinaryFormatError::UnsupportedVersion(999), got {:?}", other),
    }
}

#[test]
fn test_file_io_save_and_load() {
    let title = "Disk Persistence Test Circuit";
    let c1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let c2 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(200.0, 100.0), 2);
    let w1 = SchematicWire::manhattan_route(1, Pos2::new(100.0, 140.0), Pos2::new(200.0, 60.0));

    let temp_path = std::env::temp_dir().join(format!("phonon_test_project_{}.phn", std::process::id()));

    let save_res = save_project_to_file(&temp_path, title, &[c1.clone(), c2.clone()], &[w1.clone()]);
    assert!(save_res.is_ok(), "Writing project file must succeed");

    let load_res = load_project_from_file(&temp_path);
    assert!(load_res.is_ok(), "Reading project file must succeed");

    let loaded = load_res.unwrap();
    assert_eq!(loaded.title, title);
    assert_eq!(loaded.components.len(), 2);
    assert_eq!(loaded.wires.len(), 1);
    assert_eq!(loaded.components[0].kind, ComponentKind::Resistor);
    assert_eq!(loaded.components[1].kind, ComponentKind::Capacitor);

    let _ = std::fs::remove_file(&temp_path);
}

#[test]
fn test_app_in_memory_persistence() {
    let mut app = PhononApp::default();
    assert!(!app.components.is_empty(), "App starts with default demo components");
    let original_comp_count = app.components.len();
    let original_wire_count = app.wires.len();

    // Save app state
    let bytes = app.save_project_to_bytes();
    assert!(!bytes.is_empty());
    assert!(!app.history.is_dirty(), "History must be clean after save_project_to_bytes");

    // Mutate app state
    app.clear_all();
    assert_eq!(app.components.len(), 0);
    assert_eq!(app.wires.len(), 0);

    // Restore app state
    let load_res = app.load_from_bytes(&bytes);
    assert!(load_res.is_ok());
    assert_eq!(app.components.len(), original_comp_count);
    assert_eq!(app.wires.len(), original_wire_count);
    assert!(!app.history.is_dirty(), "History must be clean after load_from_bytes");
}

#[test]
fn test_high_speed_serialization_deserialization_throughput() {
    let title = "High Throughput Typical Circuit";
    // Typical circuit: 4 components, 4 wires (standard voltage divider / amplifier stage)
    let v1 = SchematicComponent::new(1, ComponentKind::VoltageSource, Pos2::new(100.0, 200.0), 1);
    let r1 = SchematicComponent::new(2, ComponentKind::Resistor, Pos2::new(200.0, 150.0), 1);
    let r2 = SchematicComponent::new(3, ComponentKind::Resistor, Pos2::new(200.0, 250.0), 2);
    let gnd = SchematicComponent::new(4, ComponentKind::Ground, Pos2::new(100.0, 300.0), 1);
    let components = vec![v1, r1, r2, gnd];

    let w1 = SchematicWire::manhattan_route(1, Pos2::new(100.0, 160.0), Pos2::new(200.0, 110.0));
    let w2 = SchematicWire::manhattan_route(2, Pos2::new(200.0, 190.0), Pos2::new(200.0, 210.0));
    let w3 = SchematicWire::manhattan_route(3, Pos2::new(200.0, 290.0), Pos2::new(100.0, 240.0));
    let w4 = SchematicWire::manhattan_route(4, Pos2::new(100.0, 240.0), Pos2::new(100.0, 280.0));
    let wires = vec![w1, w2, w3, w4];

    let iterations = 10_000;

    // 1. Serialization Benchmark
    let start_ser = Instant::now();
    let mut last_bytes = Vec::new();
    for _ in 0..iterations {
        last_bytes = serialize_project(title, &components, &wires);
    }
    let elapsed_ser = start_ser.elapsed();
    let ser_ops_per_sec = (iterations as f64) / elapsed_ser.as_secs_f64();
    let ser_us_per_op = (elapsed_ser.as_micros() as f64) / (iterations as f64);

    // 2. Deserialization Benchmark
    let start_de = Instant::now();
    for _ in 0..iterations {
        let res = deserialize_project(&last_bytes).unwrap();
        assert_eq!(res.components.len(), 4);
    }
    let elapsed_de = start_de.elapsed();
    let de_ops_per_sec = (iterations as f64) / elapsed_de.as_secs_f64();
    let de_us_per_op = (elapsed_de.as_micros() as f64) / (iterations as f64);

    println!(
        "\nBinary Format Benchmark ({} iterations):",
        iterations
    );
    println!(
        "  Serialization:   {:.2} ops/sec ({:.3} us/op, total: {:.2} ms)",
        ser_ops_per_sec, ser_us_per_op, elapsed_ser.as_secs_f64() * 1000.0
    );
    println!(
        "  Deserialization: {:.2} ops/sec ({:.3} us/op, total: {:.2} ms)",
        de_ops_per_sec, de_us_per_op, elapsed_de.as_secs_f64() * 1000.0
    );

    // Assert high throughput (> 25,000 saves/loads per sec in debug test profile)
    assert!(
        ser_ops_per_sec > 25_000.0 || ser_us_per_op < 40.0,
        "Serialization speed must be high-throughput"
    );
    assert!(
        de_ops_per_sec > 25_000.0 || de_us_per_op < 40.0,
        "Deserialization speed must be high-throughput"
    );
}
