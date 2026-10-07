#![deny(unsafe_code)]

//! Ergonomic floating CAD tool island anchored in the canvas viewport.

use crate::app::ToolMode;
use egui::{
    Color32, CornerRadius, FontId, Margin, Order, Pos2, Rect, RichText, Sense, Stroke, StrokeKind,
    Vec2,
};

/// Interaction action triggered from the floating CAD tools island.
#[derive(Debug, Clone, PartialEq)]
pub enum FloatingToolbarAction {
    SelectTool(ToolMode),
    Rotate,
    Mirror,
    Delete,
    Clear,
}

/// Persistent user preferences and geometry state for the floating CAD tools island.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatingToolbarState {
    pub is_visible: bool,
    pub is_collapsed: bool,
    pub custom_pos: Option<Pos2>,
}

impl Default for FloatingToolbarState {
    fn default() -> Self {
        Self {
            is_visible: true,
            is_collapsed: false,
            custom_pos: None,
        }
    }
}

impl FloatingToolbarState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Renders the floating CAD tools island over the canvas viewport in Order::Middle.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        canvas_viewport: Rect,
        active_tool: &ToolMode,
    ) -> Option<FloatingToolbarAction> {
        if !self.is_visible {
            return None;
        }

        let mut triggered_action = None;

        // Position: Anchored at TOP_LEFT of canvas viewport with 16.0 px padding if not custom
        let default_pos = canvas_viewport.min + Vec2::new(16.0, 16.0);
        let base_pos = self.custom_pos.unwrap_or(default_pos);

        // Clamp to stay inside canvas viewport
        let min_x = canvas_viewport.min.x + 8.0;
        let max_x = (canvas_viewport.max.x - 36.0).max(min_x);
        let min_y = canvas_viewport.min.y + 8.0;
        let max_y = (canvas_viewport.max.y - 36.0).max(min_y);

        let clamped_pos = Pos2::new(base_pos.x.clamp(min_x, max_x), base_pos.y.clamp(min_y, max_y));
        if self.custom_pos.is_some() {
            self.custom_pos = Some(clamped_pos);
        }

        if self.is_collapsed {
            // Collapsed 28x28 px circular badge with active tool icon
            egui::Area::new(egui::Id::new("floating_cad_island_collapsed"))
                .order(Order::Middle)
                .fixed_pos(clamped_pos)
                .show(ctx, |ui| {
                    let badge_size = Vec2::new(28.0, 28.0);
                    let (rect, response) = ui.allocate_exact_size(badge_size, Sense::click_and_drag());

                    if response.dragged() {
                        self.custom_pos = Some(clamped_pos + response.drag_delta());
                    }

                    if response.clicked() {
                        self.is_collapsed = false;
                    }

                    let painter = ui.painter();
                    let center = rect.center();
                    let radius = 13.0;

                    let (fill_color, stroke_color) = if response.hovered() {
                        (Color32::from_rgb(35, 48, 68), Color32::from_rgb(100, 160, 240))
                    } else {
                        (Color32::from_rgb(22, 28, 42), Color32::from_rgb(70, 95, 130))
                    };

                    painter.circle_filled(center, radius, fill_color);
                    painter.circle_stroke(center, radius, Stroke::new(1.5, stroke_color));

                    let tool_char = match active_tool {
                        ToolMode::Select => "V",
                        ToolMode::Wire => "W",
                        ToolMode::Bus => "B",
                        ToolMode::Probe => "P",
                        ToolMode::Place(_) | ToolMode::PlaceComponent(_) => "+",
                    };

                    let galley = painter.layout_no_wrap(
                        tool_char.to_string(),
                        FontId::proportional(12.0),
                        Color32::from_rgb(180, 220, 255),
                    );
                    painter.galley(center - galley.size() * 0.5, galley, Color32::from_rgb(180, 220, 255));

                    response.on_hover_text("CAD Tools [Click to expand, drag to relocate]");
                });
        } else {
            // Expanded pill toolbar
            egui::Area::new(egui::Id::new("floating_cad_island_expanded"))
                .order(Order::Middle)
                .fixed_pos(clamped_pos)
                .show(ctx, |ui| {
                    let frame = egui::Frame::new()
                        .fill(Color32::from_rgba_unmultiplied(20, 26, 38, 240))
                        .corner_radius(CornerRadius::same(14))
                        .stroke(Stroke::new(1.2, Color32::from_rgb(60, 80, 110)))
                        .inner_margin(Margin::symmetric(8, 4));

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

                            // 1. Ergonomic drag grip handle
                            let (grip_rect, grip_response) =
                                ui.allocate_exact_size(Vec2::new(12.0, 24.0), Sense::drag());

                            if grip_response.dragged() {
                                self.custom_pos = Some(clamped_pos + grip_response.drag_delta());
                            }

                            // Render 6-dot grip pattern
                            let grip_painter = ui.painter();
                            let dot_color = if grip_response.hovered() || grip_response.dragged() {
                                Color32::from_rgb(140, 180, 230)
                            } else {
                                Color32::from_rgb(80, 100, 130)
                            };

                            let start_x = grip_rect.min.x + 3.0;
                            let start_y = grip_rect.min.y + 4.0;
                            for col in 0..2 {
                                for row in 0..3 {
                                    let dot_pos = Pos2::new(
                                        start_x + col as f32 * 5.0,
                                        start_y + row as f32 * 7.0,
                                    );
                                    grip_painter.circle_filled(dot_pos, 1.3, dot_color);
                                }
                            }

                            grip_response.on_hover_text("Drag handle: relocate CAD island");

                            // 2. Mini chevron collapse button [<]
                            let (chev_rect, chev_response) =
                                ui.allocate_exact_size(Vec2::new(16.0, 22.0), Sense::click());

                            let chev_color = if chev_response.hovered() {
                                Color32::from_rgb(180, 210, 245)
                            } else {
                                Color32::from_rgb(110, 130, 160)
                            };

                            let chev_galley = ui.painter().layout_no_wrap(
                                "<".to_string(),
                                FontId::proportional(12.0),
                                chev_color,
                            );
                            ui.painter().galley(
                                chev_rect.center() - chev_galley.size() * 0.5,
                                chev_galley,
                                chev_color,
                            );

                            if chev_response.clicked() {
                                self.is_collapsed = true;
                            }
                            chev_response.on_hover_text("Collapse CAD island [<]");

                            // Separator
                            ui.add_space(2.0);
                            let sep_color = Color32::from_rgb(50, 65, 88);
                            let sep_top = clamped_pos.y + 6.0;
                            let sep_bottom = clamped_pos.y + 26.0;
                            ui.painter().line_segment(
                                [Pos2::new(ui.cursor().min.x, sep_top), Pos2::new(ui.cursor().min.x, sep_bottom)],
                                Stroke::new(1.0, sep_color),
                            );
                            ui.add_space(4.0);

                            // Select (V)
                            if render_tool_button(ui, "V", *active_tool == ToolMode::Select, "Select Tool [V] - Pointer & Marquee Drag Box") {
                                triggered_action = Some(FloatingToolbarAction::SelectTool(ToolMode::Select));
                            }

                            // Wire (W)
                            if render_tool_button(ui, "W", *active_tool == ToolMode::Wire, "Wire Tool [W] - Manhattan Electrical Routing") {
                                triggered_action = Some(FloatingToolbarAction::SelectTool(ToolMode::Wire));
                            }

                            // Bus (B)
                            if render_tool_button(ui, "B", *active_tool == ToolMode::Bus, "Bus Tool [B] - Multi-Signal Bus Routing") {
                                triggered_action = Some(FloatingToolbarAction::SelectTool(ToolMode::Bus));
                            }

                            // Probe (P)
                            if render_tool_button(ui, "P", *active_tool == ToolMode::Probe, "Probe Tool [P] - Interactive Voltage Telemetry") {
                                triggered_action = Some(FloatingToolbarAction::SelectTool(ToolMode::Probe));
                            }

                            // Separator
                            ui.add_space(2.0);
                            ui.painter().line_segment(
                                [Pos2::new(ui.cursor().min.x, sep_top), Pos2::new(ui.cursor().min.x, sep_bottom)],
                                Stroke::new(1.0, sep_color),
                            );
                            ui.add_space(4.0);

                            // Rotate (R)
                            if render_tool_button(ui, "R", false, "Rotate [R] - Rotate Selection Clockwise (90 deg)") {
                                triggered_action = Some(FloatingToolbarAction::Rotate);
                            }

                            // Mirror (M)
                            if render_tool_button(ui, "M", false, "Mirror [M] - Toggle Horizontal Mirroring (Flip Left/Right)") {
                                triggered_action = Some(FloatingToolbarAction::Mirror);
                            }

                            // Delete (Del)
                            if render_tool_button(ui, "Del", false, "Delete [Del] - Delete Selected Items Atomically") {
                                triggered_action = Some(FloatingToolbarAction::Delete);
                            }

                            // Clear
                            if render_tool_button(ui, "Clr", false, "Clear - Cancel Active Wire / Clear Selection") {
                                triggered_action = Some(FloatingToolbarAction::Clear);
                            }
                        });
                    });
                });
        }

        triggered_action
    }
}

fn render_tool_button(ui: &mut egui::Ui, label: &str, is_active: bool, tooltip: &str) -> bool {
    let (bg_color, fg_color, stroke_color) = if is_active {
        (
            Color32::from_rgb(35, 75, 125),
            Color32::from_rgb(255, 255, 255),
            Color32::from_rgb(80, 160, 255),
        )
    } else {
        (
            Color32::TRANSPARENT,
            Color32::from_rgb(180, 200, 220),
            Color32::TRANSPARENT,
        )
    };

    let btn_size = Vec2::new(26.0, 22.0);
    let (btn_rect, btn_resp) = ui.allocate_exact_size(btn_size, Sense::click());

    let current_bg = if is_active {
        bg_color
    } else if btn_resp.hovered() {
        Color32::from_rgb(32, 44, 62)
    } else {
        bg_color
    };

    ui.painter().rect_filled(btn_rect, CornerRadius::same(5), current_bg);
    if is_active {
        ui.painter().rect_stroke(
            btn_rect,
            CornerRadius::same(5),
            Stroke::new(1.0, stroke_color),
            StrokeKind::Inside,
        );
    }

    let text_galley = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::proportional(12.0),
        fg_color,
    );
    ui.painter().galley(
        btn_rect.center() - text_galley.size() * 0.5,
        text_galley,
        fg_color,
    );

    let clicked = btn_resp.clicked();
    btn_resp.on_hover_ui(|ui| {
        ui.label(RichText::new(tooltip).size(12.0));
    });

    clicked
}
