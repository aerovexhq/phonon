#![deny(unsafe_code)]

//! High-frequency RF network analysis, S-parameter extraction, Smith chart geometry,
//! and Harmonic Balance frequency-domain engine.

pub mod harmonic_balance;
pub mod s_parameters;
pub mod smith_chart;

pub use harmonic_balance::{
    HarmonicBalanceResult, HarmonicBalanceSolver, HarmonicComponent, NonLinearMetrics,
    NonlinearDevice,
};
pub use s_parameters::{
    format_touchstone_s2p, Complex64, FrequencySweep, MultiPortSSolver, MultiPortSSweepResult,
    SweepType, TwoPortSParameters,
};
pub use smith_chart::{
    constant_reactance_arc, constant_reactance_circle, constant_resistance_circle,
    gamma_to_normalized_z, gamma_to_z, load_stability_circle, normalized_z_to_gamma,
    source_stability_circle, standard_reactance_values, standard_resistance_values, z_to_gamma,
    SmithCircle,
};
