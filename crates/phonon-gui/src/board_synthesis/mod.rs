#![deny(unsafe_code)]

//! Physical PCB Board Synthesis, Technology Mapping & 3D Card Engine.
//!
//! Orchestrates the translation from abstract logical schematic entry to a complete,
//! physically-realizable 3D circuit card with technology-mapped IC footprints,
//! de-clumped placement organization, orthogonal copper trace routing, and
//! parasitic electromagnetic crosstalk extraction.

pub mod auto_place;
pub mod card_model;
pub mod footprint;
pub mod parasitics;
pub mod tech_mapping;
pub mod trace_router;

pub use auto_place::{AutoPlacementParams, AutoPlacer, PlacedBoardLayout};
pub use card_model::{Face3D, SolderMaskColor, SynthesizedPhysicalCard, Vertex3D};
pub use footprint::{FootprintInstance, PackageType, PhysicalPad};
pub use parasitics::{
    EmiDisturbancePair, NetParasitics, PhysicalParasiticsEngine, PhysicalWiringRealismReport,
};
pub use tech_mapping::{
    MappedPhysicalBoard, PhysicalNetConnection, SynthesisError, TechMapper, TechMappingOptions,
};
pub use trace_router::{BoardAutoRouter, CopperLayer, PhysicalTraceSegment, RoutedPhysicalNet};

use crate::schematic::components::SchematicComponent;
use crate::schematic::wire::SchematicWire;

/// Master board synthesis engine.
pub struct BoardSynthesisEngine;

impl BoardSynthesisEngine {
    /// Executes the full physical card synthesis pipeline from schematic components and wires.
    ///
    /// Pipeline Stages:
    /// 1. Technology Mapping & Abstract Gate Explosion (validates physical chip requirements).
    /// 2. De-clumping Layout Organization & Force-Directed Auto-Placement.
    /// 3. Orthogonal Dual-Layer Copper Trace Routing.
    /// 4. Parasitic RLC & Electromagnetic Disturbance (EMI) Extraction.
    /// 5. Assembly of the 3D Synthesized Card Model.
    pub fn synthesize_card(
        components: &[SchematicComponent],
        wires: &[SchematicWire],
        options: &TechMappingOptions,
        placement_params: &AutoPlacementParams,
    ) -> Result<SynthesizedPhysicalCard, SynthesisError> {
        // Stage 1: Technology Mapping & Explosion
        let mapped = TechMapper::map_schematic(components, wires, options)?;

        // Stage 2: De-clumping and Layout Organization
        let placed = AutoPlacer::declump_and_place(mapped.chips, placement_params);

        // Stage 3: Trace Routing
        let routed_nets = BoardAutoRouter::route_nets(&placed.chips, &mapped.net_connections);

        // Stage 4: Physical Wiring Realism & EMI Extraction
        let wiring_report = PhysicalParasiticsEngine::evaluate_board_realism(&routed_nets);

        // Stage 5: Assemble 3D Physical Card
        Ok(SynthesizedPhysicalCard::new(
            placed.board_width_mm,
            placed.board_height_mm,
            placed.chips,
            routed_nets,
            wiring_report,
        ))
    }
}
