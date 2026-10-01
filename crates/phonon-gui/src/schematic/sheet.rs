#![deny(unsafe_code)]

//! Tabbed multi-sheet schematic canvas management, independent viewport cameras, and cross-sheet signal routing.

use super::canvas::SchematicCanvas;
use egui::Vec2;
use std::collections::{HashMap, HashSet};

/// A single schematic sheet with its own independent infinite vector canvas and viewport camera state.
#[derive(Debug, Clone)]
pub struct SchematicSheet {
    pub id: usize,
    pub name: String,
    pub canvas: SchematicCanvas,
    pub camera_offset: Vec2,
    pub camera_zoom: f32,
}

impl SchematicSheet {
    /// Creates a new schematic sheet with default canvas and standard viewport camera.
    pub fn new(id: usize, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            canvas: SchematicCanvas::default(),
            camera_offset: Vec2::new(100.0, 100.0),
            camera_zoom: 1.0,
        }
    }
}

/// Project-wide multi-sheet canvas manager coordinating sheet tabs, active viewport, and cross-sheet nets.
#[derive(Debug, Clone)]
pub struct MultiSheetManager {
    pub sheets: Vec<SchematicSheet>,
    pub active_sheet_idx: usize,
}

impl MultiSheetManager {
    /// Initializes multi-sheet manager with a single default sheet.
    pub fn new(initial_sheet_name: &str) -> Self {
        let sheet = SchematicSheet::new(1, initial_sheet_name);
        Self {
            sheets: vec![sheet],
            active_sheet_idx: 0,
        }
    }

    /// Adds a new schematic sheet, returning the index of the newly created sheet.
    pub fn add_sheet(&mut self, name: &str) -> usize {
        let next_id = self.sheets.iter().map(|s| s.id).max().unwrap_or(0) + 1;
        let sheet = SchematicSheet::new(next_id, name);
        self.sheets.push(sheet);
        self.sheets.len() - 1
    }

    /// Removes the sheet at the specified index, preventing removal of the last remaining sheet.
    pub fn remove_sheet(&mut self, idx: usize) -> Result<(), String> {
        if self.sheets.len() <= 1 {
            return Err("Cannot remove the last remaining sheet".to_string());
        }
        if idx >= self.sheets.len() {
            return Err(format!(
                "Sheet index {} out of bounds (current sheet count: {})",
                idx,
                self.sheets.len()
            ));
        }

        self.sheets.remove(idx);

        // Adjust active sheet index if needed
        if self.active_sheet_idx >= self.sheets.len() {
            self.active_sheet_idx = self.sheets.len() - 1;
        } else if self.active_sheet_idx > idx {
            self.active_sheet_idx -= 1;
        }

        Ok(())
    }

    /// Returns an immutable reference to the currently active sheet.
    pub fn active_sheet(&self) -> &SchematicSheet {
        &self.sheets[self.active_sheet_idx]
    }

    /// Returns a mutable reference to the currently active sheet.
    pub fn active_sheet_mut(&mut self) -> &mut SchematicSheet {
        &mut self.sheets[self.active_sheet_idx]
    }

    /// Switches the active sheet to the specified index if within valid range.
    pub fn switch_sheet(&mut self, idx: usize) -> bool {
        if idx < self.sheets.len() {
            self.active_sheet_idx = idx;
            true
        } else {
            false
        }
    }

    /// Resolves all shared named nets that span across two or more sheets for project-wide global signal propagation.
    /// Returns a mapping from global net name to the list of `(sheet_id, sheet_name)` locations where it is declared.
    pub fn resolve_cross_sheet_nets(&self) -> HashMap<String, Vec<(usize, String)>> {
        let mut net_to_sheets: HashMap<String, Vec<(usize, String)>> = HashMap::new();

        for sheet in &self.sheets {
            let mut sheet_nets: HashSet<String> = HashSet::new();

            // 1. Named nets from wires
            for wire in &sheet.canvas.wires {
                if let Some(ref net) = wire.net_name {
                    let trimmed = net.trim();
                    if !trimmed.is_empty() && trimmed != "0" {
                        sheet_nets.insert(trimmed.to_string());
                    }
                }
            }

            // 2. Named nets from component properties (e.g. net, label, net_name, global_net)
            for comp in &sheet.canvas.components {
                if let Some(net) = comp.get_property("net") {
                    let trimmed = net.trim();
                    if !trimmed.is_empty() && trimmed != "0" {
                        sheet_nets.insert(trimmed.to_string());
                    }
                }
                if let Some(net) = comp.get_property("net_name") {
                    let trimmed = net.trim();
                    if !trimmed.is_empty() && trimmed != "0" {
                        sheet_nets.insert(trimmed.to_string());
                    }
                }
                if let Some(net) = comp.get_property("global_net") {
                    let trimmed = net.trim();
                    if !trimmed.is_empty() && trimmed != "0" {
                        sheet_nets.insert(trimmed.to_string());
                    }
                }
                for (key, val) in &comp.properties {
                    if (key.starts_with("net:") || key == "label") && !val.trim().is_empty() && val.trim() != "0" {
                        sheet_nets.insert(val.trim().to_string());
                    }
                }
            }

            // 3. Named nets from subcircuit instances
            for inst in &sheet.canvas.subcircuit_instances {
                for (_, ext_net) in &inst.pin_nets {
                    let trimmed = ext_net.trim();
                    if !trimmed.is_empty() && trimmed != "0" {
                        sheet_nets.insert(trimmed.to_string());
                    }
                }
            }

            // 4. Named nets from buses
            for bus in &sheet.canvas.buses {
                sheet_nets.insert(bus.signal.format_label());
                for tap in &bus.tap_offs {
                    sheet_nets.insert(tap.breakout_net.clone());
                }
            }

            for net in sheet_nets {
                net_to_sheets
                    .entry(net)
                    .or_default()
                    .push((sheet.id, sheet.name.clone()));
            }
        }

        // Only retain shared nets that appear across at least 2 distinct sheets
        net_to_sheets.retain(|_, locations| {
            let distinct_sheet_ids: HashSet<usize> = locations.iter().map(|(id, _)| *id).collect();
            distinct_sheet_ids.len() >= 2
        });

        net_to_sheets
    }
}
