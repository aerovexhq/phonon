#![deny(unsafe_code)]

use std::f64::consts::PI;

/// Simple complex number implementation in pure safe Rust for AC frequency impedance analysis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    pub fn from_real(re: f64) -> Self {
        Self { re, im: 0.0 }
    }

    pub fn norm(&self) -> f64 {
        (self.re * self.re + self.im * self.im).sqrt()
    }

    pub fn add(&self, other: Complex) -> Complex {
        Complex::new(self.re + other.re, self.im + other.im)
    }

    pub fn sub(&self, other: Complex) -> Complex {
        Complex::new(self.re - other.re, self.im - other.im)
    }

    pub fn mul(&self, other: Complex) -> Complex {
        Complex::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }

    pub fn inv(&self) -> Complex {
        let denom = self.re * self.re + self.im * self.im;
        if denom < 1e-30 {
            Complex::new(1e15, 0.0)
        } else {
            Complex::new(self.re / denom, -self.im / denom)
        }
    }

    pub fn div(&self, other: Complex) -> Complex {
        self.mul(other.inv())
    }
}

/// Specification for a decoupling capacitor tier.
#[derive(Debug, Clone)]
pub struct DecouplingCapSpec {
    pub name: String,
    /// Capacitance per unit in Farads (e.g. 47 uF = 47e-6 F).
    pub capacitance_f: f64,
    /// Equivalent Series Resistance per unit in Ohms (e.g. 3 mOhm = 0.003 Ohm).
    pub esr_ohm: f64,
    /// Equivalent Series Inductance per unit in Henries (e.g. 200 pH = 2e-10 H).
    pub esl_henry: f64,
    /// Quantity of parallel capacitors in this bank.
    pub count: usize,
}

impl DecouplingCapSpec {
    pub fn new(name: &str, capacitance_f: f64, esr_ohm: f64, esl_henry: f64, count: usize) -> Self {
        Self {
            name: name.to_string(),
            capacitance_f,
            esr_ohm,
            esl_henry,
            count: count.max(1),
        }
    }

    /// Evaluates equivalent complex admittance of the entire capacitor bank at frequency f (Hz).
    pub fn admittance_at(&self, freq_hz: f64) -> Complex {
        let omega = 2.0 * PI * freq_hz.max(1.0);
        let n = self.count as f64;

        // Effective bank parameters in parallel
        let c_tot = self.capacitance_f * n;
        let esr_tot = self.esr_ohm / n;
        let esl_tot = self.esl_henry / n;

        // Branch impedance Z = ESR + j*omega*ESL + 1/(j*omega*C)
        // Z = ESR + j*(omega*ESL - 1/(omega*C))
        let xc = 1.0 / (omega * c_tot).max(1e-25);
        let xl = omega * esl_tot;
        let z_branch = Complex::new(esr_tot, xl - xc);

        z_branch.inv()
    }
}

/// Parameters for Voltage Regulator Module (VRM) output impedance model.
#[derive(Debug, Clone)]
pub struct VrmModelParams {
    /// Nominal DC output voltage in Volts (e.g. 0.85 V).
    pub v_dd_v: f64,
    /// DC closed-loop series output resistance in Ohms (e.g. 0.05 mOhm = 5e-5 Ohm).
    pub r_dc_ohm: f64,
    /// Open-loop output series inductance in Henries (e.g. 40 nH = 4e-8 H).
    pub l_out_henry: f64,
    /// VRM closed-loop bandwidth in Hz (e.g. 100 kHz = 100e3 Hz).
    pub bandwidth_hz: f64,
    /// Low-frequency open-loop DC loop gain A0 (e.g. 2000.0).
    pub loop_gain_a0: f64,
}

impl Default for VrmModelParams {
    fn default() -> Self {
        Self {
            v_dd_v: 0.85,
            r_dc_ohm: 4.5e-5, // 0.045 mOhm
            l_out_henry: 3.5e-8, // 35 nH multiphase buck output
            bandwidth_hz: 1.2e5, // 120 kHz
            loop_gain_a0: 1500.0,
        }
    }
}

impl VrmModelParams {
    /// Evaluates VRM complex output impedance at frequency f (Hz).
    pub fn impedance_at(&self, freq_hz: f64) -> Complex {
        let omega = 2.0 * PI * freq_hz.max(1.0);

        // Open-loop output impedance: Z_ol(s) = R_dc * (1 + A0) + s * L_out
        let r_ol = self.r_dc_ohm * (1.0 + self.loop_gain_a0);
        let z_ol = Complex::new(r_ol, omega * self.l_out_henry);

        // First-order loop gain: T(s) = A0 / (1 + s / omega_bw)
        let omega_bw = 2.0 * PI * self.bandwidth_hz;
        let den_t = Complex::new(1.0, omega / omega_bw);
        let t_s = Complex::new(self.loop_gain_a0, 0.0).div(den_t);

        // Closed-loop impedance Z_cl(s) = Z_ol(s) / (1 + T(s))
        let one_plus_t = Complex::new(1.0, 0.0).add(t_s);
        z_ol.div(one_plus_t)
    }
}

/// Comprehensive multi-decade Power Delivery Network parameters.
#[derive(Debug, Clone)]
pub struct PdnNetworkParams {
    pub vrm: VrmModelParams,

    // PCB Plane & bulk decoupling
    pub pcb_plane_resistance_ohm: f64,
    pub pcb_plane_inductance_henry: f64,
    pub bulk_capacitors: Vec<DecouplingCapSpec>,

    // Package Substrate & package decoupling
    pub package_resistance_ohm: f64,
    pub package_inductance_henry: f64,
    pub package_capacitors: Vec<DecouplingCapSpec>,

    // On-Die Grid & Deep Trench Capacitors (DTC)
    pub on_die_grid_resistance_ohm: f64,
    pub on_die_grid_inductance_henry: f64,
    pub on_die_capacitors: Vec<DecouplingCapSpec>,

    // Target impedance design specifications
    pub load_step_current_a: f64,
    pub max_voltage_ripple_ratio: f64, // e.g. 0.05 = +/-5%
}

impl Default for PdnNetworkParams {
    fn default() -> Self {
        Self {
            vrm: VrmModelParams::default(),

            // PCB Plane interconnect: ~0.04 mOhm, ~2.0 pH
            pcb_plane_resistance_ohm: 4.0e-5,
            pcb_plane_inductance_henry: 2.0e-12, // 2.0 pH
            bulk_capacitors: vec![
                DecouplingCapSpec::new("PCB 47uF Bulk MLCC (1206)", 4.7e-5, 0.003, 4.5e-10, 80),
                DecouplingCapSpec::new("PCB 10uF Mid MLCC (0805)", 1.0e-5, 0.005, 3.0e-10, 120),
            ],

            // Package substrate interconnect: ~0.05 mOhm, ~1.5 pH
            package_resistance_ohm: 5.0e-5,
            package_inductance_henry: 1.5e-12, // 1.5 pH
            package_capacitors: vec![
                DecouplingCapSpec::new("Pkg 1.0uF LSC MLCC (0402)", 1.0e-6, 0.004, 8.0e-11, 150),
                DecouplingCapSpec::new("Pkg 0.22uF DSC MLCC (0201)", 2.2e-7, 0.008, 4.0e-11, 200),
            ],

            // On-Die grid and Deep Trench Capacitors (DTC) + MIM
            on_die_grid_resistance_ohm: 2.0e-5,
            on_die_grid_inductance_henry: 5.0e-13, // 0.5 pH
            on_die_capacitors: vec![
                DecouplingCapSpec::new("On-Die Deep Trench Caps (DTC)", 4.5e-6, 0.0006, 2.0e-12, 10),
                DecouplingCapSpec::new("On-Die MIM / Core C_intrinsic", 8.0e-7, 0.0015, 1.0e-12, 10),
            ],

            load_step_current_a: 800.0, // 800 A dynamic core load step
            max_voltage_ripple_ratio: 0.05, // +/- 5% allowed ripple (42.5 mV on 0.85V)
        }
    }
}

impl PdnNetworkParams {
    /// Computes the flat target impedance Z_target = (V_dd * ripple) / Delta_I.
    pub fn target_impedance_ohm(&self) -> f64 {
        let max_delta_v = self.vrm.v_dd_v * self.max_voltage_ripple_ratio;
        max_delta_v / self.load_step_current_a.max(1.0)
    }

    /// Evaluates the total on-die PDN complex impedance at frequency f (Hz).
    pub fn impedance_at_die(&self, freq_hz: f64) -> Complex {
        let omega = 2.0 * PI * freq_hz.max(1.0);

        // Stage 1: VRM output in parallel with PCB bulk capacitors
        let z_vrm = self.vrm.impedance_at(freq_hz);
        let mut y_bulk = Complex::new(0.0, 0.0);
        for cap in &self.bulk_capacitors {
            y_bulk = y_bulk.add(cap.admittance_at(freq_hz));
        }
        let z_stage1 = z_vrm.inv().add(y_bulk).inv();

        // Stage 2: In series with PCB plane impedance
        let z_pcb_plane = Complex::new(self.pcb_plane_resistance_ohm, omega * self.pcb_plane_inductance_henry);
        let z_stage2 = z_stage1.add(z_pcb_plane);

        // Stage 3: In parallel with Package capacitors (LSC/DSC)
        let mut y_pkg = Complex::new(0.0, 0.0);
        for cap in &self.package_capacitors {
            y_pkg = y_pkg.add(cap.admittance_at(freq_hz));
        }
        let z_stage3 = z_stage2.inv().add(y_pkg).inv();

        // Stage 4: In series with Package substrate/bump impedance
        let z_pkg_interconnect = Complex::new(self.package_resistance_ohm, omega * self.package_inductance_henry);
        let z_stage4 = z_stage3.add(z_pkg_interconnect);

        // Stage 5: In parallel with On-Die capacitors and grid
        let mut y_die = Complex::new(0.0, 0.0);
        for cap in &self.on_die_capacitors {
            y_die = y_die.add(cap.admittance_at(freq_hz));
        }
        let z_stage5 = z_stage4.inv().add(y_die).inv();

        // Stage 6: In series with on-die grid resistance and inductance
        let z_grid = Complex::new(self.on_die_grid_resistance_ohm, omega * self.on_die_grid_inductance_henry);
        z_stage5.add(z_grid)
    }

    /// Total on-die decoupling capacitance in Farads.
    pub fn total_on_die_capacitance_f(&self) -> f64 {
        self.on_die_capacitors
            .iter()
            .map(|c| c.capacitance_f * (c.count as f64))
            .sum()
    }

    /// Total package decoupling capacitance in Farads.
    pub fn total_package_capacitance_f(&self) -> f64 {
        self.package_capacitors
            .iter()
            .map(|c| c.capacitance_f * (c.count as f64))
            .sum()
    }

    /// Total bulk board decoupling capacitance in Farads.
    pub fn total_bulk_capacitance_f(&self) -> f64 {
        self.bulk_capacitors
            .iter()
            .map(|c| c.capacitance_f * (c.count as f64))
            .sum()
    }
}

/// A point on the frequency-dependent PDN impedance curve.
#[derive(Debug, Clone)]
pub struct ImpedanceSpectrumPoint {
    pub freq_hz: f64,
    pub impedance_ohm: f64,
    pub impedance_mohm: f64,
    pub target_impedance_mohm: f64,
    pub phase_deg: f64,
    pub violates_target: bool,
}

/// Identified anti-resonance peak in the PDN spectrum.
#[derive(Debug, Clone)]
pub struct AntiResonancePeak {
    pub name: String,
    pub freq_hz: f64,
    pub peak_impedance_mohm: f64,
    pub target_impedance_mohm: f64,
    pub margin_pct: f64,
    pub is_violation: bool,
}

/// Complete multi-decade PDN impedance profile.
#[derive(Debug, Clone)]
pub struct PdnImpedanceProfile {
    pub points: Vec<ImpedanceSpectrumPoint>,
    pub target_impedance_mohm: f64,
    pub max_impedance_mohm: f64,
    pub anti_resonances: Vec<AntiResonancePeak>,
    pub is_fully_compliant: bool,
}

/// Generates the multi-decade PDN impedance spectrum from 1 kHz to 1 GHz.
pub fn calculate_pdn_impedance_profile(
    params: &PdnNetworkParams,
    points_per_decade: usize,
) -> PdnImpedanceProfile {
    let start_exp = 3.0; // 10^3 = 1 kHz
    let end_exp = 9.0;   // 10^9 = 1 GHz
    let total_decades = end_exp - start_exp;
    let num_points = ((total_decades * (points_per_decade as f64)).round() as usize).max(30);

    let z_target_ohm = params.target_impedance_ohm();
    let z_target_mohm = z_target_ohm * 1e3;

    let mut points = Vec::with_capacity(num_points);
    let mut max_z_mohm = 0.0f64;

    for i in 0..num_points {
        let exp = start_exp + (i as f64) * (total_decades / (num_points - 1) as f64);
        let freq_hz = 10.0f64.powf(exp);

        let z_complex = params.impedance_at_die(freq_hz);
        let z_norm = z_complex.norm();
        let z_mohm = z_norm * 1e3;
        let phase_deg = z_complex.im.atan2(z_complex.re) * (180.0 / PI);

        if z_mohm > max_z_mohm {
            max_z_mohm = z_mohm;
        }

        let violates = z_mohm > z_target_mohm;

        points.push(ImpedanceSpectrumPoint {
            freq_hz,
            impedance_ohm: z_norm,
            impedance_mohm: z_mohm,
            target_impedance_mohm: z_target_mohm,
            phase_deg,
            violates_target: violates,
        });
    }

    // Peak detection: locate local maxima in |Z(f)|
    let mut anti_resonances = Vec::new();
    for i in 1..(points.len() - 1) {
        let prev = points[i - 1].impedance_mohm;
        let curr = points[i].impedance_mohm;
        let next = points[i + 1].impedance_mohm;

        if curr > prev && curr > next && curr > 0.01 {
            let f = points[i].freq_hz;
            let name = if f < 5e5 {
                "3rd Droop (VRM-Bulk Anti-Resonance)".to_string()
            } else if f < 3e7 {
                "2nd Droop (PCB-Package Anti-Resonance)".to_string()
            } else {
                "1st Droop (Package-Die Anti-Resonance)".to_string()
            };

            let margin = ((z_target_mohm - curr) / z_target_mohm) * 100.0;
            let is_violation = curr > z_target_mohm;

            anti_resonances.push(AntiResonancePeak {
                name,
                freq_hz: f,
                peak_impedance_mohm: curr,
                target_impedance_mohm: z_target_mohm,
                margin_pct: margin,
                is_violation,
            });
        }
    }

    let is_fully_compliant = max_z_mohm <= z_target_mohm;

    PdnImpedanceProfile {
        points,
        target_impedance_mohm: z_target_mohm,
        max_impedance_mohm: max_z_mohm,
        anti_resonances,
        is_fully_compliant,
    }
}
