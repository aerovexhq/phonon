#![deny(unsafe_code)]

//! Centered modal confirmation dialog intercepting destructive actions on unsaved projects.
//!
//! Enforces user confirmation when loading demos, clearing canvas, creating new projects,
//! or closing the application while unsaved modifications exist.

use egui::{vec2, Align2, Color32, Key, Pos2, Rect, RichText, Stroke};

/// Available built-in demo circuit types that can be loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DemoCircuitKind {
    VoltageDivider,
    DiodeClipper,
    BjtAmplifier,
    CmosInverter,
    NmosSwitch,
}

impl DemoCircuitKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::VoltageDivider => "Voltage Divider",
            Self::DiodeClipper => "Diode Clipper",
            Self::BjtAmplifier => "BJT CE Amplifier",
            Self::CmosInverter => "CMOS Inverter",
            Self::NmosSwitch => "NMOS Switch",
        }
    }
}

/// Action awaiting user confirmation when unsaved changes exist.
#[derive(Debug, Clone, PartialEq)]
pub enum PendingAction {
    NewProject,
    OpenProject(Option<std::path::PathBuf>),
    LoadDemo(DemoCircuitKind),
    ClearCanvas,
    CloseApp,
}

impl PendingAction {
    pub fn action_description(&self) -> String {
        match self {
            Self::NewProject => "Creating a new project".to_string(),
            Self::OpenProject(_) => "Opening a project".to_string(),
            Self::LoadDemo(demo) => format!("Loading '{}' demo", demo.display_name()),
            Self::ClearCanvas => "Clearing the canvas".to_string(),
            Self::CloseApp => "Closing Phonon Studio".to_string(),
        }
    }
}

/// Outcome of the confirmation modal dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfirmationDecision {
    None,
    SaveAndProceed,
    DiscardAndProceed,
    Cancel,
}

/// Centered confirmation modal dialog.
pub struct ConfirmationModal;

impl ConfirmationModal {
    /// Renders the confirmation modal if an action is pending. Returns the user's decision.
    pub fn show(
        ctx: &egui::Context,
        project_title: &str,
        pending_action: &PendingAction,
    ) -> ConfirmationDecision {
        let mut decision = ConfirmationDecision::None;

        // Render full-screen semi-transparent backdrop behind the modal window
        let screen_rect = ctx
            .input(|i| i.viewport().inner_rect)
            .unwrap_or(Rect::from_min_size(Pos2::ZERO, vec2(1920.0, 1080.0)));
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Middle,
            egui::Id::new("confirmation_modal_backdrop"),
        ));
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(140));

        // Keyboard hotkeys: Esc -> Cancel, Enter -> Save & Proceed
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            return ConfirmationDecision::Cancel;
        }

        let mut open_flag = true;

        let window_frame = egui::Frame::window(&ctx.global_style())
            .fill(Color32::from_rgb(20, 26, 36))
            .stroke(Stroke::new(1.0, Color32::from_rgb(60, 75, 95)));

        egui::Window::new(RichText::new("Unsaved Changes").strong())
            .open(&mut open_flag)
            .collapsible(false)
            .resizable(false)
            .fade_in(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .default_width(460.0)
            .order(egui::Order::Foreground)
            .frame(window_frame)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    ui.heading("Save changes before proceeding?");
                    ui.add_space(6.0);

                    let desc = pending_action.action_description();
                    let msg = format!(
                        "The current project '{}' has unsaved modifications.\n{} will discard all unsaved edits.",
                        project_title, desc
                    );
                    ui.label(
                        RichText::new(msg)
                            .size(13.0)
                            .color(Color32::from_rgb(200, 210, 225)),
                    );

                    ui.add_space(16.0);
                    ui.separator();
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        // Cancel button
                        if ui.button("Cancel").clicked() {
                            decision = ConfirmationDecision::Cancel;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Primary: Save & Proceed button
                            let save_btn = ui.add(
                                egui::Button::new(
                                    RichText::new("Save & Proceed")
                                        .strong()
                                        .color(Color32::from_rgb(10, 15, 25)),
                                )
                                .fill(Color32::from_rgb(56, 189, 248)),
                            );
                            if save_btn.clicked()
                                || ctx.input(|i| i.key_pressed(Key::Enter))
                            {
                                decision = ConfirmationDecision::SaveAndProceed;
                            }

                            ui.add_space(8.0);

                            // Discard Changes button
                            let discard_btn = ui.add(
                                egui::Button::new(
                                    RichText::new("Discard Changes")
                                        .color(Color32::from_rgb(248, 113, 113)),
                                )
                                .fill(Color32::from_rgb(45, 25, 30))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(180, 50, 50))),
                            );
                            if discard_btn.clicked() {
                                decision = ConfirmationDecision::DiscardAndProceed;
                            }
                        });
                    });
                });
            });

        if !open_flag {
            decision = ConfirmationDecision::Cancel;
        }

        decision
    }
}
