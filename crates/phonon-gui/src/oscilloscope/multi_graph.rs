#![deny(unsafe_code)]

//! Multi-Graph Scope Manager supporting docked Primary Scope and floating secondary popups
//! with Live streaming vs Golden Reference comparison modes.

use super::trace::WaveformTrace;
use super::OscilloscopePanel;
use egui::{Color32, Context, RichText, Window};

/// Operational mode governing trace updates during simulation runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScopeRunMode {
    #[default]
    Live,
    /// Freezes the current traces as a Golden Reference baseline for regression and A/B comparisons.
    HoldReference,
}

impl ScopeRunMode {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Live => "Live Streaming",
            Self::HoldReference => "Hold Golden Reference",
        }
    }
}

/// Independent secondary floating oscilloscope window.
#[derive(Debug, Clone)]
pub struct FloatingScopeWindow {
    pub id: String,
    pub title: String,
    pub panel: OscilloscopePanel,
    pub is_open: bool,
    pub run_mode: ScopeRunMode,
    /// Frozen golden reference traces preserved during HoldReference mode.
    pub reference_traces: Vec<WaveformTrace>,
}

impl FloatingScopeWindow {
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.to_string(),
            title: title.to_string(),
            panel: OscilloscopePanel::new(),
            is_open: true,
            run_mode: ScopeRunMode::Live,
            reference_traces: Vec::new(),
        }
    }

    /// Freezes current panel traces as the golden reference.
    pub fn freeze_reference(&mut self) {
        self.reference_traces = self.panel.traces.clone();
        for trace in &mut self.reference_traces {
            trace.name = format!("[REF] {}", trace.name);
            trace.color = Color32::from_rgba_unmultiplied(
                trace.color.r() / 2 + 60,
                trace.color.g() / 2 + 60,
                trace.color.b() / 2 + 60,
                180,
            );
        }
        self.run_mode = ScopeRunMode::HoldReference;
    }

    /// Clears the golden reference and returns to live streaming mode.
    pub fn clear_reference(&mut self) {
        self.reference_traces.clear();
        self.run_mode = ScopeRunMode::Live;
    }
}

/// Central manager orchestrating primary and secondary oscilloscope graphs.
#[derive(Debug, Clone, Default)]
pub struct MultiGraphManager {
    pub primary_scope: OscilloscopePanel,
    pub floating_scopes: Vec<FloatingScopeWindow>,
    next_scope_id: usize,
}

impl MultiGraphManager {
    pub fn new() -> Self {
        Self {
            primary_scope: OscilloscopePanel::new(),
            floating_scopes: Vec::new(),
            next_scope_id: 1,
        }
    }

    /// Spawns an independent floating oscilloscope window.
    pub fn spawn_floating_scope(&mut self, title: &str) -> String {
        let id = format!("scope_{}", self.next_scope_id);
        self.next_scope_id += 1;
        let window = FloatingScopeWindow::new(&id, title);
        self.floating_scopes.push(window);
        id
    }

    /// Clones the active primary scope traces into a new floating window.
    pub fn detach_primary_to_floating(&mut self) -> String {
        let title = format!("Oscilloscope Popup {}", self.next_scope_id);
        let id = format!("scope_{}", self.next_scope_id);
        self.next_scope_id += 1;

        let mut window = FloatingScopeWindow::new(&id, &title);
        window.panel.traces = self.primary_scope.traces.clone();
        window.panel.fft_mode = self.primary_scope.fft_mode;
        self.floating_scopes.push(window);
        id
    }

    /// Routes newly computed simulation waveform traces to primary scope and active floating scopes.
    pub fn route_simulation_traces(&mut self, traces: &[WaveformTrace]) {
        // 1. Primary scope always receives live traces
        self.primary_scope.clear();
        for t in traces {
            self.primary_scope.add_trace(t.clone());
        }

        // 2. Floating scopes receive updates according to their run mode
        for scope in &mut self.floating_scopes {
            if !scope.is_open {
                continue;
            }

            match scope.run_mode {
                ScopeRunMode::Live => {
                    scope.panel.clear();
                    for t in traces {
                        scope.panel.add_trace(t.clone());
                    }
                }
                ScopeRunMode::HoldReference => {
                    // Retain reference traces and overlay new incoming live traces
                    let mut combined = scope.reference_traces.clone();
                    for t in traces {
                        let mut live = t.clone();
                        live.name = format!("[LIVE] {}", live.name);
                        combined.push(live);
                    }
                    scope.panel.traces = combined;
                }
            }
        }
    }

    /// Renders all open floating scope windows.
    pub fn render_floating_windows(&mut self, ctx: &Context) {
        for scope in &mut self.floating_scopes {
            if !scope.is_open {
                continue;
            }

            let mut open = scope.is_open;
            let window_title = format!("{} (ID: {})", scope.title, scope.id);

            Window::new(window_title)
                .open(&mut open)
                .id(egui::Id::new(&scope.id))
                .default_size([580.0, 360.0])
                .resizable(true)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(scope.run_mode.display_name()).strong());
                        ui.separator();

                        match scope.run_mode {
                            ScopeRunMode::Live => {
                                if ui.button("Freeze as Reference").clicked() {
                                    scope.freeze_reference();
                                }
                            }
                            ScopeRunMode::HoldReference => {
                                if ui.button("Clear Reference (Return to Live)").clicked() {
                                    scope.clear_reference();
                                }
                            }
                        }

                        if ui.button("Reset View / Auto-Fit").clicked() {
                            scope.panel.request_auto_fit();
                        }
                    });

                    ui.separator();
                    scope.panel.show(ui);
                });

            scope.is_open = open;
        }

        // Retain only windows or keep in registry
        self.floating_scopes.retain(|s| s.is_open);
    }
}
