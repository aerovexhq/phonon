#![deny(unsafe_code)]

//! Power Delivery Network (PDN) & Ultra-High di/dt Dynamic Droop Co-Simulator.
//!
//! Multi-decade impedance profiling (VRM to on-die DTC), multi-stage LC droops (1st, 2nd, 3rd droop),
//! and active hardware mitigation via Digital Low-Dropout (DLDO) transient injection and clock-stretching.

pub mod dldo_mitigation;
pub mod dynamic_droop;
pub mod impedance_profile;

pub use dldo_mitigation::{
    simulate_mitigated_droop, ClockStretchParams, DldoControllerParams, MitigatedTransientResult,
    MitigationReport,
};
pub use dynamic_droop::{
    simulate_dynamic_droop, DroopStageMetrics, DynamicDroopResult, LoadStepProfile,
};
pub use impedance_profile::{
    calculate_pdn_impedance_profile, AntiResonancePeak, Complex, DecouplingCapSpec,
    ImpedanceSpectrumPoint, PdnImpedanceProfile, PdnNetworkParams, VrmModelParams,
};

/// High-level diagnostic telemetry report summarizing PDN and droop metrics.
#[derive(Debug, Clone)]
pub struct PdnTelemetryReport {
    pub v_dd_nominal_v: f64,
    pub target_impedance_mohm: f64,
    pub max_impedance_mohm: f64,
    pub impedance_compliant: bool,
    pub unmitigated_1st_droop_mv: f64,
    pub unmitigated_2nd_droop_mv: f64,
    pub unmitigated_3rd_droop_mv: f64,
    pub unmitigated_peak_droop_mv: f64,
    pub mitigated_peak_droop_mv: f64,
    pub droop_reduction_pct: f64,
    pub dldo_peak_current_a: f64,
    pub clock_stretching_triggered: bool,
    pub dc_ir_drop_mv: f64,
    pub is_overall_qualified: bool,
}

/// Unified Power Delivery Network & Dynamic Droop Co-Simulator.
#[derive(Debug, Clone)]
pub struct PdnDroopCoSimulator {
    pub params: PdnNetworkParams,
    pub load: LoadStepProfile,
    pub dldo: DldoControllerParams,
    pub stretch: ClockStretchParams,
}

impl Default for PdnDroopCoSimulator {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl PdnDroopCoSimulator {
    /// Creates a co-simulator with specified parameters.
    pub fn new(
        params: PdnNetworkParams,
        load: LoadStepProfile,
        dldo: DldoControllerParams,
        stretch: ClockStretchParams,
    ) -> Self {
        Self {
            params,
            load,
            dldo,
            stretch,
        }
    }

    /// Fast, non-blocking constructor that initializes default parameters in sub-microsecond time.
    pub fn new_fast() -> Self {
        Self {
            params: PdnNetworkParams::default(),
            load: LoadStepProfile::default(),
            dldo: DldoControllerParams::default(),
            stretch: ClockStretchParams::default(),
        }
    }

    /// Solves the multi-decade frequency impedance profile from 1 kHz to 1 GHz.
    pub fn solve_impedance(&self, points_per_decade: usize) -> PdnImpedanceProfile {
        calculate_pdn_impedance_profile(&self.params, points_per_decade)
    }

    /// Solves the 6-state multi-stage transient droop under high di/dt load step.
    pub fn solve_dynamic_droop(&self, num_points: usize) -> DynamicDroopResult {
        simulate_dynamic_droop(&self.params, &self.load, num_points)
    }

    /// Solves active hardware mitigation (DLDO transient current injection + clock-stretching).
    pub fn solve_mitigated(&self, num_points: usize) -> MitigatedTransientResult {
        simulate_mitigated_droop(&self.params, &self.load, &self.dldo, &self.stretch, num_points)
    }

    /// Evaluates DC resistive IR drop under maximum load current.
    pub fn dc_ir_drop_mv(&self) -> f64 {
        let total_current = self.load.i_base_a + self.load.delta_i_a;
        let total_r_ohm = self.params.vrm.r_dc_ohm
            + self.params.pcb_plane_resistance_ohm
            + self.params.package_resistance_ohm
            + self.params.on_die_grid_resistance_ohm;
        total_current * total_r_ohm * 1000.0
    }

    /// Generates a complete diagnostic telemetry report.
    pub fn generate_telemetry_report(&self) -> PdnTelemetryReport {
        let z_profile = self.solve_impedance(10);
        let mitigated_res = self.solve_mitigated(150);

        let unmit_peak = mitigated_res.report.unmitigated_peak_droop_mv;
        let mit_peak = mitigated_res.report.mitigated_peak_droop_mv;
        let droop_red_pct = mitigated_res.report.droop_reduction_pct;
        let budget_mv = self.params.vrm.v_dd_v * self.params.max_voltage_ripple_ratio * 1000.0;
        let is_qualified = mit_peak <= budget_mv;

        PdnTelemetryReport {
            v_dd_nominal_v: self.params.vrm.v_dd_v,
            target_impedance_mohm: z_profile.target_impedance_mohm,
            max_impedance_mohm: z_profile.max_impedance_mohm,
            impedance_compliant: z_profile.is_fully_compliant,
            unmitigated_1st_droop_mv: mitigated_res.report.unmitigated_1st_droop_mv,
            unmitigated_2nd_droop_mv: mitigated_res.unmitigated.second_droop.peak_droop_mv,
            unmitigated_3rd_droop_mv: mitigated_res.unmitigated.third_droop.peak_droop_mv,
            unmitigated_peak_droop_mv: unmit_peak,
            mitigated_peak_droop_mv: mit_peak,
            droop_reduction_pct: droop_red_pct,
            dldo_peak_current_a: mitigated_res.report.dldo_peak_current_a,
            clock_stretching_triggered: mitigated_res.report.clock_stretching_triggered,
            dc_ir_drop_mv: self.dc_ir_drop_mv(),
            is_overall_qualified: is_qualified,
        }
    }
}
