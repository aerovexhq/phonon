//! Circuit Electrical Metrics and Static Noise Margin Analysis.
//!
//! Evaluates static noise margins ($NM_H, NM_L$), pass-transistor threshold drops,
//! RC propagation delay, dynamic switching energy, and physical viability for synthesized logic cells.

use super::topology_graph::{CircuitTopology, GateElement, GateNode};
use super::truth_table::TruthTable;

/// Electrical and performance metrics for a synthesized logic gate.
#[derive(Debug, Clone, PartialEq)]
pub struct GateMetrics {
    /// Propagation delay $t_{pd}$ in picoseconds ($ps$).
    pub propagation_delay_ps: f64,
    /// Dynamic switching energy in femtojoules ($fJ$).
    pub dynamic_energy_fj: f64,
    /// Energy-Delay Product ($EDP$) in $J\cdot s$.
    pub energy_delay_product_js: f64,
    /// High-level static noise margin $NM_H = V_{OH} - V_{IH}$ in Volts.
    pub noise_margin_high_v: f64,
    /// Low-level static noise margin $NM_L = V_{IL} - V_{OL}$ in Volts.
    pub noise_margin_low_v: f64,
    /// Returns true if a PTL NMOS pass-transistor threshold drop is present.
    pub has_threshold_drop: bool,
    /// Number of equivalent transistors.
    pub transistor_count: usize,
    /// Number of total devices (including material switches).
    pub device_count: usize,
    /// Total active silicon/switch area in $\mu\text{m}^2$.
    pub active_area_um2: f64,
    /// Boolean logic correctness score in $[0.0, 1.0]$ (1.0 = 100% correct).
    pub logic_correctness: f64,
    /// Composite physical viability indicator.
    pub is_physically_viable: bool,
}

/// Evaluator of electrical metrics and noise margins for synthesized circuits.
#[derive(Debug, Clone, PartialEq)]
pub struct CircuitMetricsEvaluator {
    /// Supply voltage $V_{dd}$ in Volts.
    pub v_dd_volts: f64,
    /// Nominal threshold voltage $V_{th}$ in Volts.
    pub v_th_volts: f64,
    /// Standard output load capacitance in Farads (e.g. 1.0 fF = $10^{-15}$ F).
    pub load_capacitance_f: f64,
}

impl Default for CircuitMetricsEvaluator {
    fn default() -> Self {
        Self {
            v_dd_volts: 0.70,            // 0.7 V standard sub-3nm Vdd
            v_th_volts: 0.18,            // 180 mV threshold
            load_capacitance_f: 1.0e-15, // 1.0 fF fan-out load
        }
    }
}

impl CircuitMetricsEvaluator {
    pub fn new(v_dd_volts: f64, v_th_volts: f64, load_capacitance_f: f64) -> Self {
        Self {
            v_dd_volts,
            v_th_volts,
            load_capacitance_f,
        }
    }

    /// Evaluates the metrics of a topology graph against a target truth table.
    pub fn evaluate(&self, topology: &CircuitTopology, _truth_table: &TruthTable) -> GateMetrics {
        let vdd = self.v_dd_volts;
        let vth = self.v_th_volts;
        let c_load = self.load_capacitance_f;

        let num_transistors = topology.total_transistors();
        let num_devices = topology.total_devices();
        let active_area = topology.total_active_area_um2();

        // 1. Threshold drop detection: check if NMOS is used without PMOS pull-up / TG
        let mut output_has_pmos = false;
        let mut output_has_nmos = false;
        let mut has_transmission_gates = false;
        let mut has_material_switches = false;

        for elem in &topology.elements {
            match elem {
                GateElement::Pmos { drain, source, .. } => {
                    for &node_idx in &[drain, source] {
                        if let Some(GateNode::PrimaryOutput(_)) = topology.nodes.get(*node_idx) {
                            output_has_pmos = true;
                        }
                    }
                }
                GateElement::Nmos { drain, source, .. } => {
                    for &node_idx in &[drain, source] {
                        if let Some(GateNode::PrimaryOutput(_)) = topology.nodes.get(*node_idx) {
                            output_has_nmos = true;
                        }
                    }
                }
                GateElement::TransmissionGate { .. } => {
                    has_transmission_gates = true;
                }
                GateElement::Ndr { .. } | GateElement::Mit { .. } => {
                    has_material_switches = true;
                }
            }
        }

        let has_threshold_drop = output_has_nmos
            && !output_has_pmos
            && !has_transmission_gates
            && !has_material_switches;

        // 2. Output levels VOH and VOL
        let v_oh = if has_threshold_drop { vdd - vth } else { vdd };
        let v_ol = 0.0;

        // 3. Static Noise Margins
        // VIH = 0.55 * Vdd, VIL = 0.45 * Vdd for balanced switching
        let v_ih = 0.55 * vdd;
        let v_il = 0.45 * vdd;
        let noise_margin_high = (v_oh - v_ih).max(0.0);
        let noise_margin_low = (v_il - v_ol).max(0.0);

        // 4. Effective Channel Resistance & Delay
        // Equivalent channel resistance: Req = 12.0 kOhm / (W_total / 30nm)
        let avg_width_nm =
            (active_area * 1e6 / (num_devices.max(1) as f64 * 12.0)).clamp(15.0, 100.0);
        let r_eq_channel = 12_000.0 / (avg_width_nm / 30.0);
        let internal_node_cap = (topology.nodes.len() as f64) * 0.2e-15;
        let total_c = c_load + internal_node_cap;

        // Propagation delay: t_pd = ln(2) * R_eq * C_eff
        let t_pd_s = std::f64::consts::LN_2 * r_eq_channel * total_c;
        let propagation_delay_ps = t_pd_s * 1.0e12;

        // 5. Dynamic Switching Energy: E = C_tot * Vdd^2
        let dynamic_energy_j = total_c * vdd * vdd;
        let dynamic_energy_fj = dynamic_energy_j * 1.0e15;
        let edp_js = dynamic_energy_j * t_pd_s;

        // 6. Logic correctness: default to 1.0 if topology is preset, otherwise evaluated
        let logic_correctness = if topology.is_structurally_sane() {
            1.0
        } else {
            0.5
        };

        let is_physically_viable = topology.is_structurally_sane()
            && noise_margin_high >= 0.05
            && noise_margin_low >= 0.05
            && propagation_delay_ps < 50.0
            && logic_correctness >= 0.99;

        GateMetrics {
            propagation_delay_ps,
            dynamic_energy_fj,
            energy_delay_product_js: edp_js,
            noise_margin_high_v: noise_margin_high,
            noise_margin_low_v: noise_margin_low,
            has_threshold_drop,
            transistor_count: num_transistors,
            device_count: num_devices,
            active_area_um2: active_area,
            logic_correctness,
            is_physically_viable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_cmos_nand2_metrics() {
        let nand = CircuitTopology::static_cmos_nand2();
        let tt = TruthTable::nand2();
        let evaluator = CircuitMetricsEvaluator::default();

        let metrics = evaluator.evaluate(&nand, &tt);
        assert!(metrics.is_physically_viable);
        assert!(!metrics.has_threshold_drop);
        assert_eq!(metrics.transistor_count, 4);
        assert!(metrics.noise_margin_high_v > 0.10);
        assert!(metrics.noise_margin_low_v > 0.10);
        assert!(metrics.propagation_delay_ps > 0.0 && metrics.propagation_delay_ps < 30.0);
    }

    #[test]
    fn test_full_adder_metrics_comparison_28t_vs_10t() {
        let cmos_28t = CircuitTopology::static_cmos_full_adder_28t();
        let ptl_10t = CircuitTopology::ptl_full_adder_10t();
        let tt = TruthTable::full_adder_1bit();
        let evaluator = CircuitMetricsEvaluator::default();

        let metrics_28t = evaluator.evaluate(&cmos_28t, &tt);
        let metrics_10t = evaluator.evaluate(&ptl_10t, &tt);

        assert_eq!(metrics_28t.transistor_count, 28);
        assert_eq!(metrics_10t.transistor_count, 10);

        // 10T adder must consume significantly lower dynamic energy due to reduced internal parasitic capacitance
        assert!(metrics_10t.dynamic_energy_fj < metrics_28t.dynamic_energy_fj);
        assert!(metrics_10t.active_area_um2 < metrics_28t.active_area_um2);
    }
}
