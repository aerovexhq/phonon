#![deny(unsafe_code)]

//! DO-160G Section 22 Lightning Indirect Transient & Electro-Thermal Clamping Engine.
//!
//! Models electromagnetic indirect lightning pin injection waveforms (Waveform 4 & Waveform 5A,
//! Levels 1 through 5, up to 1600V / 1600A), on-chip TVS and SCR clamping dynamics,
//! transient power dissipation, and dynamic junction thermal rise Delta Tj(t).

/// DO-160G Section 22 Pin Injection Standard Test Waveform.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningWaveformKind {
    /// Waveform 4: Voltage impulse (6.4 us rise time, 69 us half-value decay).
    Waveform4,
    /// Waveform 5A: Current surge (40 us rise time, 120 us half-value decay).
    Waveform5A,
}

impl LightningWaveformKind {
    /// Characteristic rise time tau1 and decay time tau2 in seconds for double exponential model:
    /// V(t) = V_peak * (exp(-t/tau2) - exp(-t/tau1)) * k_norm
    pub fn time_constants(&self) -> (f64, f64, f64) {
        match self {
            Self::Waveform4 => {
                // Rise ~ 6.4 us, decay ~ 69 us
                let tau1 = 2.45e-6;
                let tau2 = 87.0e-6;
                let k_norm = 1.09;
                (tau1, tau2, k_norm)
            }
            Self::Waveform5A => {
                // Rise ~ 40 us, decay ~ 120 us
                let tau1 = 15.0e-6;
                let tau2 = 155.0e-6;
                let k_norm = 1.18;
            (tau1, tau2, k_norm)
            }
        }
    }
}

/// DO-160G Section 22 Standard Test Severity Level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightningSeverityLevel {
    /// Level 1: 100 V / 20 A (Interior well-shielded compartment).
    Level1,
    /// Level 2: 250 V / 50 A (Partially shielded avionic bay).
    Level2,
    /// Level 3: 600 V / 120 A (Standard avionic equipment bay).
    Level3,
    /// Level 4: 1000 V / 500 A (Severe unshielded cable bundle).
    Level4,
    /// Level 5: 1600 V / 1600 A (Exterior wing, nacelle, control surface skin).
    Level5,
}

impl LightningSeverityLevel {
    /// Open-circuit peak voltage Voc in Volts and short-circuit peak current Isc in Amperes.
    pub fn peak_volt_amp(&self) -> (f64, f64) {
        match self {
            Self::Level1 => (100.0, 20.0),
            Self::Level2 => (250.0, 50.0),
            Self::Level3 => (600.0, 120.0),
            Self::Level4 => (1000.0, 500.0),
            Self::Level5 => (1600.0, 1600.0),
        }
    }

    /// Source generator impedance Rs in Ohms: Rs = Voc / Isc.
    pub fn source_impedance_ohms(&self) -> f64 {
        let (voc, isc) = self.peak_volt_amp();
        voc / isc
    }
}

/// Type of clamping transient protection device.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProtectionClampDevice {
    /// TVS Diode with breakdown voltage Vbr and dynamic on-resistance Rdyn.
    TvsDiode {
        /// Breakdown knee voltage in Volts.
        v_br: f64,
        /// Dynamic resistance in Ohms.
        r_dyn: f64,
    },
    /// SCR / Snapback device with trigger voltage Vt1, holding voltage Vh, and on-resistance Ron.
    ScrSnapback {
        /// Trigger voltage in Volts.
        v_t1: f64,
        /// Holding voltage in Volts.
        v_h: f64,
        /// Dynamic on-resistance in Ohms.
        r_on: f64,
    },
}

impl Default for ProtectionClampDevice {
    fn default() -> Self {
        Self::TvsDiode {
            v_br: 7.2,
            r_dyn: 0.045,
        }
    }
}

/// Lightning transient transient timepoint sample.
#[derive(Debug, Clone, Copy)]
pub struct LightningTimeSample {
    /// Time in microseconds (us).
    pub time_us: f64,
    /// Open-circuit generator source voltage in Volts.
    pub source_voltage_v: f64,
    /// Clamped terminal voltage in Volts.
    pub clamped_voltage_v: f64,
    /// Surge current through clamping device in Amperes.
    pub surge_current_a: f64,
    /// Instantaneous power dissipation in Watts.
    pub instantaneous_power_w: f64,
    /// Dynamic junction temperature rise Delta Tj in deg C.
    pub temp_rise_deg_c: f64,
}

/// DO-160G Indirect Lightning Transient Co-Simulator.
#[derive(Debug, Clone)]
pub struct LightningIndirectSimulator {
    /// Waveform kind (Waveform 4 vs 5A).
    pub waveform: LightningWaveformKind,
    /// Severity level (Level 1 to 5).
    pub severity: LightningSeverityLevel,
    /// Active protection clamping device.
    pub protection: ProtectionClampDevice,
    /// Ambient operating temperature in deg C (default 70.0 deg C for avionics bay).
    pub ambient_temp_deg_c: f64,
    /// Junction thermal resistance Rth in K/W.
    pub r_th_k_per_w: f64,
    /// Junction thermal time constant tau_th in seconds.
    pub tau_th_s: f64,
}

impl Default for LightningIndirectSimulator {
    fn default() -> Self {
        Self {
            waveform: LightningWaveformKind::Waveform4,
            severity: LightningSeverityLevel::Level4,
            protection: ProtectionClampDevice::default(),
            ambient_temp_deg_c: 70.0,
            r_th_k_per_w: 0.0012,
            tau_th_s: 45.0e-6, // 45 us thermal mass constant
        }
    }
}

impl LightningIndirectSimulator {
    /// Create a new lightning simulator with specified parameters.
    pub fn new(
        waveform: LightningWaveformKind,
        severity: LightningSeverityLevel,
        protection: ProtectionClampDevice,
    ) -> Self {
        Self {
            waveform,
            severity,
            protection,
            ..Default::default()
        }
    }

    /// Calculate open-circuit pulse voltage at time t (seconds).
    pub fn open_circuit_voltage_at(&self, t_s: f64) -> f64 {
        if t_s <= 0.0 {
            return 0.0;
        }

        let (voc, _) = self.severity.peak_volt_amp();
        let (tau1, tau2, k_norm) = self.waveform.time_constants();

        let v = (voc / k_norm) * ((-t_s / tau2).exp() - (-t_s / tau1).exp());
        v.max(0.0)
    }

    /// Calculate clamped voltage and current at time t (seconds) given open circuit voltage.
    pub fn evaluate_clamping(&self, voc: f64) -> (f64, f64) {
        let rs = self.severity.source_impedance_ohms();

        match self.protection {
            ProtectionClampDevice::TvsDiode { v_br, r_dyn } => {
                if voc <= v_br {
                    (voc, 0.0)
                } else {
                    // I = (Voc - Vbr) / (Rs + Rdyn)
                    let i_clamp = (voc - v_br) / (rs + r_dyn);
                    let v_clamp = v_br + i_clamp * r_dyn;
                    (v_clamp, i_clamp.max(0.0))
                }
            }
            ProtectionClampDevice::ScrSnapback { v_t1, v_h, r_on } => {
                if voc <= v_t1 {
                    (voc, 0.0)
                } else {
                    // Snapback into holding state: I = (Voc - Vh) / (Rs + Ron)
                    let i_clamp = (voc - v_h) / (rs + r_on);
                    let v_clamp = v_h + i_clamp * r_on;
                    (v_clamp, i_clamp.max(0.0))
                }
            }
        }
    }

    /// Simulate full transient waveform over duration (typically 200 us to 500 us).
    pub fn simulate_transient(&self, total_duration_us: f64, points: usize) -> Vec<LightningTimeSample> {
        let pts = points.clamp(50, 1000);
        let total_time_s = total_duration_us * 1.0e-6;
        let dt = total_time_s / (pts - 1) as f64;

        let mut samples = Vec::with_capacity(pts);
        let mut temp_rise = 0.0;

        for i in 0..pts {
            let t = i as f64 * dt;
            let t_us = t * 1.0e6;

            let voc = self.open_circuit_voltage_at(t);
            let (v_clamp, i_clamp) = self.evaluate_clamping(voc);
            let p_inst = v_clamp * i_clamp;

            // Thermal convolution step: d(Delta Tj)/dt = (P * Rth - Delta Tj) / tau_th
            let d_temp = (p_inst * self.r_th_k_per_w - temp_rise) / self.tau_th_s;
            temp_rise = (temp_rise + d_temp * dt).max(0.0);

            samples.push(LightningTimeSample {
                time_us: t_us,
                source_voltage_v: voc,
                clamped_voltage_v: v_clamp,
                surge_current_a: i_clamp,
                instantaneous_power_w: p_inst,
                temp_rise_deg_c: temp_rise,
            });
        }

        samples
    }

    /// Peak clamping voltage during transient in Volts.
    pub fn peak_clamping_voltage_v(&self) -> f64 {
        let (voc, _) = self.severity.peak_volt_amp();
        let (v_clamp, _) = self.evaluate_clamping(voc);
        v_clamp
    }

    /// Peak surge current through protection clamp in Amperes.
    pub fn peak_surge_current_a(&self) -> f64 {
        let (voc, _) = self.severity.peak_volt_amp();
        let (_, i_clamp) = self.evaluate_clamping(voc);
        i_clamp
    }

    /// Peak power dissipation in Watts.
    pub fn peak_dissipation_power_w(&self) -> f64 {
        self.peak_clamping_voltage_v() * self.peak_surge_current_a()
    }

    /// Peak dynamic junction temperature in deg C.
    pub fn peak_junction_temperature_deg_c(&self) -> f64 {
        let samples = self.simulate_transient(300.0, 150);
        let max_rise = samples
            .iter()
            .map(|s| s.temp_rise_deg_c)
            .fold(0.0_f64, |a, b| a.max(b));
        self.ambient_temp_deg_c + max_rise
    }

    /// Thermal failure limit in deg C (silicon junction melting / metallization spiking ~ 350 deg C).
    pub fn thermal_failure_limit_deg_c() -> f64 {
        350.0
    }

    /// Headroom margin to thermal failure in deg C.
    pub fn thermal_failure_margin_deg_c(&self) -> f64 {
        Self::thermal_failure_limit_deg_c() - self.peak_junction_temperature_deg_c()
    }
}
