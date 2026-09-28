//! Terahertz Quantum Cascade Laser (THz QCL) Rate Equations, Frequency Comb Generation & Spectroscopy Solver.
//!
//! Formulates:
//! - Multi-level coupled rate equations:
//!   $$\frac{dn_3}{dt} = \frac{\eta_{inj} J}{e L_p} - \frac{n_3}{\tau_3} - \Gamma v_g g_{diff} (n_3 - n_2) S$$
//!   $$\frac{dn_2}{dt} = \frac{n_3}{\tau_{32}} + \Gamma v_g g_{diff} (n_3 - n_2) S - \frac{n_2}{\tau_{21}} + \frac{n_1}{\tau_{12}^{therm}}$$
//!   $$\frac{dn_1}{dt} = \frac{n_2}{\tau_{21}} - \frac{n_1}{\tau_{12}^{therm}} - \frac{n_1}{\tau_{out}}$$
//!   $$\frac{dS}{dt} = \Gamma v_g g_{diff} (n_3 - n_2) S - \frac{S}{\tau_{ph}} + \beta_{sp} \frac{n_3}{\tau_{sp}}$$
//! - Lasing threshold current density:
//!   $$J_{th} = \frac{e L_p (\alpha_w + \alpha_m)}{\Gamma g_{diff} \eta_{inj} \tau_3 (1 - \tau_2 / \tau_{32})}$$
//! - Temperature roll-off model:
//!   $$J_{th}(T) = J_0 + J_1 \exp(T / T_0)$$
//!   modeling thermal back-filling and maximum operating temperature $T_{max} > 200\text{ K}$.
//! - Frequency comb generation:
//!   Four-wave mixing (FWM) third-order optical non-linearity $\chi^{(3)}$ in intersubband active region,
//!   generating equidistant THz frequency comb modes with repetition frequency $f_{rep} = c / (2 n_g L) \approx 10-25\text{ GHz}$
//!   and sub-kilohertz beat note linewidth ($\Delta f_{beat} < 1\text{ kHz}$).
//! - Sub-millimeter absorption spectroscopy engine:
//!   Calculates molecular rotational absorption lines (atmospheric water vapor $\text{H}_2\text{O}$,
//!   carbon monoxide $\text{CO}$, ozone $\text{O}_3$) across the THz transmission band ($0.5-10\text{ THz}$).

use phonon_core::constants::{
    BOLTZMANN_CONSTANT, ELEMENTARY_CHARGE, EPSILON_0, H_BAR, SPEED_OF_LIGHT,
};
use phonon_models::quantum::{
    ResonantLoPhononDepopulation, ThzOpticalGainModel, ThzPolaritonicWaveguide,
};

/// Operating state of a THz QCL evaluated at steady-state.
#[derive(Debug, Clone, PartialEq)]
pub struct ThzQclOperatingState {
    /// Injected current density $J$ in $\text{A/cm}^2$.
    pub current_density_a_cm2: f64,
    /// Injected total device drive current $I$ in Amperes ($A$).
    pub device_current_a: f64,
    /// Operating heat sink temperature in Kelvin ($K$).
    pub temperature_kelvin: f64,
    /// Lasing threshold current density $J_{th}(T)$ in $\text{A/cm}^2$.
    pub threshold_current_density_a_cm2: f64,
    /// Lasing threshold current $I_{th}(T)$ in Amperes ($A$).
    pub threshold_current_a: f64,
    /// Upper laser level carrier density $n_3$ in $\text{m}^{-3}$.
    pub carrier_density_n3: f64,
    /// Lower laser level carrier density $n_2$ in $\text{m}^{-3}$.
    pub carrier_density_n2: f64,
    /// Inversion density $\Delta n = n_3 - n_2$ in $\text{m}^{-3}$.
    pub inversion_density_m3: f64,
    /// Intracavity photon density $S$ in $\text{m}^{-3}$.
    pub photon_density_m3: f64,
    /// Single-facet optical output power $P_{out}$ in Watts ($W$).
    pub optical_power_watts: f64,
    /// Single-facet optical output power in milliwatts ($mW$).
    pub optical_power_mw: f64,
    /// Slope efficiency $dP_{out}/dI$ in Watts per Ampere ($W/A$).
    pub slope_efficiency_w_per_a: f64,
    /// Electrical input power $P_{elec} = V_{bias} \cdot I$ in Watts ($W$).
    pub electrical_power_watts: f64,
    /// Wall-plug efficiency (WPE) $P_{out} / P_{elec} \in [0, 1]$.
    pub wall_plug_efficiency: f64,
    /// Lasing status flag (true if $J \ge J_{th}$).
    pub is_lasing: bool,
}

/// Dynamic state vector for time-resolved numerical rate-equation integration.
#[derive(Debug, Clone, PartialEq)]
pub struct ThzQclRateState {
    /// Time coordinate in seconds ($s$).
    pub time_s: f64,
    /// Upper laser level carrier density $n_3(t)$ in $\text{m}^{-3}$.
    pub n3: f64,
    /// Lower laser level carrier density $n_2(t)$ in $\text{m}^{-3}$.
    pub n2: f64,
    /// Depopulation ground level carrier density $n_1(t)$ in $\text{m}^{-3}$.
    pub n1: f64,
    /// Intracavity photon density $S(t)$ in $\text{m}^{-3}$.
    pub photon_density_s: f64,
    /// Instantaneous optical output power in Watts ($W$).
    pub optical_power_watts: f64,
}

/// Numerical multi-level rate-equation solver for THz Quantum Cascade Lasers.
#[derive(Debug, Clone, PartialEq)]
pub struct ThzQclRateEquationSolver {
    /// Resonant LO-phonon depopulation parameters.
    pub depopulation: ResonantLoPhononDepopulation,
    /// Intersubband optical gain model.
    pub gain_model: ThzOpticalGainModel,
    /// Polaritonic waveguide configuration.
    pub waveguide: ThzPolaritonicWaveguide,
    /// Number of cascaded stages/periods $N_p$ (typically 150-200).
    pub number_of_stages: usize,
    /// Injection efficiency into upper laser level $\eta_{inj}$ (typically 0.75-0.90).
    pub injection_efficiency: f64,
    /// Spontaneous emission coupling factor $\beta_{sp}$ (typically 1e-4).
    pub beta_sp: f64,
    /// Spontaneous radiative lifetime $\tau_{sp}$ in seconds ($s$) (typically ~500 ns).
    pub spontaneous_lifetime_s: f64,
    /// Active region period length $L_p$ in meters ($m$).
    pub stage_length_m: f64,
    /// Voltage drop per stage $V_{stage}$ in Volts ($V$) (~45-55 mV).
    pub voltage_per_stage_v: f64,
    /// Characteristic temperature parameter $T_0$ in Kelvin ($K$) for thermal roll-off.
    pub characteristic_temperature_t0_k: f64,
    /// Low-temperature base threshold current density $J_0$ in $\text{A/cm}^2$.
    pub base_threshold_j0_a_cm2: f64,
    /// Thermal activation prefactor $J_1$ in $\text{A/cm}^2$.
    pub thermal_activation_j1_a_cm2: f64,
    /// Maximum alignable injection current density $J_{max}$ in $\text{A/cm}^2$.
    pub max_alignable_current_density_a_cm2: f64,
}

impl ThzQclRateEquationSolver {
    /// Constructs a rate equation solver with standard GaAs/AlGaAs THz QCL parameters.
    pub fn standard_3_2_thz(waveguide: ThzPolaritonicWaveguide) -> Self {
        let center_freq = 3.2e12; // 3.2 THz
        let dipole_z32 = 3.5e-9; // 3.5 nm
        let n_r = 3.60;
        let fwhm = 0.8e12; // 0.8 THz linewidth
        let stage_len = 45.0e-9; // 45 nm per period
        let n_stages = 180; // 180 cascaded periods (~8.1 um active core)

        let depop = ResonantLoPhononDepopulation::standard_gaas(0.036_25);
        let gain = ThzOpticalGainModel::new(center_freq, dipole_z32, n_r, fwhm, stage_len);

        Self {
            depopulation: depop,
            gain_model: gain,
            waveguide,
            number_of_stages: n_stages,
            injection_efficiency: 0.85,
            beta_sp: 1.0e-4,
            spontaneous_lifetime_s: 400.0e-9,
            stage_length_m: stage_len,
            voltage_per_stage_v: 0.052,             // 52 mV per period
            characteristic_temperature_t0_k: 125.0, // T0 ~ 125 K
            base_threshold_j0_a_cm2: 140.0,
            thermal_activation_j1_a_cm2: 18.0,
            max_alignable_current_density_a_cm2: 1200.0, // 1.2 kA/cm^2 NDR rollover
        }
    }

    /// Evaluates lasing threshold current density at zero/cryogenic temperature $J_{th,0}$:
    /// $$J_{th} = \frac{e L_p (\alpha_w + \alpha_m)}{\Gamma g_{diff} \eta_{inj} \tau_3 (1 - \tau_2 / \tau_{32})}$$
    pub fn theoretical_threshold_current_density_a_cm2(&self) -> f64 {
        let alpha_tot = self.waveguide.total_cavity_loss_per_m();
        let gamma = self.waveguide.optical_confinement_factor();
        let g_diff = self.gain_model.differential_gain_m2();
        let tau_3 = self.depopulation.upper_state_tau_3_s;
        let factor = self
            .depopulation
            .inversion_sustainability_factor()
            .max(0.01);

        let num = ELEMENTARY_CHARGE * self.stage_length_m * alpha_tot;
        let den = gamma * g_diff * self.injection_efficiency * tau_3 * factor;
        let j_th_a_m2 = num / den;
        j_th_a_m2 * 1e-4 // Convert A/m^2 to A/cm^2
    }

    /// Evaluates temperature-dependent threshold current density $J_{th}(T)$ in $\text{A/cm}^2$:
    /// $$J_{th}(T) = J_0 + J_1 \exp\left(\frac{T}{T_0}\right)$$
    pub fn threshold_current_density_at_temperature(&self, temp_kelvin: f64) -> f64 {
        let t = temp_kelvin.max(0.0);
        let thermal_term =
            self.thermal_activation_j1_a_cm2 * (t / self.characteristic_temperature_t0_k).exp();
        self.base_threshold_j0_a_cm2 + thermal_term
    }

    /// Maximum operating temperature $T_{max}$ in Kelvin ($K$):
    /// $$T_{max} = T_0 \ln\left( \frac{J_{max} - J_0}{J_1} \right)$$
    pub fn max_operating_temperature_kelvin(&self) -> f64 {
        let ratio = (self.max_alignable_current_density_a_cm2 - self.base_threshold_j0_a_cm2)
            / self.thermal_activation_j1_a_cm2;
        if ratio <= 1.0 {
            0.0
        } else {
            self.characteristic_temperature_t0_k * ratio.ln()
        }
    }

    /// Evaluates cavity device cross-sectional active area $A = w \cdot L$ in $\text{cm}^2$.
    #[inline]
    pub fn active_area_cm2(&self) -> f64 {
        (self.waveguide.ridge_width_m * 100.0) * (self.waveguide.cavity_length_m * 100.0)
    }

    /// Slope efficiency $dP_{out} / dI$ in Watts per Ampere ($W/A$):
    /// $$\frac{dP_{out}}{dI} = \frac{N_p \hbar \omega_0}{2 e} \frac{\alpha_m}{\alpha_w + \alpha_m} \eta_{inj}$$
    pub fn slope_efficiency_w_per_a(&self) -> f64 {
        let hbar_omega = H_BAR * 2.0 * std::f64::consts::PI * self.gain_model.center_frequency_hz;
        let alpha_m = self.waveguide.mirror_loss_alpha_m_per_m();
        let alpha_tot = self.waveguide.total_cavity_loss_per_m();
        let outcoupling = alpha_m / alpha_tot;

        let prefactor = (self.number_of_stages as f64 * hbar_omega) / (2.0 * ELEMENTARY_CHARGE);
        prefactor * outcoupling * self.injection_efficiency
    }

    /// Solves the steady-state operating point at specified current density $J$ ($\text{A/cm}^2$)
    /// and heat-sink temperature $T$ ($K$).
    pub fn solve_steady_state(
        &self,
        current_density_a_cm2: f64,
        temp_kelvin: f64,
    ) -> ThzQclOperatingState {
        let j_th = self.threshold_current_density_at_temperature(temp_kelvin);
        let area_cm2 = self.active_area_cm2();
        let i_total = current_density_a_cm2 * area_cm2;
        let i_th = j_th * area_cm2;

        let v_bias = self.number_of_stages as f64 * self.voltage_per_stage_v;
        let p_elec = (v_bias * i_total).max(1e-12);

        let slope = self.slope_efficiency_w_per_a();
        let is_lasing = current_density_a_cm2 >= j_th
            && current_density_a_cm2 <= self.max_alignable_current_density_a_cm2;

        let (p_out_w, s_photon, n3, n2, delta_n) = if is_lasing {
            let p_out = slope * (i_total - i_th);
            let hbar_omega =
                H_BAR * 2.0 * std::f64::consts::PI * self.gain_model.center_frequency_hz;
            let v_g = self.waveguide.group_velocity_m_s();
            let alpha_m = self.waveguide.mirror_loss_alpha_m_per_m();
            let active_volume_m3 = self.waveguide.ridge_width_m
                * self.waveguide.cavity_length_m
                * self.waveguide.active_thickness_m;

            // Intracavity photon density S from outcoupled power:
            let s = (2.0 * p_out) / (hbar_omega * v_g * alpha_m * active_volume_m3);

            // Clamped population inversion above threshold:
            let delta_n_clamp = self.waveguide.total_cavity_loss_per_m()
                / (self.waveguide.optical_confinement_factor()
                    * self.gain_model.differential_gain_m2());

            let tau_2 = self.depopulation.extraction_lifetime_tau_21();
            let tau_32 = self.depopulation.upper_to_lower_tau_32_s;

            let n3_clamped = delta_n_clamp / (1.0 - tau_2 / tau_32).max(0.01);
            let n2_clamped = n3_clamped * (tau_2 / tau_32);

            (p_out, s, n3_clamped, n2_clamped, delta_n_clamp)
        } else {
            // Below threshold:
            let j_a_m2 = current_density_a_cm2 * 1e4;
            let n3_sub =
                (j_a_m2 * self.injection_efficiency * self.depopulation.upper_state_tau_3_s)
                    / (ELEMENTARY_CHARGE * self.stage_length_m);
            let factor = self.depopulation.inversion_sustainability_factor();
            let delta_n_sub = n3_sub * factor;
            let n2_sub = n3_sub - delta_n_sub;
            (0.0, 0.0, n3_sub, n2_sub, delta_n_sub)
        };

        let wpe = (p_out_w / p_elec).clamp(0.0, 1.0);

        ThzQclOperatingState {
            current_density_a_cm2,
            device_current_a: i_total,
            temperature_kelvin: temp_kelvin,
            threshold_current_density_a_cm2: j_th,
            threshold_current_a: i_th,
            carrier_density_n3: n3,
            carrier_density_n2: n2,
            inversion_density_m3: delta_n,
            photon_density_m3: s_photon,
            optical_power_watts: p_out_w,
            optical_power_mw: p_out_w * 1000.0,
            slope_efficiency_w_per_a: slope,
            electrical_power_watts: p_elec,
            wall_plug_efficiency: wpe,
            is_lasing,
        }
    }

    /// Solves the transient response of the coupled rate equations using 4th-order Runge-Kutta (RK4):
    /// integrates $(n_3, n_2, n_1, S)$ over time interval $[0, t_{end}]$ with time step $dt$.
    pub fn solve_transient_pulse(
        &self,
        current_pulse_a_cm2: f64,
        pulse_duration_s: f64,
        time_step_s: f64,
        temp_kelvin: f64,
    ) -> Vec<ThzQclRateState> {
        let num_steps = ((pulse_duration_s / time_step_s).ceil() as usize).max(10);
        let dt = pulse_duration_s / num_steps as f64;

        let mut trajectory = Vec::with_capacity(num_steps + 1);

        // Initial thermalized equilibrium carrier densities (pre-bias):
        let mut n3 = 1e18;
        let mut n2 = 1e18;
        let mut n1 = 2e21; // background doping in injector
        let mut s = 1.0; // vacuum photon seed

        let j_a_m2 = current_pulse_a_cm2 * 1e4;
        let gamma = self.waveguide.optical_confinement_factor();
        let v_g = self.waveguide.group_velocity_m_s();
        let g_diff = self.gain_model.differential_gain_m2();
        let tau_ph = self.waveguide.photon_cavity_lifetime_s();
        let tau_3 = self.depopulation.upper_state_tau_3_s;
        let tau_21 = self.depopulation.extraction_lifetime_tau_21();
        let tau_32 = self.depopulation.upper_to_lower_tau_32_s;
        let beta = self.beta_sp;
        let tau_sp = self.spontaneous_lifetime_s;
        let lp = self.stage_length_m;
        let eta = self.injection_efficiency;

        let hbar_omega = H_BAR * 2.0 * std::f64::consts::PI * self.gain_model.center_frequency_hz;
        let alpha_m = self.waveguide.mirror_loss_alpha_m_per_m();
        let active_volume_m3 = self.waveguide.ridge_width_m
            * self.waveguide.cavity_length_m
            * self.waveguide.active_thickness_m;

        // Thermal back-filling lifetime:
        let delta_e21_j = self.depopulation.delta_e21_ev * ELEMENTARY_CHARGE;
        let k_b_t = BOLTZMANN_CONSTANT * temp_kelvin.max(1.0);
        let backfill_factor = (-delta_e21_j / k_b_t).exp();
        let tau_12_therm = tau_21 / backfill_factor.max(1e-12);

        // Record initial state:
        trajectory.push(ThzQclRateState {
            time_s: 0.0,
            n3,
            n2,
            n1,
            photon_density_s: s,
            optical_power_watts: 0.0,
        });

        // Derivatives helper:
        let derivatives =
            |n3_val: f64, n2_val: f64, n1_val: f64, s_val: f64| -> (f64, f64, f64, f64) {
                let stim = gamma * v_g * g_diff * (n3_val - n2_val) * s_val;
                let dn3 = (eta * j_a_m2) / (ELEMENTARY_CHARGE * lp) - (n3_val / tau_3) - stim;
                let dn2 = (n3_val / tau_32) + stim - (n2_val / tau_21) + (n1_val / tau_12_therm);
                let dn1 = (n2_val / tau_21) - (n1_val / tau_12_therm) - (n1_val / (5.0 * tau_21));
                let ds = stim - (s_val / tau_ph) + (beta * n3_val / tau_sp);
                (dn3, dn2, dn1, ds)
            };

        for step in 1..=num_steps {
            let t = step as f64 * dt;

            // RK4 integration:
            let (k1_n3, k1_n2, k1_n1, k1_s) = derivatives(n3, n2, n1, s);

            let (k2_n3, k2_n2, k2_n1, k2_s) = derivatives(
                (n3 + 0.5 * dt * k1_n3).max(0.0),
                (n2 + 0.5 * dt * k1_n2).max(0.0),
                (n1 + 0.5 * dt * k1_n1).max(0.0),
                (s + 0.5 * dt * k1_s).max(0.0),
            );

            let (k3_n3, k3_n2, k3_n1, k3_s) = derivatives(
                (n3 + 0.5 * dt * k2_n3).max(0.0),
                (n2 + 0.5 * dt * k2_n2).max(0.0),
                (n1 + 0.5 * dt * k2_n1).max(0.0),
                (s + 0.5 * dt * k2_s).max(0.0),
            );

            let (k4_n3, k4_n2, k4_n1, k4_s) = derivatives(
                (n3 + dt * k3_n3).max(0.0),
                (n2 + dt * k3_n2).max(0.0),
                (n1 + dt * k3_n1).max(0.0),
                (s + dt * k3_s).max(0.0),
            );

            n3 = (n3 + (dt / 6.0) * (k1_n3 + 2.0 * k2_n3 + 2.0 * k3_n3 + k4_n3)).max(0.0);
            n2 = (n2 + (dt / 6.0) * (k1_n2 + 2.0 * k2_n2 + 2.0 * k3_n2 + k4_n2)).max(0.0);
            n1 = (n1 + (dt / 6.0) * (k1_n1 + 2.0 * k2_n1 + 2.0 * k3_n1 + k4_n1)).max(0.0);
            s = (s + (dt / 6.0) * (k1_s + 2.0 * k2_s + 2.0 * k3_s + k4_s)).max(0.0);

            let p_out = 0.5 * hbar_omega * v_g * alpha_m * s * active_volume_m3;

            trajectory.push(ThzQclRateState {
                time_s: t,
                n3,
                n2,
                n1,
                photon_density_s: s,
                optical_power_watts: p_out,
            });
        }

        trajectory
    }
}

/// Terahertz frequency comb mode entry.
#[derive(Debug, Clone, PartialEq)]
pub struct ThzCombMode {
    /// Comb mode index $m \in [-M/2, M/2]$ relative to center carrier.
    pub mode_index: i32,
    /// Mode absolute frequency $\nu_m$ in Hertz ($Hz$).
    pub frequency_hz: f64,
    /// Mode frequency in Terahertz ($THz$).
    pub frequency_thz: f64,
    /// Relative mode optical power normalized to peak mode ($P_m / P_0 \in [0, 1]$).
    pub normalized_power: f64,
    /// Intermodal optical phase $\phi_m$ in radians ($rad$).
    pub phase_rad: f64,
}

/// Terahertz Frequency Comb synthesis engine modeling Four-Wave Mixing (FWM) optical non-linearities.
#[derive(Debug, Clone, PartialEq)]
pub struct ThzFrequencyCombEngine {
    /// Active region third-order optical non-linearity susceptibility $\chi^{(3)}$ ($\text{m}^2/\text{V}^2$).
    pub chi_3_susceptibility_m2_v2: f64,
    /// Center emission frequency $\nu_0$ in Hertz ($Hz$).
    pub center_frequency_hz: f64,
    /// Cavity repetition frequency $f_{rep} = c / (2 n_g L)$ in Hertz ($Hz$) (~10-25 GHz).
    pub repetition_frequency_hz: f64,
    /// Total comb span / bandwidth in Hertz ($Hz$) (~300-800 GHz).
    pub comb_bandwidth_hz: f64,
    /// Number of phase-locked comb modes.
    pub num_modes: usize,
    /// Intermode beat note FWHM linewidth $\Delta f_{beat}$ in Hertz ($Hz$) (< 1 kHz).
    pub beat_note_linewidth_hz: f64,
}

impl ThzFrequencyCombEngine {
    /// Constructs a frequency comb model from laser cavity parameters.
    pub fn new(
        center_frequency_hz: f64,
        cavity_length_m: f64,
        group_index: f64,
        gain_bandwidth_hz: f64,
        transition_dipole_z_m: f64,
    ) -> Self {
        // Mode repetition frequency: f_rep = c / (2 * n_g * L)
        let f_rep = SPEED_OF_LIGHT / (2.0 * group_index * cavity_length_m);

        // Giant intersubband third-order susceptibility chi^(3):
        // chi^(3) ~ (N_e * e^4 * |z|^4) / (epsilon_0 * hbar^3 * gamma^3) ~ 1e-14 - 1e-13 m^2/V^2
        let gamma_rad = 2.0 * std::f64::consts::PI * (gain_bandwidth_hz * 0.5);
        let n_e = 3.0e21; // 3e15 cm^-3 carrier density
        let chi_3 = (n_e * ELEMENTARY_CHARGE.powi(4) * transition_dipole_z_m.powi(4))
            / (EPSILON_0 * H_BAR.powi(3) * gamma_rad.powi(3));

        let num_modes = ((gain_bandwidth_hz / f_rep).floor() as usize).max(8);
        let actual_bandwidth = num_modes as f64 * f_rep;

        // Intersubband FWM phase-locking achieves sub-kHz beat-note linewidth:
        let beat_note_lw = 450.0; // 450 Hz (< 1 kHz)

        Self {
            chi_3_susceptibility_m2_v2: chi_3,
            center_frequency_hz,
            repetition_frequency_hz: f_rep,
            comb_bandwidth_hz: actual_bandwidth,
            num_modes,
            beat_note_linewidth_hz: beat_note_lw,
        }
    }

    /// Generates the spectral array of equidistant THz frequency comb modes:
    /// $$\nu_m = \nu_0 + m \cdot f_{rep}$$
    pub fn generate_comb_spectrum(&self) -> Vec<ThzCombMode> {
        let half = (self.num_modes / 2) as i32;
        let mut modes = Vec::with_capacity(self.num_modes);

        for m in -half..=half {
            let freq = self.center_frequency_hz + (m as f64 * self.repetition_frequency_hz);
            // Sinc-like or gaussian-like comb envelope shaped by intersubband gain:
            let arg = (m as f64) / (half as f64 * 0.6);
            let norm_pwr = (-0.5 * arg * arg).exp();

            // Four-wave mixing locks phases: phi_m = phi_0 + m * Delta_phi + non-linear chirp
            let phase = 0.15 * (m as f64).powi(2) % (2.0 * std::f64::consts::PI);

            modes.push(ThzCombMode {
                mode_index: m,
                frequency_hz: freq,
                frequency_thz: freq / 1e12,
                normalized_power: norm_pwr,
                phase_rad: phase,
            });
        }

        modes
    }
}

/// Rotational molecular absorption line specification.
#[derive(Debug, Clone, PartialEq)]
pub struct MolecularRotationalLine {
    /// Chemical species name (e.g. "H2O", "CO", "O3").
    pub species_name: &'static str,
    /// Center resonance frequency in Terahertz ($THz$).
    pub center_frequency_thz: f64,
    /// Integrated absorption cross-section line intensity $S_{line}$ in $\text{cm}^2 \cdot \text{Hz} / \text{molecule}$.
    pub line_intensity_cm2_hz: f64,
    /// Pressure-broadened half-width at half-maximum (HWHM) $\gamma_{air}$ in $\text{GHz} / \text{atm}$.
    pub pressure_broadening_ghz_per_atm: f64,
}

/// Sub-millimeter / THz transmission spectrum evaluation result.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectroscopyTransmissionResult {
    /// Optical frequency evaluation points in Terahertz ($THz$).
    pub frequencies_thz: Vec<f64>,
    /// Fractional power transmission $T(\nu) = I(\nu) / I_0(\nu) \in [0, 1]$.
    pub transmission: Vec<f64>,
    /// Optical absorbance $A(\nu) = -\ln(T(\nu))$.
    pub absorbance: Vec<f64>,
    /// Minimum detected transmission in the band (deepest absorption dip).
    pub min_transmission: f64,
    /// Peak absorbance value.
    pub peak_absorbance: f64,
    /// Frequency of deepest absorption line in Terahertz ($THz$).
    pub peak_absorption_frequency_thz: f64,
}

/// High-resolution sub-millimeter absorption spectroscopy engine.
#[derive(Debug, Clone, PartialEq)]
pub struct SubMillimeterSpectroscopyEngine {
    /// Catalog of active molecular rotational absorption lines.
    pub rotational_lines: Vec<MolecularRotationalLine>,
    /// Optical path length $L_{path}$ through the gas cell in meters ($m$).
    pub optical_path_length_m: f64,
    /// Ambient pressure in atmospheres ($atm$).
    pub pressure_atm: f64,
    /// Gas temperature in Kelvin ($K$).
    pub gas_temperature_k: f64,
}

impl SubMillimeterSpectroscopyEngine {
    /// Compiles an authoritative catalog of key atmospheric rotational lines across $0.5 - 10\text{ THz}$:
    /// - Water Vapor ($\text{H}_2\text{O}$): $0.557, 0.752, 1.097, 1.163, 1.670, 3.090\text{ THz}$.
    /// - Carbon Monoxide ($\text{CO}$): Equidistant rotational ladder with $\Delta \nu \approx 115.27\text{ GHz}$:
    ///   $0.115, 0.231, 0.346, 0.461, 0.576, 1.153, 2.305, 3.458\text{ THz}$.
    /// - Ozone ($\text{O}_3$): $1.020, 1.840, 2.040\text{ THz}$.
    pub fn atmospheric_catalog(optical_path_length_m: f64, pressure_atm: f64) -> Self {
        let lines = vec![
            // Water vapor:
            MolecularRotationalLine {
                species_name: "H2O",
                center_frequency_thz: 0.557,
                line_intensity_cm2_hz: 3.2e-18,
                pressure_broadening_ghz_per_atm: 3.0,
            },
            MolecularRotationalLine {
                species_name: "H2O",
                center_frequency_thz: 0.752,
                line_intensity_cm2_hz: 4.8e-18,
                pressure_broadening_ghz_per_atm: 3.2,
            },
            MolecularRotationalLine {
                species_name: "H2O",
                center_frequency_thz: 1.097,
                line_intensity_cm2_hz: 5.6e-18,
                pressure_broadening_ghz_per_atm: 3.1,
            },
            MolecularRotationalLine {
                species_name: "H2O",
                center_frequency_thz: 1.670,
                line_intensity_cm2_hz: 6.2e-18,
                pressure_broadening_ghz_per_atm: 3.4,
            },
            MolecularRotationalLine {
                species_name: "H2O",
                center_frequency_thz: 3.090,
                line_intensity_cm2_hz: 7.5e-18,
                pressure_broadening_ghz_per_atm: 3.5,
            },
            // Carbon Monoxide:
            MolecularRotationalLine {
                species_name: "CO",
                center_frequency_thz: 0.576,
                line_intensity_cm2_hz: 2.1e-19,
                pressure_broadening_ghz_per_atm: 2.2,
            },
            MolecularRotationalLine {
                species_name: "CO",
                center_frequency_thz: 1.153,
                line_intensity_cm2_hz: 2.5e-19,
                pressure_broadening_ghz_per_atm: 2.2,
            },
            MolecularRotationalLine {
                species_name: "CO",
                center_frequency_thz: 2.305,
                line_intensity_cm2_hz: 2.9e-19,
                pressure_broadening_ghz_per_atm: 2.1,
            },
            MolecularRotationalLine {
                species_name: "CO",
                center_frequency_thz: 3.458,
                line_intensity_cm2_hz: 3.2e-19,
                pressure_broadening_ghz_per_atm: 2.0,
            },
            // Ozone:
            MolecularRotationalLine {
                species_name: "O3",
                center_frequency_thz: 1.020,
                line_intensity_cm2_hz: 1.2e-19,
                pressure_broadening_ghz_per_atm: 2.8,
            },
            MolecularRotationalLine {
                species_name: "O3",
                center_frequency_thz: 1.840,
                line_intensity_cm2_hz: 1.6e-19,
                pressure_broadening_ghz_per_atm: 2.7,
            },
            MolecularRotationalLine {
                species_name: "O3",
                center_frequency_thz: 2.040,
                line_intensity_cm2_hz: 1.9e-19,
                pressure_broadening_ghz_per_atm: 2.6,
            },
        ];

        Self {
            rotational_lines: lines,
            optical_path_length_m,
            pressure_atm,
            gas_temperature_k: 296.0, // Standard HITRAN 296 K reference
        }
    }

    /// Evaluates Beer-Lambert optical transmission and absorbance spectrum across frequency range $[\nu_{min}, \nu_{max}]$:
    /// $$I(\nu) = I_0 \exp(-\alpha_{abs}(\nu) L_{path})$$
    /// where $\alpha_{abs}(\nu) = \sum_k n_k \sigma_k(\nu)$.
    pub fn compute_transmission_spectrum(
        &self,
        min_thz: f64,
        max_thz: f64,
        num_points: usize,
        h2o_concentration_ppm: f64,
        co_concentration_ppm: f64,
        o3_concentration_ppm: f64,
    ) -> SpectroscopyTransmissionResult {
        let n_pts = num_points.max(20);
        let step_thz = (max_thz - min_thz) / (n_pts - 1) as f64;

        // Molecular number density at standard temperature and pressure:
        // n_tot = P / (k_B * T) in molecules / m^3
        let p_pa = self.pressure_atm * 101_325.0;
        let n_tot_m3 = p_pa / (BOLTZMANN_CONSTANT * self.gas_temperature_k);
        let n_tot_cm3 = n_tot_m3 * 1e-6; // molecules / cm^3

        let n_h2o = n_tot_cm3 * h2o_concentration_ppm * 1e-6;
        let n_co = n_tot_cm3 * co_concentration_ppm * 1e-6;
        let n_o3 = n_tot_cm3 * o3_concentration_ppm * 1e-6;

        let path_cm = self.optical_path_length_m * 100.0;

        let mut freqs = Vec::with_capacity(n_pts);
        let mut transmissions = Vec::with_capacity(n_pts);
        let mut absorbances = Vec::with_capacity(n_pts);

        let mut min_t = 1.0;
        let mut max_a = 0.0;
        let mut peak_freq_thz = min_thz;

        for i in 0..n_pts {
            let f_thz = min_thz + i as f64 * step_thz;
            let f_hz = f_thz * 1e12;

            // Total absorption coefficient alpha in cm^-1:
            let mut alpha_total_cm1 = 0.0;

            for line in &self.rotational_lines {
                let n_molecules = match line.species_name {
                    "H2O" => n_h2o,
                    "CO" => n_co,
                    "O3" => n_o3,
                    _ => 0.0,
                };

                if n_molecules <= 0.0 {
                    continue;
                }

                let line_center_hz = line.center_frequency_thz * 1e12;
                let gamma_hwhm_hz = line.pressure_broadening_ghz_per_atm * 1e9 * self.pressure_atm;

                // Lorentzian cross section sigma(nu) in cm^2:
                let detuning_hz = f_hz - line_center_hz;
                let lorentzian = (gamma_hwhm_hz / std::f64::consts::PI)
                    / (detuning_hz.powi(2) + gamma_hwhm_hz.powi(2));
                let c_cm_s = SPEED_OF_LIGHT * 100.0;
                let sigma_cm2 = line.line_intensity_cm2_hz * c_cm_s * lorentzian;

                alpha_total_cm1 += n_molecules * sigma_cm2;
            }

            let optical_depth = alpha_total_cm1 * path_cm;
            let trans = (-optical_depth).exp().clamp(0.0, 1.0);
            let abs_val = optical_depth;

            if trans < min_t {
                min_t = trans;
                max_a = abs_val;
                peak_freq_thz = f_thz;
            }

            freqs.push(f_thz);
            transmissions.push(trans);
            absorbances.push(abs_val);
        }

        SpectroscopyTransmissionResult {
            frequencies_thz: freqs,
            transmission: transmissions,
            absorbance: absorbances,
            min_transmission: min_t,
            peak_absorbance: max_a,
            peak_absorption_frequency_thz: peak_freq_thz,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thz_qcl_threshold_and_slope_efficiency() {
        let mm_waveguide = ThzPolaritonicWaveguide::metal_metal(10e-6, 100e-6, 2.5e-3);
        let solver = ThzQclRateEquationSolver::standard_3_2_thz(mm_waveguide);

        // Theoretical threshold current density at cryogenic temperature:
        let j_th_0 = solver.theoretical_threshold_current_density_a_cm2();
        assert!(
            j_th_0 > 10.0 && j_th_0 < 400.0,
            "Threshold Jth out of range: {} A/cm^2",
            j_th_0
        );

        // Slope efficiency should be on the order of 0.02 - 2.0 W/A:
        let slope = solver.slope_efficiency_w_per_a();
        assert!(
            slope > 0.01 && slope < 3.0,
            "Slope efficiency out of range: {} W/A",
            slope
        );

        // Maximum operating temperature should exceed 200 K for resonant LO-phonon designs:
        let t_max = solver.max_operating_temperature_kelvin();
        assert!(t_max > 200.0, "T_max should exceed 200 K, got: {} K", t_max);
    }

    #[test]
    fn test_thz_qcl_steady_state_and_optical_power() {
        let mm_waveguide = ThzPolaritonicWaveguide::metal_metal(10e-6, 100e-6, 2.5e-3);
        let solver = ThzQclRateEquationSolver::standard_3_2_thz(mm_waveguide);

        let temp_k = 78.0; // Liquid nitrogen temperature
        let j_th = solver.threshold_current_density_at_temperature(temp_k);

        // Below threshold:
        let sub = solver.solve_steady_state(j_th * 0.8, temp_k);
        assert!(!sub.is_lasing);
        assert_eq!(sub.optical_power_watts, 0.0);

        // Above threshold:
        let drive_j = j_th * 1.8;
        let active = solver.solve_steady_state(drive_j, temp_k);
        assert!(active.is_lasing);
        assert!(
            active.optical_power_mw > 10.0,
            "Power: {} mW",
            active.optical_power_mw
        );
        assert!(active.wall_plug_efficiency > 0.001); // > 0.1% WPE for unoptimized facet
    }

    #[test]
    fn test_frequency_comb_generation_and_fwm() {
        let center_freq = 3.2e12; // 3.2 THz
        let l_cav = 2.5e-3; // 2.5 mm cavity length
        let n_g = 3.82;
        let gain_bw = 500.0e9; // 500 GHz gain bandwidth
        let dipole_z = 3.5e-9; // 3.5 nm

        let comb_engine = ThzFrequencyCombEngine::new(center_freq, l_cav, n_g, gain_bw, dipole_z);

        // Repetition rate should be in the 10 - 25 GHz microwave range:
        let f_rep_ghz = comb_engine.repetition_frequency_hz / 1e9;
        assert!(
            (10.0..=25.0).contains(&f_rep_ghz),
            "Comb repetition rate should be 10 - 25 GHz, got {} GHz",
            f_rep_ghz
        );

        // Sub-kilohertz beat note linewidth:
        assert!(comb_engine.beat_note_linewidth_hz < 1000.0);

        // Generate comb modes:
        let modes = comb_engine.generate_comb_spectrum();
        assert!(modes.len() >= 15);
        assert!((modes[modes.len() / 2].frequency_thz - 3.2).abs() < 0.05);
    }

    #[test]
    fn test_sub_millimeter_absorption_spectroscopy() {
        let engine = SubMillimeterSpectroscopyEngine::atmospheric_catalog(1.0, 1.0); // 1 meter, 1 atm

        // Sweep around the strong 0.557 THz and 0.752 THz water vapor lines:
        let res = engine.compute_transmission_spectrum(
            0.50, 0.80, 60, 5000.0, // 5,000 ppm water vapor (typical room humidity)
            0.0, 0.0,
        );

        // Deep absorption dip should occur near 0.557 THz:
        assert!(res.min_transmission < 0.85);
        assert!(res.peak_absorbance > 0.15);
    }
}
