//! Multi-Valued Logic (MVL) and Non-Binary Semiconductor Computing.
//!
//! Submodules:
//! - `ternary_device`: Multi-threshold MOSFETs, dual-peak RTD, and chirality-tuned CNTFETs.
//! - `ternary_primitives`: Balanced ternary logic levels (Trit, Quat), STI, PTI, NTI, TNAND, TNOR, and TFA.
//! - `noise_margin`: 3-state Voltage Transfer Characteristics (VTC), TSNM, and thermal retention.
//! - `interconnect_scaling`: Radix information theory, Rent's rule interconnect modeling, and pin/energy metrics.

pub mod interconnect_scaling;
pub mod noise_margin;
pub mod ternary_device;
pub mod ternary_primitives;

pub use interconnect_scaling::{
    InterconnectComparison, InterconnectRentModel, InterconnectScalingEvaluator, RadixEfficiency,
};
pub use noise_margin::{TernaryNoiseMarginAnalyzer, TernaryNoiseMargins, ThermalRetentionReport};
pub use ternary_device::{
    CntfetTernaryModel, MosfetFlavor, MosfetMvlEvaluation, MultiPeakRtdModel, MultiPeakRtdParams,
    MultiThresholdMosfet, MultiThresholdMosfetParams, RtdEvaluation,
};
pub use ternary_primitives::{
    Quat, TernaryFullAdderCell, TernaryGates, TernaryInverters, TfaOutput, Trit,
};
