#![deny(unsafe_code)]

//! Distributed work-stealing scheduler, node registry, and result aggregator.
//!
//! Manages heterogeneous cluster worker nodes, dynamic parameter space partitioning,
//! fault-tolerant batch re-queueing upon node failure, and statistical yield harvesting.

use std::collections::{HashMap, VecDeque};

use super::protocol::{NodeStatus, RpcMessage, SimulationType, SweepSample, WorkerNodeInfo};

/// Aggregated statistical sweep output across all distributed cluster worker nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct ClusterSweepResult {
    /// Unique job identifier string.
    pub job_id: String,
    /// Total number of parameter samples planned in the sweep.
    pub sample_count: usize,
    /// Total number of samples successfully evaluated and aggregated.
    pub completed_count: usize,
    /// Total count of errors or simulation divergence events.
    pub error_count: u32,
    /// Total wall clock execution time in seconds.
    pub execution_time_s: f64,
    /// Measured speedup multiplier versus single-core sequential baseline.
    pub speedup_factor: f64,
    /// Arithmetic mean across all evaluated sample responses.
    pub mean: f64,
    /// Sample standard deviation.
    pub std_dev: f64,
    /// Minimum harvested response value.
    pub min: f64,
    /// Maximum harvested response value.
    pub max: f64,
    /// Percentage of samples falling within specification limits (0.0 to 100.0).
    pub yield_percentage: f64,
    /// Histogram bins formatted as `(bin_start, bin_end, sample_count)`.
    pub bins: Vec<(f64, f64, usize)>,
}

/// Accumulator tracking in-progress sweep results for a given job.
#[derive(Debug, Clone, PartialEq)]
struct JobAccumulator {
    total_samples: usize,
    collected_values: Vec<f64>,
    error_count: u32,
    total_worker_cpu_time_ms: f64,
    start_time_s: f64,
    finish_time_s: Option<f64>,
}

/// Central distributed cluster coordinator managing node health, load-balancing, and fault tolerance.
#[derive(Debug, Clone, PartialEq)]
pub struct ClusterDispatchQueue {
    /// Registry of known worker nodes keyed by unique identifier.
    nodes: HashMap<String, WorkerNodeInfo>,
    /// Ordering list of registered node IDs.
    node_ids: Vec<String>,
    /// Global unassigned pending batches queue.
    pending_batches: VecDeque<RpcMessage>,
    /// Node-specific pre-assigned batch queues for fine-grained work stealing.
    node_batch_queues: HashMap<String, VecDeque<RpcMessage>>,
    /// Batches currently in flight: maps `batch_id` -> `(assigned_node_id, batch_message)`.
    in_flight_batches: HashMap<u32, (String, RpcMessage)>,
    /// Heartbeat timeout threshold in seconds before declaring a node offline (default 5.0s).
    heartbeat_timeout_s: f64,
    /// In-progress and completed job accumulators keyed by `job_id`.
    job_accumulators: HashMap<String, JobAccumulator>,
}

impl Default for ClusterDispatchQueue {
    fn default() -> Self {
        Self::new(5.0)
    }
}

impl ClusterDispatchQueue {
    /// Creates a new cluster dispatch queue with configurable heartbeat timeout in seconds.
    pub fn new(heartbeat_timeout_s: f64) -> Self {
        Self {
            nodes: HashMap::new(),
            node_ids: Vec::new(),
            pending_batches: VecDeque::new(),
            node_batch_queues: HashMap::new(),
            in_flight_batches: HashMap::new(),
            heartbeat_timeout_s: heartbeat_timeout_s.max(0.1),
            job_accumulators: HashMap::new(),
        }
    }

    /// Sets the heartbeat timeout threshold in seconds.
    pub fn set_heartbeat_timeout_s(&mut self, timeout_s: f64) {
        self.heartbeat_timeout_s = timeout_s.max(0.1);
    }

    /// Returns the active heartbeat timeout threshold in seconds.
    pub fn heartbeat_timeout_s(&self) -> f64 {
        self.heartbeat_timeout_s
    }

    // --- Node Registry Operations ---

    /// Registers a new worker node or updates an existing record.
    pub fn add_node(&mut self, node: WorkerNodeInfo) {
        let id = node.id.clone();
        if !self.nodes.contains_key(&id) {
            self.node_ids.push(id.clone());
            self.node_batch_queues.insert(id.clone(), VecDeque::new());
        }
        self.nodes.insert(id, node);
    }

    /// Deregisters and removes a worker node, re-queueing any work currently assigned to it.
    pub fn remove_node(&mut self, node_id: &str) -> Option<WorkerNodeInfo> {
        let _ = self.handle_node_failure(node_id);
        self.node_ids.retain(|id| id != node_id);
        self.node_batch_queues.remove(node_id);
        self.nodes.remove(node_id)
    }

    /// Returns an immutable reference to a worker node by ID.
    pub fn get_node(&self, node_id: &str) -> Option<&WorkerNodeInfo> {
        self.nodes.get(node_id)
    }

    /// Returns a mutable reference to a worker node by ID.
    pub fn get_node_mut(&mut self, node_id: &str) -> Option<&mut WorkerNodeInfo> {
        self.nodes.get_mut(node_id)
    }

    /// Returns an iterator over all registered worker nodes.
    pub fn nodes(&self) -> impl Iterator<Item = &WorkerNodeInfo> {
        self.nodes.values()
    }

    /// Returns the total count of registered nodes.
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Returns count of nodes currently in Online or Idle status.
    pub fn available_node_count(&self) -> usize {
        self.nodes.values().filter(|n| n.status.is_available()).count()
    }

    /// Processes a heartbeat telemetry message from a worker node.
    pub fn heartbeat(
        &mut self,
        node_id: &str,
        timestamp_s: f64,
        cpu_usage_pct: f64,
        _memory_usage_mb: f64,
    ) -> bool {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.last_heartbeat_s = timestamp_s;
            if node.status == NodeStatus::Offline {
                node.status = NodeStatus::Online;
            } else if node.status == NodeStatus::Online && cpu_usage_pct < 10.0 {
                node.status = NodeStatus::Idle;
            }
            true
        } else {
            false
        }
    }

    /// Scans registered worker nodes and marks any node exceeding heartbeat timeout as Offline.
    /// Automatically recovers and re-queues any batches in flight on timed out nodes.
    pub fn mark_offline_timed_out(&mut self, current_time_s: f64) -> Vec<String> {
        let mut timed_out_ids = Vec::new();

        for node in self.nodes.values_mut() {
            if node.status != NodeStatus::Offline
                && (current_time_s - node.last_heartbeat_s) > self.heartbeat_timeout_s
            {
                node.status = NodeStatus::Offline;
                timed_out_ids.push(node.id.clone());
            }
        }

        for id in &timed_out_ids {
            let recovered = self.handle_node_failure(id);
            if recovered > 0 {
                // Batches re-queued to pending_batches
            }
        }

        timed_out_ids
    }

    // --- Parameter Space Partitioning ---

    /// Partitions a vector of parameter sweep samples into batches of size `chunk_size`.
    pub fn partition_samples(
        job_id: &str,
        samples: Vec<SweepSample>,
        chunk_size: usize,
        sim_type: SimulationType,
    ) -> Vec<RpcMessage> {
        let effective_chunk_size = chunk_size.max(1);
        let mut batches = Vec::new();
        let mut batch_id = 0u32;

        for chunk in samples.chunks(effective_chunk_size) {
            batches.push(RpcMessage::DispatchBatch {
                job_id: job_id.to_string(),
                batch_id,
                samples: chunk.to_vec(),
                sim_type,
            });
            batch_id += 1;
        }

        batches
    }

    /// Dispatches a parameter sweep job by partitioning samples and enqueueing batches.
    pub fn dispatch_sweep(
        &mut self,
        job_id: &str,
        samples: Vec<SweepSample>,
        chunk_size: usize,
        sim_type: SimulationType,
        start_time_s: f64,
    ) {
        let total_samples = samples.len();
        let batches = Self::partition_samples(job_id, samples, chunk_size, sim_type);

        for batch in batches {
            self.pending_batches.push_back(batch);
        }

        self.job_accumulators.insert(
            job_id.to_string(),
            JobAccumulator {
                total_samples,
                collected_values: Vec::with_capacity(total_samples),
                error_count: 0,
                total_worker_cpu_time_ms: 0.0,
                start_time_s,
                finish_time_s: None,
            },
        );
    }

    /// Pre-partitions batches directly across registered available worker queues.
    pub fn pre_distribute_batches(&mut self) {
        let available_ids: Vec<String> = self
            .node_ids
            .iter()
            .filter(|id| {
                self.nodes
                    .get(*id)
                    .map(|n| n.status != NodeStatus::Offline)
                    .unwrap_or(false)
            })
            .cloned()
            .collect();

        if available_ids.is_empty() {
            return;
        }

        let mut worker_idx = 0;
        while let Some(batch) = self.pending_batches.pop_front() {
            let target_node = &available_ids[worker_idx % available_ids.len()];
            if let Some(queue) = self.node_batch_queues.get_mut(target_node) {
                queue.push_back(batch);
            }
            worker_idx += 1;
        }
    }

    // --- Work-Stealing Scheduling ---

    /// Assigns the next batch for a specific worker node.
    ///
    /// Pulls from the worker's dedicated queue first, then from the global pending queue,
    /// and finally attempts to steal from the queue of the busiest node.
    pub fn assign_next_batch(&mut self, node_id: &str) -> Option<RpcMessage> {
        // 1. Check node-specific queue
        if let Some(queue) = self.node_batch_queues.get_mut(node_id) {
            if let Some(batch) = queue.pop_front() {
                return self.track_and_assign(node_id, batch);
            }
        }

        // 2. Check global pending queue
        if let Some(batch) = self.pending_batches.pop_front() {
            return self.track_and_assign(node_id, batch);
        }

        // 3. Work-stealing: steal from other nodes with excess queued work
        self.steal_batch(node_id)
    }

    /// Attempts to steal a queued batch from another worker node to keep the requesting worker busy.
    pub fn steal_batch(&mut self, thief_node_id: &str) -> Option<RpcMessage> {
        // Find the victim node with the largest number of queued batches
        let mut victim_id: Option<String> = None;
        let mut max_queue_len = 0;

        for (id, queue) in &self.node_batch_queues {
            if id != thief_node_id && queue.len() > max_queue_len {
                max_queue_len = queue.len();
                victim_id = Some(id.clone());
            }
        }

        if let Some(v_id) = victim_id {
            if let Some(queue) = self.node_batch_queues.get_mut(&v_id) {
                // Steal from the back (FIFO for owner, LIFO/tail steal for thieves)
                if let Some(stolen_batch) = queue.pop_back() {
                    return self.track_and_assign(thief_node_id, stolen_batch);
                }
            }
        }

        None
    }

    fn track_and_assign(&mut self, node_id: &str, batch: RpcMessage) -> Option<RpcMessage> {
        if let RpcMessage::DispatchBatch { batch_id, .. } = &batch {
            self.in_flight_batches
                .insert(*batch_id, (node_id.to_string(), batch.clone()));
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.status = NodeStatus::Busy;
            }
            Some(batch)
        } else {
            None
        }
    }

    // --- Resiliency & Fault Tolerance ---

    /// Recovers all pending and in-flight batches belonging to a failed or timed out worker node,
    /// pushing them back to the head of the global pending queue.
    pub fn handle_node_failure(&mut self, node_id: &str) -> usize {
        let mut recovered_count = 0;

        // 1. Recover in-flight batches
        let in_flight_keys: Vec<u32> = self
            .in_flight_batches
            .iter()
            .filter(|(_, (assigned_node, _))| assigned_node == node_id)
            .map(|(k, _)| *k)
            .collect();

        for k in in_flight_keys {
            if let Some((_, batch)) = self.in_flight_batches.remove(&k) {
                self.pending_batches.push_front(batch);
                recovered_count += 1;
            }
        }

        // 2. Recover pre-assigned queued batches
        if let Some(queue) = self.node_batch_queues.get_mut(node_id) {
            while let Some(batch) = queue.pop_back() {
                self.pending_batches.push_front(batch);
                recovered_count += 1;
            }
        }

        recovered_count
    }

    // --- Result Ingestion & Aggregation ---

    /// Ingests a completed batch result from a worker, updating node throughput and telemetry.
    pub fn record_batch_result(&mut self, result: &RpcMessage, current_time_s: f64) -> bool {
        if let RpcMessage::BatchResult {
            job_id,
            batch_id,
            node_id,
            execution_time_ms,
            sample_values,
            errors,
        } = result
        {
            // Remove from in-flight
            self.in_flight_batches.remove(batch_id);

            // Update worker statistics
            if let Some(node) = self.nodes.get_mut(node_id) {
                node.jobs_completed += 1;
                node.status = NodeStatus::Idle;
                node.last_heartbeat_s = current_time_s;
                let exec_s = (*execution_time_ms / 1000.0).max(0.001);
                let batch_throughput = sample_values.len() as f64 / exec_s;
                // Exponential moving average for throughput
                if node.active_throughput_jobs_sec <= 0.0 {
                    node.active_throughput_jobs_sec = batch_throughput;
                } else {
                    node.active_throughput_jobs_sec =
                        0.7 * node.active_throughput_jobs_sec + 0.3 * batch_throughput;
                }
            }

            // Ingest into accumulator
            if let Some(acc) = self.job_accumulators.get_mut(job_id) {
                acc.collected_values.extend_from_slice(sample_values);
                acc.error_count += *errors;
                acc.total_worker_cpu_time_ms += *execution_time_ms;

                if acc.collected_values.len() + acc.error_count as usize >= acc.total_samples {
                    acc.finish_time_s = Some(current_time_s);
                }
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Computes summary moments, yield capability, speedup factor, and distribution histogram.
    pub fn finalize_result(
        &self,
        job_id: &str,
        lsl: Option<f64>,
        usl: Option<f64>,
        current_time_s: f64,
    ) -> Option<ClusterSweepResult> {
        let acc = self.job_accumulators.get(job_id)?;
        let n = acc.collected_values.len();
        if n == 0 {
            return None;
        }

        let sum: f64 = acc.collected_values.iter().sum();
        let mean = sum / (n as f64);

        let variance = if n > 1 {
            let sq_sum: f64 = acc.collected_values.iter().map(|v| (v - mean).powi(2)).sum();
            sq_sum / ((n - 1) as f64)
        } else {
            0.0
        };
        let std_dev = variance.sqrt();

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        let mut passed_count = 0usize;

        for &v in &acc.collected_values {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }

            let meets_lsl = lsl.map_or(true, |l| v >= l);
            let meets_usl = usl.map_or(true, |u| v <= u);
            if meets_lsl && meets_usl {
                passed_count += 1;
            }
        }

        if min.is_infinite() {
            min = 0.0;
        }
        if max.is_infinite() {
            max = 0.0;
        }

        let yield_percentage = (passed_count as f64 / n as f64) * 100.0;

        let end_s = acc.finish_time_s.unwrap_or(current_time_s);
        let wall_time_s = (end_s - acc.start_time_s).max(0.001);
        let total_cpu_s = acc.total_worker_cpu_time_ms / 1000.0;

        let speedup_factor = if wall_time_s > 0.0 && total_cpu_s > 0.0 {
            (total_cpu_s / wall_time_s).max(1.0)
        } else {
            1.0
        };

        let bins = Self::generate_histogram_bins(&acc.collected_values, 20);

        Some(ClusterSweepResult {
            job_id: job_id.to_string(),
            sample_count: acc.total_samples,
            completed_count: n,
            error_count: acc.error_count,
            execution_time_s: wall_time_s,
            speedup_factor,
            mean,
            std_dev,
            min,
            max,
            yield_percentage,
            bins,
        })
    }

    /// Generates uniform histogram bins from raw floating point samples.
    pub fn generate_histogram_bins(
        values: &[f64],
        bin_count: usize,
    ) -> Vec<(f64, f64, usize)> {
        if values.is_empty() || bin_count == 0 {
            return Vec::new();
        }

        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;
        for &v in values {
            if v < min {
                min = v;
            }
            if v > max {
                max = v;
            }
        }

        if (max - min).abs() < 1e-12 {
            return vec![(min - 0.5, max + 0.5, values.len())];
        }

        let step = (max - min) / (bin_count as f64);
        let mut counts = vec![0usize; bin_count];

        for &v in values {
            let mut idx = ((v - min) / step).floor() as usize;
            if idx >= bin_count {
                idx = bin_count - 1;
            }
            counts[idx] += 1;
        }

        let mut bins = Vec::with_capacity(bin_count);
        for i in 0..bin_count {
            let b_start = min + (i as f64) * step;
            let b_end = b_start + step;
            bins.push((b_start, b_end, counts[i]));
        }

        bins
    }

    /// Returns the number of batches currently awaiting assignment.
    pub fn pending_batch_count(&self) -> usize {
        let node_queued: usize = self.node_batch_queues.values().map(|q| q.len()).sum();
        self.pending_batches.len() + node_queued
    }

    /// Returns the number of batches currently in flight across worker nodes.
    pub fn in_flight_batch_count(&self) -> usize {
        self.in_flight_batches.len()
    }

    /// Returns true if all pending and in-flight batches have completed.
    pub fn is_sweep_complete(&self) -> bool {
        self.pending_batch_count() == 0 && self.in_flight_batch_count() == 0
    }
}
