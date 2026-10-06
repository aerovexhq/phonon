#![deny(unsafe_code)]

//! Integration tests for RHBD Layout DRC, Fast SEL Crowbar Quenching & Autonomous Self-Healing Co-Simulator.

use phonon_solver::rhbd_self_healing::{
    CoreLifecycleState, DrcViolationSeverity, ElectronicCrowbarParams, LayoutComponentKind,
    ParasiticThyristorParams, RhbdDrcRuleType, RhbdLayoutComponent, RhbdLayoutGrid,
    RhbdSelfHealingCoSimulator, SelQuenchingSimulator, SelfHealingCluster,
};
use std::time::Instant;

#[test]
fn test_rhbd_drc_guard_ring_and_substrate_resistance() {
    let mut grid = RhbdLayoutGrid::new(25.0);

    // Single tap at origin
    grid.components.push(RhbdLayoutComponent {
        id: "TAP_ORIGIN".to_string(),
        kind: LayoutComponentKind::SubstrateTapContact,
        x_um: 0.0,
        y_um: 0.0,
        width_um: 2.0,
        height_um: 2.0,
    });

    // Close cell (distance ~ 2 um -> R_sub = 25 * 2 / 2 = 25 / 1 = ~25 Ohm? With width 10 um: R = 25 * 2 / 10 = 5 Ohm)
    grid.components.push(RhbdLayoutComponent {
        id: "CLOSE_CELL".to_string(),
        kind: LayoutComponentKind::StandardLogicCell,
        x_um: 2.0,
        y_um: 0.0,
        width_um: 10.0,
        height_um: 4.0,
    });

    // Distant unshielded cell (distance = 40 um, width = 2 um -> R_sub = 25 * 40 / 2 = 500 Ohm >> 10 Ohm)
    grid.components.push(RhbdLayoutComponent {
        id: "DISTANT_CELL".to_string(),
        kind: LayoutComponentKind::StandardLogicCell,
        x_um: 40.0,
        y_um: 0.0,
        width_um: 2.0,
        height_um: 2.0,
    });

    let violations = grid.audit_layout();
    let rsub_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_type == RhbdDrcRuleType::GuardRingSubstrateResistance)
        .collect();

    assert!(!rsub_violations.is_empty(), "Should identify substrate resistance violation");
    let distant_violation = rsub_violations
        .iter()
        .find(|v| v.component_id == "DISTANT_CELL")
        .expect("Distant cell must have R_sub violation");

    assert!(distant_violation.measured_value > 10.0);
    assert_eq!(distant_violation.severity, DrcViolationSeverity::Error);
}

#[test]
fn test_rhbd_drc_dice_node_separation() {
    let mut grid = RhbdLayoutGrid::new(25.0);

    // Tap contact
    grid.components.push(RhbdLayoutComponent {
        id: "TAP".to_string(),
        kind: LayoutComponentKind::SubstrateTapContact,
        x_um: 0.0,
        y_um: 0.0,
        width_um: 2.0,
        height_um: 2.0,
    });

    // DICE pair with critical nodes (0, 2) separated by only 3.0 um (< 5.0 um threshold)
    grid.components.push(RhbdLayoutComponent {
        id: "DICE_BAD_N0".to_string(),
        kind: LayoutComponentKind::DiceStorageCell { cell_id: 1, node_index: 0 },
        x_um: 10.0,
        y_um: 10.0,
        width_um: 2.0,
        height_um: 2.0,
    });
    grid.components.push(RhbdLayoutComponent {
        id: "DICE_BAD_N2".to_string(),
        kind: LayoutComponentKind::DiceStorageCell { cell_id: 1, node_index: 2 },
        x_um: 13.0,
        y_um: 10.0,
        width_um: 2.0,
        height_um: 2.0,
    });

    let violations = grid.audit_layout();
    let dice_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_type == RhbdDrcRuleType::DiceNodeSeparation)
        .collect();

    assert_eq!(dice_violations.len(), 1, "Exactly one critical pair violation expected");
    assert!(dice_violations[0].measured_value < 5.0);
    assert_eq!(dice_violations[0].severity, DrcViolationSeverity::Error);
}

#[test]
fn test_rhbd_drc_tmr_spatial_isolation() {
    let mut grid = RhbdLayoutGrid::new(25.0);

    grid.components.push(RhbdLayoutComponent {
        id: "TAP".to_string(),
        kind: LayoutComponentKind::SubstrateTapContact,
        x_um: 0.0,
        y_um: 0.0,
        width_um: 2.0,
        height_um: 2.0,
    });

    // TMR branches of same domain separated by 6.0 um (< 10.0 um threshold)
    grid.components.push(RhbdLayoutComponent {
        id: "TMR_B0".to_string(),
        kind: LayoutComponentKind::TmrCell { domain_id: 5, branch_index: 0 },
        x_um: 20.0,
        y_um: 20.0,
        width_um: 3.0,
        height_um: 3.0,
    });
    grid.components.push(RhbdLayoutComponent {
        id: "TMR_B1".to_string(),
        kind: LayoutComponentKind::TmrCell { domain_id: 5, branch_index: 1 },
        x_um: 26.0,
        y_um: 20.0,
        width_um: 3.0,
        height_um: 3.0,
    });

    let violations = grid.audit_layout();
    let tmr_violations: Vec<_> = violations
        .iter()
        .filter(|v| v.rule_type == RhbdDrcRuleType::TmrVoterSpatialIsolation)
        .collect();

    assert_eq!(tmr_violations.len(), 1, "TMR spatial isolation violation expected");
    assert!(tmr_violations[0].measured_value < 10.0);
    assert_eq!(tmr_violations[0].severity, DrcViolationSeverity::Error);
}

#[test]
fn test_fast_sel_crowbar_quenching_burnout_prevention() {
    let thyristor = ParasiticThyristorParams::default();
    let crowbar = ElectronicCrowbarParams {
        overcurrent_threshold_a: 0.200,
        detection_delay_ns: 10.0,
        cutoff_switch_delay_ns: 25.0,
        crowbar_clamp_resistance_ohm: 0.05,
        recovery_hold_time_us: 10.0,
    };

    assert!(
        crowbar.total_quenching_latency_ns() < 50.0,
        "Total quenching latency must be strictly < 50 ns"
    );

    let sim = SelQuenchingSimulator::new(thyristor, crowbar);
    // Strike charge 2.5 pC > 1.5 pC trigger threshold
    let result = sim.simulate_transient(2.5, 250.0, 0.5);

    assert!(result.burnout_prevented, "Thermal burnout must be averted");
    assert!(
        result.quenched_peak_temp_c < 100.0,
        "Protected junction temperature must remain well below 100 C (got {:.2} C)",
        result.quenched_peak_temp_c
    );
    assert!(
        result.unquenched_peak_temp_c > 350.0,
        "Unprotected junction must suffer thermal runaway past 350 C (got {:.2} C)",
        result.unquenched_peak_temp_c
    );
    assert!(result.quenching_latency_ns <= 35.0);
    assert!(result.energy_dissipated_uj > 0.0);
}

#[test]
fn test_autonomous_self_healing_task_migration() {
    let mut cluster = SelfHealingCluster::new_default_avionics_quad_core();

    assert_eq!(cluster.cores.len(), 6);
    assert_eq!(cluster.tasks.len(), 4);
    assert!(cluster.verify_flight_continuity());

    // Inject heavy radiation wear on primary Core 0
    cluster.inject_radiation_damage(0, 75.0, 4.2);

    let core0 = cluster.cores.iter().find(|c| c.core_id == 0).unwrap();
    assert!(
        core0.health_score < cluster.migration_threshold,
        "Core 0 health score must degrade below migration threshold"
    );
    assert_eq!(core0.status, CoreLifecycleState::Degraded);

    // Run autonomous evaluation & migration
    let migrated_count = cluster.evaluate_and_migrate(42.5);
    assert!(migrated_count >= 1, "At least one flight task must be migrated");

    let core0_after = cluster.cores.iter().find(|c| c.core_id == 0).unwrap();
    assert_eq!(
        core0_after.status,
        CoreLifecycleState::Isolated,
        "Degraded core must be isolated"
    );

    let task101 = cluster.tasks.iter().find(|t| t.task_id == 101).unwrap();
    assert_ne!(
        task101.assigned_core_id, 0,
        "Task 101 must be reassigned from Core 0 to spare"
    );
    assert!(
        task101.assigned_core_id >= 4,
        "Task 101 must be migrated to a spare core (4 or 5)"
    );

    // Verify flight continuity and zero interruptions
    assert!(cluster.verify_flight_continuity());
    assert_eq!(cluster.flight_interruptions_count, 0);
    assert_eq!(cluster.migration_history.len(), migrated_count);

    let event = &cluster.migration_history[0];
    assert!(event.migration_latency_us < 25.0);
    assert_eq!(event.source_core_id, 0);
}

#[test]
fn test_rhbd_self_healing_co_simulator_new_fast() {
    let start = Instant::now();
    let mut co_sim = RhbdSelfHealingCoSimulator::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "new_fast() must instantiate within 5ms (took {:?}",
        elapsed
    );

    let report = co_sim.report();
    assert!(report.sel_burnout_prevented);
    assert!(report.flight_continuity_certified);
    assert_eq!(report.cluster_active_cores, 4);
    assert_eq!(report.cluster_spare_cores, 2);

    let updated = co_sim.recompute();
    assert!(updated.sel_quenched_peak_temp_c < 100.0);
    assert!(updated.flight_continuity_certified);
}
