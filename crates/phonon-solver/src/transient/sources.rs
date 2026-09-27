//! Time-dependent excitation waveforms for independent voltage and current sources.

use std::f64::consts::PI;

/// Time-dependent waveform specification.
#[derive(Debug, Clone, PartialEq)]
pub enum TimeWaveform {
    /// Constant DC value.
    Dc(f64),
    /// Periodic or single pulse waveform:
    /// - `v1`: Initial value
    /// - `v2`: Pulsed value
    /// - `td`: Delay time (seconds)
    /// - `tr`: Rise time (seconds)
    /// - `tf`: Fall time (seconds)
    /// - `pw`: Pulse width (duration at `v2`, seconds)
    /// - `per`: Period (seconds, 0 or infinite for single pulse)
    Pulse {
        v1: f64,
        v2: f64,
        td: f64,
        tr: f64,
        tf: f64,
        pw: f64,
        per: f64,
    },
    /// Sinusoidal waveform:
    /// $v(t) = v_{offset} + v_{ampl} \cdot \sin(2\pi f (t - t_d)) \cdot e^{-(t - t_d)\theta}$ for $t \ge t_d$
    Sine {
        offset: f64,
        amplitude: f64,
        frequency: f64,
        delay: f64,
        damping: f64,
    },
    /// Piecewise-linear (PWL) waveform defined by time-value pairs.
    Pwl { points: Vec<(f64, f64)> },
}

impl TimeWaveform {
    /// Evaluates the waveform at time $t$.
    pub fn evaluate(&self, t: f64) -> f64 {
        match self {
            Self::Dc(val) => *val,
            Self::Pulse {
                v1,
                v2,
                td,
                tr,
                tf,
                pw,
                per,
            } => {
                if t < *td {
                    return *v1;
                }
                let mut time_rel = t - td;
                if *per > 0.0 {
                    time_rel %= *per;
                }

                let tr = tr.max(1e-18);
                let tf = tf.max(1e-18);

                if time_rel < tr {
                    // Rising edge
                    v1 + (v2 - v1) * (time_rel / tr)
                } else if time_rel < tr + pw {
                    // Flat top
                    *v2
                } else if time_rel < tr + pw + tf {
                    // Falling edge
                    let t_fall = time_rel - (tr + pw);
                    v2 + (v1 - v2) * (t_fall / tf)
                } else {
                    // Rest of period
                    *v1
                }
            }
            Self::Sine {
                offset,
                amplitude,
                frequency,
                delay,
                damping,
            } => {
                if t < *delay {
                    return *offset;
                }
                let dt = t - delay;
                let decay = if *damping != 0.0 {
                    (-dt * damping).exp()
                } else {
                    1.0
                };
                offset + amplitude * (2.0 * PI * frequency * dt).sin() * decay
            }
            Self::Pwl { points } => {
                if points.is_empty() {
                    return 0.0;
                }
                if t <= points[0].0 {
                    return points[0].1;
                }
                if t >= points.last().unwrap().0 {
                    return points.last().unwrap().1;
                }

                // Linear interpolation between bounding points
                for i in 0..points.len() - 1 {
                    let (t0, v0) = points[i];
                    let (t1, v1) = points[i + 1];
                    if t >= t0 && t <= t1 {
                        let span = (t1 - t0).max(1e-18);
                        return v0 + (v1 - v0) * ((t - t0) / span);
                    }
                }
                points.last().unwrap().1
            }
        }
    }
}
