//! Spatial Runge-Kutta 4th-order (RK4) integrator for coupled-mode equations:
//! pump, signal, and idler envelope propagation along traveling-wave transmission lines.

use phonon_models::jtwpa::ParametricProcessParams;

/// Result of a spatial coupled-mode propagation run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CoupledModeResult {
    /// Signal linear power gain G_s = |A_s(L) / A_s(0)|^2.
    pub signal_gain_linear: f64,
    /// Signal power gain in decibels G_s,dB.
    pub signal_gain_db: f64,
    /// Idler linear conversion gain G_i = |A_i(L) / A_s(0)|^2.
    pub idler_gain_linear: f64,
    /// Manley-Rowe photon conservation error |G_s - G_i - 1.0|.
    pub manley_rowe_error: f64,
    /// Phase of output signal in radians.
    pub output_signal_phase_rad: f64,
}

/// Numerical coupled-mode solver for traveling-wave parametric amplifiers.
pub struct CoupledModeSolver;

impl CoupledModeSolver {
    /// Integrates the 3WM / 4WM spatial coupled-mode equations over distance L using RK4.
    pub fn solve(params: &ParametricProcessParams, steps: usize) -> CoupledModeResult {
        let n_steps = steps.max(50);
        let dz = params.total_length_m / n_steps as f64;
        let g = params.gain_factor_per_meter;
        let dk = params.phase_mismatch_rad_per_m;

        // State vector: [Re(A_s), Im(A_s), Re(A_i), Im(A_i)]
        // Initial conditions: A_s(0) = 1.0 + 0i, A_i(0) = 0.0 + 0i
        let mut y = [1.0, 0.0, 0.0, 0.0];
        let mut z = 0.0;

        for _ in 0..n_steps {
            let k1 = derivatives(&y, z, g, dk);
            let y_mid1 = [
                y[0] + 0.5 * dz * k1[0],
                y[1] + 0.5 * dz * k1[1],
                y[2] + 0.5 * dz * k1[2],
                y[3] + 0.5 * dz * k1[3],
            ];

            let k2 = derivatives(&y_mid1, z + 0.5 * dz, g, dk);
            let y_mid2 = [
                y[0] + 0.5 * dz * k2[0],
                y[1] + 0.5 * dz * k2[1],
                y[2] + 0.5 * dz * k2[2],
                y[3] + 0.5 * dz * k2[3],
            ];

            let k3 = derivatives(&y_mid2, z + 0.5 * dz, g, dk);
            let y_end = [
                y[0] + dz * k3[0],
                y[1] + dz * k3[1],
                y[2] + dz * k3[2],
                y[3] + dz * k3[3],
            ];

            let k4 = derivatives(&y_end, z + dz, g, dk);

            for i in 0..4 {
                y[i] += (dz / 6.0) * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
            }
            z += dz;
        }

        let g_s = y[0].powi(2) + y[1].powi(2);
        let g_i = y[2].powi(2) + y[3].powi(2);
        let mr_err = (g_s - g_i - 1.0).abs();
        let phase_s = y[1].atan2(y[0]);

        CoupledModeResult {
            signal_gain_linear: g_s,
            signal_gain_db: 10.0 * g_s.max(1.0).log10(),
            idler_gain_linear: g_i,
            manley_rowe_error: mr_err,
            output_signal_phase_rad: phase_s,
        }
    }

    /// Computes the 3-dB gain bandwidth across a frequency sweep from f_start to f_end.
    pub fn compute_gain_bandwidth(
        base_params: &ParametricProcessParams,
        f_start_hz: f64,
        f_end_hz: f64,
        points: usize,
    ) -> (f64, f64, f64) {
        let n = points.max(20);
        let df = (f_end_hz - f_start_hz) / (n as f64 - 1.0);

        let mut max_gain_db = 0.0;
        let mut gains = Vec::with_capacity(n);

        for i in 0..n {
            let f = f_start_hz + i as f64 * df;
            // Phase mismatch quadratic model: Delta k(f) ~ beta2 * (f - f_0)^2
            let df_center = f - base_params.signal_freq_hz;
            let dk = base_params.phase_mismatch_rad_per_m + 5.0e-18 * df_center.powi(2);

            let mut p = *base_params;
            p.signal_freq_hz = f;
            p.phase_mismatch_rad_per_m = dk;

            let res = Self::solve(&p, 60);
            if res.signal_gain_db > max_gain_db {
                max_gain_db = res.signal_gain_db;
            }
            gains.push((f, res.signal_gain_db));
        }

        let threshold_db = max_gain_db - 3.0;
        let mut f_low = f_start_hz;
        let mut f_high = f_end_hz;
        let mut in_band = false;

        for &(f, g_db) in &gains {
            if g_db >= threshold_db && !in_band {
                f_low = f;
                in_band = true;
            } else if g_db < threshold_db && in_band {
                f_high = f;
                break;
            }
        }

        let bandwidth_hz = (f_high - f_low).max(0.0);
        (max_gain_db, bandwidth_hz, f_low)
    }
}

/// Coupled differential equations:
/// d(Re A_s)/dz = (dk / 2) Im A_s + g Re A_i
/// d(Im A_s)/dz = - (dk / 2) Re A_s - g Im A_i
/// d(Re A_i)/dz = (dk / 2) Im A_i + g Re A_s
/// d(Im A_i)/dz = - (dk / 2) Re A_i - g Im A_s
fn derivatives(y: &[f64; 4], _z: f64, g: f64, dk: f64) -> [f64; 4] {
    let half_dk = 0.5 * dk;
    let re_s = y[0];
    let im_s = y[1];
    let re_i = y[2];
    let im_i = y[3];

    [
        half_dk * im_s + g * re_i,
        -half_dk * re_s - g * im_i,
        half_dk * im_i + g * re_s,
        -half_dk * re_i - g * im_s,
    ]
}
