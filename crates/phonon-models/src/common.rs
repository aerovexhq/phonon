//! Mathematical utilities, safe exponential bounding, and Newton-Raphson voltage limiting functions.

/// Evaluates $e^v$ and its first derivative $\frac{d}{dv} e^v$ with numerical overflow protection.
/// For $v > 80.0$, smoothly transitions to a tangent line to preserve continuous gradients
/// without producing `f64::INFINITY` or `NaN`.
#[inline(always)]
pub fn safe_exp(v: f64) -> (f64, f64) {
    const MAX_EXP: f64 = 80.0;
    const MIN_EXP: f64 = -80.0;

    if v > MAX_EXP {
        let e_max = MAX_EXP.exp();
        let val = e_max * (1.0 + (v - MAX_EXP));
        let deriv = e_max;
        (val, deriv)
    } else if v < MIN_EXP {
        (0.0, 0.0)
    } else {
        let val = v.exp();
        (val, val)
    }
}

/// SPICE-standard PN junction voltage limiter (P-N diode / base-emitter / drain-bulk).
/// Prevents new iterate `v_new` from taking massive leaps into exponential overflow.
#[inline]
pub fn pn_junction_limit(v_new: f64, v_old: f64, vt: f64, vcrit: f64) -> f64 {
    if v_new > vcrit && (v_new - v_old).abs() > 2.0 * vt {
        if v_old > 0.0 {
            let arg = 1.0 + (v_new - v_old) / vt;
            if arg > 0.0 {
                v_old + vt * arg.ln()
            } else {
                vcrit
            }
        } else {
            vt * (v_new / vt).max(0.1).ln()
        }
    } else {
        v_new
    }
}

/// Computes the critical PN junction turn-on voltage:
/// $V_{crit} = V_t \ln\left( \frac{V_t}{\sqrt{2} I_S} \right)$.
#[inline(always)]
pub fn compute_vcrit(is: f64, vt: f64) -> f64 {
    vt * (vt / (std::f64::consts::SQRT_2 * is.max(1e-20))).ln()
}

/// Smooth maximum function $f(a, b) \approx \max(a, b)$ with continuous derivatives $C^\infty$:
/// $\max_\delta(a, b) = \frac{1}{2} \left[ a + b + \sqrt{(a - b)^2 + 4 \delta^2} \right]$.
#[inline(always)]
pub fn smooth_max(a: f64, b: f64, delta: f64) -> f64 {
    0.5 * (a + b + ((a - b).powi(2) + 4.0 * delta * delta).sqrt())
}

/// Smooth minimum function $f(a, b) \approx \min(a, b)$ with continuous derivatives $C^\infty$:
/// $\min_\delta(a, b) = \frac{1}{2} \left[ a + b - \sqrt{(a - b)^2 + 4 \delta^2} \right]$.
#[inline(always)]
pub fn smooth_min(a: f64, b: f64, delta: f64) -> f64 {
    0.5 * (a + b - ((a - b).powi(2) + 4.0 * delta * delta).sqrt())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_exp_bounds() {
        let (val, deriv) = safe_exp(100.0);
        assert!(!val.is_infinite());
        assert!(!val.is_nan());
        assert!(val > 0.0);
        assert!(deriv > 0.0);

        let (val_neg, deriv_neg) = safe_exp(-100.0);
        assert_eq!(val_neg, 0.0);
        assert_eq!(deriv_neg, 0.0);

        let (v_normal, d_normal) = safe_exp(1.0);
        assert!((v_normal - std::f64::consts::E).abs() < 1e-12);
        assert!((d_normal - std::f64::consts::E).abs() < 1e-12);
    }

    #[test]
    fn test_smooth_max_min() {
        let s_max = smooth_max(5.0, 3.0, 0.01);
        assert!((s_max - 5.0).abs() < 0.01);

        let s_min = smooth_min(5.0, 3.0, 0.01);
        assert!((s_min - 3.0).abs() < 0.01);
    }
}
