#![deny(unsafe_code)]

//! Quantum acoustic tensor network simulators and continuous-variable
//! fault-tolerant magic state distillation models.

mod params;

pub use params::{
    QuantumAcousticTensorDistillationMetrics, QuantumAcousticTensorDistillationParams,
};
