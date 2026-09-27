//! Distributed parallel sweep and columnar batch telemetry tests.

use phonon_cli::distributed::{ColumnarRecordBatch, DistributedCoordinator, SweepParameter};
use std::collections::HashMap;

#[test]
fn test_columnar_record_batch_and_feather_ipc_roundtrip() {
    let mut batch = ColumnarRecordBatch::new(vec![
        "time".to_string(),
        "V(out)".to_string(),
        "I(V1)".to_string(),
    ]);

    let row1 = [0.0, 1.25, -0.015];
    let row2 = [1e-6, 1.80, -0.012];
    let row3 = [2e-6, 0.45, -0.025];

    batch.push_row(&row1).unwrap();
    batch.push_row(&row2).unwrap();
    batch.push_row(&row3).unwrap();

    assert_eq!(batch.row_count(), 3);
    assert_eq!(batch.column_count(), 3);

    let v_out = batch.column("V(out)").expect("Column V(out) must exist");
    assert_eq!(v_out, &[1.25, 1.80, 0.45]);

    // Statistics
    let stats = batch.column_statistics("V(out)").unwrap();
    assert_eq!(stats.count, 3);
    assert!((stats.min - 0.45).abs() < 1e-6);
    assert!((stats.max - 1.80).abs() < 1e-6);
    assert!((stats.mean - (1.25 + 1.80 + 0.45) / 3.0).abs() < 1e-6);

    // Feather / Arrow IPC binary serialization round-trip
    let ipc_bytes = batch.to_feather_ipc_bytes();
    assert!(!ipc_bytes.is_empty());
    assert_eq!(&ipc_bytes[0..6], b"ARROW1");

    let decoded_batch =
        ColumnarRecordBatch::from_feather_ipc_bytes(&ipc_bytes).expect("Decoded batch must match");

    assert_eq!(decoded_batch.row_count(), batch.row_count());
    assert_eq!(decoded_batch.column_count(), batch.column_count());
    assert_eq!(decoded_batch.schema(), batch.schema());
    assert_eq!(
        decoded_batch.column("V(out)").unwrap(),
        batch.column("V(out)").unwrap()
    );

    // CSV format
    let csv = batch.to_csv();
    assert!(csv.contains("time,V(out),I(V1)"));
}

#[test]
fn test_distributed_coordinator_parallel_sweep() {
    let coordinator = DistributedCoordinator::new().with_worker_threads(4);

    let p1 = SweepParameter {
        target: "R1".to_string(),
        property: "resistance".to_string(),
        values: vec![100.0, 200.0, 500.0],
    };
    let p2 = SweepParameter {
        target: "V1".to_string(),
        property: "dc_value".to_string(),
        values: vec![1.0, 2.0, 5.0],
    };

    let tasks = DistributedCoordinator::generate_cartesian_tasks(&[p1, p2]);
    assert_eq!(tasks.len(), 9);

    let summary = coordinator.run_sweep(tasks, |task| {
        let r1 = *task.parameters.get("R1.resistance").unwrap();
        let v1 = *task.parameters.get("V1.dc_value").unwrap();

        // Model simple divider: V(out) = V1 / 2.0, Power = V1^2 / R1
        let mut metrics = HashMap::new();
        metrics.insert("V(out)".to_string(), v1 * 0.5);
        metrics.insert("P_diss".to_string(), (v1 * v1) / r1);

        Ok(metrics)
    });

    assert_eq!(summary.total_tasks, 9);
    assert_eq!(summary.successful_tasks, 9);
    assert_eq!(summary.failed_tasks, 0);
    assert_eq!(summary.batch.row_count(), 9);

    let v_out_stats = summary.metric_statistics.get("V(out)").unwrap();
    assert_eq!(v_out_stats.count, 9);
    assert!((v_out_stats.min - 0.5).abs() < 1e-6);
    assert!((v_out_stats.max - 2.5).abs() < 1e-6);
}
