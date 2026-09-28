//! Optical Perception, Synthetic Scene Generation & Camera Solvers
//!
//! Provides headless multi-tier offscreen perception pipelines, geometric scene intersection,
//! radiometry/photometry conversions, and Rayon parallel benchmark runners.

pub mod optical_benchmark;
pub mod perception_pipeline;

pub use optical_benchmark::{OpticalBenchmarkReport, OpticalBenchmarkRunner};
pub use perception_pipeline::{
    AabbBox, CheckerPlane, LightSource, OffscreenPerceptionEngine, OpticalRealismTier,
    PerceptionFrame, RayHit, SceneObject, Sphere, SyntheticScene,
};
