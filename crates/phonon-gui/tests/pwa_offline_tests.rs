#![deny(unsafe_code)]

//! Automated test suite for Progressive Web App (PWA) Offline ServiceWorker & Asset Cache.

use egui::Context;
use phonon_gui::widgets::pwa_offline_dialog::{
    NetworkCondition, PwaOfflineDialog, PwaTab,
};
use std::time::Instant;

#[test]
fn test_pwa_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dlg = PwaOfflineDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "PwaOfflineDialog::new_fast() must boot in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dlg.is_open);
    assert_eq!(dlg.active_tab, PwaTab::ManifestIdentity);
    assert_eq!(dlg.precache_assets.len(), 7);
    assert_eq!(dlg.storage_categories.len(), 3);
    assert_eq!(dlg.audit_items.len(), 10);
    assert!(dlg.sw_active);
}

#[test]
fn test_pwa_manifest_attributes() {
    let dlg = PwaOfflineDialog::new_fast();

    assert!(dlg.app_name.contains("Phonon Studio"));
    assert_eq!(dlg.short_name, "Phonon Studio");
    assert_eq!(dlg.start_url, "/studio/");
    assert_eq!(dlg.scope, "/studio/");
    assert_eq!(dlg.display_mode, "standalone");
    assert_eq!(dlg.theme_color, "#1e293b");
    assert_eq!(dlg.background_color, "#0f172a");
    assert!(!dlg.description.is_empty());
}

#[test]
fn test_serviceworker_precache_assets_and_actions() {
    let mut dlg = PwaOfflineDialog::new_fast();

    // Verify key precache assets
    let urls: Vec<&str> = dlg.precache_assets.iter().map(|a| a.url.as_str()).collect();
    assert!(urls.contains(&"/studio/"));
    assert!(urls.contains(&"/studio/index.html"));
    assert!(urls.contains(&"/studio/manifest.webmanifest"));
    assert!(urls.contains(&"/studio/favicon.svg"));
    assert!(urls.contains(&"/studio/wasm/phonon_gui.js"));
    assert!(urls.contains(&"/studio/wasm/phonon_gui_bg.wasm"));
    assert!(urls.contains(&"/studio/wasm/build_meta.json"));

    // Verify WASM binary size in precache
    let wasm_asset = dlg
        .precache_assets
        .iter()
        .find(|a| a.url.ends_with(".wasm"))
        .unwrap();
    assert!(wasm_asset.size_kb() > 10_000.0); // > 10 MB
    assert!(wasm_asset.caching_strategy.contains("Cache-First"));

    // Test actions
    let old_update_time = dlg.last_update_check_iso.clone();
    dlg.check_for_updates();
    assert_ne!(dlg.last_update_check_iso, old_update_time);

    dlg.purge_cache();
    assert!(dlg.precache_assets.iter().all(|a| a.is_cached));
}

#[test]
fn test_network_condition_state_transitions() {
    let mut dlg = PwaOfflineDialog::new_fast();

    assert_eq!(dlg.network_condition, NetworkCondition::OnlineFiber);
    assert!(!dlg.network_condition.is_offline());

    dlg.network_condition = NetworkCondition::OfflineAirGapped;
    assert!(dlg.network_condition.is_offline());
    assert_eq!(
        dlg.network_condition.display_name(),
        "Offline (Air-Gapped / Zero-Connectivity)"
    );

    dlg.network_condition = NetworkCondition::ThrottledSlow3g;
    assert!(!dlg.network_condition.is_offline());
    assert_eq!(dlg.network_condition.display_name(), "Throttled (Slow 3G)");

    dlg.network_condition = NetworkCondition::Online4gLte;
    assert!(!dlg.network_condition.is_offline());
    assert_eq!(dlg.network_condition.display_name(), "Online (4G LTE Mobile)");
}

#[test]
fn test_storage_quota_calculations() {
    let dlg = PwaOfflineDialog::new_fast();

    let used_bytes = dlg.total_storage_used_bytes();
    let used_mb = used_bytes as f64 / (1024.0 * 1024.0);
    assert!(used_mb > 11.0 && used_mb < 20.0, "Used storage should be ~12-16 MB, got {:.2} MB", used_mb);

    let usage_pct = dlg.storage_usage_percent();
    assert!(
        usage_pct > 0.05 && usage_pct < 1.0,
        "Storage usage should be < 1% of 10GB, got {:.3}%",
        usage_pct
    );

    assert!(dlg.persistent_storage_granted);
    assert!(dlg.indexeddb_active);
    assert_eq!(dlg.storage_categories.len(), 3);
}

#[test]
fn test_lighthouse_pwa_audit_10_criteria() {
    let dlg = PwaOfflineDialog::new_fast();

    assert_eq!(dlg.audit_items.len(), 10);
    let passed_count = dlg.audit_items.iter().filter(|i| i.is_passed).count();
    assert_eq!(passed_count, 10, "All 10 Lighthouse PWA audit criteria must pass");
    assert_eq!(dlg.audit_score, (10, 10));

    // Verify key categories
    let categories: Vec<&str> = dlg.audit_items.iter().map(|i| i.category.as_str()).collect();
    assert!(categories.contains(&"Installability"));
    assert!(categories.contains(&"Offline Capability"));
    assert!(categories.contains(&"Display & UX"));
    assert!(categories.contains(&"Branding"));
    assert!(categories.contains(&"Security"));
    assert!(categories.contains(&"Performance"));
    assert!(categories.contains(&"Reliability"));
}

#[test]
fn test_headless_egui_render_all_5_tabs() {
    let ctx = Context::default();
    let mut dlg = PwaOfflineDialog::new_fast();
    dlg.is_open = true;

    let tabs = [
        PwaTab::ManifestIdentity,
        PwaTab::ServiceWorkerCache,
        PwaTab::OfflineSimulation,
        PwaTab::StoragePersistence,
        PwaTab::PwaAuditChecklist,
    ];

    for tab in tabs {
        dlg.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dlg.ui(ui.ctx());
        });
        out.textures_delta.clear();
    }

    // Test show alias
    let mut out_show = ctx.run_ui(Default::default(), |ui| {
        dlg.show(ui.ctx());
    });
    out_show.textures_delta.clear();
}
