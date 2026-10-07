#![deny(unsafe_code)]

use egui::Context;
use phonon_gui::widgets::floquet_corner_transducer_dialog::{
    CornerTransducerTab, FloquetCornerTransducerDialog,
};
use phonon_solver::floquet_corner_transducer::TargetEntangledState;

#[test]
fn test_dialog_initialization_and_cold_boot() {
    let start = std::time::Instant::now();
    let dialog = FloquetCornerTransducerDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot initialization must be < 5ms, took {} ms",
        elapsed.as_millis()
    );
    assert!(!dialog.is_open);
    assert_eq!(
        dialog.active_tab,
        CornerTransducerTab::CornerStateTransducer
    );
    assert_eq!(dialog.cached_modes.len(), 4);
    assert!(
        dialog.cached_transduction.peak_efficiency >= 0.85,
        "Peak efficiency must be >= 85%"
    );
    assert!(
        dialog.cached_transduction.transfer_fidelity >= 0.995,
        "Transfer fidelity must be >= 0.995"
    );
    assert!(
        -dialog.cached_routing.reverse_isolation_db >= 30.0,
        "Reverse isolation must be >= 30 dB"
    );
    assert!(
        dialog.cached_entanglement.concurrence >= 0.95,
        "Concurrence must be >= 0.95"
    );
    assert!(
        dialog.cached_entanglement.chsh_parameter >= 2.75,
        "CHSH parameter must be >= 2.75"
    );
}

#[test]
fn test_dialog_tab_switching() {
    let mut dialog = FloquetCornerTransducerDialog::new_fast();

    let tabs = [
        CornerTransducerTab::CornerStateTransducer,
        CornerTransducerTab::NonReciprocalRouting,
        CornerTransducerTab::MultiQubitEntanglement,
        CornerTransducerTab::RealSpaceMetamaterial,
        CornerTransducerTab::AuditTelemetry,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        assert_eq!(dialog.active_tab, tab);
        assert!(!tab.label().is_empty());
    }
}

#[test]
fn test_dialog_recompute_and_parameter_updates() {
    let mut dialog = FloquetCornerTransducerDialog::new_fast();

    // Adjust parameters and trigger recompute
    dialog.center_freq_ghz = 5.2;
    dialog.transmon_coupling_mhz = 25.0;
    dialog.forward_exchange_mhz = 22.0;
    dialog.selected_target_state = TargetEntangledState::BellPsiPlus;
    dialog.recompute();

    assert_eq!(dialog.center_freq_ghz, 5.2);
    assert_eq!(
        dialog.selected_target_state,
        TargetEntangledState::BellPsiPlus
    );
    assert!(
        dialog.cached_transduction.peak_efficiency >= 0.85,
        "Efficiency must remain >= 85%"
    );
    assert!(
        -dialog.cached_routing.reverse_isolation_db >= 30.0,
        "Isolation must remain >= 30 dB"
    );
    assert!(
        dialog.cached_entanglement.chsh_parameter >= 2.75,
        "CHSH must remain >= 2.75"
    );
    assert!(
        dialog.cached_audit.all_passed,
        "All 10 audit criteria must pass after recompute"
    );
}

#[test]
fn test_dialog_headless_render() {
    let ctx = Context::default();
    let mut dialog = FloquetCornerTransducerDialog::new_fast();
    dialog.is_open = true;

    for tab in &[
        CornerTransducerTab::CornerStateTransducer,
        CornerTransducerTab::NonReciprocalRouting,
        CornerTransducerTab::MultiQubitEntanglement,
        CornerTransducerTab::RealSpaceMetamaterial,
        CornerTransducerTab::AuditTelemetry,
    ] {
        dialog.active_tab = *tab;
        let mut output = ctx.run_ui(Default::default(), |ui| {
            dialog.render_contents(ui);
        });
        output.textures_delta.clear();
    }
}
