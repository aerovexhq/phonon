#![deny(unsafe_code)]

//! Categorized component palette drawer widget with live multi-field search filtering for Phonon Studio.

use crate::schematic::categories::ComponentCategory;
use crate::schematic::components::ComponentKind;
use egui::{Color32, FontId, RichText, Ui};
use std::collections::HashMap;

/// Hierarchical categorized drawer palette for selecting and instantiating schematic components.
#[derive(Debug, Clone)]
pub struct ComponentPalette {
    pub search_query: String,
    pub open_categories: HashMap<ComponentCategory, bool>,
}

impl Default for ComponentPalette {
    fn default() -> Self {
        Self::new()
    }
}

impl ComponentPalette {
    pub fn new() -> Self {
        let mut open_categories = HashMap::new();
        for &cat in ComponentCategory::all_categories() {
            open_categories.insert(cat, true);
        }
        Self {
            search_query: String::new(),
            open_categories,
        }
    }

    /// Filters components across name, kind, category, and description matching query case-insensitively.
    pub fn filter_components(&self, query: &str) -> Vec<ComponentKind> {
        let q = query.trim().to_ascii_lowercase();
        if q.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        for &comp in ComponentKind::ALL {
            if comp.search_index().contains(&q) {
                results.push(comp);
            }
        }
        results
    }

    /// Renders the categorized palette and search input, returning `Some(kind)` if a component was clicked.
    pub fn render(
        &mut self,
        ui: &mut Ui,
        current_selection: Option<&ComponentKind>,
    ) -> Option<ComponentKind> {
        let mut selected_kind = None;

        // Top Search input with filter prompt (strictly zero unicode emojis)
        ui.horizontal(|ui| {
            ui.label(RichText::new("[Search]").monospace().color(Color32::from_rgb(140, 170, 200)));
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Filter components...")
                    .desired_width(ui.available_width() - 24.0),
            );
            if !self.search_query.is_empty() && ui.small_button("X").clicked() {
                self.search_query.clear();
            }
            let _ = response;
        });

        ui.add_space(6.0);

        let query = self.search_query.trim().to_string();
        if !query.is_empty() {
            // Live Search Results View
            let matches = self.filter_components(&query);
            ui.label(
                RichText::new(format!("Results ({})", matches.len()))
                    .font(FontId::proportional(11.0))
                    .color(Color32::from_rgb(160, 180, 200)),
            );
            ui.separator();

            if matches.is_empty() {
                ui.label(
                    RichText::new("No matching components")
                        .italics()
                        .color(Color32::from_rgb(130, 140, 150)),
                );
            } else {
                for kind in matches {
                    let is_active = current_selection == Some(&kind);
                    ui.group(|ui| {
                        let btn = ui.selectable_label(
                            is_active,
                            RichText::new(kind.display_name()).strong(),
                        );
                        if btn.clicked() {
                            selected_kind = Some(kind);
                        }
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!("[{}]", kind.category().display_name()))
                                    .font(FontId::proportional(10.0))
                                    .color(Color32::from_rgb(100, 180, 240)),
                            );
                            ui.label(
                                RichText::new(kind.prefix())
                                    .font(FontId::monospace(10.0))
                                    .color(Color32::from_rgb(180, 200, 160)),
                            );
                        });
                        ui.label(
                            RichText::new(kind.description())
                                .font(FontId::proportional(9.5))
                                .color(Color32::from_rgb(150, 160, 170)),
                        );
                    });
                    ui.add_space(2.0);
                }
            }
        } else {
            // Hierarchical Collapsible Drawer View
            for &cat in ComponentCategory::all_categories() {
                let default_open = *self.open_categories.get(&cat).unwrap_or(&true);
                let header = egui::CollapsingHeader::new(
                    RichText::new(cat.display_name())
                        .font(FontId::proportional(12.5))
                        .strong(),
                )
                .default_open(default_open)
                .id_salt(cat);

                header.show(ui, |ui| {
                    ui.label(
                        RichText::new(cat.description())
                            .font(FontId::proportional(9.5))
                            .italics()
                            .color(Color32::from_rgb(140, 150, 160)),
                    );
                    ui.add_space(2.0);

                    for kind in cat.components() {
                        let is_active = current_selection == Some(&kind);
                        let text = format!("{} ({})", kind.display_name(), kind.prefix());
                        if ui.selectable_label(is_active, text).clicked() {
                            selected_kind = Some(kind);
                        }
                    }
                    ui.add_space(4.0);
                });
            }
        }

        selected_kind
    }
}
