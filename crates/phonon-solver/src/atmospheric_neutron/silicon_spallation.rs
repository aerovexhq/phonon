#![deny(unsafe_code)]

//! Silicon Spallation Recoil Kinematics & Avionics DO-254 DAL-A Soft Error Rate (SER) Engine.
//!
//! Models nuclear recoil reactions 28Si(n, alpha)25Mg and 28Si(n, p)28Al, Weibull SEU
//! cross-section, Failure-In-Time (FIT) calculations across memory blocks, and fault-tolerant
//! mitigation architectures (TMR, ECC, Lockstep) to certify DO-254 DAL-A airworthiness.

use super::neutron_spectrum::AtmosphericNeutronModel;

/// Silicon nuclear reaction channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiliconReactionChannel {
    /// 28Si(n, alpha)25Mg - threshold ~2.75 MeV, Q = -2.65 MeV.
    AlphaRecoil,
    /// 28Si(n, p)28Al - threshold ~4.0 MeV, Q = -3.86 MeV.
    ProtonRecoil,
    /// High-energy multi-fragment spallation (>50 MeV).
    DeepSpallation,
}

impl SiliconReactionChannel {
    /// Threshold neutron energy in MeV.
    pub fn threshold_energy_mev(&self) -> f64 {
        match self {
            Self::AlphaRecoil => 2.75,
            Self::ProtonRecoil => 4.00,
            Self::DeepSpallation => 20.0,
        }
    }

    /// Reaction cross section in millibarns (mb) as a function of neutron energy.
    pub fn cross_section_mb(&self, energy_mev: f64) -> f64 {
        let th = self.threshold_energy_mev();
        if energy_mev < th {
            return 0.0;
        }

        let delta_e = energy_mev - th;
        match self {
            Self::AlphaRecoil => {
                // Peaked around 12-16 MeV, ~180 mb peak
                let peak = 180.0;
                let width = 6.0;
                peak * (-0.5 * ((energy_mev - 14.0) / width).powi(2)).exp()
            }
            Self::ProtonRecoil => {
                // Peaked around 14-20 MeV, ~250 mb peak
                let peak = 240.0;
                let width = 8.0;
                peak * (-0.5 * ((energy_mev - 17.0) / width).powi(2)).exp()
            }
            Self::DeepSpallation => {
                // High-energy plateau ~380 mb
                380.0 * (1.0 - (-delta_e / 25.0).exp())
            }
        }
    }

    /// Average maximum recoil ion kinetic energy in MeV given incident neutron energy.
    pub fn max_recoil_energy_mev(&self, energy_mev: f64) -> f64 {
        let th = self.threshold_energy_mev();
        if energy_mev < th {
            return 0.0;
        }

        match self {
            Self::AlphaRecoil => {
                // Alpha gets ~70% of available kinetic energy, 25Mg recoil gets ~30%
                (energy_mev - 2.65) * 0.35
            }
            Self::ProtonRecoil => {
                // 28Al recoil gets ~10-15% of available energy
                (energy_mev - 3.86) * 0.15
            }
            Self::DeepSpallation => {
                // Heavier spallation fragments get dense recoil energy
                (energy_mev * 0.25).min(35.0)
            }
        }
    }
}

/// Redundancy and Fault Mitigation Architectures for DO-254 compliance.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MitigationArchitecture {
    /// Unmitigated simplex architecture (single core, no ECC).
    Simplex,
    /// SEC-DED (Single Error Correction, Double Error Detection) ECC memory.
    EccSecDed,
    /// Dual Modular Redundancy (DMR) with fail-safe comparator (fail-silent).
    DualModularRedundancy,
    /// Triple Modular Redundancy (TMR) with hardware majority voter.
    TripleModularRedundancy,
    /// Dual-Core Lockstep with periodic background memory scrubbing.
    LockstepWithScrubbing {
        /// Scrubbing interval in seconds (e.g. 0.05 s = 50 ms).
        scrub_interval_s: f64,
    },
}

/// DO-254 Design Assurance Level (DAL) classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Do254DalLevel {
    /// DAL-A: Catastrophic failure consequence (max allowable hazard rate < 1e-9 / flight hr).
    DalA,
    /// DAL-B: Hazardous / Severe-Major consequence (< 1e-7 / flight hr).
    DalB,
    /// DAL-C: Major consequence (< 1e-5 / flight hr).
    DalC,
    /// DAL-D: Minor consequence (< 1e-3 / flight hr).
    DalD,
}

impl Do254DalLevel {
    /// Maximum allowable unmitigated catastrophic failure rate per flight hour.
    pub fn max_allowable_failure_rate_per_hour(&self) -> f64 {
        match self {
            Self::DalA => 1.0e-9,
            Self::DalB => 1.0e-7,
            Self::DalC => 1.0e-5,
            Self::DalD => 1.0e-3,
        }
    }

    /// Descriptive name of the assurance level.
    pub fn description(&self) -> &'static str {
        match self {
            Self::DalA => "DAL-A (Catastrophic, < 1e-9 / hr)",
            Self::DalB => "DAL-B (Hazardous, < 1e-7 / hr)",
            Self::DalC => "DAL-C (Major, < 1e-5 / hr)",
            Self::DalD => "DAL-D (Minor, < 1e-3 / hr)",
        }
    }
}

/// Silicon chip device technology node parameters for SEU sensitivity.
#[derive(Debug, Clone)]
pub struct SiliconDeviceParams {
    /// Critical charge Qcrit in femtocoulombs (fC) required to flip a bit cell.
    pub q_crit_fc: f64,
    /// Total number of sensitive memory bits on the chiplet (e.g. 64 Mbit = 67,108,864 bits).
    pub total_bits: u64,
    /// Saturation SEU cross-section per bit in cm^2/bit.
    pub sigma_sat_cm2_per_bit: f64,
    /// Weibull threshold energy E0 in MeV.
    pub weibull_e0_mev: f64,
    /// Weibull width parameter W in MeV.
    pub weibull_w_mev: f64,
    /// Weibull shape parameter s.
    pub weibull_s: f64,
}

impl Default for SiliconDeviceParams {
    fn default() -> Self {
        // Modern 7nm / 5nm FinFET / GAA SRAM cell
        Self {
            q_crit_fc: 0.45,
            total_bits: 64 * 1024 * 1024, // 64 Mbits
            sigma_sat_cm2_per_bit: 1.25e-14, // cm^2 / bit
            weibull_e0_mev: 4.5,
            weibull_w_mev: 28.0,
            weibull_s: 1.45,
        }
    }
}

impl SiliconDeviceParams {
    /// Calculate SEU cross-section sigma(E) in cm^2/bit for a given neutron energy.
    pub fn seu_cross_section_cm2(&self, energy_mev: f64) -> f64 {
        if energy_mev <= self.weibull_e0_mev {
            return 0.0;
        }

        let arg = (energy_mev - self.weibull_e0_mev) / self.weibull_w_mev;
        let weibull_factor = 1.0 - (-arg.powf(self.weibull_s)).exp();
        self.sigma_sat_cm2_per_bit * weibull_factor.clamp(0.0, 1.0)
    }
}

/// Silicon Spallation & Avionics Soft Error Rate (SER) Co-Simulator.
#[derive(Debug, Clone)]
pub struct SiliconSpallationEngine {
    /// Atmospheric neutron environment model.
    pub env: AtmosphericNeutronModel,
    /// Target silicon device node parameters.
    pub device: SiliconDeviceParams,
    /// Active fault mitigation architecture.
    pub mitigation: MitigationArchitecture,
    /// Target DO-254 certification level.
    pub target_dal: Do254DalLevel,
}

impl Default for SiliconSpallationEngine {
    fn default() -> Self {
        Self {
            env: AtmosphericNeutronModel::default(),
            device: SiliconDeviceParams::default(),
            mitigation: MitigationArchitecture::TripleModularRedundancy,
            target_dal: Do254DalLevel::DalA,
        }
    }
}

impl SiliconSpallationEngine {
    /// Create a new silicon spallation engine.
    pub fn new(
        env: AtmosphericNeutronModel,
        device: SiliconDeviceParams,
        mitigation: MitigationArchitecture,
        target_dal: Do254DalLevel,
    ) -> Self {
        Self {
            env,
            device,
            mitigation,
            target_dal,
        }
    }

    /// Calculate raw unmitigated Soft Error Rate per bit in FIT/bit.
    ///
    /// 1 FIT = 1 failure per 10^9 device operating hours.
    /// FIT/bit = Integral[ sigma(E) * dPhi/dE * 1e9 * 3600 dE ]
    pub fn raw_ser_fit_per_bit(&self) -> f64 {
        // Integrate using log-spaced trapezoidal integration over 1 MeV to 5000 MeV
        let n_steps = 100;
        let min_log = 0.0;
        let max_log = 3.7; // ~5000 MeV
        let step = (max_log - min_log) / (n_steps - 1) as f64;

        let mut integral = 0.0;
        for i in 0..(n_steps - 1) {
            let e1 = 10.0_f64.powf(min_log + i as f64 * step);
            let e2 = 10.0_f64.powf(min_log + (i + 1) as f64 * step);
            let de = e2 - e1;

            let s1 = self.device.seu_cross_section_cm2(e1);
            let f1 = self.env.differential_flux(e1);

            let s2 = self.device.seu_cross_section_cm2(e2);
            let f2 = self.env.differential_flux(e2);

            let trap = 0.5 * (s1 * f1 + s2 * f2) * de;
            integral += trap;
        }

        // Convert events/(s * bit) to FIT/bit: multiply by 3600 (s/hr) * 1e9 (FIT)
        integral * 3600.0 * 1.0e9
    }

    /// Calculate total raw unmitigated chip Soft Error Rate in FIT.
    pub fn raw_chip_ser_fit(&self) -> f64 {
        self.raw_ser_fit_per_bit() * self.device.total_bits as f64
    }

    /// Calculate raw unmitigated failures per flight hour.
    pub fn raw_failure_rate_per_hour(&self) -> f64 {
        self.raw_chip_ser_fit() * 1.0e-9
    }

    /// Calculate effective uncorrected / undetected failure rate per flight hour under the active mitigation.
    pub fn mitigated_failure_rate_per_hour(&self) -> f64 {
        let raw_rate = self.raw_failure_rate_per_hour();
        let slices = (self.device.total_bits / 32).max(1) as f64;
        let rate_per_slice = raw_rate / slices;

        match self.mitigation {
            MitigationArchitecture::Simplex => raw_rate,
            MitigationArchitecture::EccSecDed => {
                // SEC-DED corrects all single bit upsets in a 32-bit or 64-bit word.
                // Uncorrectable double bit upset (MBU) probability is typically 1.5% - 3.0% of single bit upsets.
                raw_rate * 0.02
            }
            MitigationArchitecture::DualModularRedundancy => {
                // DMR with fail-safe comparator catches 99.9% of faults (fails silent).
                // Undetected common-mode fault fraction ~ 0.001
                raw_rate * 0.001
            }
            MitigationArchitecture::TripleModularRedundancy => {
                // TMR with majority voter masks any single core fault in a word/slice.
                // Uncorrectable error requires 2 independent upsets in the same slice within voter window (~10 ms).
                let voter_window_hr = 0.01 / 3600.0;
                slices * (3.0 * rate_per_slice.powi(2) * voter_window_hr)
            }
            MitigationArchitecture::LockstepWithScrubbing { scrub_interval_s } => {
                // Lockstep processor comparing states every cycle, backed by memory scrubbing
                let scrub_hr = (scrub_interval_s.max(0.001)) / 3600.0;
                slices * (2.0 * rate_per_slice.powi(2) * scrub_hr)
            }
        }
    }

    /// Calculate effective mitigated chip Soft Error Rate in FIT.
    pub fn mitigated_chip_ser_fit(&self) -> f64 {
        self.mitigated_failure_rate_per_hour() * 1.0e9
    }

    /// Check whether the current architecture satisfies the target DO-254 DAL airworthiness level.
    pub fn is_dal_compliant(&self) -> bool {
        let max_rate = self.target_dal.max_allowable_failure_rate_per_hour();
        self.mitigated_failure_rate_per_hour() <= max_rate
    }

    /// Calculate DO-254 compliance safety margin in decibels (dB).
    ///
    /// Margin (dB) = 10 * log10(max_allowable_rate / actual_rate)
    /// Positive margin indicates compliant safety margin; negative indicates non-compliance.
    pub fn dal_safety_margin_db(&self) -> f64 {
        let max_rate = self.target_dal.max_allowable_failure_rate_per_hour();
        let actual_rate = self.mitigated_failure_rate_per_hour().max(1.0e-25);
        10.0 * (max_rate / actual_rate).log10()
    }
}
