#![deny(unsafe_code)]

//! Comprehensive Verification Test Suite for WebGL2 / WebGPU Compute Shader Hardware-Accelerated SPICE Co-Processor.
//!
//! Validates:
//! 1. Sub-millisecond cold boot latency (< 5ms).
//! 2. Compressed Sparse Row (CSR) matrix representation and Jacobi iteration convergence.
//! 3. Largest-Triangle-Three-Buckets (LTTB) waveform decimation algorithm accuracy and boundary handling.
//! 4. Batched multi-corner parameter sweep throughput and speedup metrics.
//! 5. GPU backend kind display mappings.
//! 6. 10-point WebGPU Hardware Acceleration Readiness Audit (10/10 passed).
//! 7. Headless egui render pass across all 5 dialog tabs.

use std::time::Instant;

use phonon_gui::widgets::webgpu_spice_dialog::{
    lttb_decimate, CsrMatrix, GpuBackendKind, WebGpuSpiceDialog, WebGpuSpiceTab,
};

#[test]
fn test_webgpu_spice_dialog_cold_boot_latency() {
    let start = Instant::now();
    let dialog = WebGpuSpiceDialog::new_fast();
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 5,
        "Cold boot instantiation took {:?}, exceeding 5ms target",
        elapsed
    );
    assert!(!dialog.is_open);
    assert_eq!(dialog.active_tab, WebGpuSpiceTab::GpuPipeline);
    assert_eq!(dialog.backend_kind, GpuBackendKind::WebGpuBrowser);
    assert!(!dialog.adapter_name.is_empty());
    assert!(!dialog.wgsl_shader_preview.is_empty());
    assert_eq!(dialog.max_workgroup_invocations, 256);
    assert_eq!(dialog.csr_matrix.num_rows, 64);
    assert_eq!(dialog.csr_matrix.num_cols, 64);

    // Also verify Default trait behaves identically
    let default_dialog = WebGpuSpiceDialog::default();
    assert_eq!(default_dialog.active_tab, WebGpuSpiceTab::GpuPipeline);
}

#[test]
fn test_csr_matrix_creation_and_jacobi_convergence() {
    let n = 8;
    let csr = CsrMatrix::new_demo_banded(n);

    assert_eq!(csr.num_rows, n);
    assert_eq!(csr.num_cols, n);
    assert_eq!(csr.row_ptr.len(), n + 1);
    assert_eq!(csr.row_ptr[0], 0);

    // Verify tridiagonal non-zero count: n diagonal + 2*(n-1) off-diagonal = 3*n - 2 = 22
    assert_eq!(csr.values.len(), 3 * n - 2);
    assert_eq!(csr.col_indices.len(), 3 * n - 2);

    // Initial state: x_0 = [0, 0, ...], rhs b = [1, 1, ...]
    let mut x = vec![0.0f64; n];
    let b = vec![1.0f64; n];
    let mut x_next = vec![0.0f64; n];

    let mut res_initial = 0.0f64;
    let mut res_final = 0.0f64;

    for step in 0..20 {
        let res = csr.jacobi_step(&x, &b, &mut x_next);
        if step == 0 {
            res_initial = res;
        }
        res_final = res;
        x.copy_from_slice(&x_next);
    }

    assert!(
        res_initial > 0.1,
        "Initial residual must be non-trivial, got {:.4}",
        res_initial
    );
    assert!(
        res_final < res_initial * 0.05,
        "Jacobi iteration must converge: initial = {:.4}, final = {:.4}",
        res_initial,
        res_final
    );
}

#[test]
fn test_lttb_decimate_waveform_reduction_and_extrema() {
    let mut raw_x = Vec::with_capacity(100);
    let mut raw_y = Vec::with_capacity(100);

    for i in 0..100 {
        let t = i as f64 * 0.1;
        // Inject a known sharp peak at index 42
        let y = if i == 42 { 25.0 } else { (t * 2.0).sin() };
        raw_x.push(t);
        raw_y.push(y);
    }

    let target_points = 20;
    let (dec_x, dec_y) = lttb_decimate(&raw_x, &raw_y, target_points);

    assert_eq!(
        dec_x.len(),
        target_points,
        "Decimated output must match target points"
    );
    assert_eq!(
        dec_y.len(),
        target_points,
        "Decimated output must match target points"
    );

    // First and last points must strictly match raw boundaries
    assert_eq!(dec_x[0], raw_x[0]);
    assert_eq!(dec_y[0], raw_y[0]);
    assert_eq!(dec_x[dec_x.len() - 1], raw_x[raw_x.len() - 1]);
    assert_eq!(dec_y[dec_y.len() - 1], raw_y[raw_y.len() - 1]);

    // The extreme spike at index 42 (y = 25.0) must be captured by LTTB
    let max_dec_y = dec_y.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        (max_dec_y - 25.0).abs() < 1e-6,
        "LTTB must preserve critical voltage spikes: expected 25.0, found {:.3}",
        max_dec_y
    );

    // Edge case 1: target points >= raw points -> returns clone of original
    let (same_x, same_y) = lttb_decimate(&raw_x, &raw_y, 100);
    assert_eq!(same_x.len(), 100);
    assert_eq!(same_y.len(), 100);

    // Edge case 2: target points < 3 -> returns clone of original
    let (fallback_x, fallback_y) = lttb_decimate(&raw_x, &raw_y, 2);
    assert_eq!(fallback_x.len(), 100);
    assert_eq!(fallback_y.len(), 100);
}

#[test]
fn test_batched_sweeps_metrics() {
    let dialog = WebGpuSpiceDialog::new_fast();

    assert_eq!(dialog.batch_size, 1024);
    assert!(
        dialog.batch_speedup_factor >= 30.0,
        "GPU batch speedup must exceed 30x, found {:.1}x",
        dialog.batch_speedup_factor
    );
    assert!(
        dialog.simulations_per_second >= 1000.0,
        "Throughput must exceed 1,000 sims/sec, found {:.0}",
        dialog.simulations_per_second
    );
    assert!(dialog.gpu_kernel_time_ms < 1.0);
    assert!(dialog.cpu_sequential_time_ms > dialog.gpu_kernel_time_ms * 20.0);
}

#[test]
fn test_gpu_backend_kind_display_names() {
    assert_eq!(
        GpuBackendKind::Vulkan.display_name(),
        "Vulkan (Native Linux/Android)"
    );
    assert_eq!(
        GpuBackendKind::Metal.display_name(),
        "Metal (Apple Silicon/macOS)"
    );
    assert_eq!(
        GpuBackendKind::Dx12.display_name(),
        "Direct3D 12 (Windows)"
    );
    assert_eq!(
        GpuBackendKind::WebGpuBrowser.display_name(),
        "WebGPU (Chromium / Firefox / Safari)"
    );
    assert_eq!(
        GpuBackendKind::CpuFallback.display_name(),
        "Pure Safe Rust CPU Fallback"
    );
}

#[test]
fn test_10_point_webgpu_readiness_audit() {
    let dialog = WebGpuSpiceDialog::new_fast();

    assert_eq!(
        dialog.audit_criteria.len(),
        10,
        "WebGPU Readiness Audit must have exactly 10 criteria"
    );

    let passed_count = dialog.audit_criteria.iter().filter(|c| c.is_passed).count();
    assert_eq!(
        passed_count, 10,
        "All 10 WebGPU Hardware Acceleration Audit criteria must pass"
    );
    assert_eq!(dialog.audit_score, (10, 10));

    let criteria_names: Vec<&str> = dialog
        .audit_criteria
        .iter()
        .map(|c| c.criterion.as_str())
        .collect();

    assert!(criteria_names.iter().any(|c| c.contains("WebGPU / WebGL2 Compute Device Discovery")));
    assert!(criteria_names.iter().any(|c| c.contains("WGSL Compute Shader Pipeline")));
    assert!(criteria_names.iter().any(|c| c.contains("CSR Sparse Matrix")));
    assert!(criteria_names.iter().any(|c| c.contains("Iterative Jacobi")));
    assert!(criteria_names.iter().any(|c| c.contains("Batched Multi-Parameter Sweep")));
    assert!(criteria_names.iter().any(|c| c.contains("LTTB Million-Point Waveform")));
    assert!(criteria_names.iter().any(|c| c.contains("Zero-Allocation Buffer Ring")));
    assert!(criteria_names.iter().any(|c| c.contains("Pure Safe Rust Fallback")));
    assert!(criteria_names.iter().any(|c| c.contains("Sub-5ms Cold Startup")));
    assert!(criteria_names.iter().any(|c| c.contains("Cross-Platform WebAssembly")));
}

#[test]
fn test_headless_egui_render_all_5_tabs() {
    let ctx = egui::Context::default();
    let mut dialog = WebGpuSpiceDialog::new_fast();
    dialog.is_open = true;

    let tabs = [
        WebGpuSpiceTab::GpuPipeline,
        WebGpuSpiceTab::SparseMnaSolver,
        WebGpuSpiceTab::BatchedSweeps,
        WebGpuSpiceTab::WaveformDecimation,
        WebGpuSpiceTab::HardwareAudit,
    ];

    for tab in tabs {
        dialog.active_tab = tab;
        let mut out = ctx.run_ui(Default::default(), |ui| {
            dialog.ui(ui.ctx());
        });
        out.textures_delta.clear();
        assert!(dialog.is_open);
    }
}
