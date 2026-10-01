#![deny(unsafe_code)]

//! Reversible command pattern history engine and non-destructive action stack for Phonon Studio.

use super::components::SchematicComponent;
use super::wire::SchematicWire;
use egui::Pos2;

/// A reversible canvas mutation command capturing visual CAD actions.
#[derive(Debug, Clone, PartialEq)]
pub enum CanvasCommand {
    /// Placed a new component on the canvas.
    AddComponent(SchematicComponent),
    /// Deleted an existing component from the canvas.
    DeleteComponent(SchematicComponent),
    /// Translated a component from a starting position to a target position.
    MoveComponent { id: usize, from: Pos2, to: Pos2 },
    /// Rotated a component from an initial orientation to a target orientation (0..4).
    RotateComponent { id: usize, from_rot: u8, to_rot: u8 },
    /// Modified component value string (e.g. resistance, capacitance, model name).
    ModifyComponentValue { id: usize, old_val: String, new_val: String },
    /// Routed and committed a new wire on the canvas.
    AddWire(SchematicWire),
    /// Deleted an existing wire from the canvas.
    DeleteWire(SchematicWire),
    /// Cleared all components and wires from the canvas.
    ClearAll {
        components: Vec<SchematicComponent>,
        wires: Vec<SchematicWire>,
    },
    /// A composite sequence of atomic canvas commands executed together.
    Batch(Vec<CanvasCommand>),
}

impl CanvasCommand {
    /// Executes or re-executes this command, mutating the canvas state.
    pub fn execute(
        &self,
        components: &mut Vec<SchematicComponent>,
        wires: &mut Vec<SchematicWire>,
    ) {
        match self {
            Self::AddComponent(comp) => {
                if !components.iter().any(|c| c.id == comp.id) {
                    components.push(comp.clone());
                }
            }
            Self::DeleteComponent(comp) => {
                components.retain(|c| c.id != comp.id);
            }
            Self::MoveComponent { id, to, .. } => {
                if let Some(comp) = components.iter_mut().find(|c| c.id == *id) {
                    comp.pos = *to;
                }
            }
            Self::RotateComponent { id, to_rot, .. } => {
                if let Some(comp) = components.iter_mut().find(|c| c.id == *id) {
                    comp.rotation = *to_rot;
                }
            }
            Self::ModifyComponentValue { id, new_val, .. } => {
                if let Some(comp) = components.iter_mut().find(|c| c.id == *id) {
                    comp.value_str = new_val.clone();
                }
            }
            Self::AddWire(wire) => {
                if !wires.iter().any(|w| w.id == wire.id) {
                    wires.push(wire.clone());
                }
            }
            Self::DeleteWire(wire) => {
                wires.retain(|w| w.id != wire.id);
            }
            Self::ClearAll { .. } => {
                components.clear();
                wires.clear();
            }
            Self::Batch(cmds) => {
                for cmd in cmds {
                    cmd.execute(components, wires);
                }
            }
        }
    }

    /// Reverses the effect of this command, returning the canvas to its prior state.
    pub fn undo(
        &self,
        components: &mut Vec<SchematicComponent>,
        wires: &mut Vec<SchematicWire>,
    ) {
        match self {
            Self::AddComponent(comp) => {
                components.retain(|c| c.id != comp.id);
            }
            Self::DeleteComponent(comp) => {
                if !components.iter().any(|c| c.id == comp.id) {
                    components.push(comp.clone());
                }
            }
            Self::MoveComponent { id, from, .. } => {
                if let Some(comp) = components.iter_mut().find(|c| c.id == *id) {
                    comp.pos = *from;
                }
            }
            Self::RotateComponent { id, from_rot, .. } => {
                if let Some(comp) = components.iter_mut().find(|c| c.id == *id) {
                    comp.rotation = *from_rot;
                }
            }
            Self::ModifyComponentValue { id, old_val, .. } => {
                if let Some(comp) = components.iter_mut().find(|c| c.id == *id) {
                    comp.value_str = old_val.clone();
                }
            }
            Self::AddWire(wire) => {
                wires.retain(|w| w.id != wire.id);
            }
            Self::DeleteWire(wire) => {
                if !wires.iter().any(|w| w.id == wire.id) {
                    wires.push(wire.clone());
                }
            }
            Self::ClearAll {
                components: saved_comps,
                wires: saved_wires,
            } => {
                *components = saved_comps.clone();
                *wires = saved_wires.clone();
            }
            Self::Batch(cmds) => {
                for cmd in cmds.iter().rev() {
                    cmd.undo(components, wires);
                }
            }
        }
    }
}

/// Bounded undo/redo history stack with dirty tracking.
#[derive(Debug, Clone, PartialEq)]
pub struct HistoryStack {
    pub undo_stack: Vec<CanvasCommand>,
    pub redo_stack: Vec<CanvasCommand>,
    pub max_depth: usize,
    pub clean_index: usize,
}

impl Default for HistoryStack {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryStack {
    /// Creates a new `HistoryStack` with default max depth (500).
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth: 500,
            clean_index: 0,
        }
    }

    /// Creates a new `HistoryStack` with a specified max depth limit.
    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
            clean_index: 0,
        }
    }

    /// Returns `true` if there are actions that can be undone.
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Returns `true` if there are actions that can be redone.
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Returns the number of commands currently available on the undo stack.
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Returns the number of commands currently available on the redo stack.
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Returns `true` if the current state differs from the last saved (clean) state.
    pub fn is_dirty(&self) -> bool {
        self.undo_stack.len() != self.clean_index
    }

    /// Marks the current history position as the clean / saved state.
    pub fn mark_clean(&mut self) {
        self.clean_index = self.undo_stack.len();
    }

    /// Clears both undo and redo stacks, resetting clean index to 0.
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.clean_index = 0;
    }

    /// Records a newly executed command onto the undo stack, invalidating the redo stack.
    ///
    /// If the stack depth exceeds `max_depth`, the oldest command is dropped.
    pub fn record(&mut self, cmd: CanvasCommand) {
        self.redo_stack.clear();
        if self.max_depth == 0 {
            return;
        }
        self.undo_stack.push(cmd);
        while self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
            if self.clean_index == 0 {
                self.clean_index = usize::MAX;
            } else {
                self.clean_index = self.clean_index.saturating_sub(1);
            }
        }
    }

    /// Reverses the most recent command on the undo stack and moves it to the redo stack.
    ///
    /// Returns `true` if an undo was performed, or `false` if the undo stack was empty.
    pub fn undo(
        &mut self,
        components: &mut Vec<SchematicComponent>,
        wires: &mut Vec<SchematicWire>,
    ) -> bool {
        if let Some(cmd) = self.undo_stack.pop() {
            cmd.undo(components, wires);
            self.redo_stack.push(cmd);
            true
        } else {
            false
        }
    }

    /// Re-applies the most recent undone command on the redo stack and moves it to the undo stack.
    ///
    /// Returns `true` if a redo was performed, or `false` if the redo stack was empty.
    pub fn redo(
        &mut self,
        components: &mut Vec<SchematicComponent>,
        wires: &mut Vec<SchematicWire>,
    ) -> bool {
        if let Some(cmd) = self.redo_stack.pop() {
            cmd.execute(components, wires);
            self.undo_stack.push(cmd);
            true
        } else {
            false
        }
    }
}
