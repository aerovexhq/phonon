#![deny(unsafe_code)]

//! Generalized Central Viewport Tab System.
//!
//! Provides a flexible, extensible multi-view tab manager for the middle viewport,
//! allowing instant switching between the 2D Schematic CAD Canvas, the 3D Synthesized
//! Physical Card Viewport, and custom registered visualization modules.

use egui::{Color32, RichText, Stroke, Ui, Vec2};
use serde::{Deserialize, Serialize};

/// Built-in and extensible tabs for the central workspace viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CentralViewportTab {
    /// Interactive 2D schematic capture canvas, wiring, and logic probing.
    Schematic,
    /// 3D synthesized physical board card, IC packages, copper traces, and abstracted ground.
    PhysicalCard3D,
    /// 2D PCB layout and copper layers viewer.
    BoardLayout2D,
}

impl CentralViewportTab {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Schematic => "Schematic Canvas",
            Self::PhysicalCard3D => "3D Physical Card",
            Self::BoardLayout2D => "2D Board Layout",
        }
    }

    pub fn icon_text(&self) -> &'static str {
        match self {
            Self::Schematic => "[~]",
            Self::PhysicalCard3D => "[3D]",
            Self::BoardLayout2D => "[PCB]",
        }
    }
}

/// Custom dynamically registered viewport tab.
#[derive(Debug, Clone)]
pub struct CustomViewportTab {
    pub id: String,
    pub title: String,
    pub icon: String,
}

/// Central Viewport Tab Manager.
#[derive(Debug, Clone)]
pub struct CentralTabManager {
    pub active_tab: CentralViewportTab,
    pub custom_tabs: Vec<CustomViewportTab>,
    pub active_custom_tab: Option<String>,
}

impl Default for CentralTabManager {
    fn default() -> Self {
        Self {
            active_tab: CentralViewportTab::Schematic,
            custom_tabs: Vec::new(),
            active_custom_tab: None,
        }
    }
}

impl CentralTabManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the active built-in viewport tab.
    pub fn set_active_tab(&mut self, tab: CentralViewportTab) {
        self.active_tab = tab;
        self.active_custom_tab = None;
    }

    /// Registers a new dynamic tab next to the central viewport.
    pub fn register_custom_tab(&mut self, id: impl Into<String>, title: impl Into<String>, icon: impl Into<String>) {
        let id_str = id.into();
        if !self.custom_tabs.iter().any(|t| t.id == id_str) {
            self.custom_tabs.push(CustomViewportTab {
                id: id_str,
                title: title.into(),
                icon: icon.into(),
            });
        }
    }

    /// Renders the central viewport tab header bar.
    pub fn render_tab_bar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

            // Built-in tabs
            let tabs = [
                CentralViewportTab::Schematic,
                CentralViewportTab::PhysicalCard3D,
            ];

            for tab in tabs {
                let is_active = self.active_custom_tab.is_none() && self.active_tab == tab;
                let (bg_color, text_color, border_stroke) = if is_active {
                    (
                        Color32::from_rgb(32, 42, 58),
                        Color32::from_rgb(100, 205, 255),
                        Stroke::new(1.0, Color32::from_rgb(60, 110, 175)),
                    )
                } else {
                    (
                        Color32::from_rgb(18, 22, 28),
                        Color32::from_rgb(150, 165, 185),
                        Stroke::NONE,
                    )
                };

                egui::Frame::new()
                    .fill(bg_color)
                    .stroke(border_stroke)
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        let text = format!("{} {}", tab.icon_text(), tab.display_name());
                        let btn = ui.add(
                            egui::Button::new(
                                RichText::new(text)
                                    .color(text_color)
                                    .size(11.5)
                                    .strong(),
                            )
                            .frame(false),
                        );
                        if btn.clicked() {
                            self.set_active_tab(tab);
                        }
                    });
            }

            // Custom registered tabs
            for custom in &self.custom_tabs {
                let is_active = self.active_custom_tab.as_deref() == Some(&custom.id);
                let (bg_color, text_color, border_stroke) = if is_active {
                    (
                        Color32::from_rgb(32, 42, 58),
                        Color32::from_rgb(100, 205, 255),
                        Stroke::new(1.0, Color32::from_rgb(60, 110, 175)),
                    )
                } else {
                    (
                        Color32::from_rgb(18, 22, 28),
                        Color32::from_rgb(150, 165, 185),
                        Stroke::NONE,
                    )
                };

                egui::Frame::new()
                    .fill(bg_color)
                    .stroke(border_stroke)
                    .corner_radius(4.0)
                    .inner_margin(egui::Margin::symmetric(10, 4))
                    .show(ui, |ui| {
                        let text = format!("{} {}", custom.icon, custom.title);
                        let btn = ui.add(
                            egui::Button::new(
                                RichText::new(text)
                                    .color(text_color)
                                    .size(11.5)
                                    .strong(),
                            )
                            .frame(false),
                        );
                        if btn.clicked() {
                            self.active_custom_tab = Some(custom.id.clone());
                        }
                    });
            }
        });
    }
}
