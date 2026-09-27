//! Safe SIMD and vectorized mathematical evaluation kernels.
//!
//! Provides lane-chunked (4-lane and 8-lane) vector routines for evaluating
//! semiconductor compact equations without allocations and without unsafe code.

pub mod diode_simd;
pub mod mosfet_simd;

pub use diode_simd::batch_evaluate_diodes_simd;
pub use mosfet_simd::{batch_evaluate_nmos_simd, MosfetBatchOutput};

use crate::common::safe_exp;

/// Vectorized safe exponential evaluation across 4 lanes: $\exp(v_i)$ and $\frac{d}{dv} \exp(v_i)$.
#[inline(always)]
pub fn safe_exp_4(v: [f64; 4]) -> ([f64; 4], [f64; 4]) {
    let (v0, d0) = safe_exp(v[0]);
    let (v1, d1) = safe_exp(v[1]);
    let (v2, d2) = safe_exp(v[2]);
    let (v3, d3) = safe_exp(v[3]);
    ([v0, v1, v2, v3], [d0, d1, d2, d3])
}

/// Vectorized safe exponential evaluation across 8 lanes: $\exp(v_i)$ and $\frac{d}{dv} \exp(v_i)$.
#[inline(always)]
pub fn safe_exp_8(v: [f64; 8]) -> ([f64; 8], [f64; 8]) {
    let (v0, d0) = safe_exp(v[0]);
    let (v1, d1) = safe_exp(v[1]);
    let (v2, d2) = safe_exp(v[2]);
    let (v3, d3) = safe_exp(v[3]);
    let (v4, d4) = safe_exp(v[4]);
    let (v5, d5) = safe_exp(v[5]);
    let (v6, d6) = safe_exp(v[6]);
    let (v7, d7) = safe_exp(v[7]);
    (
        [v0, v1, v2, v3, v4, v5, v6, v7],
        [d0, d1, d2, d3, d4, d5, d6, d7],
    )
}
