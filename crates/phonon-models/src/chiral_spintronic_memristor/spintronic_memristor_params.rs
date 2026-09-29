//! Parameters and metrics for chiral phonon-driven spintronic memristors
//! and neuromorphic acoustic crossbar accelerators.

/// Parameters for chiral phonon-driven domain-wall memristors and crossbars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpintronicMemristorParams {
    /// Acoustic SAW central drive frequency $f_{\mathrm{SAW}}$ in $\text{GHz}$ (nominal $1.0 - 6.0\text{ GHz}$).
    pub acoustic_frequency_ghz: f64,
    /// Nanowire length $L_{\mathrm{wire}}$ in micrometers (nominal $0.5 - 3.0\,\mu\text{m}$).
    pub nanowire_length_um: f64,
    /// Nanowire cross-sectional width in nanometers (nominal $20.0 - 80.0\text{ nm}$).
    pub nanowire_width_nm: f64,
    /// Chiral acoustic spin-transfer torque efficiency parameter $\eta_{\mathrm{aSTT}}$ (nominal $0.3 - 0.9$).
    pub astt_efficiency: f64,
    /// Acoustic programming pulse duration $\tau_{\mathrm{pulse}}$ in nanoseconds (nominal $0.5 - 5.0\text{ ns}$).
    pub pulse_duration_ns: f64,
    /// Acoustic pulse power in microwatts (nominal $0.2 - 3.0\,\mu\text{W}$).
    pub pulse_power_uw: f64,
    /// Thermal stability factor $\Delta E / (k_B T)$ (nominal $45.0 - 75.0$).
    pub thermal_stability_factor: f64,
    /// Magnetic tunnel junction tunnel magnetoresistance ratio $\mathrm{TMR}$ in percent (nominal $150.0 - 450.0\%$).
    pub tmr_ratio_pct: f64,
    /// Crossbar array dimension $N \times N$ (nominal $32 - 256$).
    pub crossbar_dimension: usize,
}

impl Default for SpintronicMemristorParams {
    fn default() -> Self {
        Self {
            acoustic_frequency_ghz: 2.8,
            nanowire_length_um: 1.2,
            nanowire_width_nm: 40.0,
            astt_efficiency: 0.65,
            pulse_duration_ns: 1.5,
            pulse_power_uw: 1.2,
            thermal_stability_factor: 55.0,
            tmr_ratio_pct: 280.0,
            crossbar_dimension: 64,
        }
    }
}

/// Evaluated metrics for chiral phonon-driven spintronic memristors and crossbars.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpintronicMemristorMetrics {
    /// Synaptic programming energy per switching event in femtojoules ($\le 10.0\text{ fJ}$).
    pub programming_energy_fj: f64,
    /// Non-volatile data retention time in years ($\ge 10.0\text{ years}$).
    pub retention_time_years: f64,
    /// Conductance dynamic on/off ratio $G_{\mathrm{LRS}} / G_{\mathrm{HRS}}$ ($\ge 10.0$).
    pub conductance_on_off_ratio: f64,
    /// Spike-timing-dependent plasticity (STDP) learning fidelity in percent ($\ge 95.0\%$).
    pub stdp_learning_fidelity_pct: f64,
    /// Neuromorphic crossbar compute energy efficiency in TOPS/W ($\ge 150.0\text{ TOPS/W}$).
    pub crossbar_energy_efficiency_topsw: f64,
    /// Synaptic weight programming non-linearity error in percent ($\le 2.5\%$).
    pub weight_linearity_error_pct: f64,
}

impl SpintronicMemristorParams {
    /// Creates a new parameter set for chiral spintronic memristors.
    pub fn new(
        f_ghz: f64,
        length_um: f64,
        astt: f64,
        duration_ns: f64,
        power_uw: f64,
        stability: f64,
    ) -> Self {
        Self {
            acoustic_frequency_ghz: f_ghz.clamp(0.2, 20.0),
            nanowire_length_um: length_um.clamp(0.1, 20.0),
            astt_efficiency: astt.clamp(0.1, 1.0),
            pulse_duration_ns: duration_ns.clamp(0.1, 50.0),
            pulse_power_uw: power_uw.clamp(0.01, 100.0),
            thermal_stability_factor: stability.clamp(20.0, 150.0),
            ..Default::default()
        }
    }
}
