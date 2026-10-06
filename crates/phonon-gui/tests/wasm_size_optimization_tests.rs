#![deny(unsafe_code)]

//! Automated test suite for Web Studio WASM Binary Size Optimization, Fat LTO & Cache Invalidation.

use egui::Context;
use phonon_gui::widgets::wasm_optimization_dialog::{
    WasmOptTab, WasmOptimizationDialog,
};
use std::time::Instant;

#[test]
fn test_wasm_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dlg = WasmOptimizationDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "WasmOptimizationDialog::new_fast() must boot in sub-5ms, took {:?}",
        elapsed
    );

    assert!(!dlg.is_open);
    assert_eq!(dlg.active_tab, WasmOptTab::BinarySize);
    assert_eq!(dlg.tiers.len(), 6);
    assert_eq!(dlg.networks.len(), 5);
    assert_eq!(dlg.cache_rules.len(), 4);
    assert_eq!(dlg.checklist.len(), 10);
}

#[test]
fn test_compression_reduction_and_budget() {
    let dlg = WasmOptimizationDialog::new_fast();

    let debug_tier = &dlg.tiers[0];
    let standard_release = &dlg.tiers[1];
    let fat_lto = &dlg.tiers[2];
    let wasm_opt = &dlg.tiers[3];
    let gzip = &dlg.tiers[4];
    let brotli = &dlg.tiers[5];

    // Monotonic size reduction hierarchy
    assert!(debug_tier.size_bytes > standard_release.size_bytes);
    assert!(standard_release.size_bytes > fat_lto.size_bytes);
    assert!(fat_lto.size_bytes > wasm_opt.size_bytes);
    assert!(wasm_opt.size_bytes > gzip.size_bytes);
    assert!(gzip.size_bytes > brotli.size_bytes);

    // Over-the-wire payload budget verification
    assert!(
        gzip.size_mb() <= dlg.size_budget_mb,
        "Gzip size ({:.2} MB) must satisfy budget (<= {:.1} MB)",
        gzip.size_mb(),
        dlg.size_budget_mb
    );

    // Verify reduction percentages
    let reduction_vs_debug = gzip.reduction_percent(debug_tier.size_bytes);
    assert!(
        reduction_vs_debug > 90.0,
        "Gzip payload must achieve > 90% size reduction vs unoptimized debug, got {:.1}%",
        reduction_vs_debug
    );
}

#[test]
fn test_network_profile_download_latency() {
    let dlg = WasmOptimizationDialog::new_fast();
    let gzip_bytes = dlg.tiers[4].size_bytes;

    let throttled_3g = &dlg.networks[0];
    let lte_4g = &dlg.networks[1];
    let gigabit_fiber = &dlg.networks[4];

    let t_3g = throttled_3g.download_time_s(gzip_bytes);
    let t_4g = lte_4g.download_time_s(gzip_bytes);
    let t_fiber = gigabit_fiber.download_time_s(gzip_bytes);

    assert!(t_3g > 15.0, "Throttled 3G download should exceed 15s");
    assert!(t_4g > 1.0 && t_4g < 3.0, "4G LTE download should take 1-3s, took {:.2}s", t_4g);
    assert!(t_fiber < 0.1, "Gigabit fiber download should take < 100ms, took {:.3}s", t_fiber);
    assert!(t_3g > t_4g && t_4g > t_fiber);
}

#[test]
fn test_cache_rule_matching_and_headers() {
    let dlg = WasmOptimizationDialog::new_fast();

    // 1. WASM bundle: must match immutable 1-year rule
    let wasm_rule = dlg.match_cache_rule("/studio/wasm/phonon_gui_bg.wasm");
    assert!(wasm_rule.is_some());
    let wasm_rule = wasm_rule.unwrap();
    assert_eq!(wasm_rule.max_age_seconds, 31_536_000);
    assert!(wasm_rule.is_immutable);
    assert_eq!(wasm_rule.content_type, "application/wasm");

    // 2. JS glue: must match immutable 1-year rule
    let js_rule = dlg.match_cache_rule("/studio/wasm/phonon_gui.js");
    assert!(js_rule.is_some());
    let js_rule = js_rule.unwrap();
    assert_eq!(js_rule.max_age_seconds, 31_536_000);
    assert!(js_rule.is_immutable);
    assert_eq!(js_rule.content_type, "application/javascript");

    // 3. HTML index: must have max-age=0, must-revalidate
    let html_rule = dlg.match_cache_rule("/index.html");
    assert!(html_rule.is_some());
    let html_rule = html_rule.unwrap();
    assert_eq!(html_rule.max_age_seconds, 0);
    assert!(!html_rule.is_immutable);
    assert!(html_rule.cache_control.contains("must-revalidate"));

    // 4. Favicon: static asset with 1-day caching
    let svg_rule = dlg.match_cache_rule("/studio/favicon.svg");
    assert!(svg_rule.is_some());
    let svg_rule = svg_rule.unwrap();
    assert_eq!(svg_rule.max_age_seconds, 86_400);
    assert!(!svg_rule.is_immutable);
}

#[test]
fn test_simulate_deploy_commit_and_hash_update() {
    let mut dlg = WasmOptimizationDialog::new_fast();
    let initial_commit = dlg.current_commit_hash.clone();
    let initial_wasm_sha = dlg.wasm_sha256.clone();

    assert_eq!(initial_commit, "88f1a5d");
    assert_eq!(initial_wasm_sha.len(), 64);

    dlg.simulate_deploy_commit();

    assert_ne!(dlg.current_commit_hash, initial_commit);
    assert_ne!(dlg.wasm_sha256, initial_wasm_sha);
    assert_eq!(dlg.wasm_sha256.len(), 64);
}

#[test]
fn test_symbols_breakdown_percentage_sum() {
    let dlg = WasmOptimizationDialog::new_fast();
    let total_pct: f64 = dlg.symbols.iter().map(|s| s.percentage).sum();

    assert!(
        (total_pct - 100.0).abs() < 1.0,
        "Symbol percentages must sum to ~100%, got {:.2}%",
        total_pct
    );

    // phonon-solver should be the largest component
    let max_symbol = dlg
        .symbols
        .iter()
        .max_by(|a, b| a.size_bytes.cmp(&b.size_bytes))
        .unwrap();
    assert!(max_symbol.name.contains("phonon-solver"));
}

#[test]
fn test_deployment_checklist_readiness() {
    let dlg = WasmOptimizationDialog::new_fast();

    assert_eq!(dlg.checklist.len(), 10);
    let passed_count = dlg.checklist.iter().filter(|c| c.is_passed).count();
    assert_eq!(passed_count, 10, "All 10 production deployment criteria must pass");
    assert_eq!(dlg.last_audit_score, (10, 10));
}

#[test]
fn test_headless_egui_render_all_tabs() {
    let ctx = Context::default();
    let mut dlg = WasmOptimizationDialog::new_fast();
    dlg.is_open = true;

    let tabs = [
        WasmOptTab::BinarySize,
        WasmOptTab::CacheInvalidation,
        WasmOptTab::LatencyProfiler,
        WasmOptTab::SymbolBreakdown,
        WasmOptTab::DeploymentChecklist,
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
