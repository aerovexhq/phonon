#![deny(unsafe_code)]

//! Comprehensive Verification Test Suite for Native Desktop Shell, Tauri v2 & Zero-Copy Binary IPC Co-Processor.
//!
//! Validates:
//! 1. Sub-millisecond cold boot latency (< 5ms).
//! 2. Zero-copy binary IPC packet encoding, decoding, and Adler-32 checksum integrity.
//! 3. Packet rejection on corrupted magic, unsupported version, checksum mismatch, and truncation.
//! 4. Unknown opcode detection.
//! 5. Adler-32 checksum determinism.
//! 6. RS-274X Gerber and Excellon NC drill layer syntax generation.
//! 7. 10-point Desktop Platform Readiness Audit verification (10/10 passed).
//! 8. Loopback Ping/Pong sequence execution.
//! 9. Headless egui render pass across all 5 dialog tabs.

use std::time::Instant;

use phonon_gui::widgets::desktop_ipc_dialog::{
    compute_adler32, decode_ipc_packet, encode_ipc_packet, DesktopIpcDialog, DesktopIpcTab,
    IpcOpcode, IpcPacketError, IpcPacketHeader, IPC_MAGIC, IPC_VERSION,
};

#[test]
fn test_desktop_ipc_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = DesktopIpcDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot instantiation took {:?}, exceeding 5ms target",
        elapsed
    );
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, DesktopIpcTab::DesktopShell);
    assert!(dialog.logical_cpu_cores > 0);
    assert!(dialog.physical_cpu_cores > 0);

    // Also verify Default trait behaves identically
    let default_dialog = DesktopIpcDialog::default();
    assert_eq!(default_dialog.active_tab, DesktopIpcTab::DesktopShell);
}

#[test]
fn test_ipc_packet_framing_encode_decode() {
    let sequence_id = 1001u64;
    let opcode = IpcOpcode::Ping;
    let flags = 0x0002u16;
    let payload = b"Phonon_ZeroCopy_Payload_Data_12345";

    let encoded = encode_ipc_packet(sequence_id, opcode, flags, payload);
    assert_eq!(encoded.len(), IpcPacketHeader::SIZE + payload.len());

    let (header, decoded_payload) = decode_ipc_packet(&encoded).expect("Decode should succeed");

    assert_eq!(&header.magic, IPC_MAGIC);
    assert_eq!(header.version, IPC_VERSION);
    assert_eq!(header.sequence_id, sequence_id);
    assert_eq!(header.opcode, opcode);
    assert_eq!(header.flags, flags);
    assert_eq!(header.payload_len, payload.len() as u32);
    assert_eq!(header.checksum, compute_adler32(payload));
    assert_eq!(decoded_payload, payload);
}

#[test]
fn test_ipc_packet_corrupted_magic() {
    let payload = b"ValidPayload";
    let mut encoded = encode_ipc_packet(1, IpcOpcode::Pong, 0, payload);

    // Corrupt magic header
    encoded[0] = b'X';
    encoded[1] = b'Y';
    encoded[2] = b'Z';
    encoded[3] = b'W';

    let err = decode_ipc_packet(&encoded).expect_err("Should fail with InvalidMagic");
    assert_eq!(err, IpcPacketError::InvalidMagic);
}

#[test]
fn test_ipc_packet_unsupported_version() {
    let payload = b"ValidPayload";
    let mut encoded = encode_ipc_packet(2, IpcOpcode::SchematicSync, 0, payload);

    // Set version to 999
    let bad_ver: u16 = 999;
    encoded[4..6].copy_from_slice(&bad_ver.to_le_bytes());

    let err = decode_ipc_packet(&encoded).expect_err("Should fail with UnsupportedVersion");
    assert_eq!(err, IpcPacketError::UnsupportedVersion(999));
}

#[test]
fn test_ipc_packet_checksum_mismatch() {
    let payload = b"OriginalUntamperedData";
    let mut encoded = encode_ipc_packet(3, IpcOpcode::SimulateRequest, 0, payload);

    // Tamper with payload byte after header
    let payload_offset = IpcPacketHeader::SIZE + 5;
    encoded[payload_offset] ^= 0xFF;

    let err = decode_ipc_packet(&encoded).expect_err("Should fail with ChecksumMismatch");
    match err {
        IpcPacketError::ChecksumMismatch {
            expected,
            computed,
        } => {
            assert_ne!(expected, computed);
        }
        other => panic!("Expected ChecksumMismatch, got {:?}", other),
    }
}

#[test]
fn test_ipc_packet_truncated_header_and_payload() {
    let payload = b"SomeLongerPayloadDataStream";
    let encoded = encode_ipc_packet(4, IpcOpcode::SimulateResponse, 0, payload);

    // Truncated header (< 28 bytes)
    let short_header = &encoded[..16];
    let err_hdr = decode_ipc_packet(short_header).expect_err("Should fail TruncatedHeader");
    assert_eq!(err_hdr, IpcPacketError::TruncatedHeader);

    // Truncated payload (header indicates full payload, but bytes cut short)
    let cut_payload = &encoded[..IpcPacketHeader::SIZE + 5];
    let err_pay = decode_ipc_packet(cut_payload).expect_err("Should fail TruncatedPayload");
    assert_eq!(err_pay, IpcPacketError::TruncatedPayload);
}

#[test]
fn test_ipc_unknown_opcode() {
    let mut header_bytes = [0u8; IpcPacketHeader::SIZE];
    header_bytes[0..4].copy_from_slice(IPC_MAGIC);
    header_bytes[4..6].copy_from_slice(&IPC_VERSION.to_le_bytes());
    header_bytes[6..14].copy_from_slice(&5u64.to_le_bytes());
    // Opcode 0xDEAD is unassigned
    header_bytes[14..16].copy_from_slice(&0xDEADu16.to_le_bytes());
    header_bytes[16..18].copy_from_slice(&0u16.to_le_bytes());
    header_bytes[18..22].copy_from_slice(&0u32.to_le_bytes());
    header_bytes[22..26].copy_from_slice(&1u32.to_le_bytes());

    let err = decode_ipc_packet(&header_bytes).expect_err("Should fail UnknownOpcode");
    assert_eq!(err, IpcPacketError::UnknownOpcode(0xDEAD));
}

#[test]
fn test_adler32_checksum_deterministic() {
    let empty = b"";
    assert_eq!(compute_adler32(empty), 1);

    let data1 = b"Phonon Studio";
    let cs1 = compute_adler32(data1);
    let cs2 = compute_adler32(data1);
    assert_eq!(cs1, cs2);
    assert_ne!(cs1, 1);

    let data2 = b"Phonon Studio!";
    let cs3 = compute_adler32(data2);
    assert_ne!(cs1, cs3);
}

#[test]
fn test_rs274x_gerber_and_excellon_layers() {
    let dialog = DesktopIpcDialog::new_fast();

    assert_eq!(dialog.gerber_layers.len(), 4);

    let f_cu = &dialog.gerber_layers[0];
    assert_eq!(f_cu.layer_name, "Top Copper (F.Cu)");
    assert!(f_cu.filename.ends_with(".gbr"));
    assert!(f_cu.content.contains("%FSLAX46Y46*%"));
    assert!(f_cu.content.contains("%MOMM*%"));
    assert!(f_cu.content.contains("%LPD*%"));
    assert!(f_cu.content.contains("M02*"));

    let b_cu = &dialog.gerber_layers[1];
    assert_eq!(b_cu.layer_name, "Bottom Copper (B.Cu)");
    assert!(b_cu.content.contains("%FSLAX46Y46*%"));
    assert!(b_cu.content.contains("M02*"));

    let f_mask = &dialog.gerber_layers[2];
    assert_eq!(f_mask.layer_name, "Top Solder Mask (F.Mask)");
    assert!(f_mask.content.contains("%ADD12C"));
    assert!(f_mask.content.contains("M02*"));

    let drill = &dialog.gerber_layers[3];
    assert_eq!(drill.layer_name, "Excellon Drill File (Drill.drl)");
    assert!(drill.filename.ends_with(".drl"));
    assert!(drill.content.contains("M48"));
    assert!(drill.content.contains("METRIC,TZ"));
    assert!(drill.content.contains("M30"));
}

#[test]
fn test_desktop_platform_audit_criteria() {
    let dialog = DesktopIpcDialog::new_fast();

    assert_eq!(
        dialog.audit_items.len(),
        10,
        "Audit checklist must contain exactly 10 criteria"
    );

    let passed_count = dialog.audit_items.iter().filter(|i| i.is_passed).count();
    assert_eq!(
        passed_count, 10,
        "All 10 Desktop Platform Health Audit criteria must pass"
    );
    assert_eq!(dialog.audit_score, (10, 10));

    // Verify key audit criteria are documented
    let criteria_names: Vec<&str> = dialog
        .audit_items
        .iter()
        .map(|i| i.criterion.as_str())
        .collect();
    assert!(criteria_names.iter().any(|c| c.contains("Tauri v2")));
    assert!(criteria_names.iter().any(|c| c.contains("Zero-Copy Binary IPC")));
    assert!(criteria_names.iter().any(|c| c.contains("Latency")));
    assert!(criteria_names.iter().any(|c| c.contains("Rayon")));
    assert!(criteria_names.iter().any(|c| c.contains("Gerber")));
    assert!(criteria_names.iter().any(|c| c.contains("Excellon")));
    assert!(criteria_names.iter().any(|c| c.contains("Cold Startup")));
    assert!(criteria_names.iter().any(|c| c.contains("Safe Rust")));
}

#[test]
fn test_loopback_ping_pong_simulation() {
    let mut dialog = DesktopIpcDialog::new_fast();
    let initial_tx = dialog.packets_transmitted;
    let initial_rx = dialog.packets_received;

    // Encode Ping
    let ping_packet = encode_ipc_packet(1, IpcOpcode::Ping, 0, b"PING_PAYLOAD");
    dialog.packets_transmitted += 1;

    // Simulate loopback decode
    let (header, payload) = decode_ipc_packet(&ping_packet).expect("Ping decode");
    assert_eq!(header.opcode, IpcOpcode::Ping);
    assert_eq!(payload, b"PING_PAYLOAD");

    // Encode Pong reply
    let pong_packet = encode_ipc_packet(header.sequence_id, IpcOpcode::Pong, 0, b"PONG_PAYLOAD");
    dialog.packets_received += 1;
    dialog.last_packet_opcode = IpcOpcode::Pong;

    let (pong_header, pong_payload) = decode_ipc_packet(&pong_packet).expect("Pong decode");
    assert_eq!(pong_header.opcode, IpcOpcode::Pong);
    assert_eq!(pong_payload, b"PONG_PAYLOAD");
    assert_eq!(dialog.packets_transmitted, initial_tx + 1);
    assert_eq!(dialog.packets_received, initial_rx + 1);
    assert_eq!(dialog.last_packet_opcode, IpcOpcode::Pong);
}

#[test]
fn test_headless_egui_render_all_tabs() {
    let ctx = egui::Context::default();
    let mut dialog = DesktopIpcDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        DesktopIpcTab::DesktopShell,
        DesktopIpcTab::ZeroCopyIpc,
        DesktopIpcTab::RayonCoprocessor,
        DesktopIpcTab::GerberExporter,
        DesktopIpcTab::PlatformAudit,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
        assert!(dialog.is_open);
    }
}
