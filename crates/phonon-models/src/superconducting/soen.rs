//! Superconducting Optoelectronic Neurons (SOEN).
//!
//! Models the microscopic physics of:
//! - Superconducting Nanowire Single-Photon Detectors (SNSPD) with electro-thermal hot-spot dynamics.
//! - Cryogenic semiconductor optical emitters (micro-LEDs / nanolasers) via cryogenic carrier-photon rate equations.
//! - Superconducting flux storage loops with quantized flux accumulation ($\Phi = L I + n\Phi_0$) and leaky relaxation.
//! - Somatic Josephson junctions executing $2\pi$ phase slips and self-resetting flux mechanisms.
//! - Ultra-low energy dissipation: $\sim 0.1 - 5\text{ aJ}$ per synaptic event.

use super::rcsj::JosephsonRcsjModel;
use phonon_core::{ELEMENTARY_CHARGE, FLUX_QUANTUM, PLANCK_CONSTANT, SPEED_OF_LIGHT};

/// Microscopic model for a Superconducting Nanowire Single-Photon Detector (SNSPD).
///
/// Captures photon absorption, electro-thermal hot-spot nucleation, and current diversion
/// into an inductive load / superconducting loop.
#[derive(Debug, Clone, PartialEq)]
pub struct SnspdModel {
    /// Critical current $I_c$ in Amperes ($A$) (typically $10 - 50\,\mu\text{A}$).
    pub ic: f64,
    /// Operating bias current ratio $\alpha = I_{bias} / I_c \in (0, 1)$ (typically $0.90 - 0.98$).
    pub bias_ratio: f64,
    /// Normal-state hot-spot resistance $R_{norm}$ in Ohms ($\Omega$) (typically $1000 - 5000\,\Omega$).
    pub r_norm: f64,
    /// Kinetic inductance $L_k$ in Henries ($H$) (typically $100 - 500\text{ nH}$).
    pub kinetic_inductance: f64,
    /// Thermal relaxation time $\tau_{th}$ in seconds ($s$) (typically $100 - 250\text{ ps}$).
    pub tau_th: f64,
    /// Hot-spot expansion / current diversion rise time $\tau_{rise}$ in seconds ($s$) (typically $5 - 20\text{ ps}$).
    pub tau_rise: f64,
    /// Internal Quantum Efficiency $\eta_{QE} \in [0.0, 1.0]$.
    pub quantum_efficiency: f64,
    /// Dark count rate in Hertz ($Hz$) (typically $1 - 100\text{ Hz}$).
    pub dark_count_rate_hz: f64,
}

impl Default for SnspdModel {
    fn default() -> Self {
        Self {
            ic: 25.0e-6,                // 25 uA
            bias_ratio: 0.95,           // 95% of Ic -> Ibias = 23.75 uA
            r_norm: 2000.0,             // 2.0 kOhm
            kinetic_inductance: 250e-9, // 250 nH
            tau_th: 150e-12,            // 150 ps
            tau_rise: 10e-12,           // 10 ps
            quantum_efficiency: 0.95,   // 95% QE
            dark_count_rate_hz: 10.0,   // 10 Hz
        }
    }
}

impl SnspdModel {
    /// Creates a new SNSPD model with specified physical parameters.
    pub fn new(
        ic: f64,
        bias_ratio: f64,
        r_norm: f64,
        kinetic_inductance: f64,
        tau_th: f64,
        quantum_efficiency: f64,
    ) -> Self {
        Self {
            ic: ic.max(1e-9),
            bias_ratio: bias_ratio.clamp(0.01, 0.999),
            r_norm: r_norm.max(1.0),
            kinetic_inductance: kinetic_inductance.max(1e-12),
            tau_th: tau_th.max(1e-15),
            tau_rise: (tau_th * 0.08).max(1e-15),
            quantum_efficiency: quantum_efficiency.clamp(0.0, 1.0),
            dark_count_rate_hz: 10.0,
        }
    }

    /// Constant DC bias current $I_{bias} = \alpha I_c$ in Amperes ($A$).
    #[inline]
    pub fn bias_current(&self) -> f64 {
        self.bias_ratio * self.ic
    }

    /// Evaluates the hot-spot resistance $R_{hs}(t)$ after photon absorption:
    /// $$R_{hs}(t) = R_{norm} \exp(-t / \tau_{th})$$
    #[inline]
    pub fn hotspot_resistance(&self, t_after_absorption: f64) -> f64 {
        if t_after_absorption < 0.0 {
            0.0
        } else {
            self.r_norm * (-t_after_absorption / self.tau_th).exp()
        }
    }

    /// Evaluates the diverted current pulse $I_{div}(t)$ into a load impedance $R_{load}$:
    /// $$I_{div}(t) = I_{bias} \frac{R_{hs}(t)}{R_{hs}(t) + R_{load}} \left[ 1 - \exp\left(-\frac{t}{\tau_{rise}}\right) \right]$$
    #[inline]
    pub fn diverted_current(&self, t_after_absorption: f64, r_load: f64) -> f64 {
        if t_after_absorption <= 0.0 {
            return 0.0;
        }
        let r_hs = self.hotspot_resistance(t_after_absorption);
        let ratio = r_hs / (r_hs + r_load.max(1e-6));
        let rise_factor = 1.0 - (-t_after_absorption / self.tau_rise).exp();
        self.bias_current() * ratio * rise_factor
    }

    /// Evaluates total integrated flux diverted into a load over pulse duration $5 \tau_{th}$:
    /// $$\Delta \Phi = \int_0^{5\tau_{th}} I_{div}(t) R_{load} dt$$
    pub fn diverted_flux(&self, r_load: f64, num_steps: usize) -> f64 {
        let t_total = 5.0 * self.tau_th;
        let dt = t_total / num_steps.max(10) as f64;
        let mut flux = 0.0;
        for step in 0..num_steps {
            let t = (step as f64 + 0.5) * dt;
            let i_div = self.diverted_current(t, r_load);
            flux += i_div * r_load * dt;
        }
        flux
    }

    /// Stored kinetic inductive energy $E_k = \frac{1}{2} L_k I_{bias}^2$ in Joules ($J$).
    #[inline]
    pub fn kinetic_inductive_energy(&self) -> f64 {
        let i_b = self.bias_current();
        0.5 * self.kinetic_inductance * i_b * i_b
    }

    /// Microscopic resistive Joule dissipation during a single-photon detection event:
    /// $$E_{diss} = \int_0^{5\tau_{th}} \left[ I_{div}^2(t) R_{load} + (I_{bias} - I_{div}(t))^2 R_{hs}(t) \right] dt$$
    /// In physical SNSPDs, this is typically in the sub-attojoule to few-attojoule range ($\sim 0.5 - 5\text{ aJ}$).
    pub fn dissipation_energy_joules(&self, r_load: f64) -> f64 {
        let steps = 50;
        let t_total = 5.0 * self.tau_th;
        let dt = t_total / steps as f64;
        let i_bias = self.bias_current();
        let mut total_dissipation = 0.0;

        for step in 0..steps {
            let t = (step as f64 + 0.5) * dt;
            let i_div = self.diverted_current(t, r_load);
            let r_hs = self.hotspot_resistance(t);
            let p_load = i_div * i_div * r_load;
            let i_hs = (i_bias - i_div).max(0.0);
            let p_hs = i_hs * i_hs * r_hs;
            total_dissipation += (p_load + p_hs) * dt;
        }

        total_dissipation
    }

    /// Checks if a single photon of energy $E_{ph}$ (in $eV$) triggers an absorption event.
    /// In an SNSPD at $4\text{ K}$, absorption occurs if $E_{ph} > 2\Delta$ and roll $< \eta_{QE}$.
    pub fn absorbs_photon(&self, photon_energy_ev: f64, rng_val: f64) -> bool {
        // Niobium Nitride (NbN) superconducting energy gap 2*Delta ~ 4.5 meV
        let min_energy_ev = 0.0045;
        if photon_energy_ev < min_energy_ev {
            return false;
        }
        rng_val.clamp(0.0, 1.0) < self.quantum_efficiency
    }
}

/// Cryogenic semiconductor optical emitter (micro-LED or nanolaser diode) at $4\text{ K}$.
///
/// Operates via coupled electron carrier and photon density rate equations:
/// $$\frac{dN}{dt} = \frac{\eta_d I_{in}}{q V_{act}} - \frac{N}{\tau_n} - g_0 (N - N_{tr}) S$$
/// $$\frac{dS}{dt} = \Gamma g_0 (N - N_{tr}) S - \frac{S}{\tau_p} + \Gamma \beta_{sp} \frac{N}{\tau_n}$$
#[derive(Debug, Clone, PartialEq)]
pub struct CryoOpticalEmitter {
    /// Optical emission wavelength $\lambda$ in meters ($m$) (e.g. $850\text{ nm}$).
    pub wavelength: f64,
    /// Active volume $V_{act}$ in $\text{m}^3$.
    pub active_volume: f64,
    /// Optical confinement factor $\Gamma \in (0, 1)$.
    pub confinement_factor: f64,
    /// Cryogenic carrier recombination lifetime $\tau_n$ in seconds ($s$) at $4\text{ K}$ (typically $1.0\text{ ns}$).
    pub carrier_lifetime_cryo: f64,
    /// Cryogenic photon cavity lifetime $\tau_p$ in seconds ($s$) (typically $2.0\text{ ps}$).
    pub photon_lifetime: f64,
    /// Differential optical gain coefficient $g_0$ in $\text{m}^3/\text{s}$.
    pub differential_gain: f64,
    /// Transparency carrier density $N_{tr}$ in $\text{m}^{-3}$ (at $4\text{ K}$, typically $5 \times 10^{23}\text{ m}^{-3}$).
    pub transparency_carrier_density: f64,
    /// Spontaneous emission coupling factor $\beta_{sp}$ (typically $10^{-3}$).
    pub spontaneous_emission_factor: f64,
    /// Threshold injection current $I_{th}$ in Amperes ($A$) at $4\text{ K}$ (typically $2 - 10\,\mu\text{A}$).
    pub threshold_current_a: f64,
    /// Differential quantum efficiency $\eta_d \in (0, 1)$.
    pub differential_efficiency: f64,
    /// State: instantaneous carrier density $N(t)$ in $\text{m}^{-3}$.
    pub carrier_density: f64,
    /// State: instantaneous photon density $S(t)$ in $\text{m}^{-3}$.
    pub photon_density: f64,
}

impl Default for CryoOpticalEmitter {
    fn default() -> Self {
        Self {
            wavelength: 850.0e-9,                 // 850 nm
            active_volume: 1.0e-19,               // 0.1 um^3
            confinement_factor: 0.35,             // Gamma = 0.35
            carrier_lifetime_cryo: 1.0e-9,        // 1.0 ns at 4K
            photon_lifetime: 2.0e-12,             // 2.0 ps
            differential_gain: 2.5e-12,           // 2.5e-12 m^3/s
            transparency_carrier_density: 5.0e23, // 5.0e17 cm^-3
            spontaneous_emission_factor: 1.0e-3,  // beta = 0.001
            threshold_current_a: 5.0e-6,          // 5 uA at 4K
            differential_efficiency: 0.88,        // 88%
            carrier_density: 0.0,
            photon_density: 0.0,
        }
    }
}

impl CryoOpticalEmitter {
    /// Creates a new cryogenic optical emitter with specified wavelength and active volume.
    pub fn new(wavelength: f64, active_volume: f64, threshold_current_a: f64) -> Self {
        Self {
            wavelength: wavelength.max(100e-9),
            active_volume: active_volume.max(1e-24),
            threshold_current_a: threshold_current_a.max(1e-12),
            ..Default::default()
        }
    }

    /// Single photon energy $E_{ph} = \frac{h c}{\lambda}$ in Joules ($J$).
    #[inline]
    pub fn photon_energy_joules(&self) -> f64 {
        (PLANCK_CONSTANT * SPEED_OF_LIGHT) / self.wavelength
    }

    /// Single photon energy in electron-volts ($eV$).
    #[inline]
    pub fn photon_energy_ev(&self) -> f64 {
        self.photon_energy_joules() / ELEMENTARY_CHARGE
    }

    /// Advances the optical emitter rate equations over time-step $dt$ with injected current $I_{inj}$.
    /// Returns instantaneous optical power $P_{opt}(t)$ in Watts ($W$).
    pub fn step(&mut self, dt: f64, i_inj: f64) -> f64 {
        let n = self.carrier_density;
        let s = self.photon_density;

        let pump_term = (self.differential_efficiency * i_inj.max(0.0))
            / (ELEMENTARY_CHARGE * self.active_volume);
        let srh_rad_recomb = n / self.carrier_lifetime_cryo;
        let stim_gain = self.differential_gain * (n - self.transparency_carrier_density).max(0.0);
        let stim_recomb = stim_gain * s;

        // dN/dt
        let dn_dt = pump_term - srh_rad_recomb - stim_recomb;

        // dS/dt
        let cavity_loss = s / self.photon_lifetime;
        let spon_source = self.confinement_factor
            * self.spontaneous_emission_factor
            * (n / self.carrier_lifetime_cryo);
        let ds_dt = self.confinement_factor * stim_gain * s - cavity_loss + spon_source;

        // Explicit Euler / clamped integration
        self.carrier_density = (n + dn_dt * dt).max(0.0);
        self.photon_density = (s + ds_dt * dt).max(0.0);

        // Optical output power P_opt = (eta_d * h * nu * V_act * S) / (Gamma * tau_p)
        let energy_per_photon = self.photon_energy_joules();
        (self.differential_efficiency
            * energy_per_photon
            * self.active_volume
            * self.photon_density)
            / (self.confinement_factor * self.photon_lifetime)
    }

    /// Emits an optical pulse triggered by a driver current pulse of amplitude $I_{peak}$
    /// and duration $\Delta t_{pulse}$. Returns `(pulse_energy_joules, estimated_photons)`.
    pub fn emit_pulse(&mut self, i_peak: f64, pulse_width: f64) -> (f64, f64) {
        let steps = 50;
        let dt = pulse_width / steps as f64;
        let mut total_energy = 0.0;

        for _ in 0..steps {
            let p_opt = self.step(dt, i_peak);
            total_energy += p_opt * dt;
        }

        // Cool-down relaxation steps
        for _ in 0..steps {
            let p_opt = self.step(dt, 0.0);
            total_energy += p_opt * dt;
        }

        let num_photons = total_energy / self.photon_energy_joules();
        (total_energy, num_photons)
    }
}

/// Superconducting inductive flux storage loop.
///
/// Implements dendritic flux accumulation:
/// $$\Phi_{loop}(t) = L_{loop} I_{loop}(t) + n \Phi_0$$
/// with leaky relaxation time $\tau_{leak} = L_{loop} / R_{damp}$.
#[derive(Debug, Clone, PartialEq)]
pub struct SuperconductingFluxLoop {
    /// Loop geometric inductance $L_{loop}$ in Henries ($H$) (typically $20 - 100\text{ pH}$).
    pub loop_inductance: f64,
    /// Damping / leakage resistance $R_{damp}$ in Ohms ($\Omega$) (typically $0.01 - 0.1\,\Omega$).
    pub damping_resistance: f64,
    /// Instantaneous circulating loop current $I_{loop}(t)$ in Amperes ($A$).
    pub current: f64,
    /// Stored magnetic flux $\Phi_{loop} = L_{loop} I_{loop}$ in Webers ($Wb$).
    pub stored_flux: f64,
    /// Count of discrete flux quanta $\Phi_0$ accumulated in the loop.
    pub flux_quantum_count: i64,
}

impl Default for SuperconductingFluxLoop {
    fn default() -> Self {
        Self {
            loop_inductance: 50.0e-12, // 50 pH
            damping_resistance: 0.025, // 25 mOhm -> tau_leak = 50 pH / 25 mOhm = 2.0 ns
            current: 0.0,
            stored_flux: 0.0,
            flux_quantum_count: 0,
        }
    }
}

impl SuperconductingFluxLoop {
    /// Creates a new flux storage loop with specified inductance and damping resistance.
    pub fn new(loop_inductance: f64, damping_resistance: f64) -> Self {
        Self {
            loop_inductance: loop_inductance.max(1e-15),
            damping_resistance: damping_resistance.max(1e-9),
            current: 0.0,
            stored_flux: 0.0,
            flux_quantum_count: 0,
        }
    }

    /// Characteristic leaky memory time constant $\tau_{leak} = L_{loop} / R_{damp}$ in seconds ($s$).
    #[inline]
    pub fn leak_time_constant(&self) -> f64 {
        self.loop_inductance / self.damping_resistance
    }

    /// Adds a discrete single flux quantum $\pm \Phi_0$ to the loop.
    pub fn add_flux_quantum(&mut self, sign: f64) {
        let delta_phi = sign.signum() * FLUX_QUANTUM;
        let delta_i = delta_phi / self.loop_inductance;
        self.current += delta_i;
        self.stored_flux += delta_phi;
        if sign > 0.0 {
            self.flux_quantum_count += 1;
        } else if sign < 0.0 {
            self.flux_quantum_count -= 1;
        }
    }

    /// Directly injects a continuous current increment $\Delta I$ into the loop.
    #[inline]
    pub fn inject_current(&mut self, delta_i: f64) {
        self.current += delta_i;
        self.stored_flux = self.loop_inductance * self.current;
    }

    /// Advances the loop current under external voltage $V_{in}$ and internal damping:
    /// $$L_{loop} \frac{dI}{dt} + R_{damp} I = V_{in}(t)$$
    pub fn step(&mut self, dt: f64, v_in: f64) {
        let di_dt = (v_in - self.damping_resistance * self.current) / self.loop_inductance;
        self.current += di_dt * dt;
        self.stored_flux = self.loop_inductance * self.current;
    }

    /// Resets the loop flux and circulating current to zero.
    pub fn reset_flux(&mut self) {
        self.current = 0.0;
        self.stored_flux = 0.0;
        self.flux_quantum_count = 0;
    }

    /// Total stored magnetic inductive energy $E_M = \frac{1}{2} L_{loop} I_{loop}^2$ in Joules ($J$).
    #[inline]
    pub fn stored_magnetic_energy(&self) -> f64 {
        0.5 * self.loop_inductance * self.current * self.current
    }
}

/// Performance and energy metrics for a Superconducting Optoelectronic Neuron (SOEN).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoenMetrics {
    /// Total count of somatic optical action potentials fired.
    pub spike_count: usize,
    /// Average microscopic energy dissipated per synaptic event in Joules ($J$) ($\sim 10^{-19} - 10^{-18}\text{ J}$).
    pub average_synaptic_energy_joules: f64,
    /// Average synaptic energy in attojoules ($aJ$) ($1\text{ aJ} = 10^{-18}\text{ J}$).
    pub average_synaptic_energy_attojoules: f64,
    /// Effective wall-plug energy at $300\text{ K}$ (multiplying by Carnot cryocooling overhead $\approx 1000\times$).
    pub wall_plug_energy_attojoules: f64,
    /// Somatic $2\pi$ phase-slip firing latency in picoseconds ($ps$).
    pub somatic_firing_latency_ps: f64,
    /// Dendritic flux retention time constant in nanoseconds ($ns$).
    pub flux_retention_ns: f64,
}

/// Superconducting Optoelectronic Neuron (SOEN).
///
/// Unifies:
/// 1. Synaptic optical inputs with SNSPDs and optical weight attenuation ($w_i \in [0, 1]$).
/// 2. Dendritic superconducting flux storage loop integrating diverted flux.
/// 3. Somatic Josephson junction executing thresholding via $2\pi$ phase slips.
/// 4. Cryogenic semiconductor optical emitter transmitting output photon spikes.
#[derive(Debug, Clone, PartialEq)]
pub struct SoenNeuron {
    /// Unique identifier of the neuron.
    pub id: usize,
    /// Synaptic detector model (SNSPD).
    pub snspd: SnspdModel,
    /// Dendritic superconducting flux storage loop.
    pub flux_loop: SuperconductingFluxLoop,
    /// Somatic Josephson junction (RCSJ model).
    pub somatic_jj: JosephsonRcsjModel,
    /// On-chip cryogenic optical emitter.
    pub optical_emitter: CryoOpticalEmitter,
    /// Somatic bias current $I_{bias}$ in Amperes ($A$) (typically $0.80 - 0.90 I_c$).
    pub somatic_bias_current: f64,
    /// Synaptic weights $w_i \in [0.0, 1.0]$ for each dendritic optical input channel.
    pub synaptic_weights: Vec<f64>,
    /// Somatic critical threshold flux in Webers ($Wb$) ($\sim 0.5 - 1.0 \Phi_0$).
    pub somatic_threshold_flux: f64,
    /// Total number of action potentials (optical spikes) emitted.
    pub spike_count: usize,
    /// Total cumulative energy dissipated in Joules ($J$).
    pub cumulative_energy_joules: f64,
    /// Total number of synaptic input events processed.
    pub synaptic_events_processed: usize,
    /// Timestamp of the last emitted spike in seconds ($s$).
    pub last_spike_time: Option<f64>,
}

impl SoenNeuron {
    /// Constructs a new SOEN neuron with a specified number of synaptic inputs and somatic $I_c$.
    pub fn new(id: usize, num_inputs: usize, somatic_ic: f64) -> Self {
        let somatic_ic = somatic_ic.max(1e-6);
        let somatic_rn = 5.0; // 5 Ohms
        let somatic_jj = JosephsonRcsjModel::overdamped_rsfq(somatic_ic, somatic_rn);

        let bias_ratio = 0.85;
        let somatic_bias_current = bias_ratio * somatic_ic;
        let somatic_threshold_flux = 0.70 * FLUX_QUANTUM;

        Self {
            id,
            snspd: SnspdModel::default(),
            flux_loop: SuperconductingFluxLoop::default(),
            somatic_jj,
            optical_emitter: CryoOpticalEmitter::default(),
            somatic_bias_current,
            synaptic_weights: vec![1.0; num_inputs.max(1)],
            somatic_threshold_flux,
            spike_count: 0,
            cumulative_energy_joules: 0.0,
            synaptic_events_processed: 0,
            last_spike_time: None,
        }
    }

    /// Sets the synaptic weight $w_i \in [0.0, 1.0]$ for input channel $channel_idx$.
    pub fn set_weight(&mut self, channel_idx: usize, weight: f64) {
        if channel_idx < self.synaptic_weights.len() {
            self.synaptic_weights[channel_idx] = weight.clamp(0.0, 1.0);
        }
    }

    /// Ingests incoming optical photon spikes on dendritic input channels:
    /// `input_spikes`: slice of `(channel_idx, photon_count)`.
    ///
    /// For each absorbed photon packet, diverts flux into the dendritic loop:
    /// $$\Delta \Phi = w_i \cdot \Phi_0 \cdot \min(N_{photons}, 1.0)$$
    pub fn receive_synaptic_spikes(&mut self, input_spikes: &[(usize, f64)], _current_time: f64) {
        for &(channel, photon_count) in input_spikes {
            if channel < self.synaptic_weights.len() && photon_count > 0.0 {
                let weight = self.synaptic_weights[channel];
                let diverted_flux = weight * FLUX_QUANTUM * photon_count.min(1.0);
                let delta_i = diverted_flux / self.flux_loop.loop_inductance;

                self.flux_loop.inject_current(delta_i);
                self.synaptic_events_processed += 1;

                // Microscopic energy dissipation for this synaptic event:
                // E_syn = E_diss(SNSPD) + 1/2 * L_loop * (delta_I)^2
                let e_snspd = self.snspd.dissipation_energy_joules(10.0);
                let e_loop = 0.5 * self.flux_loop.loop_inductance * delta_i * delta_i;
                self.cumulative_energy_joules += e_snspd + e_loop;
            }
        }
    }

    /// Advances the neuron dynamics over time-step $dt$.
    ///
    /// 1. Dendritic flux leaky integration: advances $I_{loop}(t)$.
    /// 2. Somatic junction total current: $I_{JJ} = I_{bias} + I_{loop}$.
    /// 3. Threshold check: if $I_{JJ} > I_c$, somatic junction undergoes a $2\pi$ phase slip.
    /// 4. Optical emission: triggers optical emitter pulse packet.
    /// 5. Self-reset: decrements loop flux by $\Phi_0$.
    ///
    /// Returns `Some(optical_pulse_energy_joules)` if an action potential was emitted, else `None`.
    pub fn step(&mut self, dt: f64, current_time: f64) -> Option<f64> {
        // Leaky dendritic relaxation
        self.flux_loop.step(dt, 0.0);

        // Total current incident on the somatic Josephson junction
        let i_somatic = self.somatic_bias_current + self.flux_loop.current;

        // Somatic thresholding: check if junction phase slips
        // If i_somatic > Ic, the junction switches to the resistive state
        if i_somatic >= self.somatic_jj.ic
            && self.flux_loop.stored_flux >= self.somatic_threshold_flux
        {
            // Somatic junction phase slips by 2*pi
            self.somatic_jj.phase += 2.0 * std::f64::consts::PI;

            // Optical pulse emission: driver delivers picosecond current pulse to the optical emitter
            let driver_pulse_current = 3.0e-6; // 3 uA driver pulse
            let pulse_duration = 3.0e-12; // 3 ps cryogenic pulse
            let (pulse_energy, _photons) = self
                .optical_emitter
                .emit_pulse(driver_pulse_current, pulse_duration);

            // Self-resetting flux mechanism: a single flux quantum leaves the dendritic loop
            self.flux_loop.add_flux_quantum(-1.0);

            // Record spike and energy
            self.spike_count += 1;
            self.last_spike_time = Some(current_time);

            // Somatic phase-slip dissipation: E_slip = Phi_0 * Ic
            let e_slip = FLUX_QUANTUM * self.somatic_jj.ic;
            self.cumulative_energy_joules += e_slip + pulse_energy;

            Some(pulse_energy)
        } else {
            // Passive emitter cooling
            self.optical_emitter.step(dt, 0.0);
            None
        }
    }

    /// Evaluates current performance and energy metrics of the neuron.
    pub fn metrics(&self) -> SoenMetrics {
        let events = self.synaptic_events_processed.max(1) as f64;
        let avg_e = self.cumulative_energy_joules / events;
        let avg_e_aj = avg_e * 1e18; // 1 aJ = 1e-18 J
        let carnot_overhead = 1000.0; // 4K cooling penalty

        // Somatic firing latency ~ tau_JTL ~ Phi_0 / (2*pi * Ic * Rn)
        let vc = self.somatic_jj.characteristic_voltage().max(1e-6);
        let latency_ps = (FLUX_QUANTUM / (2.0 * std::f64::consts::PI * vc)) * 1e12;

        SoenMetrics {
            spike_count: self.spike_count,
            average_synaptic_energy_joules: avg_e,
            average_synaptic_energy_attojoules: avg_e_aj,
            wall_plug_energy_attojoules: avg_e_aj * carnot_overhead,
            somatic_firing_latency_ps: latency_ps,
            flux_retention_ns: self.flux_loop.leak_time_constant() * 1e9,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_snspd_hotspot_and_current_diversion() {
        let snspd = SnspdModel::default();
        let i_bias = snspd.bias_current();
        assert!((i_bias - 23.75e-6).abs() < 1e-9);

        // At t = 0, diverted current is zero (rise time factor = 0)
        assert_eq!(snspd.diverted_current(0.0, 50.0), 0.0);

        // At t = 20 ps, current is diverted into load
        let i_div = snspd.diverted_current(20e-12, 50.0);
        assert!(i_div > 15.0e-6); // Significant current diverted

        // Diverted flux should be positive and finite
        let flux = snspd.diverted_flux(50.0, 100);
        assert!(flux > 0.0);
    }

    #[test]
    fn test_cryo_optical_emitter_pulse() {
        let mut emitter = CryoOpticalEmitter::default();
        let (energy, photons) = emitter.emit_pulse(20e-6, 50e-12);
        assert!(energy > 0.0);
        assert!(photons > 0.0);
        // Energy of ~a few photons at 850nm is on the order of 1e-18 J
        assert!(energy < 1e-15);
    }

    #[test]
    fn test_superconducting_flux_loop_quantization() {
        let mut loop_model = SuperconductingFluxLoop::default();
        assert_eq!(loop_model.flux_quantum_count, 0);

        loop_model.add_flux_quantum(1.0);
        assert_eq!(loop_model.flux_quantum_count, 1);
        assert!((loop_model.stored_flux - FLUX_QUANTUM).abs() < 1e-25);

        let delta_i = FLUX_QUANTUM / loop_model.loop_inductance;
        assert!((loop_model.current - delta_i).abs() < 1e-12);

        loop_model.add_flux_quantum(-1.0);
        assert_eq!(loop_model.flux_quantum_count, 0);
        assert!(loop_model.stored_flux.abs() < 1e-25);
    }

    #[test]
    fn test_soen_neuron_integration_and_firing() {
        let mut neuron = SoenNeuron::new(0, 3, 50e-6);
        assert_eq!(neuron.spike_count, 0);

        // Inject 2 optical spikes onto channel 0 with weight 1.0
        neuron.receive_synaptic_spikes(&[(0, 2.0)], 0.0);
        assert!(neuron.flux_loop.stored_flux > 0.0);

        // Advance time: neuron should fire an optical action potential
        let dt = 1e-12; // 1 ps
        let mut fired = false;
        for step in 0..100 {
            let t = step as f64 * dt;
            if let Some(pulse_energy) = neuron.step(dt, t) {
                fired = true;
                assert!(pulse_energy > 0.0);
                break;
            }
        }
        assert!(
            fired,
            "SOEN neuron should fire when flux threshold is exceeded"
        );
        assert_eq!(neuron.spike_count, 1);

        let metrics = neuron.metrics();
        // Synaptic event energy should be in the sub-attojoule to few-attojoule range
        assert!(metrics.average_synaptic_energy_attojoules < 20.0);
        assert!(metrics.average_synaptic_energy_attojoules > 0.001);
    }
}
