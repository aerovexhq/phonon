//! Unified Multi-Physics 3D Asset Ecosystem, Spatial Acceleration & Scene Architecture
//!
//! Provides hierarchical BVH spatial acceleration, multi-physics raycasting
//! (optical/LiDAR, RF electromagnetic transmission, acoustic reflection),
//! asset instancing, scene composition, and parallel Rayon benchmarks.

pub mod asset_benchmark;
pub mod asset_cache;
pub mod spatial_index;

pub use asset_benchmark::{AssetBenchmarkReport, AssetBenchmarkRunner};
pub use asset_cache::{MeshInstance, MultiPhysicsScene};
pub use spatial_index::{
    AcousticRayHit, MultiPhysicsBvh, MultiPhysicsBvhNode, OpticalRayHit, RfTransmissionResult,
    WorldTriangle,
};
