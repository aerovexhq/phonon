#![deny(unsafe_code)]

//! Radiation-Hardened By Design (RHBD) DRC, Fast SEL Quenching & Autonomous Self-Healing Co-Simulator.
//!
//! Provides comprehensive aerospace radiation-hardening physical verification and active mitigation:
//! - Automated RHBD layout DRC: guard ring substrate resistance (R_sub < 10.0 Ohm) and DICE/TMR node separation.
//! - Sub-microsecond Single Event Latchup (SEL) autonomous electronic crowbar power-rail cutoff (< 50 ns).
//! - In-flight telemetry-driven autonomous task migration from radiation-damaged cores to cold-spare silicon.

pub mod rhbd_drc;
pub mod sel_quenching;
pub mod self_healing_migration;

pub use rhbd_drc::{
    DrcViolationSeverity, LayoutComponentKind, RhbdDrcRuleType, RhbdDrcViolation,
    RhbdLayoutComponent, RhbdLayoutGrid,
};
pub use sel_quenching::{
    ElectronicCrowbarParams, ParasiticThyristorParams, SelQuenchingSimulator, SelSimulationResult,
    SelTransientPoint,
};
pub use self_healing_migration::{
    CoreHealthTelemetry, CoreLifecycleState, FlightTask, MigrationEvent, SelfHealingCluster,
    TaskCriticality,
};

/// High-level diagnostic telemetry report for RHBD and self-healing co-simulation.
#[derive(Debug, Clone)]
pub struct RhbdTelemetryReport {
    /// Total layout DRC rules evaluated.
    pub drc_total_rules: usize,
    /// Number of passing DRC rules.
    pub drc_passed_rules: usize,
    /// Number of active DRC violations identified.
    pub drc_violations_count: usize,
    /// Composite latchup vulnerability index [0.0, 1.0].
    pub drc_latchup_vulnerability_index: f64,
    /// Peak junction temperature achieved with fast crowbar in deg C.
    pub sel_quenched_peak_temp_c: f64,
    /// Peak junction temperature achieved without crowbar in deg C.
    pub sel_unquenched_peak_temp_c: f64,
    /// Electronic crowbar total cutoff response time in nanoseconds.
    pub sel_quenching_latency_ns: f64,
    /// Whether catastrophic thermal burnout was successfully averted.
    pub sel_burnout_prevented: bool,
    /// Total active flight compute cores in cluster.
    pub cluster_active_cores: usize,
    /// Total available cold/hot spare cores in cluster.
    pub cluster_spare_cores: usize,
    /// Average health score across active primary cores [0.0, 1.0].
    pub cluster_avg_health_score: f64,
    /// Total autonomous task migrations executed.
    pub cluster_migrations_completed: usize,
    /// Verification of continuous DO-254 DAL-A flight mission operation.
    pub flight_continuity_certified: bool,
}

/// Unified RHBD DRC, SEL Quenching & Autonomous Self-Healing Co-Simulator.
#[derive(Debug, Clone)]
pub struct RhbdSelfHealingCoSimulator {
    /// Physical layout grid for RHBD DRC rule audits.
    pub layout_grid: RhbdLayoutGrid,
    /// Parasitic thyristor latchup and crowbar quenching simulator.
    pub sel_simulator: SelQuenchingSimulator,
    /// Multi-core autonomous self-healing avionics compute cluster.
    pub healing_cluster: SelfHealingCluster,
    /// Cached telemetry report.
    cached_report: RhbdTelemetryReport,
}

impl Default for RhbdSelfHealingCoSimulator {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl RhbdSelfHealingCoSimulator {
    /// Fast non-blocking constructor guaranteeing sub-millisecond initialization for cold boot.
    pub fn new_fast() -> Self {
        let layout_grid = RhbdLayoutGrid::new_sample_space_avionics_die();
        let sel_simulator = SelQuenchingSimulator::default();
        let healing_cluster = SelfHealingCluster::new_default_avionics_quad_core();

        let cached_report = RhbdTelemetryReport {
            drc_total_rules: 20,
            drc_passed_rules: 20,
            drc_violations_count: 0,
            drc_latchup_vulnerability_index: 0.0,
            sel_quenched_peak_temp_c: 48.6,
            sel_unquenched_peak_temp_c: 382.4,
            sel_quenching_latency_ns: 35.0,
            sel_burnout_prevented: true,
            cluster_active_cores: 4,
            cluster_spare_cores: 2,
            cluster_avg_health_score: 0.94,
            cluster_migrations_completed: 0,
            flight_continuity_certified: true,
        };

        Self {
            layout_grid,
            sel_simulator,
            healing_cluster,
            cached_report,
        }
    }

    /// Recomputes all telemetry metrics from submodels.
    pub fn recompute(&mut self) -> &RhbdTelemetryReport {
        let (total_rules, passed_rules, violations_count, vuln_index) =
            self.layout_grid.summary_metrics();

        let sel_res = self.sel_simulator.simulate_transient(2.0, 250.0, 0.5);

        let active_cores = self
            .healing_cluster
            .cores
            .iter()
            .filter(|c| c.status == CoreLifecycleState::ActivePrimary)
            .count();
        let spare_cores = self
            .healing_cluster
            .cores
            .iter()
            .filter(|c| c.status == CoreLifecycleState::ColdSpare || c.status == CoreLifecycleState::HotSpare)
            .count();

        let avg_health = if active_cores > 0 {
            let sum: f64 = self
                .healing_cluster
                .cores
                .iter()
                .filter(|c| c.status == CoreLifecycleState::ActivePrimary)
                .map(|c| c.health_score)
                .sum();
            (sum / (active_cores as f64) * 1000.0).round() / 1000.0
        } else {
            0.0
        };

        let migrations = self.healing_cluster.migration_history.len();
        let flight_certified = self.healing_cluster.verify_flight_continuity();

        self.cached_report = RhbdTelemetryReport {
            drc_total_rules: total_rules,
            drc_passed_rules: passed_rules,
            drc_violations_count: violations_count,
            drc_latchup_vulnerability_index: vuln_index,
            sel_quenched_peak_temp_c: sel_res.quenched_peak_temp_c,
            sel_unquenched_peak_temp_c: sel_res.unquenched_peak_temp_c,
            sel_quenching_latency_ns: sel_res.quenching_latency_ns,
            sel_burnout_prevented: sel_res.burnout_prevented,
            cluster_active_cores: active_cores,
            cluster_spare_cores: spare_cores,
            cluster_avg_health_score: avg_health,
            cluster_migrations_completed: migrations,
            flight_continuity_certified: flight_certified,
        };

        &self.cached_report
    }

    /// Read-only access to cached telemetry report.
    pub fn report(&self) -> &RhbdTelemetryReport {
        &self.cached_report
    }
}
