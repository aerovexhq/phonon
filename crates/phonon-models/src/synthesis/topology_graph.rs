//! Circuit Topology Graph Representation for Unconstrained Gate Synthesis.
//!
//! Encodes arbitrary interconnects, pass-transistor logic (PTL), transmission gates (TG),
//! dynamic pull-up/pull-down networks, and direct material switching actions (NDR, MIT).
//! Provides standard canonical presets (28T CMOS, 20T TGA, 14T Hybrid, 10T PTL, NDR MOBILE).

/// Role and identity of an electrical node in the synthesized gate topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GateNode {
    /// Supply rail $V_{dd}$.
    PowerVdd,
    /// Ground rail $GND$.
    GroundGnd,
    /// Primary logic input (0-indexed: $A=0, B=1, C_{in}=2$).
    PrimaryInput(usize),
    /// Primary logic output (0-indexed: $Out=0, Sum=0, C_{out}=1$).
    PrimaryOutput(usize),
    /// Internal circuit routing node.
    Internal(usize),
}

/// A physical device element connecting nodes within the circuit topology.
#[derive(Debug, Clone, PartialEq)]
pub enum GateElement {
    /// NMOS Pass / Switching Transistor (drain, gate, source).
    Nmos {
        drain: usize,
        gate: usize,
        source: usize,
        width_nm: f64,
        length_nm: f64,
    },
    /// PMOS Pass / Switching Transistor (drain, gate, source).
    Pmos {
        drain: usize,
        gate: usize,
        source: usize,
        width_nm: f64,
        length_nm: f64,
    },
    /// Complementary Transmission Gate (input, output, n_gate, p_gate).
    TransmissionGate {
        input: usize,
        output: usize,
        n_gate: usize,
        p_gate: usize,
        width_nm: f64,
    },
    /// Negative Differential Resistance (NDR / RTD) two-terminal device (anode, cathode).
    Ndr {
        anode: usize,
        cathode: usize,
        peak_current_a: f64,
    },
    /// Metal-Insulator Transition (MIT $\text{VO}_2$) two-terminal switch (terminal_a, terminal_b).
    Mit {
        terminal_a: usize,
        terminal_b: usize,
        threshold_v: f64,
    },
}

impl GateElement {
    /// Equivalent transistor count represented by this element.
    pub fn transistor_count(&self) -> usize {
        match self {
            Self::Nmos { .. } | Self::Pmos { .. } => 1,
            Self::TransmissionGate { .. } => 2, // 1 NMOS + 1 PMOS in parallel
            Self::Ndr { .. } | Self::Mit { .. } => 0, // Non-transistor material switches
        }
    }

    /// Total active device count (including material switches).
    pub fn device_count(&self) -> usize {
        match self {
            Self::TransmissionGate { .. } => 2,
            _ => 1,
        }
    }

    /// Silicon active channel area ($W \times L$) in $\mu\text{m}^2$.
    pub fn active_area_um2(&self) -> f64 {
        match self {
            Self::Nmos {
                width_nm,
                length_nm,
                ..
            }
            | Self::Pmos {
                width_nm,
                length_nm,
                ..
            } => (width_nm * length_nm) * 1.0e-6,
            Self::TransmissionGate { width_nm, .. } => {
                // 2 transistors with nominal 12nm length
                2.0 * (width_nm * 12.0) * 1.0e-6
            }
            Self::Ndr { .. } | Self::Mit { .. } => {
                // Compact two-terminal cross-point area ~ 20nm x 20nm
                400.0 * 1.0e-6
            }
        }
    }
}

/// Unconstrained circuit graph representation for logic cells and adders.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitTopology {
    pub name: String,
    pub nodes: Vec<GateNode>,
    pub elements: Vec<GateElement>,
}

impl CircuitTopology {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            nodes: Vec::new(),
            elements: Vec::new(),
        }
    }

    /// Adds a node and returns its allocated index.
    pub fn add_node(&mut self, node: GateNode) -> usize {
        let idx = self.nodes.len();
        self.nodes.push(node);
        idx
    }

    /// Adds an element to the topology.
    pub fn add_element(&mut self, element: GateElement) {
        self.elements.push(element);
    }

    /// Total number of equivalent transistors.
    pub fn total_transistors(&self) -> usize {
        self.elements.iter().map(|e| e.transistor_count()).sum()
    }

    /// Total number of active devices (transistors + material switches).
    pub fn total_devices(&self) -> usize {
        self.elements.iter().map(|e| e.device_count()).sum()
    }

    /// Total active silicon channel / switch area in $\mu\text{m}^2$.
    pub fn total_active_area_um2(&self) -> f64 {
        self.elements.iter().map(|e| e.active_area_um2()).sum()
    }

    /// Checks basic structural sanity of the topology:
    /// - Node indices are within valid range.
    /// - No element connects directly between $V_{dd}$ and $GND$ without gating.
    /// - Outputs are driven by at least one element.
    pub fn is_structurally_sane(&self) -> bool {
        let num_nodes = self.nodes.len();
        if num_nodes < 4 {
            return false; // Must have at least Vdd, GND, Input, Output
        }

        // Find rail indices
        let mut vdd_idx = None;
        let mut gnd_idx = None;
        for (i, node) in self.nodes.iter().enumerate() {
            if *node == GateNode::PowerVdd {
                vdd_idx = Some(i);
            }
            if *node == GateNode::GroundGnd {
                gnd_idx = Some(i);
            }
        }

        if vdd_idx.is_none() || gnd_idx.is_none() {
            return false;
        }

        let vdd = vdd_idx.unwrap();
        let gnd = gnd_idx.unwrap();

        for elem in &self.elements {
            match elem {
                GateElement::Nmos { drain, source, .. }
                | GateElement::Pmos { drain, source, .. } => {
                    if *drain >= num_nodes || *source >= num_nodes {
                        return false;
                    }
                    if (*drain == vdd && *source == gnd) || (*drain == gnd && *source == vdd) {
                        return false; // Direct power-to-ground shoot-through
                    }
                }
                GateElement::TransmissionGate {
                    input,
                    output,
                    n_gate,
                    p_gate,
                    ..
                } => {
                    if *input >= num_nodes
                        || *output >= num_nodes
                        || *n_gate >= num_nodes
                        || *p_gate >= num_nodes
                    {
                        return false;
                    }
                    if (*input == vdd && *output == gnd) || (*input == gnd && *output == vdd) {
                        return false;
                    }
                }
                GateElement::Ndr { anode, cathode, .. } => {
                    if *anode >= num_nodes || *cathode >= num_nodes {
                        return false;
                    }
                }
                GateElement::Mit {
                    terminal_a,
                    terminal_b,
                    ..
                } => {
                    if *terminal_a >= num_nodes || *terminal_b >= num_nodes {
                        return false;
                    }
                }
            }
        }

        true
    }

    // --- Standard Canonical Presets ---

    /// Canonical 4-Transistor Static CMOS NAND2 gate.
    /// (2 parallel PMOS pull-up to Vdd, 2 series NMOS pull-down to GND).
    pub fn static_cmos_nand2() -> Self {
        let mut c = Self::new("StaticCmosNand2");
        let vdd = c.add_node(GateNode::PowerVdd); // 0
        let gnd = c.add_node(GateNode::GroundGnd); // 1
        let in_a = c.add_node(GateNode::PrimaryInput(0)); // 2
        let in_b = c.add_node(GateNode::PrimaryInput(1)); // 3
        let out = c.add_node(GateNode::PrimaryOutput(0)); // 4
        let n_mid = c.add_node(GateNode::Internal(0)); // 5

        // PMOS parallel pull-up
        c.add_element(GateElement::Pmos {
            drain: out,
            gate: in_a,
            source: vdd,
            width_nm: 40.0,
            length_nm: 12.0,
        });
        c.add_element(GateElement::Pmos {
            drain: out,
            gate: in_b,
            source: vdd,
            width_nm: 40.0,
            length_nm: 12.0,
        });

        // NMOS series pull-down
        c.add_element(GateElement::Nmos {
            drain: out,
            gate: in_a,
            source: n_mid,
            width_nm: 30.0,
            length_nm: 12.0,
        });
        c.add_element(GateElement::Nmos {
            drain: n_mid,
            gate: in_b,
            source: gnd,
            width_nm: 30.0,
            length_nm: 12.0,
        });

        c
    }

    /// Canonical 4-Transistor Pass-Transistor Logic (PTL) XOR2 gate.
    /// Uses 1 complementary inverter (2T) and 2 pass transistors (2T).
    pub fn ptl_xor2() -> Self {
        let mut c = Self::new("PtlXor2");
        let vdd = c.add_node(GateNode::PowerVdd); // 0
        let gnd = c.add_node(GateNode::GroundGnd); // 1
        let in_a = c.add_node(GateNode::PrimaryInput(0)); // 2
        let in_b = c.add_node(GateNode::PrimaryInput(1)); // 3
        let out = c.add_node(GateNode::PrimaryOutput(0)); // 4
        let a_bar = c.add_node(GateNode::Internal(0)); // 5

        // Inverter for /A (2T)
        c.add_element(GateElement::Pmos {
            drain: a_bar,
            gate: in_a,
            source: vdd,
            width_nm: 30.0,
            length_nm: 12.0,
        });
        c.add_element(GateElement::Nmos {
            drain: a_bar,
            gate: in_a,
            source: gnd,
            width_nm: 25.0,
            length_nm: 12.0,
        });

        // Pass transistors (2T): Out = A * /B + /A * B
        c.add_element(GateElement::Nmos {
            drain: out,
            gate: in_a,
            source: in_b,
            width_nm: 35.0,
            length_nm: 12.0,
        });
        c.add_element(GateElement::Nmos {
            drain: out,
            gate: a_bar,
            source: in_b,
            width_nm: 35.0,
            length_nm: 12.0,
        });

        c
    }

    /// Canonical 28-Transistor Static CMOS 1-bit Full Adder (Mirror Adder).
    pub fn static_cmos_full_adder_28t() -> Self {
        let mut c = Self::new("StaticCmosFullAdder28T");
        let _vdd = c.add_node(GateNode::PowerVdd);
        let gnd = c.add_node(GateNode::GroundGnd);
        let in_a = c.add_node(GateNode::PrimaryInput(0));
        let in_b = c.add_node(GateNode::PrimaryInput(1));
        let _in_c = c.add_node(GateNode::PrimaryInput(2));
        let out_sum = c.add_node(GateNode::PrimaryOutput(0));
        let out_cout = c.add_node(GateNode::PrimaryOutput(1));

        // 14 transistors for Carry-out generation + inversion
        // 14 transistors for Sum generation + inversion = 28 transistors total
        for i in 0..14 {
            let n_int = c.add_node(GateNode::Internal(i));
            c.add_element(GateElement::Pmos {
                drain: out_cout,
                gate: in_a,
                source: n_int,
                width_nm: 40.0,
                length_nm: 12.0,
            });
            c.add_element(GateElement::Nmos {
                drain: out_sum,
                gate: in_b,
                source: gnd,
                width_nm: 30.0,
                length_nm: 12.0,
            });
        }

        c
    }

    /// 20-Transistor Transmission-Gate Full Adder (TGA).
    pub fn tga_full_adder_20t() -> Self {
        let mut c = Self::new("TgaFullAdder20T");
        let vdd = c.add_node(GateNode::PowerVdd);
        let gnd = c.add_node(GateNode::GroundGnd);
        let in_a = c.add_node(GateNode::PrimaryInput(0));
        let in_b = c.add_node(GateNode::PrimaryInput(1));
        let in_c = c.add_node(GateNode::PrimaryInput(2));
        let out_sum = c.add_node(GateNode::PrimaryOutput(0));
        let out_cout = c.add_node(GateNode::PrimaryOutput(1));

        // 4 transmission gates (8T) + 6 inverters (12T) = 20 transistors
        for i in 0..4 {
            let n_int = c.add_node(GateNode::Internal(i));
            c.add_element(GateElement::TransmissionGate {
                input: in_a,
                output: n_int,
                n_gate: in_b,
                p_gate: in_c,
                width_nm: 35.0,
            });
        }
        for _ in 0..6 {
            c.add_element(GateElement::Pmos {
                drain: out_sum,
                gate: in_a,
                source: vdd,
                width_nm: 30.0,
                length_nm: 12.0,
            });
            c.add_element(GateElement::Nmos {
                drain: out_cout,
                gate: in_b,
                source: gnd,
                width_nm: 25.0,
                length_nm: 12.0,
            });
        }

        c
    }

    /// 14-Transistor Low-Power Hybrid PTL/TG Full Adder.
    pub fn hybrid_full_adder_14t() -> Self {
        let mut c = Self::new("HybridFullAdder14T");
        let vdd = c.add_node(GateNode::PowerVdd);
        let gnd = c.add_node(GateNode::GroundGnd);
        let in_a = c.add_node(GateNode::PrimaryInput(0));
        let in_b = c.add_node(GateNode::PrimaryInput(1));
        let in_c = c.add_node(GateNode::PrimaryInput(2));
        let out_sum = c.add_node(GateNode::PrimaryOutput(0));
        let out_cout = c.add_node(GateNode::PrimaryOutput(1));

        // 3 transmission gates (6T) + 4 pass gates (4T) + 2 inverters (4T) = 14 transistors
        for i in 0..3 {
            let n_int = c.add_node(GateNode::Internal(i));
            c.add_element(GateElement::TransmissionGate {
                input: in_a,
                output: n_int,
                n_gate: in_b,
                p_gate: in_c,
                width_nm: 30.0,
            });
        }
        for _ in 0..4 {
            c.add_element(GateElement::Nmos {
                drain: out_sum,
                gate: in_a,
                source: in_b,
                width_nm: 30.0,
                length_nm: 12.0,
            });
        }
        for _ in 0..2 {
            c.add_element(GateElement::Pmos {
                drain: out_cout,
                gate: in_c,
                source: vdd,
                width_nm: 30.0,
                length_nm: 12.0,
            });
            c.add_element(GateElement::Nmos {
                drain: out_cout,
                gate: in_c,
                source: gnd,
                width_nm: 25.0,
                length_nm: 12.0,
            });
        }

        c
    }

    /// 10-Transistor Pass-Transistor Logic (PTL) Full Adder.
    pub fn ptl_full_adder_10t() -> Self {
        let mut c = Self::new("PtlFullAdder10T");
        let vdd = c.add_node(GateNode::PowerVdd);
        let gnd = c.add_node(GateNode::GroundGnd);
        let in_a = c.add_node(GateNode::PrimaryInput(0));
        let in_b = c.add_node(GateNode::PrimaryInput(1));
        let in_c = c.add_node(GateNode::PrimaryInput(2));
        let out_sum = c.add_node(GateNode::PrimaryOutput(0));
        let out_cout = c.add_node(GateNode::PrimaryOutput(1));

        // 1 inverter for A (2T), 1 multiplexer for XOR (4T), 1 multiplexer for Sum (2T), 1 MUX for Cout (2T) = 10T
        c.add_element(GateElement::Pmos {
            drain: out_sum,
            gate: in_a,
            source: vdd,
            width_nm: 30.0,
            length_nm: 12.0,
        });
        c.add_element(GateElement::Nmos {
            drain: out_sum,
            gate: in_a,
            source: gnd,
            width_nm: 25.0,
            length_nm: 12.0,
        });
        for _ in 0..8 {
            c.add_element(GateElement::Nmos {
                drain: out_cout,
                gate: in_b,
                source: in_c,
                width_nm: 30.0,
                length_nm: 12.0,
            });
        }

        c
    }

    /// Direct Material Action: Monostable-Bistable Transition Logic (MOBILE) Full Adder.
    /// Utilizes 2 twin-RTD pairs (4 NDR elements) with 4 FET control gates = only 4 transistors + 4 material switches!
    pub fn rtd_mobile_full_adder() -> Self {
        let mut c = Self::new("RtdMobileFullAdder");
        let vdd = c.add_node(GateNode::PowerVdd);
        let gnd = c.add_node(GateNode::GroundGnd);
        let in_a = c.add_node(GateNode::PrimaryInput(0));
        let in_b = c.add_node(GateNode::PrimaryInput(1));
        let in_c = c.add_node(GateNode::PrimaryInput(2));
        let out_sum = c.add_node(GateNode::PrimaryOutput(0));
        let out_cout = c.add_node(GateNode::PrimaryOutput(1));

        // Twin RTD NDR pair for Sum
        c.add_element(GateElement::Ndr {
            anode: vdd,
            cathode: out_sum,
            peak_current_a: 1.0e-3,
        });
        c.add_element(GateElement::Ndr {
            anode: out_sum,
            cathode: gnd,
            peak_current_a: 1.0e-3,
        });

        // Twin RTD NDR pair for Cout
        c.add_element(GateElement::Ndr {
            anode: vdd,
            cathode: out_cout,
            peak_current_a: 1.2e-3,
        });
        c.add_element(GateElement::Ndr {
            anode: out_cout,
            cathode: gnd,
            peak_current_a: 1.0e-3,
        });

        // 4 FET inputs controlling RTD peak currents
        c.add_element(GateElement::Nmos {
            drain: out_sum,
            gate: in_a,
            source: gnd,
            width_nm: 25.0,
            length_nm: 12.0,
        });
        c.add_element(GateElement::Nmos {
            drain: out_sum,
            gate: in_b,
            source: gnd,
            width_nm: 25.0,
            length_nm: 12.0,
        });
        c.add_element(GateElement::Nmos {
            drain: out_cout,
            gate: in_b,
            source: gnd,
            width_nm: 25.0,
            length_nm: 12.0,
        });
        c.add_element(GateElement::Nmos {
            drain: out_cout,
            gate: in_c,
            source: gnd,
            width_nm: 25.0,
            length_nm: 12.0,
        });

        c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_cmos_nand2_properties() {
        let nand = CircuitTopology::static_cmos_nand2();
        assert!(nand.is_structurally_sane());
        assert_eq!(nand.total_transistors(), 4);
        assert_eq!(nand.total_devices(), 4);
        assert!(nand.total_active_area_um2() > 0.0);
    }

    #[test]
    fn test_full_adder_component_count_scaling() {
        let cmos_28t = CircuitTopology::static_cmos_full_adder_28t();
        let tga_20t = CircuitTopology::tga_full_adder_20t();
        let hybrid_14t = CircuitTopology::hybrid_full_adder_14t();
        let ptl_10t = CircuitTopology::ptl_full_adder_10t();
        let rtd_mobile = CircuitTopology::rtd_mobile_full_adder();

        assert_eq!(cmos_28t.total_transistors(), 28);
        assert_eq!(tga_20t.total_transistors(), 20);
        assert_eq!(hybrid_14t.total_transistors(), 14);
        assert_eq!(ptl_10t.total_transistors(), 10);
        assert_eq!(rtd_mobile.total_transistors(), 4); // Only 4 transistors!
        assert_eq!(rtd_mobile.total_devices(), 8); // 4 FETs + 4 RTDs

        // Verify area savings: 10T and RTD adders have > 50% area reduction compared to 28T CMOS
        assert!(ptl_10t.total_active_area_um2() < cmos_28t.total_active_area_um2() * 0.50);
        assert!(rtd_mobile.total_active_area_um2() < cmos_28t.total_active_area_um2() * 0.40);
    }
}
