//! Time-of-Flight LiDAR Simulation, BVH Spatial Acceleration & Atmospheric Point Clouds
//!
//! Provides high-performance raycasting accelerators, multi-echo pulse return,
//! Mie atmospheric extinction, backscatter clutter synthesis, and parallel benchmark runners.

pub mod bvh;
pub mod lidar_benchmark;
pub mod tof_engine;

pub use bvh::{Aabb, BvhHit, BvhNode, BvhPrimitive, BvhTree};
pub use lidar_benchmark::{LidarBenchmarkReport, LidarBenchmarkRunner};
pub use tof_engine::{LidarPoint, LidarPointCloud, TofLidarEngine};
