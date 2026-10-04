#![deny(unsafe_code)]

//! Logisim-style dual-tab left sidebar: Project Hierarchy & Assets Tree and Categorized Component Library.

use crate::schematic::categories::ComponentCategory;
use crate::schematic::components::{ComponentKind, SchematicComponent};
use crate::schematic::subcircuit_package::SubcircuitPackage;
use crate::schematic::wire::SchematicWire;
use egui::{vec2, Color32, FontId, Painter, Rect, RichText, Sense, Stroke, Ui};
use std::collections::HashMap;

/// Active tab within the left sidebar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SidebarTab {
    Hierarchy,
    #[default]
    Library,
}

impl SidebarTab {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Hierarchy => "Hierarchy & Assets",
            Self::Library => "Component Library",
        }
    }
}

/// User interaction outcome emitted by the sidebar drawer.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteAction {
    /// User selected a component kind from the library to place.
    SelectKind(ComponentKind),
    /// User selected a packaged subcircuit from the library to instantiate.
    SelectSubcircuit(String),
    /// User clicked a component in the hierarchy tree to select it.
    SelectComponent(usize),
    /// User double-clicked a component in the hierarchy tree to focus and center canvas on it.
    FocusComponent(usize),
    /// User clicked a net in the hierarchy tree to select all associated wires.
    SelectNet(String),
    /// User requested opening the subcircuit packaging dialog.
    OpenPackageDialog,
}

/// Dual-tab sidebar widget managing project hierarchy navigation and component instantiation.
#[derive(Debug, Clone)]
pub struct ComponentPalette {
    pub active_tab: SidebarTab,
    pub search_query: String,
    pub open_categories: HashMap<ComponentCategory, bool>,
    pub open_hierarchy_branches: HashMap<String, bool>,
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
            open_categories.insert(cat, false);
        }
        let mut open_hierarchy_branches = HashMap::new();
        open_hierarchy_branches.insert("sheets".to_string(), true);
        open_hierarchy_branches.insert("subcircuits".to_string(), true);
        open_hierarchy_branches.insert("components".to_string(), true);
        open_hierarchy_branches.insert("nets".to_string(), true);

        Self {
            active_tab: SidebarTab::Library,
            search_query: String::new(),
            open_categories,
            open_hierarchy_branches,
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

    /// Renders a crisp vector magnifying glass icon with the given painter.
    fn draw_vector_magnifying_glass(painter: &Painter, rect: Rect, stroke_color: Color32) {
        let center = rect.center() + vec2(-2.0, -2.0);
        let radius = 4.5;
        let stroke = Stroke::new(1.4, stroke_color);

        // Lens circle
        painter.circle_stroke(center, radius, stroke);

        // Handle line extending down-right
        let handle_start = center + vec2(radius * 0.707, radius * 0.707);
        let handle_end = handle_start + vec2(4.5, 4.5);
        painter.line_segment([handle_start, handle_end], stroke);
    }

    /// Main render entry point for the dual-tab sidebar.
    pub fn render(
        &mut self,
        ui: &mut Ui,
        project_title: &str,
        components: &[SchematicComponent],
        wires: &[SchematicWire],
        subcircuits: &[&SubcircuitPackage],
        dc_node_voltages: &HashMap<String, f64>,
        current_selection: Option<&ComponentKind>,
        selected_comp_id: Option<usize>,
    ) -> Option<PaletteAction> {
        // 1. Dual-Tab Switcher Header
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            let h_btn = ui.selectable_label(
                self.active_tab == SidebarTab::Hierarchy,
                RichText::new("Hierarchy").strong().size(11.5),
            );
            if h_btn.clicked() {
                self.active_tab = SidebarTab::Hierarchy;
            }

            let l_btn = ui.selectable_label(
                self.active_tab == SidebarTab::Library,
                RichText::new("Library").strong().size(11.5),
            );
            if l_btn.clicked() {
                self.active_tab = SidebarTab::Library;
            }
        });

        ui.add_space(6.0);
        ui.separator();

        match self.active_tab {
            SidebarTab::Hierarchy => self.render_hierarchy_tab(
                ui,
                project_title,
                components,
                wires,
                subcircuits,
                dc_node_voltages,
                selected_comp_id,
            ),
            SidebarTab::Library => self.render_library_tab(
                ui,
                subcircuits,
                current_selection,
            ),
        }
    }

    /// Renders Tab 1: Logisim-style Project Hierarchy & Assets Tree.
    fn render_hierarchy_tab(
        &mut self,
        ui: &mut Ui,
        project_title: &str,
        components: &[SchematicComponent],
        wires: &[SchematicWire],
        subcircuits: &[&SubcircuitPackage],
        dc_node_voltages: &HashMap<String, f64>,
        selected_comp_id: Option<usize>,
    ) -> Option<PaletteAction> {
        let mut action = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // Root Project Info Badge
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("[Project] {}", project_title))
                                .strong()
                                .color(Color32::from_rgb(100, 180, 240)),
                        );
                    });
                    ui.label(
                        RichText::new(format!(
                            "{} Components | {} Wires | {} Subcircuits",
                            components.len(),
                            wires.len(),
                            subcircuits.len()
                        ))
                        .font(FontId::proportional(10.0))
                        .color(Color32::from_rgb(140, 160, 180)),
                    );
                });

                ui.add_space(4.0);

                // Branch 1: Sheets
                egui::CollapsingHeader::new(
                    RichText::new("Sheets (1)")
                        .strong()
                        .color(Color32::from_rgb(200, 220, 240)),
                )
                .default_open(true)
                .id_salt("hier_sheets")
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Sheet 1: Main Schematic")
                                .italics()
                                .color(Color32::from_rgb(180, 230, 180)),
                        );
                        ui.label(RichText::new("[Active]").font(FontId::monospace(9.0)).color(Color32::from_rgb(80, 200, 120)));
                    });
                });

                ui.add_space(2.0);

                // Branch 2: Subcircuits & Reusable Blocks
                egui::CollapsingHeader::new(
                    RichText::new(format!("Subcircuits / Packages ({})", subcircuits.len()))
                        .strong()
                        .color(Color32::from_rgb(200, 220, 240)),
                )
                .default_open(true)
                .id_salt("hier_subcircuits")
                .show(ui, |ui| {
                    if subcircuits.is_empty() {
                        ui.label(
                            RichText::new("No packaged subcircuits registered")
                                .italics()
                                .color(Color32::from_rgb(130, 140, 150)),
                        );
                    } else {
                        for pkg in subcircuits {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(&pkg.name)
                                        .strong()
                                        .color(Color32::from_rgb(160, 210, 255)),
                                );
                                ui.label(
                                    RichText::new(format!("({} ports)", pkg.ports.len()))
                                        .font(FontId::proportional(10.0))
                                        .color(Color32::from_rgb(130, 150, 170)),
                                );
                            });
                        }
                    }

                    if ui.button("+ Package Selection as .phnc").clicked() {
                        action = Some(PaletteAction::OpenPackageDialog);
                    }
                });

                ui.add_space(2.0);

                // Branch 3: Circuit Components
                egui::CollapsingHeader::new(
                    RichText::new(format!("Components ({})", components.len()))
                        .strong()
                        .color(Color32::from_rgb(200, 220, 240)),
                )
                .default_open(true)
                .id_salt("hier_components")
                .show(ui, |ui| {
                    if components.is_empty() {
                        ui.label(
                            RichText::new("No components on canvas")
                                .italics()
                                .color(Color32::from_rgb(130, 140, 150)),
                        );
                    } else {
                        // Group components by kind category
                        for comp in components {
                            let is_sel = selected_comp_id == Some(comp.id);
                            let label_text = format!("{} ({})", comp.name, comp.value_str);

                            let item_resp = ui.selectable_label(
                                is_sel,
                                RichText::new(label_text).font(FontId::proportional(11.0)),
                            );

                            if item_resp.clicked() {
                                action = Some(PaletteAction::SelectComponent(comp.id));
                            }
                            if item_resp.double_clicked() {
                                action = Some(PaletteAction::FocusComponent(comp.id));
                            }
                        }
                    }
                });

                ui.add_space(2.0);

                // Branch 4: Nets & Connectivity
                let mut net_counts: HashMap<String, usize> = HashMap::new();
                for wire in wires {
                    if let Some(net) = &wire.net_name {
                        *net_counts.entry(net.clone()).or_insert(0) += 1;
                    }
                }

                egui::CollapsingHeader::new(
                    RichText::new(format!("Nets ({})", net_counts.len().max(dc_node_voltages.len())))
                        .strong()
                        .color(Color32::from_rgb(200, 220, 240)),
                )
                .default_open(false)
                .id_salt("hier_nets")
                .show(ui, |ui| {
                    if net_counts.is_empty() && dc_node_voltages.is_empty() {
                        ui.label(
                            RichText::new("Run DC (.OP) or wire circuit to view nets")
                                .italics()
                                .color(Color32::from_rgb(130, 140, 150)),
                        );
                    } else {
                        for (net, &volts) in dc_node_voltages {
                            ui.horizontal(|ui| {
                                let net_btn = ui.selectable_label(
                                    false,
                                    RichText::new(net).strong().color(Color32::from_rgb(120, 200, 240)),
                                );
                                if net_btn.clicked() {
                                    action = Some(PaletteAction::SelectNet(net.clone()));
                                }
                                ui.label(
                                    RichText::new(format!("{:.3} V", volts))
                                        .font(FontId::monospace(10.5))
                                        .color(Color32::from_rgb(180, 240, 200)),
                                );
                            });
                        }
                    }
                });
            });

        action
    }

    /// Renders Tab 2: Categorized Component Library with Vector Search Bar.
    fn render_library_tab(
        &mut self,
        ui: &mut Ui,
        subcircuits: &[&SubcircuitPackage],
        current_selection: Option<&ComponentKind>,
    ) -> Option<PaletteAction> {
        let mut action = None;

        // Modern Vector Search Bar (strictly zero unicode emojis)
        ui.horizontal(|ui| {
            // Allocate 18x18 space for vector magnifying glass
            let (rect, _) = ui.allocate_exact_size(vec2(18.0, 18.0), Sense::hover());
            let icon_color = Color32::from_rgb(120, 170, 220);
            Self::draw_vector_magnifying_glass(ui.painter(), rect, icon_color);

            let edit_w = (ui.available_width() - 56.0).max(60.0);
            let response = ui.add(
                egui::TextEdit::singleline(&mut self.search_query)
                    .hint_text("Filter components...")
                    .desired_width(edit_w),
            );

            if !self.search_query.is_empty() && ui.small_button("X").clicked() {
                self.search_query.clear();
            }

            ui.label(
                RichText::new("Ctrl+F")
                    .font(FontId::monospace(9.0))
                    .color(Color32::from_rgb(120, 140, 160)),
            );

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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for kind in matches {
                        let is_active = current_selection == Some(&kind);
                        ui.group(|ui| {
                            let btn = ui.selectable_label(
                                is_active,
                                RichText::new(kind.display_name()).strong(),
                            );
                            if btn.clicked() {
                                action = Some(PaletteAction::SelectKind(kind));
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
                });
            }
        } else {
            // Hierarchical Collapsible Library View
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    // 1. Packaged Subcircuits category
                    if !subcircuits.is_empty() {
                        egui::CollapsingHeader::new(
                            RichText::new(format!("Packaged Subcircuits ({})", subcircuits.len()))
                                .font(FontId::proportional(12.5))
                                .strong()
                                .color(Color32::from_rgb(220, 190, 100)),
                        )
                        .default_open(true)
                        .id_salt("lib_subcircuits")
                        .show(ui, |ui| {
                            for pkg in subcircuits {
                                ui.group(|ui| {
                                    if ui.button(RichText::new(&pkg.name).strong()).clicked() {
                                        action = Some(PaletteAction::SelectSubcircuit(pkg.name.clone()));
                                    }
                                    ui.label(
                                        RichText::new(&pkg.description)
                                            .font(FontId::proportional(9.5))
                                            .color(Color32::from_rgb(160, 170, 180)),
                                    );
                                });
                                ui.add_space(2.0);
                            }
                        });
                        ui.add_space(4.0);
                    }

                    // 2. Standard categories
                    for &cat in ComponentCategory::all_categories() {
                        let default_open = *self.open_categories.get(&cat).unwrap_or(&false);
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
                                    action = Some(PaletteAction::SelectKind(kind));
                                }
                            }
                            ui.add_space(4.0);
                        });
                    }
                });
        }

        action
    }
}
