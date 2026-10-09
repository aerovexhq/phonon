#![deny(unsafe_code)]

//! Live schematic Bill of Materials (BOM) generator and component aggregation.
//!
//! Inspects active schematic canvas components, groups identical devices by kind and value,
//! resolves hierarchical subcircuit instances into nested BOM items, and formats standardized part keys.

use super::canvas::SchematicCanvas;
use super::components::{ComponentKind, SchematicComponent};
use super::subcircuit::SubcircuitDefinition;
use phonon_solver::production_economics::{BomLineItem, HierarchicalBom};
use std::collections::HashMap;

/// Maps a component to a normalized part key and functional description.
pub fn component_to_part_key_and_desc(comp: &SchematicComponent) -> (String, String) {
    let val = comp.value_str.trim();
    let val_display = if val.is_empty() { "default" } else { val };

    match comp.kind {
        ComponentKind::Resistor => (
            format!("R_{}", val_display.replace(' ', "")),
            format!("Resistor {} Ohm", val_display),
        ),
        ComponentKind::Capacitor => (
            format!("C_{}", val_display.replace(' ', "")),
            format!("Capacitor {}", val_display),
        ),
        ComponentKind::Inductor => (
            format!("L_{}", val_display.replace(' ', "")),
            format!("Inductor {}", val_display),
        ),
        ComponentKind::Diode => (
            if let Some(m) = &comp.model_name {
                format!("D_{}", m)
            } else if !val.is_empty() {
                format!("D_{}", val.replace(' ', ""))
            } else {
                "D_1N4148".to_string()
            },
            "Switching Diode".to_string(),
        ),
        ComponentKind::ZenerDiode => (
            format!("D_ZENER_{}", val_display.replace(' ', "")),
            format!("Zener Diode {}", val_display),
        ),
        ComponentKind::Led => (
            format!("LED_{}", val_display.replace(' ', "")),
            format!("Light Emitting Diode {}", val_display),
        ),
        ComponentKind::SchottkyDiode => (
            format!("D_SCHOTTKY_{}", val_display.replace(' ', "")),
            format!("Schottky Diode {}", val_display),
        ),
        ComponentKind::BjtNpn => (
            if let Some(m) = &comp.model_name {
                format!("Q_{}", m)
            } else {
                "Q_BC547B".to_string()
            },
            "NPN Bipolar Junction Transistor".to_string(),
        ),
        ComponentKind::BjtPnp => (
            if let Some(m) = &comp.model_name {
                format!("Q_{}", m)
            } else {
                "Q_BC557B".to_string()
            },
            "PNP Bipolar Junction Transistor".to_string(),
        ),
        ComponentKind::Nmos => (
            if let Some(m) = &comp.model_name {
                format!("M_{}", m)
            } else {
                "Q_2N7002".to_string()
            },
            "N-Channel MOSFET".to_string(),
        ),
        ComponentKind::Pmos => (
            if let Some(m) = &comp.model_name {
                format!("M_{}", m)
            } else {
                "Q_BSS84".to_string()
            },
            "P-Channel MOSFET".to_string(),
        ),
        ComponentKind::VoltageSource
        | ComponentKind::AcVoltageSource
        | ComponentKind::CurrentSource
        | ComponentKind::Ground
        | ComponentKind::VddRail => (
            format!("PWR_{:?}_{}", comp.kind, val_display.replace(' ', "")),
            format!("{:?} {}", comp.kind, val_display),
        ),
        ComponentKind::OpAmp => (
            if let Some(m) = &comp.model_name {
                format!("IC_{}", m)
            } else {
                "IC_LM358".to_string()
            },
            "Operational Amplifier".to_string(),
        ),
        ComponentKind::Comparator => (
            if let Some(m) = &comp.model_name {
                format!("IC_{}", m)
            } else {
                "IC_LM393".to_string()
            },
            "Voltage Comparator".to_string(),
        ),
        ComponentKind::Timer555 => (
            "IC_NE555P".to_string(),
            "Precision Timer IC".to_string(),
        ),
        ComponentKind::VoltageRegulator => (
            if let Some(m) = &comp.model_name {
                format!("IC_REG_{}", m)
            } else {
                "IC_LM7805".to_string()
            },
            "Linear Voltage Regulator".to_string(),
        ),
        _ => (
            format!("{:?}_{}", comp.kind, val_display.replace(' ', "")),
            format!("{:?} {}", comp.kind, val_display),
        ),
    }
}

/// Key used to group identical components together in the BOM.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ComponentGroupKey {
    kind: ComponentKind,
    value_str: String,
    model_name: Option<String>,
}

/// Generates a complete HierarchicalBillOfMaterials directly from a live SchematicCanvas.
pub fn generate_bom_from_canvas(
    canvas: &SchematicCanvas,
    subcircuit_defs: &HashMap<String, SubcircuitDefinition>,
) -> HierarchicalBom {
    let mut items = Vec::new();

    // 1. Group discrete schematic components
    // Filter out purely virtual reference elements like Ground from purchasing BOM
    let mut groups: HashMap<ComponentGroupKey, Vec<&SchematicComponent>> = HashMap::new();

    for comp in &canvas.components {
        if comp.kind == ComponentKind::Ground {
            continue;
        }

        let key = ComponentGroupKey {
            kind: comp.kind,
            value_str: comp.value_str.clone(),
            model_name: comp.model_name.clone(),
        };
        groups.entry(key).or_default().push(comp);
    }

    // Sort group keys for deterministic BOM ordering
    let mut sorted_group_keys: Vec<ComponentGroupKey> = groups.keys().cloned().collect();
    sorted_group_keys.sort_by(|a, b| {
        format!("{:?}_{}_{:?}", a.kind, a.value_str, a.model_name)
            .cmp(&format!("{:?}_{}_{:?}", b.kind, b.value_str, b.model_name))
    });

    for (idx, key) in sorted_group_keys.iter().enumerate() {
        if let Some(comps) = groups.get(key) {
            let first = comps[0];
            let (part_key, desc) = component_to_part_key_and_desc(first);

            // Format designators: sort names e.g. "R1, R2, R3"
            let mut desig_names: Vec<String> = comps.iter().map(|c| c.name.clone()).collect();
            desig_names.sort();
            let designators_str = desig_names.join(", ");

            let item_id = format!("ITEM_{}_{}", idx + 1, part_key);

            items.push(BomLineItem::new_component(
                &item_id,
                &designators_str,
                comps.len(),
                &part_key,
                &desc,
            ));
        }
    }

    // 2. Process placed hierarchical subcircuit instances
    for (s_idx, sub_inst) in canvas.subcircuit_instances.iter().enumerate() {
        let def_name = &sub_inst.def_name;
        let mut children = Vec::new();
        let mut lump_sum_cost = 1.50;

        if let Some(def) = subcircuit_defs.get(def_name) {
            let child_bom = generate_bom_from_canvas(&def.internal_canvas, subcircuit_defs);
            children = child_bom.items;
            if !children.is_empty() {
                lump_sum_cost = 2.00;
            }
        }

        let sub_id = format!("SUB_{}_{}", s_idx + 1, def_name);
        let designator = format!("U_SUB_{}", s_idx + 1);
        let part_key = format!("PKG_{}", def_name);
        let desc = format!("Modular Subcircuit Package ({}.phnc)", def_name);

        items.push(BomLineItem::new_subcircuit(
            &sub_id,
            &designator,
            1,
            &part_key,
            &desc,
            lump_sum_cost,
            children,
        ));
    }

    HierarchicalBom { items }
}
