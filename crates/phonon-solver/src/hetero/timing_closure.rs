//! Static timing closure, critical path slack, and RC interconnect delay analyzer.
//!
//! Evaluates:
//! 1. Stage-by-stage combinational logic gate delays based on material carrier injection velocity.
//! 2. Distributed Elmore interconnect delay comparing conventional Copper vs Carbon Nanotube (CNT) bundles.
//! 3. Setup/hold timing slack, clock skew, and maximum achievable processor clock frequency \(F_{max}\).

use phonon_models::hetero::{HeteroMaterialType, ProcessorBlockType, ProcessorFloorplan};

/// Pipeline timing closure report for a processor architecture.
#[derive(Debug, Clone)]
pub struct PipelineTimingReport {
    /// Maximum clock frequency F_max [GHz]
    pub f_max_ghz: f64,
    /// Critical path clock cycle period T_clk [ps]
    pub t_clk_ps: f64,
    /// Stage identifier limiting the clock frequency (bottleneck stage)
    pub critical_stage: ProcessorBlockType,
    /// Delay of each pipeline stage [ps]
    pub stage_delays_ps: Vec<(ProcessorBlockType, f64)>,
    /// Global clock distribution skew [ps]
    pub clock_skew_ps: f64,
    /// Clock jitter margin [ps]
    pub clock_jitter_ps: f64,
}

/// Timing analyzer evaluating pipeline latency and interconnect RC delays.
#[derive(Debug, Clone)]
pub struct TimingPathAnalyzer {
    v_dd: f64,
    setup_time_ps: f64,
    clock_jitter_ps: f64,
}

impl Default for TimingPathAnalyzer {
    fn default() -> Self {
        Self {
            v_dd: 0.8,
            setup_time_ps: 8.0,
            clock_jitter_ps: 4.0,
        }
    }
}

impl TimingPathAnalyzer {
    /// Creates a new timing analyzer for supply voltage Vdd.
    pub fn new(v_dd: f64) -> Self {
        Self {
            v_dd: v_dd.max(0.4),
            setup_time_ps: 8.0,
            clock_jitter_ps: 4.0,
        }
    }

    /// Supply voltage Vdd [V].
    pub fn v_dd(&self) -> f64 {
        self.v_dd
    }

    /// Evaluates intrinsic gate delay [ps] for a device material.
    ///
    /// In nanometer transistors, gate delay is governed by carrier thermal injection velocity
    /// and supply voltage overdrive:
    /// \[\tau_{gate} \propto \frac{C_L V_{dd}}{I_{on}} \propto \frac{L_g}{v_{inj}} \cdot \frac{V_{dd, nominal}}{V_{dd}}\]
    pub fn evaluate_gate_delay_ps(&self, material: HeteroMaterialType) -> f64 {
        let props = material.properties();
        let base_delay_si_ps = 2.4; // 2.4 ps for 3nm Silicon GAA at nominal 0.8V
        let v_inj_si = HeteroMaterialType::SiliconGaa
            .properties()
            .injection_velocity_cm_s;
        let vdd_scaling = 0.8 / self.v_dd;

        // Gate delay scales inversely with injection velocity and directly with Vdd scaling factor
        base_delay_si_ps * (v_inj_si / props.injection_velocity_cm_s.max(1e5)) * vdd_scaling
    }

    /// Evaluates distributed RC Elmore interconnect delay [ps] across distance L [um].
    pub fn evaluate_interconnect_delay_ps(
        &self,
        material: HeteroMaterialType,
        wire_length_um: f64,
    ) -> f64 {
        let l_m = wire_length_um * 1.0e-6;
        let props = material.properties();

        if material == HeteroMaterialType::CntBundleInterconnect {
            // Ballistic transport in CNT bundles:
            // High Fermi velocity vf ~ 8e5 m/s, reduced kinetic inductance and quantum capacitance
            // Propagation velocity v_prop ~ 1/3 speed of light in dielectric ~ 100 um / ps
            let t_flight_ps = wire_length_um / 100.0;
            let quantum_rc_ps = 0.5; // Minimal quantum contact RC
            t_flight_ps + quantum_rc_ps
        } else {
            // Conventional diffusive Copper / barrier wire
            let w_m = 30.0e-9; // 30 nm width
            let h_m = 60.0e-9; // 60 nm height
            let a_wire = w_m * h_m;
            let r_wire = props.interconnect_resistivity_ohm_m * (l_m / a_wire);
            let c_per_m = 1.8e-10; // 0.18 pF/mm
            let c_wire = c_per_m * l_m;

            // Elmore delay: tau = 0.5 * R_wire * C_wire [s] -> / 1e-12 for ps
            let elmore_delay_s = 0.5 * r_wire * c_wire;
            let driver_rc_s = 200.0 * c_wire; // 200 ohm equivalent driver resistance
            (elmore_delay_s + driver_rc_s) * 1.0e12
        }
    }

    /// Evaluates timing closure across all pipeline stages of the processor floorplan.
    pub fn evaluate_floorplan_timing(
        &self,
        floorplan: &ProcessorFloorplan,
    ) -> PipelineTimingReport {
        // Pipeline logic depths (number of FO4 equivalent gate stages per stage)
        let pipeline_stages = [
            (ProcessorBlockType::InstructionFetch, 14.0, 80.0), // (type, gate_depth, bus_length_um)
            (ProcessorBlockType::InstructionDecode, 12.0, 90.0),
            (ProcessorBlockType::ExecutionAlu, 20.0, 100.0), // Deepest combinational path (64-bit ALU)
            (ProcessorBlockType::MemoryAccess, 16.0, 110.0),
            (ProcessorBlockType::WriteBack, 10.0, 60.0),
        ];

        let mut stage_delays = Vec::new();
        let mut max_delay_ps = 0.0;
        let mut critical_stage = ProcessorBlockType::ExecutionAlu;

        // Clock distribution skew depends on the clock spine material
        let clock_block = floorplan.get_block(ProcessorBlockType::ClockDistribution);
        let clock_material = clock_block
            .map(|b| b.material)
            .unwrap_or(HeteroMaterialType::SiliconGaa);

        let clock_skew_ps = if clock_material == HeteroMaterialType::CntBundleInterconnect {
            3.5 // Ultra-low skew in ballistic CNT H-tree
        } else {
            7.5 // Standard copper clock spine skew
        };

        for (block_type, depth, bus_um) in pipeline_stages {
            let block = floorplan.get_block(block_type);
            let material = block
                .map(|b| b.material)
                .unwrap_or(HeteroMaterialType::SiliconGaa);

            let tau_gate = self.evaluate_gate_delay_ps(material);
            let logic_delay_ps = depth * tau_gate;

            let wire_material = if clock_material == HeteroMaterialType::CntBundleInterconnect {
                HeteroMaterialType::CntBundleInterconnect
            } else {
                HeteroMaterialType::SiliconGaa
            };
            let wire_delay_ps = self.evaluate_interconnect_delay_ps(wire_material, bus_um);

            let total_stage_delay_ps = logic_delay_ps + wire_delay_ps;
            stage_delays.push((block_type, total_stage_delay_ps));

            if total_stage_delay_ps > max_delay_ps {
                max_delay_ps = total_stage_delay_ps;
                critical_stage = block_type;
            }
        }

        // Clock period T_clk = max_stage_delay + t_setup + t_skew + t_jitter
        let t_clk_ps = max_delay_ps + self.setup_time_ps + clock_skew_ps + self.clock_jitter_ps;
        let f_max_ghz = 1000.0 / t_clk_ps; // 1000 ps = 1 ns -> 1/ns = 1 GHz

        PipelineTimingReport {
            f_max_ghz,
            t_clk_ps,
            critical_stage,
            stage_delays_ps: stage_delays,
            clock_skew_ps,
            clock_jitter_ps: self.clock_jitter_ps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use phonon_models::hetero::{BlockAllocationMap, RiscVFloorplanBuilder};

    #[test]
    fn test_gate_delay_scaling_ingaas_vs_silicon() {
        let analyzer = TimingPathAnalyzer::default();
        let tau_si = analyzer.evaluate_gate_delay_ps(HeteroMaterialType::SiliconGaa);
        let tau_ingaas = analyzer.evaluate_gate_delay_ps(HeteroMaterialType::InGaAsNmos);

        // InGaAs gate delay should be significantly lower (< 50% of Silicon)
        assert!(tau_ingaas < tau_si * 0.55);
    }

    #[test]
    fn test_interconnect_delay_cnt_vs_copper() {
        let analyzer = TimingPathAnalyzer::default();
        let wire_um = 200.0;
        let t_cu = analyzer.evaluate_interconnect_delay_ps(HeteroMaterialType::SiliconGaa, wire_um);
        let t_cnt = analyzer
            .evaluate_interconnect_delay_ps(HeteroMaterialType::CntBundleInterconnect, wire_um);

        // CNT bundle interconnect delay must be significantly faster than copper
        assert!(t_cnt < t_cu * 0.50);
    }

    #[test]
    fn test_floorplan_timing_closure_speedup() {
        let alloc_si = BlockAllocationMap::uniform_silicon();
        let alloc_hetero = BlockAllocationMap::synthesized_heterogeneous();

        let fp_si = RiscVFloorplanBuilder::build(&alloc_si);
        let fp_hetero = RiscVFloorplanBuilder::build(&alloc_hetero);

        let analyzer = TimingPathAnalyzer::default();
        let timing_si = analyzer.evaluate_floorplan_timing(&fp_si);
        let timing_hetero = analyzer.evaluate_floorplan_timing(&fp_hetero);

        // Heterogeneous processor must achieve higher F_max (> 25% speedup)
        assert!(timing_hetero.f_max_ghz > timing_si.f_max_ghz * 1.25);
        assert!(timing_hetero.t_clk_ps < timing_si.t_clk_ps);
    }
}
