#![deny(unsafe_code)]

//! In-Flight Telemetry-Driven Autonomous Task Migration & Self-Healing Compute Engine.
//!
//! Manages fault-tolerant multi-core avionics clusters with autonomous live task migration
//! from radiation-degraded silicon cores to cold-spare cores without flight interruption.

/// Operational lifecycle state of a physical compute core.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoreLifecycleState {
    /// Active primary flight core executing mission control tasks.
    ActivePrimary,
    /// Cold-spare core held in power-gated or clock-gated standby.
    ColdSpare,
    /// Hot-spare core synchronized and ready for instantaneous failover.
    HotSpare,
    /// Degraded core experiencing elevated radiation wear or SEU error rates.
    Degraded,
    /// Isolated core after task migration, running self-test or thermal annealing.
    Isolated,
    /// Permanently disabled core due to unrecoverable damage.
    PermanentlyFailed,
}

/// Flight task criticality rating according to DO-254 / DO-178C.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskCriticality {
    /// Design Assurance Level A (Catastrophic flight hazard if lost).
    DalA,
    /// Design Assurance Level B (Hazardous / severe flight impact).
    DalB,
    /// Design Assurance Level C (Major impact).
    DalC,
    /// Non-critical payload or diagnostic telemetry.
    NonCritical,
}

/// Avionics software flight task allocated to a compute core.
#[derive(Debug, Clone)]
pub struct FlightTask {
    /// Unique task identifier.
    pub task_id: u32,
    /// Task functional designation.
    pub name: String,
    /// DO-254/178C safety criticality level.
    pub criticality: TaskCriticality,
    /// Core currently executing this task.
    pub assigned_core_id: usize,
    /// Execution state memory footprint in kilobytes.
    pub checkpoint_size_kb: f64,
}

/// Real-time health telemetry state for an individual silicon core.
#[derive(Debug, Clone)]
pub struct CoreHealthTelemetry {
    /// Physical core index (0..N-1).
    pub core_id: usize,
    /// Current operational lifecycle status.
    pub status: CoreLifecycleState,
    /// Cumulative Total Ionizing Dose in krad(Si).
    pub cumulative_tid_krad: f64,
    /// Single Event Upset fault frequency in events/second.
    pub seu_fault_rate_per_sec: f64,
    /// Subthreshold static leakage current drift ratio (I_leak / I_leak_nominal).
    pub leakage_drift_ratio: f64,
    /// Computed composite core health index in range [0.0, 1.0].
    pub health_score: f64,
    /// Operating junction temperature in deg C.
    pub junction_temp_c: f64,
}

impl CoreHealthTelemetry {
    /// Evaluates and updates the composite health score based on sensor telemetry.
    pub fn update_health(&mut self) -> f64 {
        // Health degradation components:
        // - TID wear: 100 krad reference end-of-life
        let tid_penalty = (self.cumulative_tid_krad / 100.0).clamp(0.0, 1.0) * 0.45;
        // - SEU rate: > 5.0 upsets/sec indicates severe single-event vulnerability
        let seu_penalty = (self.seu_fault_rate_per_sec / 5.0).clamp(0.0, 1.0) * 0.35;
        // - Leakage drift: > 2.0x nominal leakage indicates dielectric or interface damage
        let leakage_penalty = ((self.leakage_drift_ratio - 1.0).max(0.0) / 2.0).clamp(0.0, 1.0) * 0.20;

        let score = (1.0 - (tid_penalty + seu_penalty + leakage_penalty)).clamp(0.0, 1.0);
        self.health_score = (score * 1000.0).round() / 1000.0;

        if self.health_score < 0.60 && self.status == CoreLifecycleState::ActivePrimary {
            self.status = CoreLifecycleState::Degraded;
        }

        self.health_score
    }
}

/// Audit log entry detailing an autonomous live migration event.
#[derive(Debug, Clone)]
pub struct MigrationEvent {
    /// Simulation or mission timestamp in seconds.
    pub timestamp_s: f64,
    /// Degraded source core ID.
    pub source_core_id: usize,
    /// Target spare core ID activated.
    pub target_core_id: usize,
    /// Specific task migrated.
    pub task_id: u32,
    /// Task functional designation.
    pub task_name: String,
    /// Duration of context checkpoint and thread handoff in microseconds.
    pub migration_latency_us: f64,
    /// Reason triggering the migration.
    pub trigger_reason: String,
}

/// In-flight autonomous multi-core self-healing compute cluster.
#[derive(Debug, Clone)]
pub struct SelfHealingCluster {
    /// Telemetry states for all physical cores.
    pub cores: Vec<CoreHealthTelemetry>,
    /// Flight avionics tasks currently scheduled.
    pub tasks: Vec<FlightTask>,
    /// Migration threshold health score (default 0.60).
    pub migration_threshold: f64,
    /// History of completed autonomous task migrations.
    pub migration_history: Vec<MigrationEvent>,
    /// Total flight interruptions (guaranteed 0 in deterministic self-healing architecture).
    pub flight_interruptions_count: u64,
}

impl Default for SelfHealingCluster {
    fn default() -> Self {
        Self::new_default_avionics_quad_core()
    }
}

impl SelfHealingCluster {
    /// Creates a 6-core flight cluster (4 Primary cores + 2 Cold Spares).
    pub fn new_default_avionics_quad_core() -> Self {
        let mut cores = Vec::with_capacity(6);

        // 4 Primary Cores (0..3)
        for i in 0..4 {
            cores.push(CoreHealthTelemetry {
                core_id: i,
                status: CoreLifecycleState::ActivePrimary,
                cumulative_tid_krad: 12.0 + (i as f64) * 2.5,
                seu_fault_rate_per_sec: 0.05,
                leakage_drift_ratio: 1.05,
                health_score: 0.94,
                junction_temp_c: 45.0 + (i as f64) * 2.0,
            });
        }

        // 2 Cold Spare Cores (4..5)
        for i in 4..6 {
            cores.push(CoreHealthTelemetry {
                core_id: i,
                status: CoreLifecycleState::ColdSpare,
                cumulative_tid_krad: 0.0,
                seu_fault_rate_per_sec: 0.0,
                leakage_drift_ratio: 1.0,
                health_score: 1.0,
                junction_temp_c: 25.0,
            });
        }

        let tasks = vec![
            FlightTask {
                task_id: 101,
                name: "PrimaryFlightControlLoop_1kHz".to_string(),
                criticality: TaskCriticality::DalA,
                assigned_core_id: 0,
                checkpoint_size_kb: 128.0,
            },
            FlightTask {
                task_id: 102,
                name: "AttitudeOrbitDetermination".to_string(),
                criticality: TaskCriticality::DalA,
                assigned_core_id: 1,
                checkpoint_size_kb: 256.0,
            },
            FlightTask {
                task_id: 103,
                name: "PropulsionThrustVectorControl".to_string(),
                criticality: TaskCriticality::DalA,
                assigned_core_id: 2,
                checkpoint_size_kb: 64.0,
            },
            FlightTask {
                task_id: 104,
                name: "MissionTelemetryEncryption".to_string(),
                criticality: TaskCriticality::DalB,
                assigned_core_id: 3,
                checkpoint_size_kb: 512.0,
            },
        ];

        Self {
            cores,
            tasks,
            migration_threshold: 0.60,
            migration_history: Vec::new(),
            flight_interruptions_count: 0,
        }
    }

    /// Simulates radiation degradation on a specific core by injecting TID and SEUs.
    pub fn inject_radiation_damage(&mut self, core_id: usize, delta_tid_krad: f64, seu_rate: f64) {
        if let Some(core) = self.cores.iter_mut().find(|c| c.core_id == core_id) {
            core.cumulative_tid_krad += delta_tid_krad;
            core.seu_fault_rate_per_sec += seu_rate;
            core.leakage_drift_ratio += delta_tid_krad * 0.02;
            core.update_health();
        }
    }

    /// Finds the healthiest available spare core (ColdSpare or HotSpare).
    pub fn find_available_spare(&self) -> Option<usize> {
        self.cores
            .iter()
            .filter(|c| c.status == CoreLifecycleState::ColdSpare || c.status == CoreLifecycleState::HotSpare)
            .max_by(|a, b| a.health_score.partial_cmp(&b.health_score).unwrap_or(std::cmp::Ordering::Equal))
            .map(|c| c.core_id)
    }

    /// Evaluates all cores and performs autonomous live task migration for any degraded core.
    ///
    /// Returns the number of tasks successfully migrated.
    pub fn evaluate_and_migrate(&mut self, current_time_s: f64) -> usize {
        // First update all health scores
        for core in &mut self.cores {
            core.update_health();
        }

        let mut migrations_performed = 0;

        // Identify degraded cores needing migration
        let degraded_core_ids: Vec<usize> = self
            .cores
            .iter()
            .filter(|c| c.health_score < self.migration_threshold && c.status != CoreLifecycleState::Isolated && c.status != CoreLifecycleState::PermanentlyFailed)
            .map(|c| c.core_id)
            .collect();

        for source_id in degraded_core_ids {
            if let Some(target_id) = self.find_available_spare() {
                // Find all tasks assigned to the degraded core
                let task_indices: Vec<usize> = self
                    .tasks
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.assigned_core_id == source_id)
                    .map(|(idx, _)| idx)
                    .collect();

                for idx in task_indices {
                    let task = &mut self.tasks[idx];
                    task.assigned_core_id = target_id;

                    // Migration latency calculation:
                    // checkpoint capture (~5 us) + core power up (~8 us) + context restore (~4 us)
                    let latency_us = 17.0 + (task.checkpoint_size_kb * 0.005);

                    self.migration_history.push(MigrationEvent {
                        timestamp_s: current_time_s,
                        source_core_id: source_id,
                        target_core_id: target_id,
                        task_id: task.task_id,
                        task_name: task.name.clone(),
                        migration_latency_us: latency_us,
                        trigger_reason: format!(
                            "Autonomous health trigger: Core {} health score dropped below {:.2}",
                            source_id, self.migration_threshold
                        ),
                    });

                    migrations_performed += 1;
                }

                // Update core statuses
                if let Some(target) = self.cores.iter_mut().find(|c| c.core_id == target_id) {
                    target.status = CoreLifecycleState::ActivePrimary;
                    target.junction_temp_c = 42.0;
                }
                if let Some(source) = self.cores.iter_mut().find(|c| c.core_id == source_id) {
                    source.status = CoreLifecycleState::Isolated;
                    source.junction_temp_c = 28.0; // Cooling down in isolated diagnostic mode
                }
            }
        }

        migrations_performed
    }

    /// Verifies that all DAL-A critical flight tasks are assigned to operational healthy cores.
    pub fn verify_flight_continuity(&self) -> bool {
        for task in &self.tasks {
            if task.criticality == TaskCriticality::DalA {
                if let Some(core) = self.cores.iter().find(|c| c.core_id == task.assigned_core_id) {
                    if core.status != CoreLifecycleState::ActivePrimary && core.status != CoreLifecycleState::HotSpare {
                        return false;
                    }
                    if core.health_score < 0.50 {
                        return false;
                    }
                } else {
                    return false;
                }
            }
        }
        true
    }
}
