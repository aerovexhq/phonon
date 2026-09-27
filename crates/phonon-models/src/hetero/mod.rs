//! Heterogeneous CPU architecture synthesis and multi-material allocation.
//!
//! Submodules:
//! - `material_allocation`: Heterogeneous semiconductor materials database and block allocation maps.
//! - `cpu_floorplan`: 2D geometric macro block placement, power density, and RISC-V 5-stage pipeline floorplans.
//! - `thermo_mechanical`: CTE mismatch stress, Black's electromigration model, and 2D thermal hotspot solvers.

pub mod cpu_floorplan;
pub mod material_allocation;
pub mod thermo_mechanical;

pub use cpu_floorplan::{CpuMacroBlock, ProcessorFloorplan, RiscVFloorplanBuilder};
pub use material_allocation::{
    BlockAllocationMap, HeteroMaterialProperties, HeteroMaterialType, ProcessorBlockType,
};
pub use thermo_mechanical::{
    BlackElectromigrationModel, BlockStressReport, ThermalHotspotReport, ThermalHotspotSolver,
    ThermoMechanicalStressModel,
};
