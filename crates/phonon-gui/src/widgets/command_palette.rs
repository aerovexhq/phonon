#![deny(unsafe_code)]

//! Searchable command palette modal overlay with zero-allocation fuzzy search and keyboard navigation.

use crate::actions::{ActionCategory, ActionId, ActionRegistry};
use egui::{
    Color32, CornerRadius, FontId, Key, Margin, Order, Pos2, Rect, RichText, Sense, Stroke,
    StrokeKind, TextEdit, Vec2,
};

/// Interactive state for the command palette modal overlay.
#[derive(Debug, Clone)]
pub struct CommandPalette {
    pub is_open: bool,
    pub search_query: String,
    pub selected_index: usize,
    pub request_focus: bool,
}

impl Default for CommandPalette {
    fn default() -> Self {
        Self {
            is_open: false,
            search_query: String::new(),
            selected_index: 0,
            request_focus: false,
        }
    }
}

impl CommandPalette {
    pub fn new() -> Self {
        Self::default()
    }

    /// Opens the command palette and resets query and selection state.
    pub fn open(&mut self) {
        self.is_open = true;
        self.search_query.clear();
        self.selected_index = 0;
        self.request_focus = true;
    }

    /// Closes the command palette.
    pub fn close(&mut self) {
        self.is_open = false;
        self.search_query.clear();
    }

    /// Toggles the command palette open/closed.
    pub fn toggle(&mut self) {
        if self.is_open {
            self.close();
        } else {
            self.open();
        }
    }

    /// Renders the command palette modal overlay.
    /// Returns `Some(ActionId)` if an action was selected and confirmed by the user.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        registry: &ActionRegistry,
    ) -> Option<ActionId> {
        if !self.is_open {
            return None;
        }

        let mut executed_action = None;

        let screen_rect = ctx
            .input(|i| i.viewport().inner_rect)
            .unwrap_or(Rect::from_min_size(Pos2::ZERO, Vec2::new(1920.0, 1080.0)));
        let painter = ctx.layer_painter(egui::LayerId::new(
            Order::Foreground,
            egui::Id::new("command_palette_backdrop"),
        ));
        painter.rect_filled(
            screen_rect,
            0.0,
            Color32::from_rgba_unmultiplied(10, 15, 24, 180),
        );

        let matches = registry.search(&self.search_query);
        let match_count = matches.len();

        // Keyboard navigation within the command palette
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            self.close();
            return None;
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowDown)) {
            if match_count > 0 {
                self.selected_index = (self.selected_index + 1) % match_count;
            }
        }

        if ctx.input(|i| i.key_pressed(Key::ArrowUp)) {
            if match_count > 0 {
                self.selected_index = if self.selected_index == 0 {
                    match_count - 1
                } else {
                    self.selected_index - 1
                };
            }
        }

        if ctx.input(|i| i.key_pressed(Key::Enter)) {
            if let Some(action) = matches.get(self.selected_index) {
                executed_action = Some(action.id);
                self.close();
                return executed_action;
            }
        }

        // Clamp selected index if query reduced results
        if match_count > 0 && self.selected_index >= match_count {
            self.selected_index = match_count - 1;
        }

        let modal_width = 560.0_f32.min(screen_rect.width() - 40.0);
        let modal_pos = Pos2::new(
            screen_rect.center().x - modal_width * 0.5,
            screen_rect.min.y + (screen_rect.height() * 0.15).max(40.0),
        );

        egui::Area::new(egui::Id::new("command_palette_area"))
            .order(Order::Foreground)
            .fixed_pos(modal_pos)
            .show(ctx, |ui| {
                let frame = egui::Frame::new()
                    .fill(Color32::from_rgb(20, 26, 38))
                    .corner_radius(CornerRadius::same(10))
                    .stroke(Stroke::new(1.5, Color32::from_rgb(70, 100, 140)))
                    .inner_margin(Margin::same(12));

                frame.show(ui, |ui| {
                    ui.set_width(modal_width);

                    // Search input bar
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(">")
                                .strong()
                                .size(16.0)
                                .color(Color32::from_rgb(100, 180, 255)),
                        );

                        let text_edit = TextEdit::singleline(&mut self.search_query)
                            .hint_text("Type a command or search actions...")
                            .font(FontId::proportional(15.0))
                            .desired_width(modal_width - 50.0);

                        let output = ui.add(text_edit);
                        if self.request_focus {
                            output.request_focus();
                            self.request_focus = false;
                        }
                    });

                    ui.add_space(8.0);
                    ui.separator();
                    ui.add_space(4.0);

                    // Action item list
                    let max_visible_items = 8;
                    let display_items = matches.iter().take(max_visible_items).enumerate();

                    if match_count == 0 {
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("No matching commands found")
                                .italics()
                                .color(Color32::from_rgb(140, 150, 165)),
                        );
                        ui.add_space(8.0);
                    } else {
                        for (idx, action) in display_items {
                            let is_selected = idx == self.selected_index;
                            let item_rect = ui.available_rect_before_wrap();
                            let row_height = 36.0;
                            let row_rect = Rect::from_min_size(
                                item_rect.min,
                                Vec2::new(ui.available_width(), row_height),
                            );

                            let response = ui.allocate_rect(row_rect, Sense::click());

                            let bg_color = if is_selected {
                                Color32::from_rgba_unmultiplied(40, 75, 120, 200)
                            } else if response.hovered() {
                                Color32::from_rgba_unmultiplied(30, 45, 65, 180)
                            } else {
                                Color32::TRANSPARENT
                            };

                            ui.painter().rect_filled(
                                row_rect,
                                CornerRadius::same(6),
                                bg_color,
                            );

                            if is_selected {
                                ui.painter().rect_stroke(
                                    row_rect,
                                    CornerRadius::same(6),
                                    Stroke::new(1.0, Color32::from_rgb(80, 150, 240)),
                                    StrokeKind::Inside,
                                );
                            }

                            // Layout row contents
                            let inner_rect = row_rect.shrink2(Vec2::new(8.0, 6.0));
                            let mut child_ui = ui.new_child(
                                egui::UiBuilder::new()
                                    .max_rect(inner_rect)
                                    .layout(egui::Layout::left_to_right(egui::Align::Center)),
                            );

                            // Category badge
                            let (badge_bg, badge_fg) = match action.category {
                                ActionCategory::File => (
                                    Color32::from_rgb(35, 55, 80),
                                    Color32::from_rgb(120, 180, 240),
                                ),
                                ActionCategory::Edit => (
                                    Color32::from_rgb(50, 45, 75),
                                    Color32::from_rgb(190, 150, 250),
                                ),
                                ActionCategory::View => (
                                    Color32::from_rgb(35, 65, 55),
                                    Color32::from_rgb(120, 220, 170),
                                ),
                                ActionCategory::Simulate => (
                                    Color32::from_rgb(65, 50, 30),
                                    Color32::from_rgb(250, 180, 80),
                                ),
                                ActionCategory::Tools => (
                                    Color32::from_rgb(30, 60, 70),
                                    Color32::from_rgb(100, 210, 230),
                                ),
                            };

                            let badge_text = action.category.display_name();
                            let badge_galley = child_ui.painter().layout_no_wrap(
                                badge_text.to_string(),
                                FontId::proportional(11.0),
                                badge_fg,
                            );
                            let badge_padding = Vec2::new(6.0, 3.0);
                            let badge_size = badge_galley.size() + badge_padding * 2.0;
                            let (badge_alloc, _) = child_ui.allocate_exact_size(badge_size, Sense::hover());
                            child_ui.painter().rect_filled(
                                badge_alloc,
                                CornerRadius::same(4),
                                badge_bg,
                            );
                            child_ui.painter().galley(
                                badge_alloc.min + badge_padding,
                                badge_galley,
                                badge_fg,
                            );

                            child_ui.add_space(8.0);

                            // Action Title
                            let title_color = if is_selected {
                                Color32::from_rgb(255, 255, 255)
                            } else {
                                Color32::from_rgb(215, 225, 235)
                            };
                            child_ui.label(
                                RichText::new(action.title)
                                    .strong()
                                    .size(13.0)
                                    .color(title_color),
                            );

                            // Action Description (subtle)
                            child_ui.label(
                                RichText::new(format!("- {}", action.description))
                                    .size(11.0)
                                    .color(Color32::from_rgb(130, 145, 165)),
                            );

                            // Shortcut badge aligned to right
                            if let Some(shortcut) = action.shortcut {
                                child_ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let sc_galley = ui.painter().layout_no_wrap(
                                            shortcut.to_string(),
                                            FontId::monospace(11.0),
                                            Color32::from_rgb(170, 190, 210),
                                        );
                                        let sc_padding = Vec2::new(6.0, 2.0);
                                        let sc_size = sc_galley.size() + sc_padding * 2.0;
                                        let (sc_alloc, _) = ui.allocate_exact_size(sc_size, Sense::hover());
                                        ui.painter().rect_filled(
                                            sc_alloc,
                                            CornerRadius::same(4),
                                            Color32::from_rgb(28, 36, 50),
                                        );
                                        ui.painter().rect_stroke(
                                            sc_alloc,
                                            CornerRadius::same(4),
                                            Stroke::new(1.0, Color32::from_rgb(55, 70, 95)),
                                            StrokeKind::Outside,
                                        );
                                        ui.painter().galley(
                                            sc_alloc.min + sc_padding,
                                            sc_galley,
                                            Color32::from_rgb(170, 190, 210),
                                        );
                                    },
                                );
                            }

                            if response.clicked() {
                                executed_action = Some(action.id);
                                self.close();
                            }
                        }
                    }

                    // Footer hints
                    ui.add_space(4.0);
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Up/Down to navigate")
                                .size(10.0)
                                .color(Color32::from_rgb(110, 125, 145)),
                        );
                        ui.label(
                            RichText::new("| Enter to select")
                                .size(10.0)
                                .color(Color32::from_rgb(110, 125, 145)),
                        );
                        ui.label(
                            RichText::new("| Esc to dismiss")
                                .size(10.0)
                                .color(Color32::from_rgb(110, 125, 145)),
                        );
                    });
                });
            });

        executed_action
    }
}
