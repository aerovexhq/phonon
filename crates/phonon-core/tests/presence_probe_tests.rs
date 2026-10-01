#![deny(unsafe_code)]

//! Integration verification suite for AerovexPresenceProbe and atomic Seqlock AerovexShmBackend.
//!
//! Validates sub-millisecond presence detection, stale heartbeat rejection, torn-read detection,
//! optimistic Seqlock lock-free reads, adaptive auto-promotion and fallback dynamics,
//! quaternion re-normalization, and >1,000,000 ticks/sec ingestion throughput.

use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use phonon_core::{
    create_mock_shm_buffer, safe_read_shm_slot, ActuatorInputs, AerovexPresenceProbe,
    AerovexShmBackend, AutoSelectingDynamicsBackend, DynamicsError, PhysicsDynamicsBackend,
    ProbeStatus, ShmSlotData,
};

#[test]
fn test_probe_not_running_when_file_missing() {
    let missing_path = PathBuf::from("/tmp/aerovex_missing_test_probe_847291.bin");
    let _ = std::fs::remove_file(&missing_path);

    let status = AerovexPresenceProbe::probe_path(&missing_path);
    assert_eq!(status, ProbeStatus::NotRunning);
    assert!(!AerovexPresenceProbe::is_available_at(&missing_path));
}

#[test]
fn test_probe_invalid_format_bad_magic() {
    let temp_path = PathBuf::from("/tmp/aerovex_bad_magic_test_probe_847292.bin");
    let mut bad_buf = vec![0u8; 64];
    bad_buf[0..4].copy_from_slice(b"BADM");
    std::fs::write(&temp_path, &bad_buf).expect("Failed to write bad magic file");

    let status = AerovexPresenceProbe::probe_path(&temp_path);
    let _ = std::fs::remove_file(&temp_path);

    match status {
        ProbeStatus::InvalidFormat(msg) => {
            assert!(
                msg.contains("Invalid magic header"),
                "Expected error message mentioning invalid magic, got: {}",
                msg
            );
        }
        other => panic!("Expected ProbeStatus::InvalidFormat, got {:?}", other),
    }
    assert!(!AerovexPresenceProbe::is_available_at(&temp_path));
}

#[test]
fn test_probe_stale_heartbeat() {
    let temp_path = PathBuf::from("/tmp/aerovex_stale_test_probe_847293.bin");
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    // Heartbeat timestamp 3000 ms ago (exceeds MAX_HEARTBEAT_AGE_MS = 1500 ms)
    let stale_ms = now_epoch_ms.saturating_sub(3000);
    let buf = create_mock_shm_buffer(2, 1, Some(stale_ms), &[(0, 2, ShmSlotData::default())]);
    std::fs::write(&temp_path, &buf).expect("Failed to write stale SHM file");

    let status = AerovexPresenceProbe::probe_path(&temp_path);
    let _ = std::fs::remove_file(&temp_path);

    match status {
        ProbeStatus::StaleHeartbeat { age_ms } => {
            assert!(
                age_ms >= 1500.0,
                "Heartbeat age {:.2} ms should exceed 1500.0 ms threshold",
                age_ms
            );
        }
        other => panic!("Expected ProbeStatus::StaleHeartbeat, got {:?}", other),
    }
    assert!(!AerovexPresenceProbe::is_available_at(&temp_path));
}

#[test]
fn test_probe_available_valid_shm() {
    let temp_path = PathBuf::from("/tmp/aerovex_valid_test_probe_847294.bin");
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let buf = create_mock_shm_buffer(2, 8, Some(now_epoch_ms), &[(0, 2, ShmSlotData::default())]);
    std::fs::write(&temp_path, &buf).expect("Failed to write valid SHM file");

    let status = AerovexPresenceProbe::probe_path(&temp_path);
    let is_avail = AerovexPresenceProbe::is_available_at(&temp_path);
    let _ = std::fs::remove_file(&temp_path);

    assert!(is_avail);
    match status {
        ProbeStatus::Available {
            active_worlds,
            version,
            latency_us,
            heartbeat_age_ms,
        } => {
            assert_eq!(active_worlds, 8);
            assert_eq!(version, 2);
            assert!(
                latency_us < 1000.0,
                "Probe latency {:.2} us exceeds 1000.0 us threshold",
                latency_us
            );
            assert!(heartbeat_age_ms < 1500.0);
        }
        other => panic!("Expected ProbeStatus::Available, got {:?}", other),
    }
}

#[test]
fn test_shm_backend_safe_read_seqlock() {
    let expected = ShmSlotData {
        sim_time_s: 12.375,
        total_ticks: 1234,
        position_m: [15.5, -24.2, -150.0],
        orientation_quat: [0.7071067811865476, 0.0, 0.7071067811865476, 0.0],
        velocity_m_per_s: [8.5, -2.1, 0.4],
        angular_velocity_rad_per_s: [0.02, -0.05, 0.01],
    };
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let buf = create_mock_shm_buffer(2, 1, Some(now_epoch_ms), &[(0, 4, expected.clone())]);

    let slot = safe_read_shm_slot(&buf, 0, 0).expect("safe_read_shm_slot failed");

    assert_eq!(slot.sim_time_s, expected.sim_time_s);
    assert_eq!(slot.total_ticks, expected.total_ticks);
    assert_eq!(slot.position_m, expected.position_m);
    assert_eq!(slot.velocity_m_per_s, expected.velocity_m_per_s);
    assert_eq!(
        slot.angular_velocity_rad_per_s,
        expected.angular_velocity_rad_per_s
    );
    for i in 0..4 {
        assert!(
            (slot.orientation_quat[i] - expected.orientation_quat[i]).abs() < 1e-12,
            "Quaternion component mismatch at index {}",
            i
        );
    }

    // Verify AerovexShmBackend with mock buffer
    let mut shm_backend = AerovexShmBackend::new();
    shm_backend.set_mock_buffer(Some(buf));
    let inputs = ActuatorInputs::default();
    let telem = shm_backend
        .step(0.01, &inputs)
        .expect("Shm step failed")
        .clone();
    assert_eq!(telem.position_m, expected.position_m);
    assert_eq!(telem.velocity_m_per_s, expected.velocity_m_per_s);
    assert_eq!(telem.sim_time_s, expected.sim_time_s);
    assert_eq!(telem.step_count, expected.total_ticks);
    assert!(shm_backend.is_healthy());
    assert_eq!(
        shm_backend.info().name,
        "Aerovex POSIX Shared Memory Connector"
    );
}

#[test]
fn test_shm_backend_detects_torn_read() {
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    // Sequence number 5 is odd, indicating writer collision / active write
    let buf = create_mock_shm_buffer(2, 1, Some(now_epoch_ms), &[(0, 5, ShmSlotData::default())]);

    let result = safe_read_shm_slot(&buf, 0, 0);
    assert!(result.is_err());
    match result.unwrap_err() {
        DynamicsError::StepFailed(msg) => {
            assert!(
                msg.contains("writer active"),
                "Expected writer active error, got: {}",
                msg
            );
        }
        other => panic!("Expected StepFailed error, got {:?}", other),
    }
}

#[test]
fn test_auto_selecting_backend_defaults_to_reference() {
    let missing_path = PathBuf::from("/tmp/aerovex_missing_autoselect_847295.bin");
    let _ = std::fs::remove_file(&missing_path);

    let mut backend = AutoSelectingDynamicsBackend::with_path(&missing_path, 0, 0);
    let inputs = ActuatorInputs::hover(3.6775);

    let telem = backend.step(0.01, &inputs).expect("Step failed").clone();

    assert_eq!(
        backend.active_backend_name(),
        "Reference Dynamics Engine (RK4 6-DOF)"
    );
    assert!(!backend.is_shm_active());
    assert!(backend.is_healthy());
    assert!(telem.sim_time_s >= 0.01);
}

#[test]
fn test_auto_selecting_backend_promotes_to_shm() {
    let shm_path = PathBuf::from("/tmp/aerovex_promote_test_847296.bin");
    let expected = ShmSlotData {
        sim_time_s: 5.0,
        total_ticks: 500,
        position_m: [120.0, -80.0, -250.0],
        orientation_quat: [1.0, 0.0, 0.0, 0.0],
        velocity_m_per_s: [12.0, 1.5, -0.5],
        angular_velocity_rad_per_s: [0.01, 0.02, 0.03],
    };
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let buf = create_mock_shm_buffer(2, 1, Some(now_epoch_ms), &[(0, 2, expected)]);
    std::fs::write(&shm_path, &buf).expect("Failed to write mock SHM file");

    let mut backend = AutoSelectingDynamicsBackend::with_path(&shm_path, 0, 0);
    let inputs = ActuatorInputs::default();

    let telem = backend.step(0.01, &inputs).expect("Step failed").clone();
    let _ = std::fs::remove_file(&shm_path);

    assert_eq!(
        backend.active_backend_name(),
        "Aerovex POSIX Shared Memory Connector"
    );
    assert!(backend.is_shm_active());
    assert_eq!(telem.position_m, [120.0, -80.0, -250.0]);
    assert_eq!(telem.velocity_m_per_s, [12.0, 1.5, -0.5]);
    assert_eq!(telem.sim_time_s, 5.0);
    assert_eq!(telem.step_count, 500);
}

#[test]
fn test_auto_selecting_backend_falls_back_on_shm_loss() {
    let shm_path = PathBuf::from("/tmp/aerovex_fallback_test_847297.bin");
    let initial_slot = ShmSlotData {
        sim_time_s: 2.0,
        total_ticks: 200,
        position_m: [50.0, 0.0, -100.0],
        orientation_quat: [1.0, 0.0, 0.0, 0.0],
        velocity_m_per_s: [5.0, 0.0, 0.0],
        angular_velocity_rad_per_s: [0.0, 0.0, 0.0],
    };
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let buf = create_mock_shm_buffer(2, 1, Some(now_epoch_ms), &[(0, 2, initial_slot)]);
    std::fs::write(&shm_path, &buf).expect("Failed to write mock SHM file");

    let mut backend = AutoSelectingDynamicsBackend::with_path(&shm_path, 0, 0);
    let inputs = ActuatorInputs::hover(3.6775);

    // Step 1: Ingests from SHM
    let telem1 = backend.step(0.01, &inputs).expect("Step 1 failed").clone();
    assert_eq!(
        backend.active_backend_name(),
        "Aerovex POSIX Shared Memory Connector"
    );
    assert!(backend.is_shm_active());
    assert_eq!(telem1.position_m, [50.0, 0.0, -100.0]);

    // Simulate daemon termination by removing the SHM file
    let _ = std::fs::remove_file(&shm_path);

    // Step 2: Falls back smoothly to Reference backend without interruption
    let telem2 = backend.step(0.01, &inputs).expect("Step 2 failed").clone();
    assert_eq!(
        backend.active_backend_name(),
        "Reference Dynamics Engine (RK4 6-DOF)"
    );
    assert!(!backend.is_shm_active());
    assert!(backend.is_healthy());
    assert!(telem2.sim_time_s > 2.0);
}

#[test]
fn test_shm_step_throughput_benchmark() {
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let slot_data = ShmSlotData {
        sim_time_s: 10.0,
        total_ticks: 1000,
        position_m: [10.0, 20.0, -30.0],
        orientation_quat: [1.0, 0.0, 0.0, 0.0],
        velocity_m_per_s: [5.0, 0.0, -1.0],
        angular_velocity_rad_per_s: [0.01, 0.02, 0.03],
    };
    let buf = create_mock_shm_buffer(2, 1, Some(now_epoch_ms), &[(0, 8, slot_data)]);

    let iterations = 100_000;
    let start = Instant::now();
    for _ in 0..iterations {
        let slot = safe_read_shm_slot(&buf, 0, 0).expect("Safe read failed in benchmark");
        assert_eq!(slot.total_ticks, 1000);
    }
    let elapsed = start.elapsed();
    let ticks_per_sec = iterations as f64 / elapsed.as_secs_f64();

    println!(
        "SHM Read Throughput: {:.2} ticks/sec ({:.2} ms for {} iterations)",
        ticks_per_sec,
        elapsed.as_secs_f64() * 1000.0,
        iterations
    );

    assert!(
        ticks_per_sec > 1_000_000.0,
        "Measured throughput {:.2} ticks/sec is below 1,000,000 ticks/sec threshold",
        ticks_per_sec
    );
}

#[test]
fn test_quaternion_renormalization_on_shm_read() {
    let unnormalized = ShmSlotData {
        sim_time_s: 1.0,
        total_ticks: 1,
        position_m: [0.0, 0.0, 0.0],
        orientation_quat: [2.0, 2.0, 2.0, 2.0], // Non-unit norm: sqrt(4*4) = 4.0
        velocity_m_per_s: [0.0, 0.0, 0.0],
        angular_velocity_rad_per_s: [0.0, 0.0, 0.0],
    };
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let buf = create_mock_shm_buffer(2, 1, Some(now_epoch_ms), &[(0, 2, unnormalized)]);

    let slot = safe_read_shm_slot(&buf, 0, 0).expect("Read failed");
    let q = slot.orientation_quat;
    let norm = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();

    assert!(
        (norm - 1.0).abs() < 1e-12,
        "Normalized quaternion norm {} should be 1.0 within 1e-12 tolerance",
        norm
    );
    for i in 0..4 {
        assert!(
            (q[i] - 0.5).abs() < 1e-12,
            "Quaternion component [{}] = {}, expected 0.5",
            i,
            q[i]
        );
    }
}

#[test]
fn test_probe_latency_sub_millisecond() {
    let temp_path = PathBuf::from("/tmp/aerovex_latency_test_probe_847298.bin");
    let now_epoch_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let buf = create_mock_shm_buffer(2, 1, Some(now_epoch_ms), &[(0, 2, ShmSlotData::default())]);
    std::fs::write(&temp_path, &buf).expect("Failed to write mock SHM file");

    let start = Instant::now();
    let status = AerovexPresenceProbe::probe_path(&temp_path);
    let duration = start.elapsed();
    let duration_us = duration.as_nanos() as f64 / 1000.0;
    let _ = std::fs::remove_file(&temp_path);

    assert!(matches!(status, ProbeStatus::Available { .. }));
    println!("Measured probe latency: {:.2} us", duration_us);

    assert!(
        duration_us < 500.0,
        "Probe latency {:.2} us exceeds 500.0 us threshold",
        duration_us
    );
}
