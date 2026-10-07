#![deny(unsafe_code)]

//! Centralized CAD action registry, category management, and zero-allocation fuzzy search.

/// Categorical domain of a user-facing CAD action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionCategory {
    File,
    Edit,
    View,
    Simulate,
    Tools,
}

impl ActionCategory {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::File => "File",
            Self::Edit => "Edit",
            Self::View => "View",
            Self::Simulate => "Simulate",
            Self::Tools => "Tools",
        }
    }
}

/// Unique identifiers for every executable CAD action across Phonon Studio.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ActionId {
    NewProject,
    OpenProject,
    SaveProject,
    SaveProjectAs,
    ClearCanvas,
    CloseApp,
    ExportNetlist,
    Undo,
    Redo,
    Delete,
    Duplicate,
    SelectAll,
    ClearSelection,
    RotateClockwise,
    MirrorComponent,
    ToggleFloatingToolbar,
    ToggleGrid,
    ZoomFit,
    TogglePalette,
    ToggleOscilloscope,
    ToggleThermal,
    ToggleErcOverlay,
    RunSimulation,
    RunErc,
    ToolSelect,
    ToolWire,
    ToolBus,
    ToolProbe,
    ClearWire,
    OpenCommandPalette,
    OpenPreferences,
}

/// Metadata definition of a single CAD studio action.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionDef {
    pub id: ActionId,
    pub title: &'static str,
    pub description: &'static str,
    pub category: ActionCategory,
    pub shortcut: Option<&'static str>,
    pub search_key: String,
}

/// Zero-allocation fuzzy matching between a pre-lowercased query and a pre-lowercased target.
/// Returns None if query is not a subsequence of target, or Some(score) based on match quality.
pub fn fuzzy_match_score(query: &str, target: &str) -> Option<i32> {
    if query.is_empty() {
        return Some(0);
    }

    let mut score = 0;
    let mut consecutive = 0;
    let mut prev_idx = None;

    let mut target_chars = target.char_indices();
    for qc in query.chars() {
        let mut matched = false;
        while let Some((idx, tc)) = target_chars.next() {
            if qc == tc {
                matched = true;
                score += 10;
                if let Some(prev) = prev_idx {
                    if idx == prev + 1 {
                        consecutive += 1;
                        score += consecutive * 15;
                    } else {
                        consecutive = 0;
                    }
                }
                // Bonus if match is at the start of a word
                if idx == 0 || target.as_bytes().get(idx - 1) == Some(&b' ') {
                    score += 25;
                }
                prev_idx = Some(idx);
                break;
            }
        }
        if !matched {
            return None;
        }
    }

    Some(score)
}

/// Central registry maintaining all registered CAD commands and their shortcuts.
#[derive(Debug, Clone)]
pub struct ActionRegistry {
    actions: Vec<ActionDef>,
}

impl Default for ActionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ActionRegistry {
    /// Creates and populates the default action registry with standard CAD tools and actions.
    pub fn new() -> Self {
        let mut registry = Self {
            actions: Vec::with_capacity(32),
        };

        // File
        registry.register(
            ActionId::NewProject,
            "New Project",
            "Clear current schematic canvas and start fresh",
            ActionCategory::File,
            Some("Ctrl+N"),
        );
        registry.register(
            ActionId::OpenProject,
            "Open Project...",
            "Browse and open a saved schematic project from storage",
            ActionCategory::File,
            Some("Ctrl+O"),
        );
        registry.register(
            ActionId::SaveProject,
            "Save Project",
            "Save current schematic project modifications",
            ActionCategory::File,
            Some("Ctrl+S"),
        );
        registry.register(
            ActionId::SaveProjectAs,
            "Save Project As...",
            "Save current schematic project under a new title",
            ActionCategory::File,
            Some("Ctrl+Shift+S"),
        );
        registry.register(
            ActionId::ClearCanvas,
            "Clear Canvas...",
            "Remove all components and wires from canvas with confirmation",
            ActionCategory::Edit,
            None,
        );
        registry.register(
            ActionId::CloseApp,
            "Close Application",
            "Close Phonon Studio CAD workspace",
            ActionCategory::File,
            Some("Ctrl+Q"),
        );
        registry.register(
            ActionId::ExportNetlist,
            "Export SPICE Netlist",
            "View and export SPICE 3f5 netlist syntax",
            ActionCategory::File,
            Some("Ctrl+E"),
        );
        registry.register(
            ActionId::OpenPreferences,
            "Preferences...",
            "Configure application preferences, theme colors, keybindings, and history depth",
            ActionCategory::File,
            Some("Ctrl+,"),
        );

        // Edit
        registry.register(
            ActionId::Undo,
            "Undo",
            "Reverse the previous canvas mutation",
            ActionCategory::Edit,
            Some("Ctrl+Z"),
        );
        registry.register(
            ActionId::Redo,
            "Redo",
            "Re-apply the undone mutation",
            ActionCategory::Edit,
            Some("Ctrl+Y"),
        );
        registry.register(
            ActionId::Delete,
            "Delete Selected",
            "Delete all selected components and wires atomically",
            ActionCategory::Edit,
            Some("Del"),
        );
        registry.register(
            ActionId::Duplicate,
            "Duplicate Selected",
            "Duplicate selected items offset by (+40, +40)",
            ActionCategory::Edit,
            Some("Ctrl+D"),
        );
        registry.register(
            ActionId::SelectAll,
            "Select All",
            "Select all components and wires on the active sheet",
            ActionCategory::Edit,
            Some("Ctrl+A"),
        );
        registry.register(
            ActionId::ClearSelection,
            "Clear Selection",
            "Deselect all components and wires",
            ActionCategory::Edit,
            Some("Esc"),
        );
        registry.register(
            ActionId::RotateClockwise,
            "Rotate Clockwise",
            "Rotate selected or active component clockwise by 90 degrees",
            ActionCategory::Edit,
            Some("R"),
        );
        registry.register(
            ActionId::MirrorComponent,
            "Mirror Component",
            "Toggle horizontal mirroring of selected or active component",
            ActionCategory::Edit,
            Some("M"),
        );

        // View
        registry.register(
            ActionId::ToggleFloatingToolbar,
            "Toggle Floating CAD Tools",
            "Show or hide the floating CAD tools island",
            ActionCategory::View,
            Some("H"),
        );
        registry.register(
            ActionId::ToggleGrid,
            "Toggle Grid",
            "Show or hide schematic grid lines and dots",
            ActionCategory::View,
            Some("G"),
        );
        registry.register(
            ActionId::ZoomFit,
            "Zoom to Fit",
            "Center and fit canvas viewport to bounding box",
            ActionCategory::View,
            Some("F"),
        );
        registry.register(
            ActionId::TogglePalette,
            "Toggle Component Palette",
            "Show or hide the component palette sidebar",
            ActionCategory::View,
            None,
        );
        registry.register(
            ActionId::ToggleOscilloscope,
            "Toggle Oscilloscope",
            "Show or hide the waveform oscilloscope dock",
            ActionCategory::View,
            None,
        );
        registry.register(
            ActionId::ToggleThermal,
            "Toggle Thermal Badges",
            "Show or hide component temperature telemetry badges",
            ActionCategory::View,
            None,
        );
        registry.register(
            ActionId::ToggleErcOverlay,
            "Toggle ERC Overlay",
            "Show or hide visual Electrical Rules Check diagnostics",
            ActionCategory::View,
            None,
        );
        registry.register(
            ActionId::OpenCommandPalette,
            "Command Palette",
            "Open searchable CAD action launcher",
            ActionCategory::View,
            Some("Ctrl+K"),
        );

        // Simulate
        registry.register(
            ActionId::RunSimulation,
            "Run DC Operating Point",
            "Execute non-linear Newton-Raphson DC solve",
            ActionCategory::Simulate,
            Some("F5"),
        );
        registry.register(
            ActionId::RunErc,
            "Run Electrical Rules Check",
            "Evaluate topology for floating nodes, short circuits and contention",
            ActionCategory::Simulate,
            Some("F7"),
        );

        // Tools
        registry.register(
            ActionId::ToolSelect,
            "Select Tool",
            "Interactive selection and marquee bounding box tool",
            ActionCategory::Tools,
            Some("V"),
        );
        registry.register(
            ActionId::ToolWire,
            "Wire Tool",
            "Orthogonal Manhattan electrical routing wire tool",
            ActionCategory::Tools,
            Some("W"),
        );
        registry.register(
            ActionId::ToolBus,
            "Bus Tool",
            "High-density vectorized multi-signal schematic bus tool",
            ActionCategory::Tools,
            Some("B"),
        );
        registry.register(
            ActionId::ToolProbe,
            "Voltage Probe Tool",
            "Interactive pin voltage telemetry inspector",
            ActionCategory::Tools,
            Some("P"),
        );
        registry.register(
            ActionId::ClearWire,
            "Clear Active Wire",
            "Cancel active wire routing operation",
            ActionCategory::Tools,
            None,
        );

        registry
    }

    /// Registers a new action definition.
    pub fn register(
        &mut self,
        id: ActionId,
        title: &'static str,
        description: &'static str,
        category: ActionCategory,
        shortcut: Option<&'static str>,
    ) {
        let search_key = format!(
            "{} {} {} {}",
            title.to_lowercase(),
            description.to_lowercase(),
            category.display_name().to_lowercase(),
            shortcut.unwrap_or("").to_lowercase()
        );

        self.actions.push(ActionDef {
            id,
            title,
            description,
            category,
            shortcut,
            search_key,
        });
    }

    /// Returns a slice of all registered actions.
    pub fn actions(&self) -> &[ActionDef] {
        &self.actions
    }

    /// Returns a slice of all registered action definitions.
    pub fn all(&self) -> &[ActionDef] {
        &self.actions
    }

    /// Look up an action definition by its identifier.
    pub fn get(&self, id: ActionId) -> Option<&ActionDef> {
        self.actions.iter().find(|a| a.id == id)
    }

    /// Performs fuzzy search query matching against all registered actions.
    /// Returns matching action definitions sorted in descending relevance order.
    pub fn search(&self, query: &str) -> Vec<&ActionDef> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return self.actions.iter().collect();
        }

        let mut scored: Vec<(&ActionDef, i32)> = Vec::with_capacity(self.actions.len());
        for action in &self.actions {
            if let Some(score) = fuzzy_match_score(&q, &action.search_key) {
                scored.push((action, score));
            }
        }

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().map(|(a, _)| a).collect()
    }
}
