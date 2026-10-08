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
    HalfAdder,
    BasicGates,
    QuantumMetamaterial,
    Alu4Bit,
    RfTransceiver,
    TopologicalQuantumProcessor,
}

impl DemoCircuitKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::VoltageDivider => "Voltage Divider",
            Self::DiodeClipper => "Diode Clipper",
            Self::BjtAmplifier => "BJT CE Amplifier",
            Self::CmosInverter => "CMOS Inverter",
            Self::NmosSwitch => "NMOS Switch",
            Self::HalfAdder => "Half Adder Logic",
            Self::BasicGates => "Basic Logic Gates",
            Self::QuantumMetamaterial => "Topological Quantum Metamaterials",
            Self::Alu4Bit => "4-Bit ALU Processor Slice",
            Self::RfTransceiver => "RF Microwave Heterodyne Transceiver Front-End",
            Self::TopologicalQuantumProcessor => "Topological Quantum Acoustic Metamaterial Processor",
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
        painter.rect_filled(screen_rect, 0.0, Color32::from_black_alpha(160));

        // Keyboard hotkeys: Esc -> Cancel, Enter -> Save & Proceed
        if ctx.input(|i| i.key_pressed(Key::Escape)) {
            return ConfirmationDecision::Cancel;
        }

        let window_frame = egui::Frame::window(&ctx.global_style())
            .fill(Color32::from_rgb(17, 24, 39))
            .stroke(Stroke::new(1.0, Color32::from_rgb(51, 65, 85)))
            .inner_margin(egui::Margin::symmetric(28, 24));

        egui::Window::new("Unsaved Changes Confirmation")
            .title_bar(false) // Custom header bar so title is mathematically centered across full popup width, ignoring [x]
            .collapsible(false)
            .resizable(false)
            .fade_in(false)
            .anchor(Align2::CENTER_CENTER, vec2(0.0, 0.0))
            .default_width(480.0)
            .order(egui::Order::Foreground)
            .frame(window_frame)
            .show(ctx, |ui| {
                // Eliminate hover size expansion across the entire modal
                ui.style_mut().visuals.widgets.hovered.expansion = 0.0;
                ui.style_mut().visuals.widgets.active.expansion = 0.0;

                ui.vertical(|ui| {
                    // Header Bar: Mathematically centered title across full popup width, ignoring the close [x]
                    let header_rect = ui.allocate_space(vec2(ui.available_width(), 26.0)).1;

                    // Centered title text
                    ui.painter().text(
                        header_rect.center(),
                        Align2::CENTER_CENTER,
                        "Unsaved Changes",
                        egui::FontId::proportional(17.0),
                        Color32::from_rgb(241, 245, 249),
                    );

                    // Close [x] button on the far right without displacing the title center
                    let close_btn_rect = Rect::from_center_size(
                        Pos2::new(header_rect.max.x - 12.0, header_rect.center().y),
                        vec2(22.0, 22.0),
                    );
                    let close_resp = ui.put(
                        close_btn_rect,
                        egui::Button::new(
                            RichText::new("✕")
                                .size(13.0)
                                .color(Color32::from_rgb(148, 163, 184)),
                        )
                        .frame(false),
                    );
                    if close_resp.clicked() {
                        decision = ConfirmationDecision::Cancel;
                    }

                    ui.add_space(14.0);
                    ui.label(
                        RichText::new("Save changes before proceeding?")
                            .size(15.0)
                            .strong()
                            .color(Color32::from_rgb(226, 232, 240)),
                    );
                    ui.add_space(8.0);

                    let desc = pending_action.action_description();
                    let msg = format!(
                        "The current project '{}' has unsaved modifications.\n{} will discard all unsaved edits.",
                        project_title, desc
                    );
                    ui.label(
                        RichText::new(msg)
                            .size(13.5)
                            .color(Color32::from_rgb(148, 163, 184)),
                    );

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(16.0);

                    // Button Row: Professional dark slate / deep navy styling, fixed sizes, no hover jump
                    ui.horizontal(|ui| {
                        // Cancel button: Professional neutral dark slate
                        let cancel_btn = ui.add_sized(
                            vec2(90.0, 34.0),
                            egui::Button::new(
                                RichText::new("Cancel")
                                    .size(13.0)
                                    .color(Color32::from_rgb(203, 213, 225)),
                            )
                            .fill(Color32::from_rgb(30, 41, 59))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(71, 85, 105))),
                        );
                        if cancel_btn.clicked() {
                            decision = ConfirmationDecision::Cancel;
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // Primary: Save & Proceed button (Deep slate navy, highly professional)
                            let save_btn = ui.add_sized(
                                vec2(136.0, 34.0),
                                egui::Button::new(
                                    RichText::new("Save & Proceed")
                                        .size(13.0)
                                        .strong()
                                        .color(Color32::from_rgb(241, 245, 249)),
                                )
                                .fill(Color32::from_rgb(30, 58, 138))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(59, 130, 246))),
                            );
                            if save_btn.clicked() || ctx.input(|i| i.key_pressed(Key::Enter)) {
                                decision = ConfirmationDecision::SaveAndProceed;
                            }

                            ui.add_space(12.0);

                            // Discard Changes button: Deep crimson slate, fixed size, no hover popping
                            let discard_btn = ui.add_sized(
                                vec2(136.0, 34.0),
                                egui::Button::new(
                                    RichText::new("Discard Changes")
                                        .size(13.0)
                                        .color(Color32::from_rgb(248, 113, 113)),
                                )
                                .fill(Color32::from_rgb(55, 20, 26))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(153, 27, 27))),
                            );
                            if discard_btn.clicked() {
                                decision = ConfirmationDecision::DiscardAndProceed;
                            }
                        });
                    });
                });
            });

        decision
    }
}
