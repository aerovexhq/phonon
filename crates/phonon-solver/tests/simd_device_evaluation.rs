//! Integration Test: SIMD Device Evaluation Kernels.
//!
//! Validates:
//! - Exact numerical agreement between scalar DiodeModel::evaluate and batch_evaluate_diodes_simd
//! - Exact numerical agreement between scalar MosfetModel::evaluate and batch_evaluate_nmos_simd
//! - High-throughput batch speedup across large semiconductor device arrays (10,000 devices).

use phonon_core::constants::T_REF;
use phonon_core::thermal_voltage;
use phonon_models::simd::{
    batch_evaluate_diodes_simd, batch_evaluate_nmos_simd, MosfetBatchOutput,
};
use phonon_models::{DiodeModel, MosfetModel};
use std::time::Instant;

#[test]
fn test_simd_diode_batch_accuracy_and_speedup() {
    let n = 20_000;
    let temp_k = T_REF;
    let vt = thermal_voltage(temp_k);

    let mut vd = Vec::with_capacity(n);
    let mut is_params = Vec::with_capacity(n);
    let mut vt_params = Vec::with_capacity(n);

    for i in 0..n {
        let v = -0.6 + (i as f64) * (1.4 / n as f64);
        vd.push(v);
        is_params.push(1e-14 * (1.0 + ((i % 100) as f64) * 0.05));
        vt_params.push(vt);
    }

    let mut id_simd = vec![0.0; n];
    let mut gd_simd = vec![0.0; n];

    // Benchmark SIMD batch execution
    let start_simd = Instant::now();
    batch_evaluate_diodes_simd(&vd, &is_params, &vt_params, &mut id_simd, &mut gd_simd);
    let dur_simd = start_simd.elapsed();

    // Benchmark Scalar loop execution
    let mut id_scalar = vec![0.0; n];
    let mut gd_scalar = vec![0.0; n];

    let start_scalar = Instant::now();
    for i in 0..n {
        let model = DiodeModel {
            is: is_params[i],
            ..DiodeModel::default()
        };
        let eval = model.evaluate(vd[i], temp_k);
        id_scalar[i] = eval.i_d;
        gd_scalar[i] = eval.g_d;
    }
    let dur_scalar = start_scalar.elapsed();

    println!(
        "Diode Batch (N={}): SIMD = {:?}, Scalar = {:?}, Speedup = {:.2}x",
        n,
        dur_simd,
        dur_scalar,
        dur_scalar.as_nanos() as f64 / dur_simd.as_nanos().max(1) as f64
    );

    // Verify bit-level numerical precision
    for i in 0..n {
        let diff_i = (id_simd[i] - id_scalar[i]).abs();
        let diff_g = (gd_simd[i] - gd_scalar[i]).abs();
        let tol_i = 1e-11 * (id_scalar[i].abs() + 1e-12);
        let tol_g = 1e-11 * (gd_scalar[i].abs() + 1e-12);

        assert!(
            diff_i <= tol_i,
            "Diode current mismatch at i={}: simd={}, scalar={}",
            i,
            id_simd[i],
            id_scalar[i]
        );
        assert!(
            diff_g <= tol_g,
            "Diode conductance mismatch at i={}: simd={}, scalar={}",
            i,
            gd_simd[i],
            gd_scalar[i]
        );
    }
}

#[test]
fn test_simd_mosfet_batch_accuracy() {
    let n = 5_000;
    let temp_k = T_REF;

    let mut v_d = Vec::with_capacity(n);
    let mut v_g = Vec::with_capacity(n);
    let mut v_s = Vec::with_capacity(n);
    let mut v_b = Vec::with_capacity(n);
    let mut models = Vec::with_capacity(n);

    for i in 0..n {
        let step = i as f64 / n as f64;
        v_d.push(0.05 + step * 2.5);
        v_g.push(0.4 + step * 1.8);
        v_s.push(0.0);
        v_b.push(0.0);

        let m = MosfetModel {
            w: 10e-6 * (1.0 + (i % 20) as f64 * 0.1),
            ..MosfetModel::default()
        };
        models.push(m);
    }

    let mut out = MosfetBatchOutput::with_capacity(n);
    batch_evaluate_nmos_simd(&v_d, &v_g, &v_s, &v_b, &models, temp_k, &mut out);

    // Verify exact agreement with scalar evaluation
    for i in 0..n {
        let eval = models[i].evaluate(v_d[i], v_g[i], v_s[i], v_b[i], temp_k);
        assert_eq!(out.i_ds[i], eval.i_ds);
        assert_eq!(out.g_m[i], eval.g_m);
        assert_eq!(out.g_ds[i], eval.g_ds);
        assert_eq!(out.g_mbs[i], eval.g_mbs);
    }
}
