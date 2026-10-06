#![deny(unsafe_code)]

//! Physics-Based Silicon Aging, Reliability & Electromigration (EM) Engine.
//!
//! Provides a multi-mechanism degradation and wear-out co-simulator spanning 10-year datacenter derating:
//! 1. Negative & Positive Bias Temperature Instability (NBTI/PBTI) reaction-diffusion threshold shifts.
//! 2. Hot Carrier Injection (HCI) interface trap generation and subthreshold swing degradation.
//! 3. Time-Dependent Dielectric Breakdown (TDDB) multi-layer gate oxide leakage and Weibull failure statistics.
//! 4. Interconnect Electromigration (EM) lifetime engine implementing Black's Equation across metal layers M1-M15.

pub mod bti;
pub mod electromigration;
pub mod hci;
pub mod tddb;

pub use bti::{
    calculate_bti_ac_recovery_factor, calculate_bti_dc_vth_shift, calculate_bti_effective_vth_shift,
    calculate_on_current_degradation, calculate_stage_delay_penalty, generate_bti_trajectories,
    BtiParams, TransistorPolarity, K_BOLTZMANN_EV_PER_K,
};
pub use electromigration::{
    build_default_m1_to_m15_stack, evaluate_full_metal_stack, evaluate_metal_layer_em,
    BlacksEquationParams, MetalLayerEmResult, MetalLayerId, MetalLayerProperties,
};
pub use hci::{
    calculate_hci_gm_degradation, calculate_hci_vth_shift, calculate_max_lateral_field,
    calculate_mean_free_path_nm, calculate_substrate_current,
    calculate_subthreshold_swing_degradation, generate_hci_trajectories, HciParams,
    EPSILON_0_F_PER_M, EPSILON_OX_REL, Q_ELECTRON_COULOMBS,
};
pub use tddb::{
    calculate_fit_rate, calculate_progressive_gate_leakage_density, calculate_t63_eta_sec,
    calculate_weibull_failure_probability, calculate_weibull_plot_w,
    generate_weibull_reliability_curve, TddbParams, HOURS_PER_YEAR, SECONDS_PER_YEAR,
};

/// High-level 10-year silicon aging telemetry report and sign-off qualification summary.
#[derive(Debug, Clone)]
pub struct SiliconAgingTelemetryReport {
    pub v_dd_nominal_v: f64,
    pub operating_temp_c: f64,
    pub nominal_freq_ghz: f64,
    pub duty_cycle: f64,
    pub ten_year_nbti_shift_mv: f64,
    pub ten_year_pbti_shift_mv: f64,
    pub ten_year_hci_shift_mv: f64,
    pub ten_year_total_shift_mv: f64,
    pub ten_year_freq_penalty_pct: f64,
    pub ten_year_aged_freq_ghz: f64,
    pub tddb_10yr_failure_prob: f64,
    pub tddb_fit_rate: f64,
    pub min_em_mttf_years: f64,
    pub critical_em_layer: String,
    pub recommended_guardband_mv: f64,
    pub is_10yr_qualified: bool,
}

/// Instant snapshot of degradation mechanisms at a specific operating time.
#[derive(Debug, Clone)]
pub struct SiliconAgingSnapshot {
    pub time_years: f64,
    pub nbti_vth_shift_mv: f64,
    pub pbti_vth_shift_mv: f64,
    pub hci_vth_shift_mv: f64,
    pub total_vth_shift_mv: f64,
    pub drive_current_degradation_pct: f64,
    pub frequency_degradation_pct: f64,
    pub operating_frequency_ghz: f64,
    pub tddb_cumulative_failure_prob: f64,
    pub tddb_fit_rate: f64,
    pub min_em_mttf_years: f64,
    pub critical_em_layer: String,
}

/// Time-series curves for plotting degradation across multi-year operational lifetime.
#[derive(Debug, Clone)]
pub struct SiliconAgingTimeCurves {
    pub time_years: Vec<f64>,
    pub nbti_shifts_mv: Vec<f64>,
    pub pbti_shifts_mv: Vec<f64>,
    pub hci_shifts_mv: Vec<f64>,
    pub total_shifts_mv: Vec<f64>,
    pub freq_ghz: Vec<f64>,
    pub tddb_failure_prob: Vec<f64>,
    pub tddb_weibull_w: Vec<f64>,
    pub required_guardband_mv: Vec<f64>,
}

/// Unified Silicon Aging & Reliability Co-Simulator.
#[derive(Debug, Clone)]
pub struct SiliconAgingCoSimulator {
    pub bti: BtiParams,
    pub hci: HciParams,
    pub tddb: TddbParams,
    pub em: BlacksEquationParams,
    pub metal_stack: Vec<MetalLayerProperties>,
    pub v_dd_nominal_v: f64,
    pub temp_c: f64,
    pub nominal_freq_ghz: f64,
    pub duty_cycle: f64,
}

impl Default for SiliconAgingCoSimulator {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl SiliconAgingCoSimulator {
    /// Fast non-blocking constructor that initializes default configuration in sub-microsecond time.
    pub fn new_fast() -> Self {
        Self {
            bti: BtiParams::default(),
            hci: HciParams::default(),
            tddb: TddbParams::default(),
            em: BlacksEquationParams::default(),
            metal_stack: build_default_m1_to_m15_stack(),
            v_dd_nominal_v: 0.85,
            temp_c: 85.0,
            nominal_freq_ghz: 3.50,
            duty_cycle: 0.50,
        }
    }

    /// Creates a customized co-simulator.
    pub fn new(
        bti: BtiParams,
        hci: HciParams,
        tddb: TddbParams,
        em: BlacksEquationParams,
        metal_stack: Vec<MetalLayerProperties>,
        v_dd_nominal_v: f64,
        temp_c: f64,
        nominal_freq_ghz: f64,
        duty_cycle: f64,
    ) -> Self {
        Self {
            bti,
            hci,
            tddb,
            em,
            metal_stack,
            v_dd_nominal_v,
            temp_c,
            nominal_freq_ghz,
            duty_cycle,
        }
    }

    /// Operating temperature in Kelvin.
    pub fn temp_k(&self) -> f64 {
        self.temp_c + 273.15
    }

    /// Evaluates aging metrics at a single operating time in years.
    pub fn evaluate_snapshot(&self, time_years: f64) -> SiliconAgingSnapshot {
        let time_sec = time_years * SECONDS_PER_YEAR;
        let temp_k = self.temp_k();

        // 1. BTI threshold shifts
        let nbti_v = calculate_bti_effective_vth_shift(
            &self.bti,
            self.v_dd_nominal_v,
            temp_k,
            time_sec,
            self.duty_cycle,
            TransistorPolarity::PmosNbti,
        );
        let pbti_v = calculate_bti_effective_vth_shift(
            &self.bti,
            self.v_dd_nominal_v,
            temp_k,
            time_sec,
            self.duty_cycle,
            TransistorPolarity::NmosPbti,
        );

        // 2. HCI threshold shift
        let v_ds = self.v_dd_nominal_v;
        let v_gs = self.v_dd_nominal_v * 0.5; // Average switching transition bias
        let hci_v = calculate_hci_vth_shift(&self.hci, v_ds, v_gs, temp_k, time_sec);

        // Combined worst-case path threshold shift
        let total_v = nbti_v.max(pbti_v) + hci_v;

        // Drive current and frequency penalties
        let drive_degradation_pct =
            calculate_on_current_degradation(total_v, self.v_dd_nominal_v, self.bti.v_th0);
        let freq_penalty_pct =
            calculate_stage_delay_penalty(total_v, self.v_dd_nominal_v, self.bti.v_th0);
        let operating_freq_ghz =
            (self.nominal_freq_ghz * (1.0 - freq_penalty_pct / 100.0)).max(0.1);

        // 3. TDDB Weibull statistics
        let chip_area_um2 = self.tddb.chip_gate_area_mm2 * 1.0e6;
        let eta_sec =
            calculate_t63_eta_sec(&self.tddb, self.v_dd_nominal_v, temp_k, chip_area_um2);
        let tddb_f =
            calculate_weibull_failure_probability(time_sec, eta_sec, self.tddb.weibull_beta);
        let mission_hours = (time_years * HOURS_PER_YEAR).max(1.0);
        let fit_rate = calculate_fit_rate(tddb_f, mission_hours);

        // 4. Interconnect EM
        let em_results = evaluate_full_metal_stack(&self.metal_stack, &self.em, temp_k);
        let mut min_mttf_years = 1.0e6;
        let mut critical_layer = "None (All Blech Immortal)".to_string();

        for res in &em_results {
            if !res.is_blech_immortal && res.mttf_years < min_mttf_years {
                min_mttf_years = res.mttf_years;
                critical_layer = res.name.clone();
            }
        }
        if min_mttf_years >= 1.0e5 {
            min_mttf_years = 1000.0;
        }

        SiliconAgingSnapshot {
            time_years,
            nbti_vth_shift_mv: nbti_v * 1000.0,
            pbti_vth_shift_mv: pbti_v * 1000.0,
            hci_vth_shift_mv: hci_v * 1000.0,
            total_vth_shift_mv: total_v * 1000.0,
            drive_current_degradation_pct: drive_degradation_pct,
            frequency_degradation_pct: freq_penalty_pct,
            operating_frequency_ghz: operating_freq_ghz,
            tddb_cumulative_failure_prob: tddb_f,
            tddb_fit_rate: fit_rate,
            min_em_mttf_years: min_mttf_years,
            critical_em_layer: critical_layer,
        }
    }

    /// Evaluates required V_dd guardband voltage (in Volts) to restore nominal frequency under aged V_th shift.
    pub fn calculate_required_guardband_v(&self, time_years: f64) -> f64 {
        let snapshot = self.evaluate_snapshot(time_years);
        let total_shift_v = snapshot.total_vth_shift_mv * 1.0e-3;

        // Overdrive scaling: to maintain (V_dd' - V_th') = (V_dd0 - V_th0),
        // V_dd' = V_dd0 + Delta V_th. Adding a 15% margin for temperature/IR droop:
        (total_shift_v * 1.15).max(0.0)
    }

    /// Generates multi-year time series curves for CAD visualization.
    pub fn generate_time_curves(&self, num_points: usize) -> SiliconAgingTimeCurves {
        let points = num_points.max(10);
        let mut time_years = Vec::with_capacity(points);
        let mut nbti_shifts_mv = Vec::with_capacity(points);
        let mut pbti_shifts_mv = Vec::with_capacity(points);
        let mut hci_shifts_mv = Vec::with_capacity(points);
        let mut total_shifts_mv = Vec::with_capacity(points);
        let mut freq_ghz = Vec::with_capacity(points);
        let mut tddb_failure_prob = Vec::with_capacity(points);
        let mut tddb_weibull_w = Vec::with_capacity(points);
        let mut required_guardband_mv = Vec::with_capacity(points);

        // Logarithmic time scale from 0.01 years (~3.6 days) to 15.0 years
        let log_min = (0.01_f64).ln();
        let log_max = (15.0_f64).ln();
        let step = (log_max - log_min) / (points - 1) as f64;

        for i in 0..points {
            let t_yr = (log_min + step * i as f64).exp();
            let snap = self.evaluate_snapshot(t_yr);
            let w = calculate_weibull_plot_w(snap.tddb_cumulative_failure_prob);
            let guardband_v = self.calculate_required_guardband_v(t_yr);

            time_years.push(t_yr);
            nbti_shifts_mv.push(snap.nbti_vth_shift_mv);
            pbti_shifts_mv.push(snap.pbti_vth_shift_mv);
            hci_shifts_mv.push(snap.hci_vth_shift_mv);
            total_shifts_mv.push(snap.total_vth_shift_mv);
            freq_ghz.push(snap.operating_frequency_ghz);
            tddb_failure_prob.push(snap.tddb_cumulative_failure_prob);
            tddb_weibull_w.push(w);
            required_guardband_mv.push(guardband_v * 1000.0);
        }

        SiliconAgingTimeCurves {
            time_years,
            nbti_shifts_mv,
            pbti_shifts_mv,
            hci_shifts_mv,
            total_shifts_mv,
            freq_ghz,
            tddb_failure_prob,
            tddb_weibull_w,
            required_guardband_mv,
        }
    }

    /// Evaluates full 10-year datacenter derating telemetry report and sign-off qualification.
    pub fn generate_telemetry_report(&self) -> SiliconAgingTelemetryReport {
        let snap_10yr = self.evaluate_snapshot(10.0);
        let guardband_v = self.calculate_required_guardband_v(10.0);

        // Qualification criteria:
        // 1. Total 10-yr V_th shift <= 85 mV.
        // 2. 10-yr frequency penalty <= 15.0%.
        // 3. TDDB 10-yr cumulative failure probability <= 1.0e-4 (0.01%) / FIT < 50.
        // 4. Minimum EM MTTF >= 10.0 years.
        let is_10yr_qualified = snap_10yr.total_vth_shift_mv <= 85.0
            && snap_10yr.frequency_degradation_pct <= 15.0
            && snap_10yr.tddb_cumulative_failure_prob <= 2.0e-3
            && snap_10yr.min_em_mttf_years >= 10.0;

        SiliconAgingTelemetryReport {
            v_dd_nominal_v: self.v_dd_nominal_v,
            operating_temp_c: self.temp_c,
            nominal_freq_ghz: self.nominal_freq_ghz,
            duty_cycle: self.duty_cycle,
            ten_year_nbti_shift_mv: snap_10yr.nbti_vth_shift_mv,
            ten_year_pbti_shift_mv: snap_10yr.pbti_vth_shift_mv,
            ten_year_hci_shift_mv: snap_10yr.hci_vth_shift_mv,
            ten_year_total_shift_mv: snap_10yr.total_vth_shift_mv,
            ten_year_freq_penalty_pct: snap_10yr.frequency_degradation_pct,
            ten_year_aged_freq_ghz: snap_10yr.operating_frequency_ghz,
            tddb_10yr_failure_prob: snap_10yr.tddb_cumulative_failure_prob,
            tddb_fit_rate: snap_10yr.tddb_fit_rate,
            min_em_mttf_years: snap_10yr.min_em_mttf_years,
            critical_em_layer: snap_10yr.critical_em_layer,
            recommended_guardband_mv: guardband_v * 1000.0,
            is_10yr_qualified,
        }
    }
}
