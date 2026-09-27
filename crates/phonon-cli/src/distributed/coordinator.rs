//! Multi-threaded / distributed simulation coordinator for large-scale parametric sweeps
//! and Monte Carlo studies across cloud worker nodes.

use super::telemetry_arrow::{ColumnStatistics, ColumnarRecordBatch};
use rayon::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

/// Individual parameter specification for multidimensional grid or Monte Carlo sweeps.
#[derive(Debug, Clone, PartialEq)]
pub struct SweepParameter {
    /// Target component name (e.g. "R1", "V1", "M1").
    pub target: String,
    /// Property name (e.g. "resistance", "dc_value", "w", "l", "temp").
    pub property: String,
    /// Discrete values to evaluate.
    pub values: Vec<f64>,
}

/// A single discrete simulation task within a distributed sweep job.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulationTask {
    /// Task identifier.
    pub task_id: usize,
    /// Parameter assignments for this specific run: mapping `target.property -> value`.
    pub parameters: HashMap<String, f64>,
}

/// Aggregated results and statistical metrics from a completed parallel sweep.
#[derive(Debug, Clone)]
pub struct SweepResultSummary {
    /// Total tasks dispatched.
    pub total_tasks: usize,
    /// Successfully converged tasks.
    pub successful_tasks: usize,
    /// Failed or diverging tasks.
    pub failed_tasks: usize,
    /// Wall-clock time spent in seconds.
    pub elapsed_wall_seconds: f64,
    /// Statistical summary for each metric output across all runs.
    pub metric_statistics: HashMap<String, ColumnStatistics>,
    /// Columnar record batch containing the collected results across all tasks.
    pub batch: ColumnarRecordBatch,
}

/// Multi-worker distributed sweep coordinator.
#[derive(Debug, Clone, Default)]
pub struct DistributedCoordinator {
    /// Number of worker threads (defaults to available hardware threads if None).
    pub worker_threads: Option<usize>,
}

impl DistributedCoordinator {
    /// Creates a coordinator with default thread configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the number of worker threads.
    pub fn with_worker_threads(mut self, threads: usize) -> Self {
        self.worker_threads = Some(threads);
        self
    }

    /// Generates Cartesian product tasks from a list of sweep parameters.
    pub fn generate_cartesian_tasks(parameters: &[SweepParameter]) -> Vec<SimulationTask> {
        if parameters.is_empty() {
            return vec![SimulationTask {
                task_id: 0,
                parameters: HashMap::new(),
            }];
        }

        let mut current_combinations: Vec<HashMap<String, f64>> = vec![HashMap::new()];

        for param in parameters {
            let key = format!("{}.{}", param.target, param.property);
            let mut next_combinations = Vec::new();
            for val in &param.values {
                for comb in &current_combinations {
                    let mut new_comb = comb.clone();
                    new_comb.insert(key.clone(), *val);
                    next_combinations.push(new_comb);
                }
            }
            current_combinations = next_combinations;
        }

        current_combinations
            .into_iter()
            .enumerate()
            .map(|(task_id, parameters)| SimulationTask {
                task_id,
                parameters,
            })
            .collect()
    }

    /// Executes a parallel parameter sweep using Rayon work-stealing parallelism.
    ///
    /// The user supplies a closure `sim_fn(&SimulationTask) -> Result<HashMap<String, f64>, String>`
    /// which executes a single simulation and returns output scalar metrics (e.g. "V(out)", "gain", "power").
    pub fn run_sweep<F>(&self, tasks: Vec<SimulationTask>, sim_fn: F) -> SweepResultSummary
    where
        F: Fn(&SimulationTask) -> Result<HashMap<String, f64>, String> + Sync + Send,
    {
        let total_tasks = tasks.len();
        let start_time = Instant::now();

        let success_count = AtomicUsize::new(0);
        let failure_count = AtomicUsize::new(0);

        // Execute parallel map over tasks
        let results: Vec<(usize, Option<HashMap<String, f64>>)> = tasks
            .par_iter()
            .map(|task| match sim_fn(task) {
                Ok(metrics) => {
                    success_count.fetch_add(1, Ordering::Relaxed);
                    (task.task_id, Some(metrics))
                }
                Err(_) => {
                    failure_count.fetch_add(1, Ordering::Relaxed);
                    (task.task_id, None)
                }
            })
            .collect();

        let elapsed = start_time.elapsed().as_secs_f64();

        // Assemble columnar schema from task parameters and metric names
        let mut param_keys: Vec<String> = tasks
            .first()
            .map(|t| t.parameters.keys().cloned().collect())
            .unwrap_or_default();
        param_keys.sort();

        let mut metric_keys: Vec<String> = results
            .iter()
            .find_map(|(_, m)| m.as_ref().map(|map| map.keys().cloned().collect()))
            .unwrap_or_default();
        metric_keys.sort();

        let mut all_columns = Vec::new();
        all_columns.push("task_id".to_string());
        all_columns.extend(param_keys.clone());
        all_columns.extend(metric_keys.clone());

        let mut batch = ColumnarRecordBatch::new(all_columns);

        let mut metric_values_map: HashMap<String, Vec<f64>> = HashMap::new();
        for key in &metric_keys {
            metric_values_map.insert(key.clone(), Vec::new());
        }

        for (task, (_, maybe_metric)) in tasks.iter().zip(results.iter()) {
            if let Some(metrics) = maybe_metric {
                let mut row = Vec::with_capacity(batch.column_count());
                row.push(task.task_id as f64);
                for p_key in &param_keys {
                    row.push(*task.parameters.get(p_key).unwrap_or(&0.0));
                }
                for m_key in &metric_keys {
                    let val = *metrics.get(m_key).unwrap_or(&0.0);
                    row.push(val);
                    metric_values_map.get_mut(m_key).unwrap().push(val);
                }
                let _ = batch.push_row(&row);
            }
        }

        let mut metric_statistics = HashMap::new();
        for (m_key, vals) in metric_values_map {
            metric_statistics.insert(m_key, ColumnStatistics::compute(&vals));
        }

        SweepResultSummary {
            total_tasks,
            successful_tasks: success_count.load(Ordering::Relaxed),
            failed_tasks: failure_count.load(Ordering::Relaxed),
            elapsed_wall_seconds: elapsed,
            metric_statistics,
            batch,
        }
    }
}
