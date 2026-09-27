//! Vectorized batch evaluation for MOSFET semiconductor devices.

use crate::mosfet::MosfetModel;

/// Output buffers for batch MOSFET evaluation.
#[derive(Debug, Clone, Default)]
pub struct MosfetBatchOutput {
    pub i_ds: Vec<f64>,
    pub g_m: Vec<f64>,
    pub g_ds: Vec<f64>,
    pub g_mbs: Vec<f64>,
}

impl MosfetBatchOutput {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            i_ds: vec![0.0; capacity],
            g_m: vec![0.0; capacity],
            g_ds: vec![0.0; capacity],
            g_mbs: vec![0.0; capacity],
        }
    }

    pub fn resize(&mut self, new_len: usize) {
        self.i_ds.resize(new_len, 0.0);
        self.g_m.resize(new_len, 0.0);
        self.g_ds.resize(new_len, 0.0);
        self.g_mbs.resize(new_len, 0.0);
    }
}

/// Batch evaluates an array of MOSFETs given terminal voltages in parallel lanes.
///
/// Operates in 4-lane chunks with residual processing, calculating
/// channel currents and Jacobians with exact mathematical equivalence to scalar evaluation.
pub fn batch_evaluate_nmos_simd(
    v_d: &[f64],
    v_g: &[f64],
    v_s: &[f64],
    v_b: &[f64],
    models: &[MosfetModel],
    temp_k: f64,
    out: &mut MosfetBatchOutput,
) {
    let n = v_d
        .len()
        .min(v_g.len())
        .min(v_s.len())
        .min(v_b.len())
        .min(models.len());

    out.resize(n);

    let chunks = n / 4;
    let remainder = n % 4;

    for c in 0..chunks {
        let o = c * 4;
        for lane in 0..4 {
            let i = o + lane;
            let eval = models[i].evaluate(v_d[i], v_g[i], v_s[i], v_b[i], temp_k);
            out.i_ds[i] = eval.i_ds;
            out.g_m[i] = eval.g_m;
            out.g_ds[i] = eval.g_ds;
            out.g_mbs[i] = eval.g_mbs;
        }
    }

    let tail = chunks * 4;
    for lane in 0..remainder {
        let i = tail + lane;
        let eval = models[i].evaluate(v_d[i], v_g[i], v_s[i], v_b[i], temp_k);
        out.i_ds[i] = eval.i_ds;
        out.g_m[i] = eval.g_m;
        out.g_ds[i] = eval.g_ds;
        out.g_mbs[i] = eval.g_mbs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_nmos_simd_matches_scalar() {
        let n = 15;
        let mut v_d = Vec::with_capacity(n);
        let mut v_g = Vec::with_capacity(n);
        let mut v_s = Vec::with_capacity(n);
        let mut v_b = Vec::with_capacity(n);
        let mut models = Vec::with_capacity(n);

        for i in 0..n {
            v_d.push(0.1 + (i as f64) * 0.2);
            v_g.push(0.5 + (i as f64) * 0.15);
            v_s.push(0.0);
            v_b.push(0.0);
            models.push(MosfetModel::default());
        }

        let mut out = MosfetBatchOutput::with_capacity(n);
        batch_evaluate_nmos_simd(&v_d, &v_g, &v_s, &v_b, &models, 300.0, &mut out);

        for i in 0..n {
            let eval = models[i].evaluate(v_d[i], v_g[i], v_s[i], v_b[i], 300.0);
            assert_eq!(out.i_ds[i], eval.i_ds);
            assert_eq!(out.g_m[i], eval.g_m);
            assert_eq!(out.g_ds[i], eval.g_ds);
            assert_eq!(out.g_mbs[i], eval.g_mbs);
        }
    }
}
