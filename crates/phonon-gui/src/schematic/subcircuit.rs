#![deny(unsafe_code)]

//! Hierarchical subcircuit macro-modeling, pin mapping, nested schematic encapsulation, and flat MNA netlist compilation.

use super::canvas::SchematicCanvas;
use super::circuit_compiler::{compile_schematic, compile_schematic_with_labels};
use super::components::ComponentKind;
use egui::{Color32, FontId, Painter, Pos2, Rect, Stroke, StrokeKind, Vec2};
use std::collections::HashMap;

/// Electrical pin direction on a hierarchical subcircuit boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PinDirection {
    Input,
    Output,
    Inout,
}

/// Boundary pin definition for a hierarchical subcircuit block.
#[derive(Debug, Clone, PartialEq)]
pub struct SubcircuitPin {
    pub name: String,
    pub direction: PinDirection,
    pub local_net: String,
    pub pos_offset: Pos2,
}

impl SubcircuitPin {
    /// Creates a new subcircuit pin.
    pub fn new(
        name: impl Into<String>,
        direction: PinDirection,
        local_net: impl Into<String>,
        pos_offset: Pos2,
    ) -> Self {
        Self {
            name: name.into(),
            direction,
            local_net: local_net.into(),
            pos_offset,
        }
    }
}

/// Reusable hierarchical subcircuit definition enclosing an internal schematic canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct SubcircuitDefinition {
    pub name: String,
    pub pins: Vec<SubcircuitPin>,
    pub internal_canvas: SchematicCanvas,
    pub width: f32,
    pub height: f32,
}

impl SubcircuitDefinition {
    /// Creates a new subcircuit definition with calculated or clamped dimensions.
    pub fn new(name: &str, pins: Vec<SubcircuitPin>, internal_canvas: SchematicCanvas) -> Self {
        let mut min_x = -50.0f32;
        let mut max_x = 50.0f32;
        let mut min_y = -35.0f32;
        let mut max_y = 35.0f32;

        for pin in &pins {
            min_x = min_x.min(pin.pos_offset.x - 20.0);
            max_x = max_x.max(pin.pos_offset.x + 20.0);
            min_y = min_y.min(pin.pos_offset.y - 15.0);
            max_y = max_y.max(pin.pos_offset.y + 15.0);
        }

        let width = (max_x - min_x).max(100.0);
        let height = (max_y - min_y).max(60.0);

        Self {
            name: name.to_string(),
            pins,
            internal_canvas,
            width,
            height,
        }
    }

    /// Looks up a boundary pin by name.
    pub fn pin_by_name(&self, name: &str) -> Option<&SubcircuitPin> {
        self.pins.iter().find(|p| p.name == name)
    }

    /// Overrides default computed dimensions for this subcircuit block.
    pub fn with_dimensions(mut self, width: f32, height: f32) -> Self {
        self.width = width;
        self.height = height;
        self
    }
}

/// Placed instance of a hierarchical subcircuit on a schematic canvas.
#[derive(Debug, Clone, PartialEq)]
pub struct SubcircuitInstance {
    pub id: usize,
    pub def_name: String,
    pub pos: Pos2,
    pub rotation: u8,
    pub pin_nets: HashMap<String, String>,
}

impl SubcircuitInstance {
    /// Constructs a new subcircuit instance.
    pub fn new(id: usize, def_name: impl Into<String>, pos: Pos2) -> Self {
        Self {
            id,
            def_name: def_name.into(),
            pos,
            rotation: 0,
            pin_nets: HashMap::new(),
        }
    }

    /// Builder method attaching an external net mapping for an internal pin.
    pub fn with_pin_net(
        mut self,
        pin_name: impl Into<String>,
        external_net: impl Into<String>,
    ) -> Self {
        self.pin_nets.insert(pin_name.into(), external_net.into());
        self
    }

    /// Explicitly maps an internal pin name to an external net name.
    pub fn map_pin(&mut self, pin_name: impl Into<String>, external_net: impl Into<String>) {
        self.pin_nets.insert(pin_name.into(), external_net.into());
    }

    /// Computes the world-space coordinate of a subcircuit pin accounting for instance rotation.
    pub fn pin_world_pos(&self, pin: &SubcircuitPin) -> Pos2 {
        let offset = match self.rotation % 4 {
            0 => pin.pos_offset.to_vec2(),
            1 => Vec2::new(-pin.pos_offset.y, pin.pos_offset.x),
            2 => Vec2::new(-pin.pos_offset.x, -pin.pos_offset.y),
            3 => Vec2::new(pin.pos_offset.y, -pin.pos_offset.x),
            _ => pin.pos_offset.to_vec2(),
        };
        self.pos + offset
    }

    /// Calculates bounding rectangle in world space.
    pub fn bounds(&self, def: Option<&SubcircuitDefinition>) -> Rect {
        let (w, h) = if let Some(d) = def {
            (d.width, d.height)
        } else {
            (120.0, 80.0)
        };
        let (half_w, half_h) = if self.rotation % 2 == 1 {
            (h * 0.5, w * 0.5)
        } else {
            (w * 0.5, h * 0.5)
        };
        Rect::from_center_size(self.pos, Vec2::new(half_w * 2.0, half_h * 2.0))
    }

    /// Tests if a world coordinate falls within this instance's symbol bounds.
    pub fn contains(&self, p: Pos2, def: Option<&SubcircuitDefinition>) -> bool {
        self.bounds(def).contains(p)
    }

    /// Renders the subcircuit block symbol, pin ports, and designator labels.
    pub fn render(
        &self,
        painter: &Painter,
        canvas: &SchematicCanvas,
        def: Option<&SubcircuitDefinition>,
        is_selected: bool,
    ) {
        let screen_center = canvas.world_to_screen(self.pos);
        let (w, h) = if let Some(d) = def {
            (d.width, d.height)
        } else {
            (120.0, 80.0)
        };
        let (half_w, half_h) = if self.rotation % 2 == 1 {
            (h * 0.5 * canvas.zoom, w * 0.5 * canvas.zoom)
        } else {
            (w * 0.5 * canvas.zoom, h * 0.5 * canvas.zoom)
        };

        let rect = Rect::from_center_size(
            screen_center,
            Vec2::new(half_w * 2.0, half_h * 2.0),
        );

        let bg_color = Color32::from_rgb(26, 32, 44);
        let border_stroke = if is_selected {
            Stroke::new(2.0, Color32::from_rgb(100, 200, 255))
        } else {
            Stroke::new(1.5, Color32::from_rgb(70, 130, 210))
        };

        painter.rect_filled(rect, 4.0, bg_color);
        painter.rect_stroke(rect, 4.0, border_stroke, StrokeKind::Middle);

        // Subcircuit definition name centered
        painter.text(
            rect.center() + Vec2::new(0.0, -8.0),
            egui::Align2::CENTER_CENTER,
            &self.def_name,
            FontId::proportional(12.0 * canvas.zoom.clamp(0.8, 1.4)),
            Color32::from_rgb(230, 240, 255),
        );

        // Instance identifier: X<id>
        painter.text(
            rect.center() + Vec2::new(0.0, 8.0),
            egui::Align2::CENTER_CENTER,
            format!("X{}", self.id),
            FontId::monospace(10.0 * canvas.zoom.clamp(0.8, 1.4)),
            Color32::from_rgb(140, 180, 230),
        );

        // Render pins if definition is available
        if let Some(d) = def {
            for pin in &d.pins {
                let pin_world = self.pin_world_pos(pin);
                let pin_screen = canvas.world_to_screen(pin_world);

                painter.circle_filled(
                    pin_screen,
                    3.5 * canvas.zoom.clamp(0.8, 1.4),
                    Color32::from_rgb(0, 210, 240),
                );

                let label_pos = pin_screen
                    + Vec2::new(
                        if pin.pos_offset.x < 0.0 { 6.0 } else { -6.0 },
                        0.0,
                    );
                let align = if pin.pos_offset.x < 0.0 {
                    egui::Align2::LEFT_CENTER
                } else {
                    egui::Align2::RIGHT_CENTER
                };
                painter.text(
                    label_pos,
                    align,
                    &pin.name,
                    FontId::monospace(9.0 * canvas.zoom.clamp(0.8, 1.4)),
                    Color32::from_rgb(180, 220, 240),
                );
            }
        }
    }
}

/// Precomputed internal component descriptor used for ultra-fast instantiation flattening.
#[derive(Debug, Clone)]
struct SubcircuitInternalComp {
    name: String,
    kind: ComponentKind,
    pins: Vec<(String, String)>, // (pin_name, local_net)
    value_str: String,
    model_name: Option<String>,
}

/// Extracts template component descriptions and local net associations from a subcircuit definition.
fn build_template_descriptors(def: &SubcircuitDefinition) -> Vec<SubcircuitInternalComp> {
    // If internal canvas has wires, compile to resolve pin-to-net mappings
    let compiled_pin_to_net = if !def.internal_canvas.wires.is_empty() {
        compile_schematic_with_labels(
            &def.internal_canvas.components,
            &def.internal_canvas.wires,
            &def.internal_canvas.net_labels,
        )
        .ok()
        .map(|c| c.pin_to_net)
    } else {
        None
    };

    let mut descriptors = Vec::with_capacity(def.internal_canvas.components.len());

    for comp in &def.internal_canvas.components {
        if comp.kind == ComponentKind::Ground {
            continue;
        }

        let pin_defs = comp.kind.pin_definitions();
        let mut pin_entries = Vec::with_capacity(pin_defs.len());

        for (pin_idx, (p_name, _)) in pin_defs.iter().enumerate() {
            // 1. Check direct property override "net:<pin_name>"
            let explicit_prop = comp.get_property(&format!("net:{}", p_name));

            let local_net = if let Some(n) = explicit_prop {
                n.to_string()
            } else if let Some(ref pin_map) = compiled_pin_to_net {
                // 2. Resolved net from wires
                pin_map
                    .get(&(comp.name.clone(), p_name.to_string()))
                    .cloned()
                    .unwrap_or_else(|| format!("{}_{}", comp.name, p_name))
            } else {
                // 3. Match component pin coordinates with SubcircuitPin pos_offset
                let pin_world = comp.pin_world_pos(pin_idx).unwrap_or(comp.pos);
                if let Some(matching_pin) = def
                    .pins
                    .iter()
                    .find(|p| (p.pos_offset - pin_world).length() <= 6.0)
                {
                    matching_pin.local_net.clone()
                } else {
                    format!("{}_{}", comp.name, p_name)
                }
            };

            pin_entries.push((p_name.to_string(), local_net));
        }

        descriptors.push(SubcircuitInternalComp {
            name: comp.name.clone(),
            kind: comp.kind,
            pins: pin_entries,
            value_str: comp.value_str.clone(),
            model_name: comp.model_name.clone(),
        });
    }

    descriptors
}

/// Flattens a top-level canvas containing subcircuit instances into a flat MNA SPICE netlist prefixing internal nodes with `X<id>_`.
pub fn flatten_hierarchical_netlist(
    canvas: &SchematicCanvas,
    definitions: &HashMap<String, SubcircuitDefinition>,
) -> Result<String, String> {
    flatten_hierarchical_netlist_with_instances(canvas, &canvas.subcircuit_instances, definitions)
}

/// Flattens a top-level canvas and an explicit slice of subcircuit instances into a flat MNA SPICE netlist.
pub fn flatten_hierarchical_netlist_with_instances(
    canvas: &SchematicCanvas,
    instances: &[SubcircuitInstance],
    definitions: &HashMap<String, SubcircuitDefinition>,
) -> Result<String, String> {
    // 1. Validate that all instance definition names exist
    for inst in instances {
        if !definitions.contains_key(&inst.def_name) {
            return Err(format!(
                "Subcircuit definition '{}' not found for instance X{}",
                inst.def_name, inst.id
            ));
        }
    }

    // 2. Pre-compile template descriptors for each definition referenced
    let mut template_cache: HashMap<&str, Vec<SubcircuitInternalComp>> = HashMap::new();
    for inst in instances {
        if !template_cache.contains_key(inst.def_name.as_str()) {
            if let Some(def) = definitions.get(&inst.def_name) {
                template_cache.insert(&inst.def_name, build_template_descriptors(def));
            }
        }
    }

    // 3. Pre-allocate buffer for high-throughput netlist generation
    let mut out = String::with_capacity(32 * 1024 + instances.len() * 256);
    out.push_str("* Exported from Phonon CAD Schematic (Hierarchical Flattened)\n");
    out.push_str(".TEMP 27.0\n");

    // 4. Emit top-level canvas primitive components if present
    if !canvas.components.is_empty() {
        if let Ok(compiled) = compile_schematic_with_labels(&canvas.components, &canvas.wires, &canvas.net_labels) {
            for line in compiled.spice_netlist.lines() {
                if !line.starts_with('*')
                    && !line.starts_with(".TEMP")
                    && !line.starts_with(".OP")
                    && !line.starts_with(".END")
                    && !line.starts_with(".SUBCKT")
                    && !line.starts_with(".ENDS")
                    && !line.is_empty()
                {
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }
    }

    // 5. Flatten each subcircuit instance into inlined primitive SPICE cards
    for inst in instances {
        let def = definitions.get(&inst.def_name).unwrap();
        let comps = template_cache.get(inst.def_name.as_str()).unwrap();

        for comp in comps {
            let flat_comp_name = format!("{}_X{}", comp.name, inst.id);
            let mut resolved_nets: Vec<String> = Vec::with_capacity(comp.pins.len());

            for (_, local_net) in &comp.pins {
                let resolved = if local_net == "0"
                    || local_net.eq_ignore_ascii_case("gnd")
                    || local_net.eq_ignore_ascii_case("ground")
                {
                    "0".to_string()
                } else if let Some(ext) = inst.pin_nets.get(local_net) {
                    ext.clone()
                } else if let Some(matching_pin) = def
                    .pins
                    .iter()
                    .find(|p| &p.local_net == local_net || &p.name == local_net)
                {
                    if let Some(ext) = inst.pin_nets.get(&matching_pin.name) {
                        ext.clone()
                    } else {
                        // Unconnected pin node
                        format!("X{}_{}", inst.id, local_net)
                    }
                } else {
                    // Internal subcircuit node: prefix with X<id>_
                    format!("X{}_{}", inst.id, local_net)
                };

                resolved_nets.push(resolved);
            }

            match comp.kind {
                ComponentKind::Resistor => {
                    let n1 = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let n2 = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    out.push_str(&format!("{} {} {} {}\n", flat_comp_name, n1, n2, comp.value_str));
                }
                ComponentKind::Capacitor => {
                    let n1 = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let n2 = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    out.push_str(&format!("{} {} {} {}\n", flat_comp_name, n1, n2, comp.value_str));
                }
                ComponentKind::Inductor => {
                    let n1 = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let n2 = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    out.push_str(&format!("{} {} {} {}\n", flat_comp_name, n1, n2, comp.value_str));
                }
                ComponentKind::VoltageSource => {
                    let p = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let n = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    out.push_str(&format!("{} {} {} DC {}\n", flat_comp_name, p, n, comp.value_str));
                }
                ComponentKind::CurrentSource => {
                    let p = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let n = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    out.push_str(&format!("{} {} {} DC {}\n", flat_comp_name, p, n, comp.value_str));
                }
                ComponentKind::Diode => {
                    let a = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let k = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    let model = comp.model_name.as_deref().unwrap_or("1N4148");
                    out.push_str(&format!("{} {} {} {}\n", flat_comp_name, a, k, model));
                }
                ComponentKind::Nmos => {
                    let d = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let g = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    let s = resolved_nets.get(2).map(|s| s.as_str()).unwrap_or("0");
                    let b = resolved_nets.get(3).map(|s| s.as_str()).unwrap_or("0");
                    out.push_str(&format!("{} {} {} {} {} NMOS\n", flat_comp_name, d, g, s, b));
                }
                ComponentKind::Pmos => {
                    let d = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let g = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    let s = resolved_nets.get(2).map(|s| s.as_str()).unwrap_or("0");
                    let b = resolved_nets.get(3).map(|s| s.as_str()).unwrap_or("0");
                    out.push_str(&format!("{} {} {} {} {} PMOS\n", flat_comp_name, d, g, s, b));
                }
                ComponentKind::BjtNpn => {
                    let c = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let b = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    let e = resolved_nets.get(2).map(|s| s.as_str()).unwrap_or("0");
                    out.push_str(&format!("{} {} {} {} NPN\n", flat_comp_name, c, b, e));
                }
                ComponentKind::BjtPnp => {
                    let c = resolved_nets.get(0).map(|s| s.as_str()).unwrap_or("0");
                    let b = resolved_nets.get(1).map(|s| s.as_str()).unwrap_or("0");
                    let e = resolved_nets.get(2).map(|s| s.as_str()).unwrap_or("0");
                    out.push_str(&format!("{} {} {} {} PNP\n", flat_comp_name, c, b, e));
                }
                ComponentKind::Ground => {}
                _ => {
                    let nets_str = resolved_nets.join(" ");
                    out.push_str(&format!("{} {} {}\n", flat_comp_name, nets_str, comp.value_str));
                }
            }
        }
    }

    out.push_str(".OP\n");
    out.push_str(".END\n");

    Ok(out)
}
