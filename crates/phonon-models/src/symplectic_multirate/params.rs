#![deny(unsafe_code)]

//! Physical parameters and numerical co-simulation metrics for the Phonon Studio
//! Real-Time High-Order Symplectic Integration & Multi-Rate Co-Simulation Engine.

/// Configuration parameters for high-order symplectic multi-rate integration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymplecticMultirateParams {
    /// Fast subsystem micro-step time in seconds (clamp [1.0e-9, 1.0e-5], default 1.0e-7 s).
    pub fast_dt_s: f64,
    /// Slow subsystem macro-step time in seconds (clamp [1.0e-6, 1.0e-3], default 2.0833333333333333e-5 s).
    pub slow_dt_s: f64,
    /// Integration order for Runge-Kutta kernel (clamp [2, 6], default 4 for GLRK4).
    pub order: u8,
    /// Relative local truncation error tolerance (clamp [1.0e-12, 1.0e-2], default 1.0e-6).
    pub tolerance: f64,
    /// Whether Milne adaptive step control is enabled (default true).
    pub adaptive_step: bool,
}

impl Default for SymplecticMultirateParams {
    fn default() -> Self {
        Self {
            fast_dt_s: 1.0e-7,
            slow_dt_s: 2.0833333333333333e-5,
            order: 4,
            tolerance: 1.0e-6,
            adaptive_step: true,
        }
    }
}

impl SymplecticMultirateParams {
    /// Constructs a new configuration clamped to physically valid numerical bounds.
    pub fn new(
        fast_dt_s: f64,
        slow_dt_s: f64,
        order: u8,
        tolerance: f64,
        adaptive_step: bool,
    ) -> Self {
        Self {
            fast_dt_s: fast_dt_s.clamp(1.0e-9, 1.0e-5),
            slow_dt_s: slow_dt_s.clamp(1.0e-6, 1.0e-3),
            order: order.clamp(2, 6),
            tolerance: tolerance.clamp(1.0e-12, 1.0e-2),
            adaptive_step,
        }
    }

    /// Evaluates the discrete sub-cycling ratio M = round(slow_dt_s / fast_dt_s).
    #[inline]
    pub fn subcycling_ratio(&self) -> usize {
        (self.slow_dt_s / self.fast_dt_s).round().max(1.0) as usize
    }
}

/// Evaluated metrics and invariants from a symplectic multi-rate simulation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SymplecticMultirateMetrics {
    /// Maximum observed absolute drift in total Hamiltonian energy relative to initial.
    pub hamiltonian_drift_max: f64,
    /// Total count of micro-steps executed by the fast partition.
    pub fast_steps_count: u64,
    /// Total count of macro-steps executed by the slow partition.
    pub slow_steps_count: u64,
    /// Maximum observed local Milne truncation error norm.
    pub milne_error_max: f64,
    /// Flag indicating whether the symplectic 2-form or energy invariance was preserved within tolerance.
    pub is_symplectic_invariant_preserved: bool,
    /// Effective integration throughput in steps per second.
    pub throughput_steps_per_sec: f64,
}

impl Default for SymplecticMultirateMetrics {
    fn default() -> Self {
        Self {
            hamiltonian_drift_max: 0.0,
            fast_steps_count: 0,
            slow_steps_count: 0,
            milne_error_max: 0.0,
            is_symplectic_invariant_preserved: true,
            throughput_steps_per_sec: 0.0,
        }
    }
}

impl SymplecticMultirateMetrics {
    /// Constructs a new metrics instance with zero-initialized counters and invariant asserted.
    pub fn new() -> Self {
        Self::default()
    }
}
