#![deny(unsafe_code)]

//! Test suite for Phase 332: Distributed Cloud Parameter Sweep Cluster Engine.
//!
//! Verifies:
//! - Worker registration, heartbeat tracking, and node timeout detection.
//! - Parameter space partitioning into balanced batches.
//! - Work-stealing load balancing across heterogeneous worker speeds.
//! - Worker failure / dropout fault tolerance and batch re-queueing.
//! - Multi-node result aggregation, statistical moments, yield %, and histogram generation.
//! - Wire protocol serialization and deserialization roundtrips.

use std::collections::HashMap;

use phonon_solver::cluster::{
    ClusterDispatchQueue, NodeStatus, ProtocolError, RpcMessage, SimulatedClusterWorker,
    SimulationType, SweepSample, WorkerNodeInfo, PROTOCOL_MAGIC,
};

#[test]
fn test_rpc_message_wire_serialization_roundtrip() {
    // 1. RegisterNode roundtrip
    let node = WorkerNodeInfo::new("node-alpha", "10.0.0.1:8080", 8, 16384);
    let msg1 = RpcMessage::RegisterNode { node };
    let bytes1 = msg1.to_bytes();
    assert_eq!(&bytes1[0..8], &PROTOCOL_MAGIC);
    let decoded1 = RpcMessage::from_bytes(&bytes1).expect("Failed to deserialize RegisterNode");
    assert_eq!(msg1, decoded1);

    // 2. Heartbeat roundtrip
    let msg2 = RpcMessage::Heartbeat {
        node_id: "node-alpha".to_string(),
        timestamp_s: 123.456,
        cpu_usage_pct: 78.5,
        memory_usage_mb: 4096.0,
    };
    let bytes2 = msg2.to_bytes();
    let decoded2 = RpcMessage::from_bytes(&bytes2).expect("Failed to deserialize Heartbeat");
    assert_eq!(msg2, decoded2);

    // 3. DispatchBatch roundtrip
    let mut overrides = HashMap::new();
    overrides.insert("R1".to_string(), 1000.0);
    overrides.insert("C1".to_string(), 1e-9);
    let sample = SweepSample::with_overrides(42, overrides);
    let msg3 = RpcMessage::DispatchBatch {
        job_id: "job-001".to_string(),
        batch_id: 7,
        samples: vec![sample],
        sim_type: SimulationType::MonteCarlo,
    };
    let bytes3 = msg3.to_bytes();
    let decoded3 = RpcMessage::from_bytes(&bytes3).expect("Failed to deserialize DispatchBatch");
    assert_eq!(msg3, decoded3);

    // 4. BatchResult roundtrip
    let msg4 = RpcMessage::BatchResult {
        job_id: "job-001".to_string(),
        batch_id: 7,
        node_id: "node-alpha".to_string(),
        execution_time_ms: 15.2,
        sample_values: vec![2.48, 2.51, 2.49],
        errors: 0,
    };
    let bytes4 = msg4.to_bytes();
    let decoded4 = RpcMessage::from_bytes(&bytes4).expect("Failed to deserialize BatchResult");
    assert_eq!(msg4, decoded4);

    // 5. DeregisterNode roundtrip
    let msg5 = RpcMessage::DeregisterNode {
        node_id: "node-gamma".to_string(),
    };
    let bytes5 = msg5.to_bytes();
    let decoded5 = RpcMessage::from_bytes(&bytes5).expect("Failed to deserialize DeregisterNode");
    assert_eq!(msg5, decoded5);

    // 6. Corrupt header detection
    let mut corrupt_bytes = bytes1.clone();
    corrupt_bytes[0] = b'X';
    let err = RpcMessage::from_bytes(&corrupt_bytes);
    assert_eq!(err, Err(ProtocolError::InvalidMagic));
}

#[test]
fn test_worker_registration_heartbeat_and_timeout_detection() {
    let mut queue = ClusterDispatchQueue::new(3.0); // 3-second heartbeat timeout
    assert_eq!(queue.node_count(), 0);

    let n1 = WorkerNodeInfo::new("node-1", "192.168.1.10", 4, 8192);
    let n2 = WorkerNodeInfo::new("node-2", "192.168.1.11", 8, 16384);
    queue.add_node(n1);
    queue.add_node(n2);

    assert_eq!(queue.node_count(), 2);
    assert_eq!(queue.available_node_count(), 2);

    // Send heartbeats at t = 10.0s
    let ok = queue.heartbeat("node-1", 10.0, 5.0, 500.0);
    assert!(ok);
    let ok2 = queue.heartbeat("node-2", 10.0, 12.0, 800.0);
    assert!(ok2);

    // Node 1 receives heartbeat at t = 12.5s, Node 2 does not
    queue.heartbeat("node-1", 12.5, 4.0, 510.0);

    // At current_time = 13.5s:
    // Node 1 elapsed = 1.0s <= 3.0s (healthy)
    // Node 2 elapsed = 3.5s > 3.0s (timed out!)
    let timed_out = queue.mark_offline_timed_out(13.5);
    assert_eq!(timed_out, vec!["node-2".to_string()]);

    let node2 = queue.get_node("node-2").expect("node-2 must exist");
    assert_eq!(node2.status, NodeStatus::Offline);

    let node1 = queue.get_node("node-1").expect("node-1 must exist");
    assert_ne!(node1.status, NodeStatus::Offline);

    // Node 2 recovers and sends heartbeat at t = 14.0s
    queue.heartbeat("node-2", 14.0, 2.0, 800.0);
    let node2_recovered = queue.get_node("node-2").unwrap();
    assert_eq!(node2_recovered.status, NodeStatus::Online);
}

#[test]
fn test_parameter_space_partitioning_into_balanced_batches() {
    let samples: Vec<SweepSample> = (0..250)
        .map(|i| {
            let mut overrides = HashMap::new();
            overrides.insert("idx".to_string(), i as f64);
            SweepSample::with_overrides(i, overrides)
        })
        .collect();

    let chunk_size = 50;
    let batches = ClusterDispatchQueue::partition_samples(
        "job-partition-test",
        samples,
        chunk_size,
        SimulationType::MonteCarlo,
    );

    // 250 samples / 50 chunk_size = 5 batches
    assert_eq!(batches.len(), 5);

    for (i, batch) in batches.iter().enumerate() {
        if let RpcMessage::DispatchBatch {
            job_id,
            batch_id,
            samples,
            sim_type,
        } = batch
        {
            assert_eq!(job_id, "job-partition-test");
            assert_eq!(*batch_id, i as u32);
            assert_eq!(samples.len(), 50);
            assert_eq!(*sim_type, SimulationType::MonteCarlo);
        } else {
            panic!("Expected DispatchBatch message");
        }
    }
}

#[test]
fn test_work_stealing_load_balancing_heterogeneous_workers() {
    let mut queue = ClusterDispatchQueue::new(5.0);

    // Fast worker: 16 cores
    let fast_node = WorkerNodeInfo::new("fast-worker", "10.0.0.1", 16, 32768);
    // Slow worker: 2 cores
    let slow_node = WorkerNodeInfo::new("slow-worker", "10.0.0.2", 2, 4096);

    queue.add_node(fast_node);
    queue.add_node(slow_node);

    // Create 120 samples chunked into 12 batches of 10 samples
    let samples: Vec<SweepSample> = (0..120).map(SweepSample::new).collect();
    queue.dispatch_sweep(
        "job-stealing",
        samples,
        10,
        SimulationType::MonteCarlo,
        0.0,
    );

    // Pre-distribute batches across node queues: 6 to fast, 6 to slow
    queue.pre_distribute_batches();

    let mut fast_worker = SimulatedClusterWorker::new("fast-worker", "10.0.0.1", 16, 32768)
        .with_speed(3.0); // 3x speed multiplier
    let mut slow_worker = SimulatedClusterWorker::new("slow-worker", "10.0.0.2", 2, 4096)
        .with_speed(0.5); // 0.5x speed multiplier

    let mut sim_clock_s = 0.0;
    let dt = 0.01;

    // Simulation loop
    for _ in 0..1000 {
        sim_clock_s += dt;

        // Assign to fast worker if idle
        if fast_worker.is_ready() {
            if let Some(batch) = queue.assign_next_batch("fast-worker") {
                fast_worker.receive_batch(batch);
            }
        }

        // Assign to slow worker if idle
        if slow_worker.is_ready() {
            if let Some(batch) = queue.assign_next_batch("slow-worker") {
                slow_worker.receive_batch(batch);
            }
        }

        // Step workers
        if let Some(res) = fast_worker.step(dt) {
            queue.record_batch_result(&res, sim_clock_s);
        }
        if let Some(res) = slow_worker.step(dt) {
            queue.record_batch_result(&res, sim_clock_s);
        }

        if queue.is_sweep_complete() {
            break;
        }
    }

    assert!(
        queue.is_sweep_complete(),
        "Sweep must complete via work stealing"
    );

    // Verify fast worker completed significantly more batches than slow worker due to work-stealing
    assert!(
        fast_worker.total_jobs_completed > slow_worker.total_jobs_completed,
        "Fast worker ({}) should complete more jobs than slow worker ({})",
        fast_worker.total_jobs_completed,
        slow_worker.total_jobs_completed
    );

    assert_eq!(
        fast_worker.total_jobs_completed + slow_worker.total_jobs_completed,
        12,
        "Total completed batches must equal 12"
    );
}

#[test]
fn test_worker_failure_dropout_fault_tolerance_requeue() {
    let mut queue = ClusterDispatchQueue::new(2.0); // 2-second timeout

    let n1 = WorkerNodeInfo::new("node-alpha", "192.168.1.1", 4, 8192);
    let n2 = WorkerNodeInfo::new("node-bravo", "192.168.1.2", 4, 8192);
    queue.add_node(n1);
    queue.add_node(n2);

    let samples: Vec<SweepSample> = (0..60).map(SweepSample::new).collect();
    queue.dispatch_sweep(
        "job-fault-tolerance",
        samples,
        20,
        SimulationType::MonteCarlo,
        0.0,
    );
    // 3 batches of 20 samples

    // Initial heartbeats
    queue.heartbeat("node-alpha", 0.0, 5.0, 500.0);
    queue.heartbeat("node-bravo", 0.0, 5.0, 500.0);

    // Assign batch 0 to alpha, batch 1 to bravo
    let b0 = queue.assign_next_batch("node-alpha").expect("Batch 0 assigned");
    let b1 = queue.assign_next_batch("node-bravo").expect("Batch 1 assigned");

    assert_eq!(queue.in_flight_batch_count(), 2);
    assert_eq!(queue.pending_batch_count(), 1);

    // Bravo completes its batch successfully
    let mut w_bravo = SimulatedClusterWorker::new("node-bravo", "192.168.1.2", 4, 8192);
    let res1 = w_bravo.execute_batch_immediate(b1);
    queue.record_batch_result(&res1, 0.5);

    assert_eq!(queue.in_flight_batch_count(), 1);

    // Bravo keeps heartbeating normally at t = 2.0s
    queue.heartbeat("node-bravo", 2.0, 5.0, 500.0);

    // Alpha crashes and stops heartbeating. At t = 2.5s, alpha times out.
    let timed_out = queue.mark_offline_timed_out(2.5);
    assert_eq!(timed_out, vec!["node-alpha".to_string()]);

    // Alpha's in-flight batch (batch 0) must be automatically recovered and re-queued
    assert_eq!(
        queue.in_flight_batch_count(),
        0,
        "In-flight batch on failed node must be cleared"
    );
    assert_eq!(
        queue.pending_batch_count(),
        2,
        "Batch 0 must be re-queued back to pending along with batch 2"
    );

    // Bravo (healthy) requests next batch and gets the recovered batch!
    let recovered_batch = queue
        .assign_next_batch("node-bravo")
        .expect("Bravo must pick up recovered batch");

    if let RpcMessage::DispatchBatch { batch_id, .. } = recovered_batch {
        assert_eq!(batch_id, 0, "Recovered batch 0 must be reassigned first");
    } else {
        panic!("Expected DispatchBatch");
    }

    let res0 = w_bravo.execute_batch_immediate(b0);
    queue.record_batch_result(&res0, 1.0);

    // Bravo also picks up batch 2
    let b2 = queue.assign_next_batch("node-bravo").expect("Batch 2 assigned");
    let res2 = w_bravo.execute_batch_immediate(b2);
    queue.record_batch_result(&res2, 1.5);

    assert!(queue.is_sweep_complete());

    let final_res = queue
        .finalize_result("job-fault-tolerance", None, None, 1.5)
        .expect("Result must finalize");

    assert_eq!(final_res.completed_count, 60);
    assert_eq!(final_res.error_count, 0);
}

#[test]
fn test_multi_node_result_aggregation_and_yield_calculations() {
    let mut queue = ClusterDispatchQueue::new(5.0);

    let n1 = WorkerNodeInfo::new("w1", "10.0.0.1", 8, 16384);
    let n2 = WorkerNodeInfo::new("w2", "10.0.0.2", 8, 16384);
    queue.add_node(n1);
    queue.add_node(n2);

    let mut w1 = SimulatedClusterWorker::new("w1", "10.0.0.1", 8, 16384);
    let mut w2 = SimulatedClusterWorker::new("w2", "10.0.0.2", 8, 16384);

    let samples: Vec<SweepSample> = (0..200).map(SweepSample::new).collect();
    queue.dispatch_sweep(
        "job-yield-agg",
        samples,
        50,
        SimulationType::MonteCarlo,
        0.0,
    );

    let mut sim_clock = 0.0;
    while !queue.is_sweep_complete() {
        if let Some(b) = queue.assign_next_batch("w1") {
            let res = w1.execute_batch_immediate(b);
            sim_clock += 0.05;
            queue.record_batch_result(&res, sim_clock);
        }
        if let Some(b) = queue.assign_next_batch("w2") {
            let res = w2.execute_batch_immediate(b);
            sim_clock += 0.05;
            queue.record_batch_result(&res, sim_clock);
        }
    }

    // Specification limits: LSL = 2.40, USL = 2.60
    let res = queue
        .finalize_result("job-yield-agg", Some(2.40), Some(2.60), sim_clock)
        .expect("Final result must be populated");

    assert_eq!(res.sample_count, 200);
    assert_eq!(res.completed_count, 200);
    assert_eq!(res.error_count, 0);

    // Mean should be centered near 2.50
    assert!(
        (res.mean - 2.50).abs() < 0.05,
        "Mean ({}) should be close to 2.50",
        res.mean
    );
    assert!(res.std_dev > 0.0, "StdDev must be positive");
    assert!(res.min <= res.mean);
    assert!(res.max >= res.mean);

    // Yield percentage should be between 60% and 99% for +/- 1.25 sigma
    assert!(
        res.yield_percentage >= 50.0 && res.yield_percentage <= 100.0,
        "Yield percentage ({}) must be physically plausible",
        res.yield_percentage
    );

    // Histogram verification
    assert_eq!(res.bins.len(), 20, "Must generate 20 histogram bins");
    let total_hist_count: usize = res.bins.iter().map(|(_, _, c)| *c).sum();
    assert_eq!(
        total_hist_count, 200,
        "Histogram sum must equal total samples"
    );

    for (b_start, b_end, _) in &res.bins {
        assert!(b_end >= b_start, "Histogram bin bounds must be ordered");
    }
}
