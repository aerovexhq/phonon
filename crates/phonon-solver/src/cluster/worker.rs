#![deny(unsafe_code)]

//! Simulated cluster worker daemon executing numerical simulation batches.
//!
//! Emulates hardware execution workloads, memory footprint, CPU utilization,
//! heartbeat reporting, and numerical response calculations for Monte Carlo,
//! S-parameter, and DC corner sweeps.

use std::collections::HashMap;

use super::protocol::{NodeStatus, RpcMessage, SimulationType, SweepSample, WorkerNodeInfo};

/// Simulated worker node executing assigned batches with telemetry and hardware performance profiles.
#[derive(Debug, Clone, PartialEq)]
pub struct SimulatedClusterWorker {
    /// Worker node descriptor and network information.
    pub info: WorkerNodeInfo,
    /// Live CPU utilization percentage (0.0% to 100.0%).
    pub current_cpu_pct: f64,
    /// Live physical memory utilization in megabytes.
    pub current_mem_mb: f64,
    /// Baseline idle memory consumption in megabytes.
    pub base_mem_mb: f64,
    /// Batch message currently being computed.
    pub current_batch: Option<RpcMessage>,
    /// Accumulated compute elapsed time on current batch in seconds.
    pub sim_progress_s: f64,
    /// Estimated target duration for current batch in seconds.
    pub batch_duration_s: f64,
    /// Relative compute execution speed factor (1.0 = baseline, 2.0 = 2x faster).
    pub sim_speed_multiplier: f64,
    /// Artificial failure / error rate for fault testing (0.0 = reliable, 1.0 = always fail).
    pub failure_rate: f64,
    /// Cumulative count of completed batches.
    pub total_jobs_completed: usize,
}

impl SimulatedClusterWorker {
    /// Creates a new simulated worker node with given hardware capacity.
    pub fn new(
        id: impl Into<String>,
        address: impl Into<String>,
        core_count: usize,
        memory_mb: usize,
    ) -> Self {
        let base_mem = (memory_mb as f64 * 0.05).max(128.0);
        let mut info = WorkerNodeInfo::new(id, address, core_count, memory_mb);
        info.status = NodeStatus::Idle;

        Self {
            info,
            current_cpu_pct: 2.0,
            current_mem_mb: base_mem,
            base_mem_mb: base_mem,
            current_batch: None,
            sim_progress_s: 0.0,
            batch_duration_s: 0.0,
            sim_speed_multiplier: 1.0,
            failure_rate: 0.0,
            total_jobs_completed: 0,
        }
    }

    /// Sets the compute speed multiplier of this worker.
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.sim_speed_multiplier = speed.max(0.01);
        self
    }

    /// Sets the artificial error / failure rate of this worker.
    pub fn with_failure_rate(mut self, rate: f64) -> Self {
        self.failure_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Returns true if this worker is currently idle and ready for a batch.
    pub fn is_ready(&self) -> bool {
        self.current_batch.is_none() && self.info.status != NodeStatus::Offline
    }

    /// Assigns a new simulation batch to this worker.
    pub fn receive_batch(&mut self, batch: RpcMessage) {
        if let RpcMessage::DispatchBatch { samples, .. } = &batch {
            let sample_count = samples.len();
            // Estimate duration based on sample count, core count, and speed multiplier
            let base_time_per_sample = 0.002; // 2 ms per sample base
            let parallel_scale = (self.info.core_count as f64).sqrt().max(1.0);
            self.batch_duration_s =
                (sample_count as f64 * base_time_per_sample) / (parallel_scale * self.sim_speed_multiplier);
            self.sim_progress_s = 0.0;
            self.current_batch = Some(batch);
            self.info.status = NodeStatus::Busy;

            // Update live telemetry under load
            let core_load = (self.info.core_count as f64 * 4.0).min(30.0);
            self.current_cpu_pct = (65.0 + core_load * self.sim_speed_multiplier).min(98.5);
            self.current_mem_mb = self.base_mem_mb + (sample_count as f64 * 0.8);
        }
    }

    /// Steps the worker simulation clock by `dt_s`.
    ///
    /// If the current batch completes, returns the resulting `BatchResult` message.
    pub fn step(&mut self, dt_s: f64) -> Option<RpcMessage> {
        if self.info.status == NodeStatus::Offline {
            return None;
        }

        let batch = self.current_batch.as_ref()?;
        self.sim_progress_s += dt_s * self.sim_speed_multiplier;

        if self.sim_progress_s >= self.batch_duration_s {
            // Batch completed
            let result = self.compute_batch_result(batch);
            self.current_batch = None;
            self.info.status = NodeStatus::Idle;
            self.total_jobs_completed += 1;
            self.info.jobs_completed = self.total_jobs_completed;

            // Reset idle resource usage
            self.current_cpu_pct = (1.5 + (self.info.core_count as f64 * 0.1)).min(5.0);
            self.current_mem_mb = self.base_mem_mb;
            self.sim_progress_s = 0.0;
            self.batch_duration_s = 0.0;

            Some(result)
        } else {
            None
        }
    }

    /// Synchronously executes an entire batch immediately without waiting for clock steps.
    pub fn execute_batch_immediate(&mut self, batch: RpcMessage) -> RpcMessage {
        let result = self.compute_batch_result(&batch);
        self.total_jobs_completed += 1;
        self.info.jobs_completed = self.total_jobs_completed;
        self.info.status = NodeStatus::Idle;
        result
    }

    /// Generates a heartbeat telemetry RPC message for the coordinator.
    pub fn generate_heartbeat(&self, timestamp_s: f64) -> RpcMessage {
        RpcMessage::Heartbeat {
            node_id: self.info.id.clone(),
            timestamp_s,
            cpu_usage_pct: self.current_cpu_pct,
            memory_usage_mb: self.current_mem_mb,
        }
    }

    /// Internal evaluation of simulation samples inside a dispatch batch.
    fn compute_batch_result(&self, batch: &RpcMessage) -> RpcMessage {
        if let RpcMessage::DispatchBatch {
            job_id,
            batch_id,
            samples,
            sim_type,
        } = batch
        {
            let mut sample_values = Vec::with_capacity(samples.len());
            let mut errors = 0u32;

            for sample in samples {
                // Determine whether artificial failure triggered
                if self.failure_rate > 0.0 && Self::pseudo_random_unit(sample.sample_id, 9999) < self.failure_rate {
                    errors += 1;
                    continue;
                }

                let val = Self::evaluate_sample(sample, *sim_type);
                sample_values.push(val);
            }

            let exec_ms = (self.batch_duration_s * 1000.0).max(1.0);

            RpcMessage::BatchResult {
                job_id: job_id.clone(),
                batch_id: *batch_id,
                node_id: self.info.id.clone(),
                execution_time_ms: exec_ms,
                sample_values,
                errors,
            }
        } else {
            RpcMessage::BatchResult {
                job_id: "unknown".to_string(),
                batch_id: 0,
                node_id: self.info.id.clone(),
                execution_time_ms: 0.0,
                sample_values: Vec::new(),
                errors: 1,
            }
        }
    }

    /// Evaluates synthetic physical response for an individual sample point.
    fn evaluate_sample(sample: &SweepSample, sim_type: SimulationType) -> f64 {
        match sim_type {
            SimulationType::MonteCarlo => {
                Self::evaluate_monte_carlo(&sample.parameter_overrides, sample.sample_id)
            }
            SimulationType::SParameterSweep => {
                Self::evaluate_s_parameter(&sample.parameter_overrides, sample.sample_id)
            }
            SimulationType::DcCornerSweep => {
                Self::evaluate_dc_corner(&sample.parameter_overrides, sample.sample_id)
            }
        }
    }

    fn evaluate_monte_carlo(overrides: &HashMap<String, f64>, sample_id: u64) -> f64 {
        // If R1/R2 or L/C specified in overrides, compute physical circuit transfer
        if let (Some(&r1), Some(&r2)) = (overrides.get("R1"), overrides.get("R2")) {
            let vin = overrides.get("VIN").copied().unwrap_or(5.0);
            (vin * r2) / (r1 + r2).max(1e-9)
        } else if let (Some(&l), Some(&c)) = (overrides.get("L1"), overrides.get("C1")) {
            1.0 / (2.0 * std::f64::consts::PI * (l * c).max(1e-18).sqrt())
        } else {
            // Realistic Gaussian distributed response centered at 2.50 with sigma 0.08
            let u1 = Self::pseudo_random_unit(sample_id, 101);
            let u2 = Self::pseudo_random_unit(sample_id, 202);
            let z0 = (-2.0 * u1.max(1e-7).ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
            2.50 + 0.08 * z0
        }
    }

    fn evaluate_s_parameter(overrides: &HashMap<String, f64>, sample_id: u64) -> f64 {
        let f0 = 5.0e9; // 5 GHz resonance center
        let q = 25.0; // Quality factor
        let freq = overrides.get("freq").copied().unwrap_or_else(|| {
            4.0e9 + (sample_id as f64 % 100.0) * 2.0e7
        });

        let delta_f = (freq - f0) / f0;
        let denom = 1.0 + 4.0 * q * q * delta_f * delta_f;
        let s21_linear = 1.0 / denom.sqrt();
        // Convert to dB return loss / insertion
        20.0 * s21_linear.max(1e-6).log10()
    }

    fn evaluate_dc_corner(overrides: &HashMap<String, f64>, sample_id: u64) -> f64 {
        let vdd = overrides.get("VDD").copied().unwrap_or(3.3);
        let temp_k = overrides.get("TEMP").copied().unwrap_or(300.0);
        let corner_idx = (sample_id % 3) as f64; // 0 = TT, 1 = SS, 2 = FF

        let vth_nominal = 0.70;
        let temp_drift = (temp_k - 300.0) * (-0.0012);
        let corner_drift = match corner_idx as usize {
            1 => 0.05,  // Slow-Slow
            2 => -0.05, // Fast-Fast
            _ => 0.0,   // Typical-Typical
        };

        (vdd * 0.5) + vth_nominal + temp_drift + corner_drift
    }

    /// Deterministic pseudo-random number in range (0.0, 1.0) based on seed and salt.
    fn pseudo_random_unit(seed: u64, salt: u64) -> f64 {
        let mut x = seed.wrapping_add(salt).wrapping_mul(0x9E3779B97F4A7C15);
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D049BB133111EB);
        let bits = x ^ (x >> 31);
        ((bits >> 11) as f64) / ((1u64 << 53) as f64)
    }
}
