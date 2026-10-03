#![deny(unsafe_code)]

//! Verification test suite for Phase 332: Interactive Distributed Cluster Dashboard GUI.
//!
//! Verifies:
//! - Dialog initialization and default 4-node cluster topology.
//! - Parameter sweep dispatch and clock progress stepping.
//! - Dynamic worker node addition and fault dropout simulation.
//! - Headless egui Context render pass.

use phonon_gui::widgets::cluster_dashboard_dialog::ClusterDashboardDialog;
use phonon_solver::cluster::{NodeStatus, SimulationType};

#[test]
fn test_cluster_dashboard_dialog_initialization_defaults() {
    let dialog = ClusterDashboardDialog::new();

    // Dialog closed by default
    assert!(!dialog.is_open, "Dialog must be closed by default");
    assert!(!dialog.is_running, "Sweep must not be running initially");

    // Default configuration parameters
    assert_eq!(dialog.total_samples, 2000);
    assert_eq!(dialog.batch_size, 100);
    assert_eq!(dialog.sim_type, SimulationType::MonteCarlo);
    assert!(dialog.enable_lsl);
    assert!(dialog.enable_usl);
    assert!(dialog.active_result.is_none());

    // Default 4-node topology
    assert_eq!(dialog.workers.len(), 4, "Default topology must have 4 workers");
    assert_eq!(dialog.scheduler.node_count(), 4, "Scheduler must have 4 nodes");

    let ids: Vec<&str> = dialog.workers.iter().map(|w| w.info.id.as_str()).collect();
    assert!(ids.contains(&"node-alpha"));
    assert!(ids.contains(&"node-beta"));
    assert!(ids.contains(&"node-gamma"));
    assert!(ids.contains(&"node-delta"));

    let total_cores: usize = dialog.workers.iter().map(|w| w.info.core_count).sum();
    assert_eq!(total_cores, 36, "4 default nodes must provide 36 aggregate cores");

    for w in &dialog.workers {
        assert_eq!(w.info.status, NodeStatus::Idle);
    }
}

#[test]
fn test_cluster_sweep_dispatch_and_progress_stepping() {
    let mut dialog = ClusterDashboardDialog::new();
    dialog.total_samples = 200;
    dialog.batch_size = 50;

    // Dispatch cluster sweep
    dialog.dispatch_sweep();

    assert!(dialog.is_running, "Cluster sweep must be marked running");
    assert_eq!(dialog.total_job_samples, 200);
    assert_eq!(dialog.completed_samples, 0);
    assert!(dialog.scheduler.pending_batch_count() > 0);

    // Step simulation clock until sweep completes
    let dt = 0.05;
    let mut steps = 0;
    while dialog.is_running && steps < 500 {
        dialog.step_cluster(dt);
        steps += 1;
    }

    assert!(
        !dialog.is_running,
        "Cluster sweep must complete within step limit"
    );
    assert_eq!(
        dialog.completed_samples, 200,
        "All 200 samples must be completed"
    );
    assert!(
        dialog.active_result.is_some(),
        "Active result must be populated after sweep finish"
    );

    let res = dialog.active_result.as_ref().unwrap();
    assert_eq!(res.sample_count, 200);
    assert_eq!(res.completed_count, 200);
    assert!(res.mean > 0.0);
    assert!(res.std_dev >= 0.0);
    assert!(res.yield_percentage > 0.0 && res.yield_percentage <= 100.0);
    assert!(!res.bins.is_empty(), "Histogram bins must be generated");
}

#[test]
fn test_dynamic_worker_addition_and_dropout_simulation() {
    let mut dialog = ClusterDashboardDialog::new();
    assert_eq!(dialog.workers.len(), 4);

    // 1. Dynamically provision a new worker node
    dialog.add_worker_node();
    assert_eq!(dialog.workers.len(), 5);
    assert_eq!(dialog.scheduler.node_count(), 5);

    let new_worker = dialog.workers.last().unwrap();
    assert_eq!(new_worker.info.id, "node-worker-5");
    assert_eq!(new_worker.info.core_count, 8);
    assert!(dialog.status_msg.contains("Worker 'node-worker-5' added"));

    // 2. Dispatch a sweep
    dialog.total_samples = 300;
    dialog.batch_size = 50;
    dialog.dispatch_sweep();

    // Step once to assign batches
    dialog.step_cluster(0.01);

    // 3. Simulate sudden worker failure / dropout
    let dropped_id = dialog.simulate_dropout();
    assert!(dropped_id.is_some(), "Must drop an active node");
    let dropped_id_str = dropped_id.unwrap();

    let dropped_worker = dialog
        .workers
        .iter()
        .find(|w| w.info.id == dropped_id_str)
        .expect("Dropped worker must exist");
    assert_eq!(dropped_worker.info.status, NodeStatus::Offline);

    assert!(dialog.status_msg.contains("dropped offline"));

    // 4. Continue stepping: cluster must tolerate dropout and finish sweep using remaining nodes
    let mut steps = 0;
    while dialog.is_running && steps < 800 {
        dialog.step_cluster(0.05);
        steps += 1;
    }

    assert!(
        !dialog.is_running,
        "Cluster must achieve complete sweep despite worker dropout"
    );
    assert_eq!(dialog.completed_samples, 300);
    assert!(dialog.active_result.is_some());
}

#[test]
fn test_headless_egui_render_pass() {
    let mut dialog = ClusterDashboardDialog::new();
    dialog.is_open = true;

    // Pre-populate with completed sweep results
    dialog.total_samples = 100;
    dialog.batch_size = 50;
    dialog.dispatch_sweep();
    while dialog.is_running {
        dialog.step_cluster(0.05);
    }

    // Execute headless egui frame
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(Default::default(), |ui| {
        dialog.render_content(ui);
    });
    output.textures_delta.clear();

    assert!(dialog.is_open);
    assert!(dialog.active_result.is_some());
}
