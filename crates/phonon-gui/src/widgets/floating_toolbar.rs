#![deny(unsafe_code)]

//! Ergonomic floating CAD tool island anchored in the canvas viewport.

use crate::app::ToolMode;
use crate::theme::PhononTheme;
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

/// Themed visual styling tokens for the floating CAD tools island.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FloatingToolbarTheme {
    pub bg_fill: Color32,
    pub stroke_color: Color32,
    pub badge_hover_fill: Color32,
    pub dot_normal: Color32,
    pub dot_hovered: Color32,
    pub chevron_normal: Color32,
    pub chevron_hovered: Color32,
    pub separator: Color32,
    pub btn_active_bg: Color32,
    pub btn_active_fg: Color32,
    pub btn_active_stroke: Color32,
    pub btn_hover_bg: Color32,
    pub btn_hover_fg: Color32,
    pub btn_normal_fg: Color32,
}

impl FloatingToolbarTheme {
    /// Derives the floating toolbar color tokens dynamically from the active `PhononTheme`.
    pub fn from_theme(theme: &PhononTheme) -> Self {
        let is_dark = theme.is_dark();

        let bg_fill = if is_dark {
            Color32::from_rgba_unmultiplied(
                theme.component_body.r(),
                theme.component_body.g(),
                theme.component_body.b(),
                240,
            )
        } else if theme.component_body.r() > 230
            && theme.component_body.g() > 230
            && theme.component_body.b() > 230
        {
            Color32::from_rgba_unmultiplied(255, 255, 255, 248)
        } else {
            Color32::from_rgba_unmultiplied(
                theme.component_body.r(),
                theme.component_body.g(),
                theme.component_body.b(),
                245,
            )
        };

        let stroke_color = if is_dark {
            Color32::from_rgba_unmultiplied(
                theme.grid_dot.r(),
                theme.grid_dot.g(),
                theme.grid_dot.b(),
                200,
            )
        } else {
            Color32::from_rgba_unmultiplied(
                theme.grid_dot.r(),
                theme.grid_dot.g(),
                theme.grid_dot.b(),
                230,
            )
        };

        let badge_hover_fill = if is_dark {
            let r = theme.component_body.r().saturating_add(16);
            let g = theme.component_body.g().saturating_add(16);
            let b = theme.component_body.b().saturating_add(16);
            Color32::from_rgba_unmultiplied(r, g, b, 255)
        } else {
            let r = theme.component_body.r().saturating_sub(12);
            let g = theme.component_body.g().saturating_sub(12);
            let b = theme.component_body.b().saturating_sub(12);
            Color32::from_rgba_unmultiplied(r, g, b, 255)
        };

        let dot_normal = theme.text_secondary;
        let dot_hovered = theme.accent_primary;

        let chevron_normal = theme.text_secondary;
        let chevron_hovered = theme.accent_primary;

        let separator = Color32::from_rgba_unmultiplied(
            theme.grid_dot.r(),
            theme.grid_dot.g(),
            theme.grid_dot.b(),
            if is_dark { 160 } else { 190 },
        );

        let (btn_active_bg, btn_active_fg, btn_active_stroke) = if is_dark {
            (
                Color32::from_rgba_unmultiplied(
                    theme.accent_primary.r(),
                    theme.accent_primary.g(),
                    theme.accent_primary.b(),
                    75,
                ),
                theme.text_primary,
                theme.accent_primary,
            )
        } else {
            (
                Color32::from_rgba_unmultiplied(
                    theme.accent_primary.r(),
                    theme.accent_primary.g(),
                    theme.accent_primary.b(),
                    40,
                ),
                theme.accent_primary,
                theme.accent_primary,
            )
        };

        let btn_hover_bg = if is_dark {
            Color32::from_rgba_unmultiplied(
                theme.grid_dot.r(),
                theme.grid_dot.g(),
                theme.grid_dot.b(),
                80,
            )
        } else {
            Color32::from_rgba_unmultiplied(
                theme.grid_dot.r(),
                theme.grid_dot.g(),
                theme.grid_dot.b(),
                120,
            )
        };

        let btn_hover_fg = theme.text_primary;
        let btn_normal_fg = theme.text_secondary;

        Self {
            bg_fill,
            stroke_color,
            badge_hover_fill,
            dot_normal,
            dot_hovered,
            chevron_normal,
            chevron_hovered,
            separator,
            btn_active_bg,
            btn_active_fg,
            btn_active_stroke,
            btn_hover_bg,
            btn_hover_fg,
            btn_normal_fg,
        }
    }
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

    /// Renders the floating CAD tools island over the canvas viewport in Order::Middle,
    /// dynamically adapting all colors and tokens to the active `PhononTheme`.
    pub fn show(
        &mut self,
        ctx: &egui::Context,
        canvas_viewport: Rect,
        active_tool: &ToolMode,
        theme: &PhononTheme,
    ) -> Option<FloatingToolbarAction> {
        if !self.is_visible {
            return None;
        }

        let toolbar_theme = FloatingToolbarTheme::from_theme(theme);
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
                        (toolbar_theme.badge_hover_fill, toolbar_theme.dot_hovered)
                    } else {
                        (toolbar_theme.bg_fill, toolbar_theme.stroke_color)
                    };

                    painter.circle_filled(center, radius, fill_color);
                    painter.circle_stroke(center, radius, Stroke::new(1.5, stroke_color));

                    let tool_char = match active_tool {
                        ToolMode::Select => "V",
                        ToolMode::Wire => "W",
                        ToolMode::Bus => "B",
                        ToolMode::Probe => "P",
                        ToolMode::NetLabel => "L",
                        ToolMode::Place(_) | ToolMode::PlaceComponent(_) => "+",
                    };

                    let text_color = if response.hovered() {
                        toolbar_theme.dot_hovered
                    } else {
                        toolbar_theme.btn_active_stroke
                    };

                    let galley = painter.layout_no_wrap(
                        tool_char.to_string(),
                        FontId::proportional(12.0),
                        text_color,
                    );
                    painter.galley(center - galley.size() * 0.5, galley, text_color);

                    response.on_hover_text("CAD Tools [Click to expand, drag to relocate]");
                });
        } else {
            // Expanded pill toolbar
            egui::Area::new(egui::Id::new("floating_cad_island_expanded"))
                .order(Order::Middle)
                .fixed_pos(clamped_pos)
                .show(ctx, |ui| {
                    let frame = egui::Frame::new()
                        .fill(toolbar_theme.bg_fill)
                        .corner_radius(CornerRadius::same(14))
                        .stroke(Stroke::new(1.2, toolbar_theme.stroke_color))
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
                                toolbar_theme.dot_hovered
                            } else {
                                toolbar_theme.dot_normal
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
                                toolbar_theme.chevron_hovered
                            } else {
                                toolbar_theme.chevron_normal
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
                            let sep_color = toolbar_theme.separator;
                            let sep_top = clamped_pos.y + 6.0;
                            let sep_bottom = clamped_pos.y + 26.0;
                            ui.painter().line_segment(
                                [Pos2::new(ui.cursor().min.x, sep_top), Pos2::new(ui.cursor().min.x, sep_bottom)],
                                Stroke::new(1.0, sep_color),
                            );
                            ui.add_space(4.0);

                            // Select (V)
                            if render_tool_button(ui, "V", *active_tool == ToolMode::Select, "Select Tool [V] - Pointer & Marquee Drag Box", &toolbar_theme) {
                                triggered_action = Some(FloatingToolbarAction::SelectTool(ToolMode::Select));
                            }

                            // Wire (W)
                            if render_tool_button(ui, "W", *active_tool == ToolMode::Wire, "Wire Tool [W] - Manhattan Electrical Routing", &toolbar_theme) {
                                triggered_action = Some(FloatingToolbarAction::SelectTool(ToolMode::Wire));
                            }

                            // Bus (B)
                            if render_tool_button(ui, "B", *active_tool == ToolMode::Bus, "Bus Tool [B] - Multi-Signal Bus Routing", &toolbar_theme) {
                                triggered_action = Some(FloatingToolbarAction::SelectTool(ToolMode::Bus));
                            }

                            // Probe (P)
                            if render_tool_button(ui, "P", *active_tool == ToolMode::Probe, "Probe Tool [P] - Interactive Voltage Telemetry", &toolbar_theme) {
                                triggered_action = Some(FloatingToolbarAction::SelectTool(ToolMode::Probe));
                            }

                            // Net Label (L)
                            if render_tool_button(ui, "L", *active_tool == ToolMode::NetLabel, "Net Label Tool [L] - Logical Net & Off-Sheet Wire Labeling", &toolbar_theme) {
                                triggered_action = Some(FloatingToolbarAction::SelectTool(ToolMode::NetLabel));
                            }

                            // Separator
                            ui.add_space(2.0);
                            ui.painter().line_segment(
                                [Pos2::new(ui.cursor().min.x, sep_top), Pos2::new(ui.cursor().min.x, sep_bottom)],
                                Stroke::new(1.0, sep_color),
                            );
                            ui.add_space(4.0);

                            // Rotate (R)
                            if render_tool_button(ui, "R", false, "Rotate [R] - Rotate Selection Clockwise (90 deg)", &toolbar_theme) {
                                triggered_action = Some(FloatingToolbarAction::Rotate);
                            }

                            // Mirror (M)
                            if render_tool_button(ui, "M", false, "Mirror [M] - Toggle Horizontal Mirroring (Flip Left/Right)", &toolbar_theme) {
                                triggered_action = Some(FloatingToolbarAction::Mirror);
                            }

                            // Delete (Del)
                            if render_tool_button(ui, "Del", false, "Delete [Del] - Delete Selected Items Atomically", &toolbar_theme) {
                                triggered_action = Some(FloatingToolbarAction::Delete);
                            }

                            // Clear
                            if render_tool_button(ui, "Clr", false, "Clear - Cancel Active Wire / Clear Selection", &toolbar_theme) {
                                triggered_action = Some(FloatingToolbarAction::Clear);
                            }
                        });
                    });
                });
        }

        triggered_action
    }

    /// Backwards-compatible convenience method using default Dark Navy theme.
    pub fn show_default(
        &mut self,
        ctx: &egui::Context,
        canvas_viewport: Rect,
        active_tool: &ToolMode,
    ) -> Option<FloatingToolbarAction> {
        let default_theme = crate::theme::ThemePreset::DarkNavy.to_theme();
        self.show(ctx, canvas_viewport, active_tool, &default_theme)
    }
}

fn render_tool_button(
    ui: &mut egui::Ui,
    label: &str,
    is_active: bool,
    tooltip: &str,
    theme: &FloatingToolbarTheme,
) -> bool {
    let (bg_color, fg_color, stroke_color) = if is_active {
        (
            theme.btn_active_bg,
            theme.btn_active_fg,
            theme.btn_active_stroke,
        )
    } else {
        (
            Color32::TRANSPARENT,
            theme.btn_normal_fg,
            Color32::TRANSPARENT,
        )
    };

    let btn_size = Vec2::new(26.0, 22.0);
    let (btn_rect, btn_resp) = ui.allocate_exact_size(btn_size, Sense::click());

    let current_bg = if is_active {
        bg_color
    } else if btn_resp.hovered() {
        theme.btn_hover_bg
    } else {
        bg_color
    };

    let current_fg = if is_active {
        fg_color
    } else if btn_resp.hovered() {
        theme.btn_hover_fg
    } else {
        fg_color
    };

    ui.painter().rect_filled(btn_rect, CornerRadius::same(5), current_bg);
    if is_active {
        ui.painter().rect_stroke(
            btn_rect,
            CornerRadius::same(5),
            Stroke::new(1.0, stroke_color),
            StrokeKind::Inside,
        );
    } else if btn_resp.hovered() {
        ui.painter().rect_stroke(
            btn_rect,
            CornerRadius::same(5),
            Stroke::new(
                0.8,
                Color32::from_rgba_unmultiplied(
                    theme.stroke_color.r(),
                    theme.stroke_color.g(),
                    theme.stroke_color.b(),
                    80,
                ),
            ),
            StrokeKind::Inside,
        );
    }

    let text_galley = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::proportional(12.0),
        current_fg,
    );
    ui.painter().galley(
        btn_rect.center() - text_galley.size() * 0.5,
        text_galley,
        current_fg,
    );

    let clicked = btn_resp.clicked();
    btn_resp.on_hover_ui(|ui| {
        ui.label(RichText::new(tooltip).size(12.0));
    });

    clicked
}
