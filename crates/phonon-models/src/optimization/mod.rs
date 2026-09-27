//! Inverse device design, automated material discovery, and multi-objective Pareto optimization.

pub mod adjoint_refiner;
pub mod genome;
pub mod nsga2;
pub mod physical_fitness;
pub mod roadmap_targets;

pub use adjoint_refiner::{AdjointRefiner, OptimizationTarget, ParameterSensitivities};
pub use genome::{
    ArchitectureType, ChannelMaterial, GeneBounds, OptContactMetal, OptGateDielectric,
    TransistorGenome,
};
pub use nsga2::{FastRng, Individual, Nsga2Config, Nsga2Optimizer};
pub use physical_fitness::{evaluate_transistor_fitness, FitnessEvaluation};
pub use roadmap_targets::{IrdsNodeTarget, RoadmapComplianceReport};
