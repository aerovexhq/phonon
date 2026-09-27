//! Vectorized batch evaluation for semiconductor PN diodes.

use super::safe_exp_4;
use crate::common::safe_exp;

/// Vectorized batch evaluation of diode currents $ and conductances  = \frac{\partial I_D}{\partial V_D}$.
///
/// Computes results in 4-lane parallel chunks with residual tail processing.
/// Guarantees exact IEEE 754 floating point consistency with scalar diode evaluation.
pub fn batch_evaluate_diodes_simd(
    vd: &[f64],
    is_params: &[f64],
    vt_params: &[f64],
    id_out: &mut [f64],
    gd_out: &mut [f64],
) {
    let n = vd
        .len()
        .min(is_params.len())
        .min(vt_params.len())
        .min(id_out.len())
        .min(gd_out.len());

    let chunks = n / 4;
    let remainder = n % 4;

    for c in 0..chunks {
        let offset = c * 4;
        let v_chunk = [vd[offset], vd[offset + 1], vd[offset + 2], vd[offset + 3]];
        let is_chunk = [
            is_params[offset],
            is_params[offset + 1],
            is_params[offset + 2],
            is_params[offset + 3],
        ];
        let vt_chunk = [
            vt_params[offset],
            vt_params[offset + 1],
            vt_params[offset + 2],
            vt_params[offset + 3],
        ];

        let arg_chunk = [
            v_chunk[0] / vt_chunk[0],
            v_chunk[1] / vt_chunk[1],
            v_chunk[2] / vt_chunk[2],
            v_chunk[3] / vt_chunk[3],
        ];

        let (exp_val, exp_deriv) = safe_exp_4(arg_chunk);

        id_out[offset] = is_chunk[0] * (exp_val[0] - 1.0);
        id_out[offset + 1] = is_chunk[1] * (exp_val[1] - 1.0);
        id_out[offset + 2] = is_chunk[2] * (exp_val[2] - 1.0);
        id_out[offset + 3] = is_chunk[3] * (exp_val[3] - 1.0);

        gd_out[offset] = (is_chunk[0] / vt_chunk[0]) * exp_deriv[0];
        gd_out[offset + 1] = (is_chunk[1] / vt_chunk[1]) * exp_deriv[1];
        gd_out[offset + 2] = (is_chunk[2] / vt_chunk[2]) * exp_deriv[2];
        gd_out[offset + 3] = (is_chunk[3] / vt_chunk[3]) * exp_deriv[3];
    }

    let tail_offset = chunks * 4;
    for i in 0..remainder {
        let idx = tail_offset + i;
        let arg = vd[idx] / vt_params[idx];
        let (e, de) = safe_exp(arg);
        id_out[idx] = is_params[idx] * (e - 1.0);
        gd_out[idx] = (is_params[idx] / vt_params[idx]) * de;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diode::DiodeModel;

    #[test]
    fn test_batch_diode_simd_matches_scalar() {
        let n = 23; // Test chunked + tail remainder
        let mut vd = Vec::with_capacity(n);
        let mut is_params = Vec::with_capacity(n);
        let mut vt_params = Vec::with_capacity(n);

        let temp_k = phonon_core::constants::T_REF;
        let vt = phonon_core::thermal_voltage(temp_k);

        for i in 0..n {
            let v = -0.5 + (i as f64) * 0.08;
            vd.push(v);
            is_params.push(1e-14 * (1.0 + (i as f64) * 0.1));
            vt_params.push(vt);
        }

        let mut id_simd = vec![0.0; n];
        let mut gd_simd = vec![0.0; n];

        batch_evaluate_diodes_simd(&vd, &is_params, &vt_params, &mut id_simd, &mut gd_simd);

        for i in 0..n {
            let model = DiodeModel {
                is: is_params[i],
                ..DiodeModel::default()
            };
            let eval = model.evaluate(vd[i], temp_k);

            let diff_i = (id_simd[i] - eval.i_d).abs();
            let diff_g = (gd_simd[i] - eval.g_d).abs();

            assert!(
                diff_i < 1e-12 * (eval.i_d.abs() + 1.0),
                "Mismatch in current at i={}: simd={}, scalar={}",
                i,
                id_simd[i],
                eval.i_d
            );
            assert!(
                diff_g < 1e-12 * (eval.g_d.abs() + 1.0),
                "Mismatch in conductance at i={}: simd={}, scalar={}",
                i,
                gd_simd[i],
                eval.g_d
            );
        }
    }
}
