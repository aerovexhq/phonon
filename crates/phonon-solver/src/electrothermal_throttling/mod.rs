#![deny(unsafe_code)]

pub mod advanced_cooling;
pub mod dvfs_controller;
pub mod thermal_runaway;

pub use advanced_cooling::{
    calculate_immersion_performance, calculate_microchannel_performance, calculate_rohsenow_heat_flux,
    calculate_zuber_chf, generate_boiling_curve, BoilingCurvePoint, CoolantFluid,
    ImmersionCoolingParams, ImmersionFluidKind, ImmersionFluidProperties, ImmersionPerformance,
    LiquidCoolantProperties, MicrochannelParams, MicrochannelPerformance, TimAgingModel,
    TimAgingPoint, COPPER_THERMAL_CONDUCTIVITY, GRAVITY_ACCEL,
};

pub use dvfs_controller::{
    run_closed_loop_transient, DvfsControllerConfig, PState, TransientSimulationResult,
    TransientStepRecord, WorkloadProfile,
};

pub use thermal_runaway::{
    calculate_dynamic_power, calculate_leakage_current, calculate_leakage_power,
    calculate_leakage_temp_derivative, generate_bifurcation_curve, solve_thermal_equilibrium,
    BifurcationCurve, DynamicPowerParams, EquilibriumResult, LeakageModelParams,
    ThermalStabilityStatus, K_BOLTZMANN, Q_ELEM, T_ZERO_CELSIUS_K,
};

/// High-level selection of primary datacenter cooling architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoolingArchitecture {
    LiquidMicrochannelColdPlate,
    DielectricTwoPhaseImmersion,
    ForcedAirHeatsink,
}

/// Comprehensive diagnostic telemetry report for the electro-thermal throttling co-simulator.
#[derive(Debug, Clone)]
pub struct ThrottlingTelemetryReport {
    pub cooling_architecture: CoolingArchitecture,
    pub junction_temperature_c: f64,
    pub ambient_temperature_c: f64,
    pub dynamic_power_w: f64,
    pub leakage_power_w: f64,
    pub total_power_w: f64,
    pub leakage_fraction_pct: f64,
    pub stability_factor: f64,
    pub stability_status: ThermalStabilityStatus,
    pub total_thermal_resistance_k_w: f64,
    pub tim_thermal_resistance_k_w: f64,
    pub cooling_thermal_resistance_k_w: f64,
    pub current_frequency_ghz: f64,
    pub current_voltage_v: f64,
    pub active_p_state_name: String,
    pub is_throttling_active: bool,
    pub is_clock_gating_active: bool,
    pub chf_margin_pct: Option<f64>,
}

/// Primary coordinator for closed-loop dynamic electro-thermal simulation.
#[derive(Debug, Clone)]
pub struct ElectrothermalCoSimulator {
    pub leak_params: LeakageModelParams,
    pub dyn_params: DynamicPowerParams,
    pub dvfs_config: DvfsControllerConfig,
    pub cooling_architecture: CoolingArchitecture,
    pub microchannel: MicrochannelParams,
    pub immersion: ImmersionCoolingParams,
    pub tim_aging: TimAgingModel,
    pub spreader_resistance_k_w: f64,
    pub air_heatsink_resistance_k_w: f64,
    pub ambient_temp_c: f64,
    pub die_thermal_mass_j_k: f64,
}

impl Default for ElectrothermalCoSimulator {
    fn default() -> Self {
        Self::new_fast()
    }
}

impl ElectrothermalCoSimulator {
    /// Instant, non-blocking constructor with sub-millisecond initialization latency.
    pub fn new_fast() -> Self {
        Self {
            leak_params: LeakageModelParams::default(),
            dyn_params: DynamicPowerParams::default(),
            dvfs_config: DvfsControllerConfig::default(),
            cooling_architecture: CoolingArchitecture::LiquidMicrochannelColdPlate,
            microchannel: MicrochannelParams::default(),
            immersion: ImmersionCoolingParams::default(),
            tim_aging: TimAgingModel::default(),
            spreader_resistance_k_w: 0.025,
            air_heatsink_resistance_k_w: 0.22,
            ambient_temp_c: 25.0,
            die_thermal_mass_j_k: 0.085,
        }
    }

    /// Evaluates the convective/caloric thermal resistance of the active cooling mechanism.
    pub fn cooling_thermal_resistance(&self) -> f64 {
        match self.cooling_architecture {
            CoolingArchitecture::LiquidMicrochannelColdPlate => {
                let perf = calculate_microchannel_performance(&self.microchannel);
                perf.total_coldplate_resistance_k_w
            }
            CoolingArchitecture::DielectricTwoPhaseImmersion => {
                // Estimated under baseline operating temperature
                let perf = calculate_immersion_performance(&self.immersion, 75.0);
                perf.boiling_thermal_resistance_k_w
            }
            CoolingArchitecture::ForcedAirHeatsink => self.air_heatsink_resistance_k_w,
        }
    }

    /// Evaluates the total package thermal resistance: R_th = R_TIM + R_spreader + R_cooling.
    pub fn total_thermal_resistance(&self) -> f64 {
        let r_tim = self.tim_aging.total_resistance_k_w();
        let r_spreader = if self.cooling_architecture == CoolingArchitecture::DielectricTwoPhaseImmersion {
            // In direct immersion, fluid directly cools the die/lid with zero or minimal spreader resistance
            0.005
        } else {
            self.spreader_resistance_k_w
        };
        let r_cooling = self.cooling_thermal_resistance();
        r_tim + r_spreader + r_cooling
    }

    /// Computes full diagnostic telemetry report.
    pub fn compute_telemetry(&self) -> ThrottlingTelemetryReport {
        let r_th = self.total_thermal_resistance();
        let r_tim = self.tim_aging.total_resistance_k_w();
        let r_cooling = self.cooling_thermal_resistance();

        // Check equilibrium at nominal frequency
        let f_nom_ghz = self.dvfs_config.f_max_ghz;
        let v_nom = self.dvfs_config.voltage_for_frequency(f_nom_ghz);

        let eq = solve_thermal_equilibrium(
            &self.leak_params,
            &self.dyn_params,
            v_nom,
            f_nom_ghz * 1e9,
            1.0,
            self.ambient_temp_c,
            r_th,
        );

        let is_throttling = eq.junction_temp_c > self.dvfs_config.target_temp_c;
        let is_clock_gating = eq.junction_temp_c >= self.dvfs_config.critical_temp_c;

        // Current regulated frequency
        let current_freq = if is_throttling {
            let temp_diff = eq.junction_temp_c - self.dvfs_config.target_temp_c;
            (f_nom_ghz - self.dvfs_config.kp * temp_diff).clamp(self.dvfs_config.f_min_ghz, self.dvfs_config.f_max_ghz)
        } else {
            f_nom_ghz
        };
        let current_volt = self.dvfs_config.voltage_for_frequency(current_freq);
        let p_state_idx = self.dvfs_config.select_p_state(current_freq);
        let p_state_name = self.dvfs_config.p_states.get(p_state_idx).map(|p| p.name.clone()).unwrap_or_else(|| "Custom".to_string());

        let chf_margin = if self.cooling_architecture == CoolingArchitecture::DielectricTwoPhaseImmersion {
            let perf = calculate_immersion_performance(&self.immersion, eq.junction_temp_c);
            Some(perf.chf_margin_percentage)
        } else {
            None
        };

        ThrottlingTelemetryReport {
            cooling_architecture: self.cooling_architecture,
            junction_temperature_c: eq.junction_temp_c,
            ambient_temperature_c: self.ambient_temp_c,
            dynamic_power_w: eq.dynamic_power_w,
            leakage_power_w: eq.leakage_power_w,
            total_power_w: eq.total_power_w,
            leakage_fraction_pct: eq.leakage_percentage,
            stability_factor: eq.stability_factor,
            stability_status: eq.status,
            total_thermal_resistance_k_w: r_th,
            tim_thermal_resistance_k_w: r_tim,
            cooling_thermal_resistance_k_w: r_cooling,
            current_frequency_ghz: current_freq,
            current_voltage_v: current_volt,
            active_p_state_name: p_state_name,
            is_throttling_active: is_throttling,
            is_clock_gating_active: is_clock_gating,
            chf_margin_pct: chf_margin,
        }
    }

    /// Runs dynamic closed-loop transient simulation.
    pub fn run_transient_simulation(&self) -> TransientSimulationResult {
        let r_th = self.total_thermal_resistance();
        let workload = WorkloadProfile::default();
        run_closed_loop_transient(
            &self.dvfs_config,
            &self.leak_params,
            &self.dyn_params,
            &workload,
            self.ambient_temp_c,
            r_th,
            self.die_thermal_mass_j_k,
            50.0, // 50 ms total duration
            0.1,  // 0.1 ms step
        )
    }

    /// Generates bifurcation sweep curves for thermal runaway analysis.
    pub fn generate_bifurcation_curve(&self, num_points: usize) -> BifurcationCurve {
        let r_th = self.total_thermal_resistance();
        let f_nom_ghz = self.dvfs_config.f_max_ghz;
        let v_nom = self.dvfs_config.voltage_for_frequency(f_nom_ghz);

        thermal_runaway::generate_bifurcation_curve(
            &self.leak_params,
            &self.dyn_params,
            v_nom,
            f_nom_ghz * 1e9,
            1.0,
            self.ambient_temp_c,
            r_th,
            num_points,
        )
    }

    /// Generates two-phase immersion boiling curve.
    pub fn generate_boiling_curve(&self, num_points: usize) -> Vec<BoilingCurvePoint> {
        advanced_cooling::generate_boiling_curve(&self.immersion, num_points)
    }

    /// Generates TIM pump-out aging curve over power cycles.
    pub fn generate_tim_aging_curve(&self, max_cycles: usize, num_points: usize) -> Vec<TimAgingPoint> {
        self.tim_aging.generate_aging_trajectory(max_cycles, num_points)
    }

    /// Computes microchannel hydrodynamic and thermal performance.
    pub fn compute_microchannel_performance(&self) -> MicrochannelPerformance {
        calculate_microchannel_performance(&self.microchannel)
    }

    /// Computes two-phase immersion boiling performance.
    pub fn compute_immersion_performance(&self, surface_temp_c: f64) -> ImmersionPerformance {
        calculate_immersion_performance(&self.immersion, surface_temp_c)
    }
}
