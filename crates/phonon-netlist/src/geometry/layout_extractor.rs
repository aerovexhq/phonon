//! Layout parasitics extractor synthesizing physical geometries into CircuitGraph components.

use phonon_core::CircuitGraph;
use phonon_models::MosfetModel;

/// Physical geometric dimensions and diffusion parasitics of a fabricated transistor.
#[derive(Debug, Clone, PartialEq)]
pub struct TransistorLayout {
    pub name: String,
    /// Channel drawn width W in meters.
    pub width: f64,
    /// Channel drawn length L in meters.
    pub length: f64,
    /// Drain diffusion area AD in square meters.
    pub drain_area: f64,
    /// Source diffusion area AS in square meters.
    pub source_area: f64,
    /// Drain diffusion perimeter PD in meters.
    pub drain_perimeter: f64,
    /// Source diffusion perimeter PS in meters.
    pub source_perimeter: f64,
    /// Diffusion sheet resistance R_sheet in Ohms per square.
    pub sheet_resistance: f64,
    /// Bottom-wall zero-bias junction capacitance C_j in Farads per square meter.
    pub cj_area: f64,
    /// Sidewall zero-bias junction capacitance C_jsw in Farads per meter.
    pub cj_sidewall: f64,
}

impl TransistorLayout {
    /// Creates a transistor layout description with typical 130nm default parasitics.
    pub fn new(name: &str, width_m: f64, length_m: f64) -> Self {
        // Standard diffusion contact extension ~ 2 * L_min = 0.3 um
        let diff_len = 0.3e-6;
        let area = width_m * diff_len;
        let perim = 2.0 * (width_m + diff_len);
        Self {
            name: name.to_string(),
            width: width_m,
            length: length_m,
            drain_area: area,
            source_area: area,
            drain_perimeter: perim,
            source_perimeter: perim,
            sheet_resistance: 120.0,
            cj_area: 8.0e-4,
            cj_sidewall: 2.5e-10,
        }
    }

    /// Computes total parasitic junction capacitance at the drain node in Farads:
    /// $$C_{drain} = AD \cdot C_j + PD \cdot C_{jsw}$$
    pub fn drain_capacitance(&self) -> f64 {
        self.drain_area * self.cj_area + self.drain_perimeter * self.cj_sidewall
    }

    /// Computes total parasitic junction capacitance at the source node in Farads:
    /// $$C_{source} = AS \cdot C_j + PS \cdot C_{jsw}$$
    pub fn source_capacitance(&self) -> f64 {
        self.source_area * self.cj_area + self.source_perimeter * self.cj_sidewall
    }

    /// Computes series diffusion resistance in Ohms:
    /// $$R_{diff} = R_{sheet} \cdot \frac{L_{diff}}{W}$$
    pub fn diffusion_series_resistance(&self) -> f64 {
        let l_diff = self.drain_area / self.width.max(1e-9);
        self.sheet_resistance * (l_diff / self.width.max(1e-9))
    }

    /// Generates a calibrated analytical compact MOSFET model with parasitic parameters.
    pub fn to_compact_model(&self) -> MosfetModel {
        MosfetModel {
            w: self.width,
            l: self.length,
            ..MosfetModel::default()
        }
    }

    /// Synthesizes the transistor and its parasitic diffusion capacitances into a CircuitGraph.
    pub fn synthesize_into_graph(
        &self,
        graph: &mut CircuitGraph,
        drain_node: &str,
        gate_node: &str,
        source_node: &str,
        bulk_node: &str,
    ) -> Result<(), String> {
        // Add core MOSFET element
        graph
            .add_mosfet(&self.name, drain_node, gate_node, source_node, bulk_node)
            .map_err(|e| format!("Failed to add MOSFET {}: {:?}", self.name, e))?;

        // Add drain junction parasitic capacitor: drain to bulk
        let c_drain = self.drain_capacitance();
        if c_drain > 1e-18 {
            let c_d_name = format!("{}_Cdiff_d", self.name);
            graph
                .add_capacitor(&c_d_name, drain_node, bulk_node, c_drain, None)
                .map_err(|e| format!("Failed to add drain parasitic capacitor: {:?}", e))?;
        }

        // Add source junction parasitic capacitor: source to bulk
        let c_source = self.source_capacitance();
        if c_source > 1e-18 {
            let c_s_name = format!("{}_Cdiff_s", self.name);
            graph
                .add_capacitor(&c_s_name, source_node, bulk_node, c_source, None)
                .map_err(|e| format!("Failed to add source parasitic capacitor: {:?}", e))?;
        }

        Ok(())
    }
}
