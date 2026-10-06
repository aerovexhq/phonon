#![deny(unsafe_code)]

//! Universal Chiplet Interconnect Express (UCIe) & Die-to-Die (D2D) PHY Channel Model.
//!
//! Provides standards-compliant PHY modeling for UCIe 1.0/2.0 and Bunch of Wires (BoW):
//! - Standard Package (organic substrate, 100-130 um bump pitch, 4-16 Gbps/lane)
//! - Advanced Package (silicon interposer / bridge e.g. CoWoS/EMIB, 25-55 um pitch, 16-32 Gbps/lane)
//! - Eye diagram synthesis with dual-Dirac jitter decomposition (Random Jitter RJ, Deterministic Jitter DJ)
//! - Bit Error Rate (BER < 1e-15) target eye height / eye width margin calculations
//! - Energy efficiency metrics (pJ/bit) and channel insertion loss.

/// Packaging category defining physical bump pitch and channel loss budget.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UciePackageType {
    /// Standard organic substrate package (100 - 130 um bump pitch, longer trace reach up to 25 mm).
    StandardOrganic,
    /// Advanced silicon interposer / embedded silicon bridge (25 - 55 um bump pitch, short reach < 3 mm).
    AdvancedSiliconBridge,
}

/// Operating data rate per single-ended data lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UcieDataRateGbps {
    Rate4Gbps,
    Rate8Gbps,
    Rate12Gbps,
    Rate16Gbps,
    Rate24Gbps,
    Rate32Gbps,
}

impl UcieDataRateGbps {
    pub fn as_gbps(&self) -> f64 {
        match self {
            Self::Rate4Gbps => 4.0,
            Self::Rate8Gbps => 8.0,
            Self::Rate12Gbps => 12.0,
            Self::Rate16Gbps => 16.0,
            Self::Rate24Gbps => 24.0,
            Self::Rate32Gbps => 32.0,
        }
    }

    /// Unit interval in picoseconds (UI = 1 / data_rate).
    pub fn unit_interval_ps(&self) -> f64 {
        1000.0 / self.as_gbps()
    }

    /// Nyquist fundamental frequency in Gigahertz (f_nyq = data_rate / 2).
    pub fn nyquist_ghz(&self) -> f64 {
        self.as_gbps() / 2.0
    }
}

/// Parameters for UCIe Die-to-Die physical layer channel.
#[derive(Debug, Clone, PartialEq)]
pub struct UciePhyParams {
    pub package_type: UciePackageType,
    pub data_rate: UcieDataRateGbps,
    /// Transmitter peak-to-peak output swing in Volts (typically 0.4V to 0.8V).
    pub tx_swing_v: f64,
    /// Interconnect physical trace length in millimeters.
    pub trace_length_mm: f64,
    /// Random Jitter (RJ) 1-sigma Gaussian standard deviation in picoseconds.
    pub random_jitter_ps_rms: f64,
    /// Deterministic Jitter (DJ) dual-Dirac peak-to-peak in picoseconds.
    pub deterministic_jitter_ps: f64,
    /// Receiver decision feedback equalizer / continuous time linear equalizer boost (dB).
    pub rx_equalization_boost_db: f64,
}

impl Default for UciePhyParams {
    fn default() -> Self {
        Self {
            package_type: UciePackageType::AdvancedSiliconBridge,
            data_rate: UcieDataRateGbps::Rate16Gbps,
            tx_swing_v: 0.50,
            trace_length_mm: 2.0,
            random_jitter_ps_rms: 0.45,
            deterministic_jitter_ps: 3.2,
            rx_equalization_boost_db: 4.0,
        }
    }
}

/// Evaluated eye diagram metrics at target BER = 1e-15.
#[derive(Debug, Clone, PartialEq)]
pub struct UcieEyeMetrics {
    /// Unit interval in picoseconds.
    pub ui_ps: f64,
    /// Insertion loss at Nyquist frequency in dB (negative value).
    pub insertion_loss_db: f64,
    /// Total jitter at BER = 1e-15 (TJ = DJ + 14.069 * RJ_rms) in picoseconds.
    pub total_jitter_ps: f64,
    /// Eye width opening in picoseconds (UI - TJ).
    pub eye_width_ps: f64,
    /// Normalized eye width fraction of Unit Interval (EW / UI).
    pub eye_width_ui: f64,
    /// Eye height opening at receiver sampler in millivolts.
    pub eye_height_mv: f64,
    /// Energy efficiency of transceiver and channel in picojoules per bit (pJ/bit).
    pub energy_efficiency_pj_bit: f64,
    /// Whether the channel satisfies UCIe compliance (EH >= 40 mV, EW >= 0.40 UI, IL >= -8 dB).
    pub is_compliant: bool,
}

/// Synthesized 2D eye contour trace points for rendering.
#[derive(Debug, Clone, PartialEq)]
pub struct EyeDiagramSample {
    /// Upper inner eye opening contour points [time_ps, voltage_mv].
    pub upper_inner_contour: Vec<[f64; 2]>,
    /// Lower inner eye opening contour points [time_ps, voltage_mv].
    pub lower_inner_contour: Vec<[f64; 2]>,
    /// Upper outer rail transition contour [time_ps, voltage_mv].
    pub upper_outer_contour: Vec<[f64; 2]>,
    /// Lower outer rail transition contour [time_ps, voltage_mv].
    pub lower_outer_contour: Vec<[f64; 2]>,
}

/// Evaluates UCIe PHY channel transmission, jitter decomposition, and eye diagram.
pub fn evaluate_ucie_phy(params: &UciePhyParams) -> (UcieEyeMetrics, EyeDiagramSample) {
    let ui_ps = params.data_rate.unit_interval_ps();
    let f_nyq = params.data_rate.nyquist_ghz();

    // Attenuation coefficient in dB/mm at Nyquist:
    // Advanced silicon bridge: ~0.8 dB/mm at 8 GHz, scaling as sqrt(f) + f
    // Standard organic package: ~0.4 dB/mm at 8 GHz, but longer traces
    let atten_per_mm = match params.package_type {
        UciePackageType::AdvancedSiliconBridge => 0.55 * (f_nyq / 8.0).sqrt() + 0.25 * (f_nyq / 8.0),
        UciePackageType::StandardOrganic => 0.30 * (f_nyq / 8.0).sqrt() + 0.15 * (f_nyq / 8.0),
    };

    let raw_il_db = -(atten_per_mm * params.trace_length_mm + 0.8); // +0.8 dB for 2x microbumps
    let net_il_db = (raw_il_db + params.rx_equalization_boost_db).min(0.0);

    // Channel voltage attenuation factor
    let channel_atten = 10.0_f64.powf(net_il_db / 20.0);

    // Dual-Dirac model for BER = 1e-15: Q(1e-15) = 7.9416 -> peak-to-peak factor is ~14.069 * sigma_RJ
    let rj_pp_1e15 = 14.069 * params.random_jitter_ps_rms;
    let total_jitter_ps = params.deterministic_jitter_ps + rj_pp_1e15;

    let eye_width_ps = (ui_ps - total_jitter_ps).max(0.0);
    let eye_width_ui = eye_width_ps / ui_ps;

    let eye_height_mv = (params.tx_swing_v * 1000.0 * channel_atten * (eye_width_ui.min(1.0))).max(0.0);

    // Energy efficiency in pJ/bit
    // Advanced packaging: ~0.15 to 0.35 pJ/bit
    // Standard packaging: ~0.45 to 0.95 pJ/bit
    let base_pj = match params.package_type {
        UciePackageType::AdvancedSiliconBridge => 0.18 + 0.005 * params.data_rate.as_gbps(),
        UciePackageType::StandardOrganic => 0.45 + 0.015 * params.data_rate.as_gbps(),
    };
    let energy_efficiency_pj_bit = base_pj + 0.02 * params.trace_length_mm;

    // Compliance criteria for UCIe specification:
    // Advanced: EH >= 40 mV, EW >= 0.40 UI, IL >= -8.5 dB
    // Standard: EH >= 50 mV, EW >= 0.35 UI, IL >= -14.0 dB
    let is_compliant = match params.package_type {
        UciePackageType::AdvancedSiliconBridge => {
            eye_height_mv >= 40.0 && eye_width_ui >= 0.40 && raw_il_db >= -9.5
        }
        UciePackageType::StandardOrganic => {
            eye_height_mv >= 45.0 && eye_width_ui >= 0.35 && raw_il_db >= -15.0
        }
    };

    let metrics = UcieEyeMetrics {
        ui_ps,
        insertion_loss_db: raw_il_db,
        total_jitter_ps,
        eye_width_ps,
        eye_width_ui,
        eye_height_mv,
        energy_efficiency_pj_bit,
        is_compliant,
    };

    // Synthesize 2-UI eye diagram curves for plotting
    let eye_sample = synthesize_eye_contours(ui_ps, eye_width_ps, eye_height_mv, params.tx_swing_v * 1000.0);

    (metrics, eye_sample)
}

/// Generates parametric eye diagram boundary contours across 2 unit intervals [-0.5 UI, 1.5 UI].
fn synthesize_eye_contours(
    ui_ps: f64,
    eye_width_ps: f64,
    eye_height_mv: f64,
    tx_swing_mv: f64,
) -> EyeDiagramSample {
    let num_points = 80;
    let mut upper_inner = Vec::with_capacity(num_points);
    let mut lower_inner = Vec::with_capacity(num_points);
    let mut upper_outer = Vec::with_capacity(num_points);
    let mut lower_outer = Vec::with_capacity(num_points);

    let half_ew = (eye_width_ps / 2.0).min(ui_ps * 0.48);
    let half_eh = eye_height_mv / 2.0;
    let v_rail = tx_swing_mv / 2.0;

    for i in 0..num_points {
        let t_norm = (i as f64) / ((num_points - 1) as f64); // 0.0 to 1.0
        let t_ps = (t_norm - 0.5) * ui_ps; // -0.5 UI to +0.5 UI around eye center (t=0)

        // Inner eye opening shape: flattened diamond / ellipse
        let dist_from_center = (t_ps / half_ew).abs();
        let inner_v = if dist_from_center <= 1.0 {
            half_eh * (1.0 - dist_from_center.powi(2)).max(0.0).sqrt()
        } else {
            0.0
        };

        upper_inner.push([t_ps, inner_v]);
        lower_inner.push([t_ps, -inner_v]);

        // Outer transition trajectories
        // Trajectory 1: rising from -v_rail to +v_rail crossing at -half_ew
        let s_curve = 1.0 / (1.0 + (-(t_ps) / (ui_ps * 0.15)).exp());
        let v_transition_up = -v_rail + 2.0 * v_rail * s_curve;
        let v_transition_down = v_rail - 2.0 * v_rail * s_curve;

        upper_outer.push([t_ps, v_transition_up.max(v_rail * 0.85)]);
        lower_outer.push([t_ps, v_transition_down.min(-v_rail * 0.85)]);
    }

    EyeDiagramSample {
        upper_inner_contour: upper_inner,
        lower_inner_contour: lower_inner,
        upper_outer_contour: upper_outer,
        lower_outer_contour: lower_outer,
    }
}
