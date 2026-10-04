#![deny(unsafe_code)]

//! Comprehensive test suite for Phonon Studio Scalable Binary History Format.
//!
//! Validates:
//! 1. Opcode serialization and deserialization across all opcodes:
//!    AddComponent, DeleteComponent, MoveComponent, RotateComponent,
//!    ModifyComponentValue, AddWire, DeleteWire, ClearAll, Batch.
//! 2. Adler-32 checksum calculation, known test vectors, and payload integrity.
//! 3. Corrupted checksum detection (BinaryHistoryError::ChecksumMismatch).
//! 4. Unsupported version and invalid magic rejection.
//! 5. On-disk history serialization with tail truncation and clean index adjustment.
//! 6. Full reversibility: executing commands, undoing them, re-executing them,
//!    ensuring canvas integrity and determinism.

use egui::Pos2;
use phonon_gui::schematic::{
    adler32, deserialize_history, read_command, serialize_history, write_command, ActionOpcode,
    BinaryHistoryError, CanvasCommand, ComponentKind, HistoryStack, SchematicComponent,
    SchematicWire, HISTORY_MAGIC, HISTORY_VERSION,
};

#[test]
fn test_action_opcode_mapping_and_errors() {
    assert_eq!(ActionOpcode::from_u8(0x01).unwrap(), ActionOpcode::AddComponent);
    assert_eq!(ActionOpcode::from_u8(0x02).unwrap(), ActionOpcode::DeleteComponent);
    assert_eq!(ActionOpcode::from_u8(0x03).unwrap(), ActionOpcode::MoveComponent);
    assert_eq!(ActionOpcode::from_u8(0x04).unwrap(), ActionOpcode::RotateComponent);
    assert_eq!(ActionOpcode::from_u8(0x05).unwrap(), ActionOpcode::ModifyComponentValue);
    assert_eq!(ActionOpcode::from_u8(0x06).unwrap(), ActionOpcode::AddWire);
    assert_eq!(ActionOpcode::from_u8(0x07).unwrap(), ActionOpcode::DeleteWire);
    assert_eq!(ActionOpcode::from_u8(0x08).unwrap(), ActionOpcode::ClearAll);
    assert_eq!(ActionOpcode::from_u8(0x09).unwrap(), ActionOpcode::Batch);

    assert_eq!(
        ActionOpcode::from_u8(0x00),
        Err(BinaryHistoryError::UnknownOpcode(0x00))
    );
    assert_eq!(
        ActionOpcode::from_u8(0x0A),
        Err(BinaryHistoryError::UnknownOpcode(0x0A))
    );
    assert_eq!(
        ActionOpcode::from_u8(0xFF),
        Err(BinaryHistoryError::UnknownOpcode(0xFF))
    );
}

#[test]
fn test_opcode_roundtrip_add_component() {
    let mut comp = SchematicComponent::new(101, ComponentKind::Resistor, Pos2::new(120.0, 240.0), 101);
    comp.rotation = 2;
    comp.value_str = "4.7k".to_string();
    comp.properties.push(("tolerance".to_string(), "1%".to_string()));

    let cmd = CanvasCommand::AddComponent(comp);
    let mut bytes = Vec::new();
    write_command(&cmd, &mut bytes);

    assert_eq!(bytes[0], ActionOpcode::AddComponent as u8);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read AddComponent");
    assert_eq!(cursor, bytes.len());
    assert_eq!(cmd, read_cmd);
}

#[test]
fn test_opcode_roundtrip_delete_component() {
    let mut comp = SchematicComponent::new(202, ComponentKind::Capacitor, Pos2::new(300.0, 150.0), 202);
    comp.rotation = 3;
    comp.value_str = "100nF".to_string();
    comp.properties.push(("voltage_rating".to_string(), "50V".to_string()));

    let cmd = CanvasCommand::DeleteComponent(comp);
    let mut bytes = Vec::new();
    write_command(&cmd, &mut bytes);

    assert_eq!(bytes[0], ActionOpcode::DeleteComponent as u8);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read DeleteComponent");
    assert_eq!(cursor, bytes.len());
    assert_eq!(cmd, read_cmd);
}

#[test]
fn test_opcode_roundtrip_move_component() {
    let cmd = CanvasCommand::MoveComponent {
        id: 42,
        from: Pos2::new(100.5, 200.25),
        to: Pos2::new(350.0, 420.75),
    };

    let mut bytes = Vec::new();
    write_command(&cmd, &mut bytes);

    assert_eq!(bytes[0], ActionOpcode::MoveComponent as u8);
    // 1 opcode + 8 id + 4*4 coords = 25 bytes
    assert_eq!(bytes.len(), 25);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read MoveComponent");
    assert_eq!(cursor, bytes.len());
    assert_eq!(cmd, read_cmd);
}

#[test]
fn test_opcode_roundtrip_rotate_component() {
    let cmd = CanvasCommand::RotateComponent {
        id: 77,
        from_rot: 1,
        to_rot: 3,
    };

    let mut bytes = Vec::new();
    write_command(&cmd, &mut bytes);

    assert_eq!(bytes[0], ActionOpcode::RotateComponent as u8);
    // 1 opcode + 8 id + 1 from_rot + 1 to_rot = 11 bytes
    assert_eq!(bytes.len(), 11);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read RotateComponent");
    assert_eq!(cursor, bytes.len());
    assert_eq!(cmd, read_cmd);
}

#[test]
fn test_opcode_roundtrip_modify_component_value() {
    let cmd = CanvasCommand::ModifyComponentValue {
        id: 88,
        old_val: "10k_POT".to_string(),
        new_val: "22k_POT_LIN".to_string(),
    };

    let mut bytes = Vec::new();
    write_command(&cmd, &mut bytes);

    assert_eq!(bytes[0], ActionOpcode::ModifyComponentValue as u8);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read ModifyComponentValue");
    assert_eq!(cursor, bytes.len());
    assert_eq!(cmd, read_cmd);

    // Empty string values
    let cmd_empty = CanvasCommand::ModifyComponentValue {
        id: 89,
        old_val: String::new(),
        new_val: String::new(),
    };
    let mut bytes_empty = Vec::new();
    write_command(&cmd_empty, &mut bytes_empty);
    let mut cursor_empty = 0;
    let read_empty = read_command(&bytes_empty, &mut cursor_empty).expect("Failed to read empty value");
    assert_eq!(cursor_empty, bytes_empty.len());
    assert_eq!(cmd_empty, read_empty);
}

#[test]
fn test_opcode_roundtrip_add_wire() {
    let wire = SchematicWire::manhattan_route_with_net(
        501,
        Pos2::new(50.0, 60.0),
        Pos2::new(200.0, 180.0),
        Some("CLK_IN".to_string()),
    );

    let cmd = CanvasCommand::AddWire(wire);
    let mut bytes = Vec::new();
    write_command(&cmd, &mut bytes);

    assert_eq!(bytes[0], ActionOpcode::AddWire as u8);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read AddWire");
    assert_eq!(cursor, bytes.len());
    assert_eq!(cmd, read_cmd);
}

#[test]
fn test_opcode_roundtrip_delete_wire() {
    let wire = SchematicWire::manhattan_route_with_net(
        502,
        Pos2::new(10.0, 20.0),
        Pos2::new(100.0, 20.0),
        None,
    );

    let cmd = CanvasCommand::DeleteWire(wire);
    let mut bytes = Vec::new();
    write_command(&cmd, &mut bytes);

    assert_eq!(bytes[0], ActionOpcode::DeleteWire as u8);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read DeleteWire");
    assert_eq!(cursor, bytes.len());
    assert_eq!(cmd, read_cmd);
}

#[test]
fn test_opcode_roundtrip_clear_all() {
    let c1 = SchematicComponent::new(1, ComponentKind::Inductor, Pos2::new(50.0, 50.0), 1);
    let c2 = SchematicComponent::new(2, ComponentKind::Ground, Pos2::new(50.0, 100.0), 2);
    let w1 = SchematicWire::manhattan_route_with_net(
        1,
        Pos2::new(50.0, 50.0),
        Pos2::new(50.0, 100.0),
        Some("GND".to_string()),
    );

    let cmd = CanvasCommand::ClearAll {
        components: vec![c1, c2],
        wires: vec![w1],
    };

    let mut bytes = Vec::new();
    write_command(&cmd, &mut bytes);

    assert_eq!(bytes[0], ActionOpcode::ClearAll as u8);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read ClearAll");
    assert_eq!(cursor, bytes.len());
    assert_eq!(cmd, read_cmd);
}

#[test]
fn test_opcode_roundtrip_batch_and_nested() {
    let comp = SchematicComponent::new(301, ComponentKind::VoltageSource, Pos2::new(80.0, 90.0), 301);
    let wire = SchematicWire::manhattan_route_with_net(
        601,
        Pos2::new(80.0, 90.0),
        Pos2::new(150.0, 90.0),
        Some("VCC".to_string()),
    );

    let sub1 = CanvasCommand::AddComponent(comp);
    let sub2 = CanvasCommand::AddWire(wire);
    let sub3 = CanvasCommand::MoveComponent {
        id: 301,
        from: Pos2::new(80.0, 90.0),
        to: Pos2::new(100.0, 110.0),
    };

    let nested_batch = CanvasCommand::Batch(vec![sub1.clone(), sub2.clone()]);
    let root_batch = CanvasCommand::Batch(vec![nested_batch, sub3]);

    let mut bytes = Vec::new();
    write_command(&root_batch, &mut bytes);

    assert_eq!(bytes[0], ActionOpcode::Batch as u8);

    let mut cursor = 0;
    let read_cmd = read_command(&bytes, &mut cursor).expect("Failed to read Batch");
    assert_eq!(cursor, bytes.len());
    assert_eq!(root_batch, read_cmd);
}

#[test]
fn test_adler32_known_vectors_and_properties() {
    // Empty buffer: s1 = 1, s2 = 0 -> 1
    assert_eq!(adler32(b""), 1);

    // Standard RFC 1950 reference vector: "Wikipedia" -> 0x11E60398
    assert_eq!(adler32(b"Wikipedia"), 0x11E60398);

    // "123456789" standard checksum
    assert_eq!(adler32(b"123456789"), 0x091E01DE);

    // Non-zero data produces deterministic, different checksums
    let c1 = adler32(b"Phonon Studio CAD Engine");
    let c2 = adler32(b"Phonon Studio CAD Enginf");
    assert_ne!(c1, c2);
}

#[test]
fn test_serialize_history_checksum_and_magic() {
    let mut history = HistoryStack::new();
    history.record(CanvasCommand::MoveComponent {
        id: 1,
        from: Pos2::new(0.0, 0.0),
        to: Pos2::new(10.0, 20.0),
    });

    let bytes = serialize_history(&history, 50);
    assert!(bytes.len() >= 14);

    // Verify magic bytes
    assert_eq!(&bytes[0..4], HISTORY_MAGIC);

    // Verify version
    let version = u16::from_le_bytes(bytes[4..6].try_into().unwrap());
    assert_eq!(version, HISTORY_VERSION);

    // Verify Adler-32 footer matches payload
    let payload_len = bytes.len() - 4;
    let computed_chk = adler32(&bytes[..payload_len]);
    let stored_chk = u32::from_le_bytes(bytes[payload_len..].try_into().unwrap());
    assert_eq!(computed_chk, stored_chk);
}

#[test]
fn test_corrupted_checksum_detection() {
    let mut history = HistoryStack::new();
    history.record(CanvasCommand::MoveComponent {
        id: 5,
        from: Pos2::new(10.0, 20.0),
        to: Pos2::new(30.0, 40.0),
    });

    let mut bytes = serialize_history(&history, 50);

    // Corrupt a byte in the payload
    let target_idx = 8;
    bytes[target_idx] ^= 0x55;

    let res = deserialize_history(&bytes, 500);
    match res {
        Err(BinaryHistoryError::ChecksumMismatch { expected, computed }) => {
            assert_ne!(expected, computed);
        }
        other => panic!("Expected ChecksumMismatch error, got {:?}", other),
    }

    // Corrupt the checksum bytes directly
    let mut bytes2 = serialize_history(&history, 50);
    let chk_start = bytes2.len() - 4;
    bytes2[chk_start] ^= 0xFF;

    let res2 = deserialize_history(&bytes2, 500);
    match res2 {
        Err(BinaryHistoryError::ChecksumMismatch { expected, computed }) => {
            assert_ne!(expected, computed);
        }
        other => panic!("Expected ChecksumMismatch error, got {:?}", other),
    }
}

#[test]
fn test_unsupported_version_and_invalid_magic_rejection() {
    let mut history = HistoryStack::new();
    history.record(CanvasCommand::MoveComponent {
        id: 1,
        from: Pos2::new(0.0, 0.0),
        to: Pos2::new(1.0, 1.0),
    });
    let bytes = serialize_history(&history, 50);

    // 1. Invalid Magic
    let mut bad_magic_bytes = bytes.clone();
    bad_magic_bytes[0..4].copy_from_slice(b"BADM");
    // Update checksum so magic check fails first
    let payload_len = bad_magic_bytes.len() - 4;
    let chk = adler32(&bad_magic_bytes[..payload_len]);
    bad_magic_bytes[payload_len..].copy_from_slice(&chk.to_le_bytes());

    let res_magic = deserialize_history(&bad_magic_bytes, 500);
    assert_eq!(res_magic, Err(BinaryHistoryError::InvalidMagic));

    // 2. Unsupported Version
    let mut bad_ver_bytes = bytes.clone();
    bad_ver_bytes[4..6].copy_from_slice(&99u16.to_le_bytes());
    let payload_len = bad_ver_bytes.len() - 4;
    let chk = adler32(&bad_ver_bytes[..payload_len]);
    bad_ver_bytes[payload_len..].copy_from_slice(&chk.to_le_bytes());

    let res_ver = deserialize_history(&bad_ver_bytes, 500);
    assert_eq!(res_ver, Err(BinaryHistoryError::UnsupportedVersion(99)));

    // 3. Truncated Data (< 14 bytes)
    assert_eq!(deserialize_history(&[], 500), Err(BinaryHistoryError::TruncatedData));
    assert_eq!(
        deserialize_history(&bytes[..10], 500),
        Err(BinaryHistoryError::TruncatedData)
    );
}

#[test]
fn test_on_disk_history_serialization_tail_truncation() {
    let mut history = HistoryStack::with_max_depth(500);

    // Push 100 commands onto the undo stack
    for i in 1..=100 {
        history.record(CanvasCommand::MoveComponent {
            id: i,
            from: Pos2::new(i as f32, 0.0),
            to: Pos2::new(i as f32, 10.0),
        });
    }
    assert_eq!(history.undo_count(), 100);

    // Set clean index at action 80
    history.clean_index = 80;

    // Serialize with on_disk_limit = 25
    let bytes = serialize_history(&history, 25);

    // Deserialization with in_memory_limit = 300
    let deserialized = deserialize_history(&bytes, 300).expect("Deserialization must succeed");

    // Verify exactly the last 25 commands were retained (ids 76..=100)
    assert_eq!(deserialized.undo_count(), 25);
    assert_eq!(deserialized.max_depth, 300);

    for (idx, cmd) in deserialized.undo_stack.iter().enumerate() {
        let expected_id = 76 + idx;
        match cmd {
            CanvasCommand::MoveComponent { id, .. } => {
                assert_eq!(*id, expected_id);
            }
            other => panic!("Expected MoveComponent, found {:?}", other),
        }
    }

    // Verify adjusted clean index:
    // skip_count = 100 - 25 = 75
    // clean_index 80 - 75 = 5
    assert_eq!(deserialized.clean_index, 5);

    // If clean index was before the truncation window (e.g. 50 < 75)
    history.clean_index = 50;
    let bytes_truncated_clean = serialize_history(&history, 25);
    let des_trunc = deserialize_history(&bytes_truncated_clean, 300).unwrap();
    assert_eq!(des_trunc.clean_index, usize::MAX);
    assert!(des_trunc.is_dirty());
}

#[test]
fn test_on_disk_history_limit_zero_and_oversized() {
    let mut history = HistoryStack::with_max_depth(100);
    for i in 1..=10 {
        history.record(CanvasCommand::MoveComponent {
            id: i,
            from: Pos2::new(0.0, 0.0),
            to: Pos2::new(1.0, 1.0),
        });
    }
    history.clean_index = 4;

    // Limit 0: drops all commands
    let bytes_zero = serialize_history(&history, 0);
    let des_zero = deserialize_history(&bytes_zero, 100).unwrap();
    assert_eq!(des_zero.undo_count(), 0);
    assert_eq!(des_zero.clean_index, usize::MAX);

    // Limit larger than stack: preserves all commands and clean index
    let bytes_large = serialize_history(&history, 50);
    let des_large = deserialize_history(&bytes_large, 100).unwrap();
    assert_eq!(des_large.undo_count(), 10);
    assert_eq!(des_large.clean_index, 4);
    assert_eq!(des_large.undo_stack, history.undo_stack);
}

#[test]
fn test_full_canvas_reversibility_and_determinism() {
    let mut components: Vec<SchematicComponent> = Vec::new();
    let mut wires: Vec<SchematicWire> = Vec::new();
    let mut history = HistoryStack::with_max_depth(100);

    // Define 8 distinct mutation commands
    let c1 = SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(100.0, 100.0), 1);
    let c2 = SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(200.0, 100.0), 2);
    let w1 = SchematicWire::manhattan_route_with_net(
        1,
        Pos2::new(100.0, 100.0),
        Pos2::new(200.0, 100.0),
        Some("NET_A".to_string()),
    );
    let w2 = SchematicWire::manhattan_route_with_net(
        2,
        Pos2::new(200.0, 100.0),
        Pos2::new(200.0, 200.0),
        Some("NET_B".to_string()),
    );

    let commands = vec![
        CanvasCommand::AddComponent(c1.clone()),
        CanvasCommand::AddComponent(c2.clone()),
        CanvasCommand::AddWire(w1.clone()),
        CanvasCommand::MoveComponent {
            id: 1,
            from: Pos2::new(100.0, 100.0),
            to: Pos2::new(120.0, 140.0),
        },
        CanvasCommand::RotateComponent {
            id: 1,
            from_rot: 0,
            to_rot: 1,
        },
        CanvasCommand::ModifyComponentValue {
            id: 1,
            old_val: "1k".to_string(),
            new_val: "4.7k".to_string(),
        },
        CanvasCommand::DeleteWire(w1.clone()),
        CanvasCommand::Batch(vec![
            CanvasCommand::MoveComponent {
                id: 2,
                from: Pos2::new(200.0, 100.0),
                to: Pos2::new(220.0, 120.0),
            },
            CanvasCommand::AddWire(w2.clone()),
        ]),
    ];

    // Execute and record all 8 commands
    for cmd in &commands {
        cmd.execute(&mut components, &mut wires);
        history.record(cmd.clone());
    }

    assert_eq!(components.len(), 2);
    assert_eq!(wires.len(), 1);
    assert_eq!(wires[0].id, 2);

    let comp1 = components.iter().find(|c| c.id == 1).unwrap();
    assert_eq!(comp1.pos, Pos2::new(120.0, 140.0));
    assert_eq!(comp1.rotation, 1);
    assert_eq!(comp1.value_str, "4.7k");

    let comp2 = components.iter().find(|c| c.id == 2).unwrap();
    assert_eq!(comp2.pos, Pos2::new(220.0, 120.0));

    // Serialize history to binary format and deserialize back
    let serialized_bytes = serialize_history(&history, 50);
    let mut restored_history = deserialize_history(&serialized_bytes, 100)
        .expect("History deserialization must succeed");

    assert_eq!(restored_history.undo_count(), 8);
    assert_eq!(restored_history.redo_count(), 0);

    // Snapshot target mutated state for later determinism comparison
    let saved_comps = components.clone();
    let saved_wires = wires.clone();

    // Undo all 8 commands in reverse order
    for _ in 0..8 {
        assert!(restored_history.undo(&mut components, &mut wires));
    }

    // Verify canvas is completely empty and clean
    assert!(components.is_empty(), "Canvas components must be completely empty after full undo");
    assert!(wires.is_empty(), "Canvas wires must be completely empty after full undo");
    assert!(!restored_history.can_undo());
    assert_eq!(restored_history.redo_count(), 8);

    // Redo all 8 commands forward
    for _ in 0..8 {
        assert!(restored_history.redo(&mut components, &mut wires));
    }

    // Verify exact determinism matching post-execution snapshot
    assert_eq!(components, saved_comps);
    assert_eq!(wires, saved_wires);
    assert_eq!(restored_history.undo_count(), 8);
    assert_eq!(restored_history.redo_count(), 0);
}

#[test]
fn test_clear_all_reversibility() {
    let mut components = vec![
        SchematicComponent::new(1, ComponentKind::Resistor, Pos2::new(10.0, 10.0), 1),
        SchematicComponent::new(2, ComponentKind::Capacitor, Pos2::new(20.0, 20.0), 2),
    ];
    let mut wires = vec![
        SchematicWire::manhattan_route_with_net(1, Pos2::new(10.0, 10.0), Pos2::new(20.0, 20.0), None),
    ];
    let mut stack = HistoryStack::new();

    let cmd = CanvasCommand::ClearAll {
        components: components.clone(),
        wires: wires.clone(),
    };
    cmd.execute(&mut components, &mut wires);
    stack.record(cmd);

    assert!(components.is_empty());
    assert!(wires.is_empty());

    // Undo ClearAll restores everything
    assert!(stack.undo(&mut components, &mut wires));
    assert_eq!(components.len(), 2);
    assert_eq!(wires.len(), 1);

    // Redo ClearAll empties canvas again
    assert!(stack.redo(&mut components, &mut wires));
    assert!(components.is_empty());
    assert!(wires.is_empty());
}
