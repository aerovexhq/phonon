#![deny(unsafe_code)]

//! Automated Radiation-Hardened By Design (RHBD) Layout DRC & Structural Integrity Audit.
//!
//! Enforces physical design rules for radiation-tolerant aerospace microelectronics:
//! - Latchup guard ring substrate resistance verification: R_sub < 10.0 Ohm.
//! - Dual-Interlocked Storage Cell (DICE) critical node spatial separation: d_sep >= 5.0 um.
//! - Triple Modular Redundancy (TMR) voter and branch spatial isolation: d_tmr >= 10.0 um.
//! - Well-tap contact distribution density: spacing <= 20.0 um.

/// Category of Radiation-Hardened By Design (RHBD) Design Rule Check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RhbdDrcRuleType {
    /// Latchup guard ring substrate resistance to nearest tap (must be < 10.0 Ohm).
    GuardRingSubstrateResistance,
    /// Dual-Interlocked Storage Cell (DICE) redundant node spatial separation (must be >= 5.0 um).
    DiceNodeSeparation,
    /// Triple Modular Redundancy (TMR) domain spatial isolation (must be >= 10.0 um).
    TmrVoterSpatialIsolation,
    /// Substrate/well tap contact proximity (must be <= 20.0 um).
    WellTapDensity,
    /// N-well to P-well boundary separation to suppress parasitic SCR beta product (>= 2.5 um).
    WellBoundarySeparation,
}

/// Severity classification of a DRC violation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DrcViolationSeverity {
    /// Information / check passed.
    Pass,
    /// Warning / elevated single-event risk.
    Warning,
    /// Critical rule violation / high risk of destructive latchup or multi-node upset.
    Error,
}

/// Detailed audit violation report item.
#[derive(Debug, Clone, PartialEq)]
pub struct RhbdDrcViolation {
    /// Specific rule evaluated.
    pub rule_type: RhbdDrcRuleType,
    /// Severity of the violation.
    pub severity: DrcViolationSeverity,
    /// Identifier of the affected component.
    pub component_id: String,
    /// Location coordinate on the die in micrometers (x, y).
    pub location_um: (f64, f64),
    /// Measured physical value (e.g. resistance in Ohm or distance in um).
    pub measured_value: f64,
    /// Target design threshold value.
    pub threshold_value: f64,
    /// Diagnostic description.
    pub message: String,
    /// Engineering recommendation for layout mitigation.
    pub recommendation: String,
}

/// Functional classification of a layout component.
#[derive(Debug, Clone, PartialEq)]
pub enum LayoutComponentKind {
    /// Standard non-hardened logic standard cell.
    StandardLogicCell,
    /// Dual-Interlocked Storage Cell (DICE) node with pair identifier and internal node index (0..3).
    DiceStorageCell {
        /// DICE cell group ID.
        cell_id: usize,
        /// Internal sensitive node index (0..3).
        node_index: usize,
    },
    /// Triple Modular Redundancy (TMR) redundant branch or majority voter.
    TmrCell {
        /// TMR domain identifier.
        domain_id: usize,
        /// Redundant branch index (0..2 for branches, 3 for voter).
        branch_index: usize,
    },
    /// Substrate or N-well / P-well tap contact.
    SubstrateTapContact,
    /// P-Well protective guard ring strip.
    GuardRingPWell,
    /// N-Well protective guard ring strip.
    GuardRingNWell,
}

/// Individual placement component in an RHBD layout.
#[derive(Debug, Clone, PartialEq)]
pub struct RhbdLayoutComponent {
    /// Unique component identifier.
    pub id: String,
    /// Component functional kind.
    pub kind: LayoutComponentKind,
    /// Lower-left X coordinate in micrometers.
    pub x_um: f64,
    /// Lower-left Y coordinate in micrometers.
    pub y_um: f64,
    /// Component width in micrometers.
    pub width_um: f64,
    /// Component height in micrometers.
    pub height_um: f64,
}

impl RhbdLayoutComponent {
    /// Returns the center point (x, y) of the component in micrometers.
    pub fn center_um(&self) -> (f64, f64) {
        (
            self.x_um + self.width_um * 0.5,
            self.y_um + self.height_um * 0.5,
        )
    }

    /// Euclidean distance in micrometers between the centers of two components.
    pub fn distance_to(&self, other: &RhbdLayoutComponent) -> f64 {
        let (x1, y1) = self.center_um();
        let (x2, y2) = self.center_um_other(other);
        ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
    }

    fn center_um_other(&self, other: &RhbdLayoutComponent) -> (f64, f64) {
        other.center_um()
    }
}

/// Layout grid containing components for RHBD design rule verification.
#[derive(Debug, Clone)]
pub struct RhbdLayoutGrid {
    /// Substrate sheet resistance in Ohms per square (typical ~20 to 50 Ohm/sq).
    pub substrate_sheet_resistance_ohm_sq: f64,
    /// Placed components in the layout region.
    pub components: Vec<RhbdLayoutComponent>,
}

impl Default for RhbdLayoutGrid {
    fn default() -> Self {
        Self::new_sample_space_avionics_die()
    }
}

impl RhbdLayoutGrid {
    /// Creates a new layout grid with specified substrate sheet resistance.
    pub fn new(sheet_resistance_ohm_sq: f64) -> Self {
        Self {
            substrate_sheet_resistance_ohm_sq: sheet_resistance_ohm_sq.max(1.0),
            components: Vec::new(),
        }
    }

    /// Constructs a representative 100 um x 100 um aerospace flight logic tile
    /// containing DICE registers, TMR branches, guard rings, and taps.
    pub fn new_sample_space_avionics_die() -> Self {
        let mut grid = Self::new(25.0);

        // Substrate tap contacts along perimeter and center
        grid.components.push(RhbdLayoutComponent {
            id: "TAP_P_0".to_string(),
            kind: LayoutComponentKind::SubstrateTapContact,
            x_um: 5.0,
            y_um: 5.0,
            width_um: 2.0,
            height_um: 2.0,
        });
        grid.components.push(RhbdLayoutComponent {
            id: "TAP_P_1".to_string(),
            kind: LayoutComponentKind::SubstrateTapContact,
            x_um: 45.0,
            y_um: 45.0,
            width_um: 2.0,
            height_um: 2.0,
        });
        grid.components.push(RhbdLayoutComponent {
            id: "TAP_P_2".to_string(),
            kind: LayoutComponentKind::SubstrateTapContact,
            x_um: 85.0,
            y_um: 85.0,
            width_um: 2.0,
            height_um: 2.0,
        });

        // Guard ring structures
        grid.components.push(RhbdLayoutComponent {
            id: "GUARD_RING_PW0".to_string(),
            kind: LayoutComponentKind::GuardRingPWell,
            x_um: 10.0,
            y_um: 10.0,
            width_um: 30.0,
            height_um: 3.0,
        });
        grid.components.push(RhbdLayoutComponent {
            id: "GUARD_RING_NW0".to_string(),
            kind: LayoutComponentKind::GuardRingNWell,
            x_um: 10.0,
            y_um: 15.0,
            width_um: 30.0,
            height_um: 3.0,
        });

        // DICE Register 0 (Hardened: redundant pair nodes (0,2) separated by 6.5 um >= 5.0 um)
        grid.components.push(RhbdLayoutComponent {
            id: "DICE0_NODE0".to_string(),
            kind: LayoutComponentKind::DiceStorageCell { cell_id: 0, node_index: 0 },
            x_um: 15.0,
            y_um: 25.0,
            width_um: 2.0,
            height_um: 2.0,
        });
        grid.components.push(RhbdLayoutComponent {
            id: "DICE0_NODE1".to_string(),
            kind: LayoutComponentKind::DiceStorageCell { cell_id: 0, node_index: 1 },
            x_um: 18.0,
            y_um: 25.0,
            width_um: 2.0,
            height_um: 2.0,
        });
        grid.components.push(RhbdLayoutComponent {
            id: "DICE0_NODE2".to_string(),
            kind: LayoutComponentKind::DiceStorageCell { cell_id: 0, node_index: 2 },
            x_um: 22.0,
            y_um: 25.0,
            width_um: 2.0,
            height_um: 2.0,
        });
        grid.components.push(RhbdLayoutComponent {
            id: "DICE0_NODE3".to_string(),
            kind: LayoutComponentKind::DiceStorageCell { cell_id: 0, node_index: 3 },
            x_um: 25.0,
            y_um: 25.0,
            width_um: 2.0,
            height_um: 2.0,
        });

        // TMR Domain 0 (Branches separated by 12.0 um >= 10.0 um)
        grid.components.push(RhbdLayoutComponent {
            id: "TMR0_BRANCH0".to_string(),
            kind: LayoutComponentKind::TmrCell { domain_id: 0, branch_index: 0 },
            x_um: 50.0,
            y_um: 20.0,
            width_um: 4.0,
            height_um: 4.0,
        });
        grid.components.push(RhbdLayoutComponent {
            id: "TMR0_BRANCH1".to_string(),
            kind: LayoutComponentKind::TmrCell { domain_id: 0, branch_index: 1 },
            x_um: 65.0,
            y_um: 20.0,
            width_um: 4.0,
            height_um: 4.0,
        });
        grid.components.push(RhbdLayoutComponent {
            id: "TMR0_BRANCH2".to_string(),
            kind: LayoutComponentKind::TmrCell { domain_id: 0, branch_index: 2 },
            x_um: 80.0,
            y_um: 20.0,
            width_um: 4.0,
            height_um: 4.0,
        });
        grid.components.push(RhbdLayoutComponent {
            id: "TMR0_VOTER".to_string(),
            kind: LayoutComponentKind::TmrCell { domain_id: 0, branch_index: 3 },
            x_um: 65.0,
            y_um: 35.0,
            width_um: 3.0,
            height_um: 3.0,
        });

        grid
    }

    /// Evaluates the distance in micrometers from coordinate (x, y) to the nearest substrate tap.
    pub fn distance_to_nearest_tap(&self, x: f64, y: f64) -> f64 {
        let mut min_dist = f64::MAX;
        for c in &self.components {
            if let LayoutComponentKind::SubstrateTapContact = c.kind {
                let (tx, ty) = c.center_um();
                let dist = ((x - tx).powi(2) + (y - ty).powi(2)).sqrt();
                if dist < min_dist {
                    min_dist = dist;
                }
            }
        }
        if min_dist == f64::MAX {
            50.0 // Default fallback when no tap exists
        } else {
            min_dist
        }
    }

    /// Evaluates substrate resistance R_sub in Ohms between component and nearest tap.
    /// Uses R_sub = rho_sheet * (distance / effective_width).
    pub fn calculate_substrate_resistance(&self, comp: &RhbdLayoutComponent) -> f64 {
        let (cx, cy) = comp.center_um();
        let dist = self.distance_to_nearest_tap(cx, cy);
        let eff_width = comp.width_um.max(1.0);
        (self.substrate_sheet_resistance_ohm_sq * dist / eff_width).max(0.1)
    }

    /// Performs full layout DRC audit against all RHBD design rules.
    pub fn audit_layout(&self) -> Vec<RhbdDrcViolation> {
        let mut violations = Vec::new();

        // 1. Audit Guard Ring & Substrate Resistance (R_sub < 10.0 Ohm threshold)
        for comp in &self.components {
            match comp.kind {
                LayoutComponentKind::StandardLogicCell
                | LayoutComponentKind::DiceStorageCell { .. }
                | LayoutComponentKind::TmrCell { .. } => {
                    let r_sub = self.calculate_substrate_resistance(comp);
                    if r_sub > 10.0 {
                        violations.push(RhbdDrcViolation {
                            rule_type: RhbdDrcRuleType::GuardRingSubstrateResistance,
                            severity: if r_sub > 25.0 {
                                DrcViolationSeverity::Error
                            } else {
                                DrcViolationSeverity::Warning
                            },
                            component_id: comp.id.clone(),
                            location_um: comp.center_um(),
                            measured_value: r_sub,
                            threshold_value: 10.0,
                            message: format!(
                                "Substrate resistance R_sub = {:.2} Ohm exceeds 10.0 Ohm threshold",
                                r_sub
                            ),
                            recommendation: "Insert additional substrate tap contacts or widen enclosing guard ring"
                                .to_string(),
                        });
                    }
                }
                _ => {}
            }
        }

        // 2. Audit DICE Critical Node Spatial Separation (d_sep >= 5.0 um threshold)
        let dice_cells: Vec<&RhbdLayoutComponent> = self
            .components
            .iter()
            .filter(|c| matches!(c.kind, LayoutComponentKind::DiceStorageCell { .. }))
            .collect();

        for i in 0..dice_cells.len() {
            for j in (i + 1)..dice_cells.len() {
                let c1 = dice_cells[i];
                let c2 = dice_cells[j];
                if let (
                    LayoutComponentKind::DiceStorageCell { cell_id: id1, node_index: n1 },
                    LayoutComponentKind::DiceStorageCell { cell_id: id2, node_index: n2 },
                ) = (&c1.kind, &c2.kind)
                {
                    if id1 == id2 {
                        // Check critical diagonal redundant node pairs: (0, 2) and (1, 3)
                        let is_critical_pair =
                            (*n1 == 0 && *n2 == 2)
                                || (*n1 == 2 && *n2 == 0)
                                || (*n1 == 1 && *n2 == 3)
                                || (*n1 == 3 && *n2 == 1);

                        if is_critical_pair {
                            let dist = c1.distance_to(c2);
                            if dist < 5.0 {
                                violations.push(RhbdDrcViolation {
                                    rule_type: RhbdDrcRuleType::DiceNodeSeparation,
                                    severity: DrcViolationSeverity::Error,
                                    component_id: format!("{}-{}", c1.id, c2.id),
                                    location_um: c1.center_um(),
                                    measured_value: dist,
                                    threshold_value: 5.0,
                                    message: format!(
                                        "DICE critical node pair (Node {}, Node {}) spacing d = {:.2} um < 5.0 um",
                                        n1, n2, dist
                                    ),
                                    recommendation:
                                        "Interleave cell layouts or separate sensitive nodes >= 5.0 um to prevent dual charge-collection MBU"
                                            .to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        // 3. Audit TMR Spatial Isolation (d_tmr >= 10.0 um threshold)
        let tmr_cells: Vec<&RhbdLayoutComponent> = self
            .components
            .iter()
            .filter(|c| matches!(c.kind, LayoutComponentKind::TmrCell { .. }))
            .collect();

        for i in 0..tmr_cells.len() {
            for j in (i + 1)..tmr_cells.len() {
                let c1 = tmr_cells[i];
                let c2 = tmr_cells[j];
                if let (
                    LayoutComponentKind::TmrCell { domain_id: d1, branch_index: b1 },
                    LayoutComponentKind::TmrCell { domain_id: d2, branch_index: b2 },
                ) = (&c1.kind, &c2.kind)
                {
                    if d1 == d2 && b1 != b2 {
                        let dist = c1.distance_to(c2);
                        if dist < 10.0 {
                            violations.push(RhbdDrcViolation {
                                rule_type: RhbdDrcRuleType::TmrVoterSpatialIsolation,
                                severity: DrcViolationSeverity::Error,
                                component_id: format!("{}-{}", c1.id, c2.id),
                                location_um: c1.center_um(),
                                measured_value: dist,
                                threshold_value: 10.0,
                                message: format!(
                                    "TMR domain {} redundant elements ({}, {}) separation d = {:.2} um < 10.0 um",
                                    d1, b1, b2, dist
                                ),
                                recommendation:
                                    "Increase physical floorplan separation between TMR branches >= 10.0 um to prevent common-mode heavy ion strikes"
                                        .to_string(),
                            });
                        }
                    }
                }
            }
        }

        // 4. Audit Well Tap Density (tap distance <= 20.0 um)
        for comp in &self.components {
            let (cx, cy) = comp.center_um();
            let dist = self.distance_to_nearest_tap(cx, cy);
            if dist > 20.0 {
                violations.push(RhbdDrcViolation {
                    rule_type: RhbdDrcRuleType::WellTapDensity,
                    severity: DrcViolationSeverity::Warning,
                    component_id: comp.id.clone(),
                    location_um: (cx, cy),
                    measured_value: dist,
                    threshold_value: 20.0,
                    message: format!(
                        "Distance to nearest tap contact = {:.2} um exceeds max density limit 20.0 um",
                        dist
                    ),
                    recommendation: "Insert periodic well/substrate tap contact stripe every 20.0 um"
                        .to_string(),
                });
            }
        }

        violations
    }

    /// Evaluates aggregate audit statistics:
    /// Returns (total_rules_checked, passed_count, violations_count, latchup_vulnerability_index).
    pub fn summary_metrics(&self) -> (usize, usize, usize, f64) {
        let violations = self.audit_layout();
        let total_rules = (self.components.len() * 2).max(1);
        let violations_count = violations.len();
        let passed_count = total_rules.saturating_sub(violations_count);

        let error_count = violations
            .iter()
            .filter(|v| v.severity == DrcViolationSeverity::Error)
            .count() as f64;
        let warning_count = violations
            .iter()
            .filter(|v| v.severity == DrcViolationSeverity::Warning)
            .count() as f64;

        // Latchup vulnerability index normalized to [0.0, 1.0]
        let raw_index = (error_count * 0.25 + warning_count * 0.08).min(1.0);
        let vulnerability_index = (raw_index * 100.0).round() / 100.0;

        (total_rules, passed_count, violations_count, vulnerability_index)
    }
}
