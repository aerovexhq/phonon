#![deny(unsafe_code)]

//! Real-time bidirectional SPICE netlist synchronization engine.
//!
//! Synchronizes visual CAD schematic canvas topology with SPICE netlist representation,
//! featuring incremental delta reconciliation, layout coordinate preservation, and
//! 64-bit canvas hash debouncing.

use super::canvas::SchematicCanvas;
use super::circuit_compiler::compile_schematic;
use super::components::{ComponentKind, SchematicComponent};
use egui::Pos2;
use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};

/// Delta record tracking modifications applied during netlist synchronization to canvas.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SyncDelta {
    pub added_count: usize,
    pub updated_count: usize,
    pub removed_count: usize,
}

/// Errors that can occur during netlist parsing and synchronization.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NetlistSyncError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Invalid component line: {0}")]
    InvalidComponent(String),
    #[error("Unknown component type for designator: {0}")]
    UnknownComponentType(String),
}

/// Bidirectional SPICE netlist synchronization engine.
#[derive(Debug, Clone)]
pub struct NetlistSyncEngine {
    pub last_canvas_hash: u64,
    pub last_netlist_text: String,
}

impl Default for NetlistSyncEngine {
    fn default() -> Self {
        Self {
            last_canvas_hash: 0,
            last_netlist_text: String::new(),
        }
    }
}

impl NetlistSyncEngine {
    /// Creates a new netlist synchronization engine instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes a 64-bit topological hash of canvas components and wires.
    pub fn compute_canvas_hash(canvas: &SchematicCanvas) -> u64 {
        let mut hasher = DefaultHasher::new();
        canvas.components.len().hash(&mut hasher);
        for comp in &canvas.components {
            comp.id.hash(&mut hasher);
            comp.name.hash(&mut hasher);
            (comp.kind as usize).hash(&mut hasher);
            comp.pos.x.to_bits().hash(&mut hasher);
            comp.pos.y.to_bits().hash(&mut hasher);
            comp.rotation.hash(&mut hasher);
            comp.mirrored.hash(&mut hasher);
            comp.value_str.hash(&mut hasher);
            comp.properties.hash(&mut hasher);
        }
        canvas.wires.len().hash(&mut hasher);
        for wire in &canvas.wires {
            wire.id.hash(&mut hasher);
            wire.net_name.hash(&mut hasher);
            wire.segments.len().hash(&mut hasher);
            for seg in &wire.segments {
                seg.start.x.to_bits().hash(&mut hasher);
                seg.start.y.to_bits().hash(&mut hasher);
                seg.end.x.to_bits().hash(&mut hasher);
                seg.end.y.to_bits().hash(&mut hasher);
            }
        }
        hasher.finish()
    }

    /// Compiles canvas components and wires into SPICE netlist text representation.
    ///
    /// Returns reference to internal synced SPICE netlist string. Caches compilation
    /// if the canvas hash has not changed.
    pub fn sync_from_canvas(&mut self, canvas: &SchematicCanvas) -> &str {
        let hash = Self::compute_canvas_hash(canvas);
        if hash == self.last_canvas_hash && !self.last_netlist_text.is_empty() {
            return &self.last_netlist_text;
        }

        match compile_schematic(&canvas.components, &canvas.wires) {
            Ok(compiled) => {
                self.last_netlist_text = compiled.spice_netlist;
            }
            Err(_) => {
                let mut lines = vec![
                    "* Exported from Phonon CAD Schematic".to_string(),
                    ".TEMP 27.0".to_string(),
                ];
                for comp in &canvas.components {
                    if comp.kind != ComponentKind::Ground {
                        lines.push(format!("{} 0 0 {}", comp.name, comp.value_str));
                    }
                }
                lines.push(".END".to_string());
                self.last_netlist_text = lines.join("\n");
            }
        }
        self.last_canvas_hash = hash;
        &self.last_netlist_text
    }

    /// Parses netlist lines, reconciles components by designator/id, updating values
    /// or adding new components, while preserving existing component coordinates to
    /// prevent disrupting user layouts.
    pub fn sync_to_canvas(
        &mut self,
        netlist: &str,
        canvas: &mut SchematicCanvas,
    ) -> Result<SyncDelta, NetlistSyncError> {
        let mut parsed_items = Vec::new();

        for line in netlist.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('*')
                || trimmed.starts_with(';')
                || trimmed.starts_with('#')
            {
                continue;
            }
            if trimmed.starts_with('.') {
                continue;
            }

            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            if tokens.len() < 3 {
                return Err(NetlistSyncError::InvalidComponent(line.to_string()));
            }

            let designator = tokens[0].to_string();
            let first_char = designator
                .chars()
                .next()
                .unwrap_or(' ')
                .to_ascii_uppercase();

            let (kind, value) = match first_char {
                'R' => {
                    if tokens.len() < 4 {
                        return Err(NetlistSyncError::InvalidComponent(line.to_string()));
                    }
                    (ComponentKind::Resistor, tokens[3].to_string())
                }
                'C' => {
                    if tokens.len() < 4 {
                        return Err(NetlistSyncError::InvalidComponent(line.to_string()));
                    }
                    (ComponentKind::Capacitor, tokens[3].to_string())
                }
                'L' => {
                    if tokens.len() < 4 {
                        return Err(NetlistSyncError::InvalidComponent(line.to_string()));
                    }
                    (ComponentKind::Inductor, tokens[3].to_string())
                }
                'V' => {
                    if tokens.len() < 4 {
                        return Err(NetlistSyncError::InvalidComponent(line.to_string()));
                    }
                    (ComponentKind::VoltageSource, tokens[3..].join(" "))
                }
                'I' => {
                    if tokens.len() < 4 {
                        return Err(NetlistSyncError::InvalidComponent(line.to_string()));
                    }
                    (ComponentKind::CurrentSource, tokens[3..].join(" "))
                }
                'D' => {
                    if tokens.len() < 4 {
                        return Err(NetlistSyncError::InvalidComponent(line.to_string()));
                    }
                    (ComponentKind::Diode, tokens[3].to_string())
                }
                'M' => {
                    let val = tokens[tokens.len() - 1].to_string();
                    (ComponentKind::Nmos, val)
                }
                'Q' => {
                    let val = tokens[tokens.len() - 1].to_string();
                    (ComponentKind::BjtNpn, val)
                }
                'X' => {
                    let upper = designator.to_ascii_uppercase();
                    let kind = if upper.starts_with("XOP") {
                        ComponentKind::OpAmp
                    } else if upper.starts_with("XINV") {
                        ComponentKind::Inverter
                    } else if upper.starts_with("XNAND") {
                        ComponentKind::NandGate
                    } else if upper.starts_with("XNOR") {
                        ComponentKind::NorGate
                    } else if upper.starts_with("XMUX") {
                        ComponentKind::Mux2to1
                    } else {
                        ComponentKind::OpAmp
                    };
                    let val = tokens[tokens.len() - 1].to_string();
                    (kind, val)
                }
                _ => return Err(NetlistSyncError::UnknownComponentType(tokens[0].to_string())),
            };

            parsed_items.push((designator, kind, value));
        }

        let mut added_count = 0;
        let mut updated_count = 0;
        let mut matched_ids = HashSet::new();

        for (designator, kind, value) in parsed_items {
            if let Some(existing) = canvas
                .components
                .iter_mut()
                .find(|c| c.name.eq_ignore_ascii_case(&designator))
            {
                matched_ids.insert(existing.id);
                if existing.value_str != value {
                    existing.value_str = value;
                    updated_count += 1;
                }
            } else {
                let next_id = canvas.components.iter().map(|c| c.id).max().unwrap_or(0) + 1;
                let idx = canvas.components.len() as f32;
                let pos = Pos2::new(100.0 + (idx % 6.0) * 100.0, 100.0 + (idx / 6.0).floor() * 80.0);
                let mut comp = SchematicComponent::new(next_id, kind, pos, 0);
                comp.name = designator;
                comp.value_str = value;
                canvas.components.push(comp);
                matched_ids.insert(next_id);
                added_count += 1;
            }
        }

        let before_len = canvas.components.len();
        canvas.components.retain(|c| {
            c.kind == ComponentKind::Ground || matched_ids.contains(&c.id)
        });
        let removed_count = before_len - canvas.components.len();

        self.last_canvas_hash = Self::compute_canvas_hash(canvas);
        self.last_netlist_text = netlist.to_string();

        Ok(SyncDelta {
            added_count,
            updated_count,
            removed_count,
        })
    }

    /// Returns whether the internal netlist cache is up-to-date with current canvas state.
    pub fn is_up_to_date(&self, canvas: &SchematicCanvas) -> bool {
        Self::compute_canvas_hash(canvas) == self.last_canvas_hash
    }

    /// Returns the currently cached netlist text.
    pub fn cached_netlist(&self) -> &str {
        &self.last_netlist_text
    }

    /// Invalidates the hash cache forcing re-compilation on next sync.
    pub fn invalidate(&mut self) {
        self.last_canvas_hash = 0;
    }
}
