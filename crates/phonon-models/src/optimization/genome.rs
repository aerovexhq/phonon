//! Transistor Genome definition and parameter search space for automated inverse device design.
//!
//! Encapsulates discrete structural architecture choices, chemical material selections,
//! and continuous geometric / doping parameters for sub-10nm transistor optimization.

/// Transistor structural 3D architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArchitectureType {
    /// Gate-All-Around horizontal stacked nanosheets (Samsung MBCFET / TSMC N2 / Intel 20A).
    GaaNanosheet,
    /// Complementary FET with monolithic vertical N-over-P or P-over-N stacking.
    Cfet,
    /// Negative Capacitance FET with integrated ferroelectric HZO gate sub-stack.
    Ncfet,
    /// Tri-gate / FinFET baseline architecture.
    FinFet,
}

/// Chemical species for semiconductor channel conduction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelMaterial {
    /// Crystalline Silicon: $E_g = 1.12\text{ eV}, \mu_n = 0.14\text{ m}^2/\text{Vs}$.
    Silicon,
    /// Strained Germanium: $E_g = 0.66\text{ eV}, \mu_p = 0.19\text{ m}^2/\text{Vs}$.
    StrainedGermanium,
    /// Indium Gallium Arsenide ($\text{In}_{0.53}\text{Ga}_{0.47}\text{As}$): $E_g = 0.74\text{ eV}, \mu_n \approx 1.2\text{ m}^2/\text{Vs}$.
    InGaAs,
    /// 2D Monolayer $\text{MoS}_2$: $E_g = 1.80\text{ eV}$, monolayer thickness $0.65\text{ nm}$.
    MoS2,
    /// 2D Monolayer $\text{WS}_2$: $E_g = 2.00\text{ eV}$, monolayer thickness $0.65\text{ nm}$.
    WS2,
}

impl ChannelMaterial {
    /// Energy bandgap in electron-volts ($eV$) at 300 K.
    #[inline]
    pub fn bandgap_ev(&self) -> f64 {
        match self {
            Self::Silicon => 1.12,
            Self::StrainedGermanium => 0.66,
            Self::InGaAs => 0.74,
            Self::MoS2 => 1.80,
            Self::WS2 => 2.00,
        }
    }

    /// Low-field carrier mobility in $\text{m}^2 / (\text{V}\cdot\text{s})$.
    #[inline]
    pub fn carrier_mobility_m2_per_vs(&self) -> f64 {
        match self {
            Self::Silicon => 0.14,           // 1400 cm^2/Vs
            Self::StrainedGermanium => 0.19, // 1900 cm^2/Vs (holes)
            Self::InGaAs => 1.10,            // 11,000 cm^2/Vs
            Self::MoS2 => 0.04,              // 400 cm^2/Vs
            Self::WS2 => 0.06,               // 600 cm^2/Vs
        }
    }

    /// Relative permittivity $\epsilon_r$.
    #[inline]
    pub fn relative_permittivity(&self) -> f64 {
        match self {
            Self::Silicon => 11.7,
            Self::StrainedGermanium => 16.0,
            Self::InGaAs => 13.9,
            Self::MoS2 => 4.0,
            Self::WS2 => 4.5,
        }
    }

    /// Effective carrier mass ratio $m^* / m_0$.
    #[inline]
    pub fn effective_mass_ratio(&self) -> f64 {
        match self {
            Self::Silicon => 0.26,
            Self::StrainedGermanium => 0.15,
            Self::InGaAs => 0.041,
            Self::MoS2 => 0.45,
            Self::WS2 => 0.35,
        }
    }

    /// Returns true if this is a 2D atomically thin monolayer material.
    #[inline]
    pub fn is_2d_material(&self) -> bool {
        matches!(self, Self::MoS2 | Self::WS2)
    }
}

/// Gate dielectric material species for inverse optimization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptGateDielectric {
    /// Silicon Dioxide $\text{SiO}_2$: $\epsilon_r = 3.9, E_g = 9.0\text{ eV}$.
    SiO2,
    /// Hafnium Oxide $\text{HfO}_2$: $\epsilon_r = 25.0, E_g = 5.7\text{ eV}$.
    HfO2,
    /// Aluminum Oxide $\text{Al}_2\text{O}_3$: $\epsilon_r = 9.0, E_g = 8.8\text{ eV}$.
    Al2O3,
    /// Zirconium Oxide $\text{ZrO}_2$: $\epsilon_r = 22.0, E_g = 5.8\text{ eV}$.
    ZrO2,
    /// Ferroelectric $\text{Hf}_{0.5}\text{Zr}_{0.5}\text{O}_2$ (HZO) negative capacitance layer.
    HzoFerroelectric,
}

impl OptGateDielectric {
    #[inline]
    pub fn relative_permittivity(&self) -> f64 {
        match self {
            Self::SiO2 => 3.9,
            Self::HfO2 => 25.0,
            Self::Al2O3 => 9.0,
            Self::ZrO2 => 22.0,
            Self::HzoFerroelectric => -30.0, // Effective negative capacitance in stabilized regime
        }
    }
}

/// Contact metal / silicide species for source and drain interfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OptContactMetal {
    NiSi,
    CoSi2,
    PtSi,
    TiN,
    Ruthenium,
}

impl OptContactMetal {
    /// Specific contact resistivity $\rho_c$ in $\Omega\cdot\text{cm}^2$.
    #[inline]
    pub fn specific_resistivity_ohm_cm2(&self) -> f64 {
        match self {
            Self::NiSi => 1.5e-9,
            Self::CoSi2 => 2.0e-9,
            Self::PtSi => 1.0e-9,
            Self::TiN => 1.8e-9,
            Self::Ruthenium => 8.0e-10, // Ultra-low contact resistance
        }
    }

    /// Metal workfunction in $eV$.
    #[inline]
    pub fn workfunction_ev(&self) -> f64 {
        match self {
            Self::NiSi => 4.65,
            Self::CoSi2 => 4.70,
            Self::PtSi => 4.90,
            Self::TiN => 4.60,
            Self::Ruthenium => 4.71,
        }
    }
}

/// Parameter bounds for continuous optimization variables.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneBounds {
    pub gate_length_nm_range: (f64, f64),
    pub channel_thickness_nm_range: (f64, f64),
    pub channel_width_nm_range: (f64, f64),
    pub eot_nm_range: (f64, f64),
    pub sd_doping_range: (f64, f64),
    pub body_doping_range: (f64, f64),
    pub workfunction_ev_range: (f64, f64),
}

impl Default for GeneBounds {
    fn default() -> Self {
        Self {
            gate_length_nm_range: (5.0, 20.0),      // 5 nm to 20 nm
            channel_thickness_nm_range: (1.5, 6.0), // 1.5 nm to 6.0 nm
            channel_width_nm_range: (10.0, 60.0),   // 10 nm to 60 nm
            eot_nm_range: (0.5, 1.8),               // 0.5 nm to 1.8 nm EOT
            sd_doping_range: (5.0e19, 5.0e20),      // 5e19 to 5e20 cm^-3
            body_doping_range: (1.0e15, 1.0e18),    // 1e15 to 1e18 cm^-3
            workfunction_ev_range: (4.1, 5.1),      // 4.1 eV to 5.1 eV
        }
    }
}

/// Complete parameter specification (genome) of a candidate transistor.
#[derive(Debug, Clone, PartialEq)]
pub struct TransistorGenome {
    /// 3D architecture class.
    pub architecture: ArchitectureType,
    /// Channel material chemistry.
    pub channel_material: ChannelMaterial,
    /// Gate dielectric chemistry.
    pub gate_dielectric: OptGateDielectric,
    /// Contact metal species.
    pub contact_species: OptContactMetal,
    /// Physical gate length $L_g$ in nanometers ($nm$).
    pub gate_length_nm: f64,
    /// Physical channel thickness $T_{ch}$ in nanometers ($nm$).
    pub channel_thickness_nm: f64,
    /// Physical channel width $W_{ch}$ in nanometers ($nm$).
    pub channel_width_nm: f64,
    /// Equivalent Oxide Thickness $\text{EOT}$ in nanometers ($nm$).
    pub eot_nm: f64,
    /// Source/Drain doping concentration in $\text{cm}^{-3}$.
    pub sd_doping_cm3: f64,
    /// Channel body doping concentration in $\text{cm}^{-3}$.
    pub body_doping_cm3: f64,
    /// Gate workfunction $\Phi_m$ in $eV$.
    pub workfunction_ev: f64,
    /// Number of stacked nanosheet channels ($N_{sheets} \in [1, 4]$).
    pub num_sheets: usize,
    /// Supply voltage $V_{dd}$ in Volts ($V$).
    pub v_dd_volts: f64,
}

impl TransistorGenome {
    /// Canonical 2nm-node Gate-All-Around (GAA) nanosheet candidate preset.
    pub fn n2_gaa_nanosheet_preset() -> Self {
        Self {
            architecture: ArchitectureType::GaaNanosheet,
            channel_material: ChannelMaterial::Silicon,
            gate_dielectric: OptGateDielectric::HfO2,
            contact_species: OptContactMetal::Ruthenium,
            gate_length_nm: 12.0,
            channel_thickness_nm: 5.0,
            channel_width_nm: 30.0,
            eot_nm: 0.85,
            sd_doping_cm3: 2.0e20,
            body_doping_cm3: 1.0e16,
            workfunction_ev: 4.65,
            num_sheets: 3,
            v_dd_volts: 0.70,
        }
    }

    /// Monolithic 3D Complementary FET (CFET) candidate preset.
    pub fn cfet_preset() -> Self {
        Self {
            architecture: ArchitectureType::Cfet,
            channel_material: ChannelMaterial::StrainedGermanium,
            gate_dielectric: OptGateDielectric::HfO2,
            contact_species: OptContactMetal::NiSi,
            gate_length_nm: 10.0,
            channel_thickness_nm: 4.0,
            channel_width_nm: 25.0,
            eot_nm: 0.75,
            sd_doping_cm3: 3.0e20,
            body_doping_cm3: 1.0e16,
            workfunction_ev: 4.70,
            num_sheets: 2,
            v_dd_volts: 0.65,
        }
    }

    /// Ultra-scaled 2D TMD ($\text{MoS}_2$) sub-5nm channel candidate preset.
    pub fn tmd_ultra_scaled_preset() -> Self {
        Self {
            architecture: ArchitectureType::GaaNanosheet,
            channel_material: ChannelMaterial::MoS2,
            gate_dielectric: OptGateDielectric::Al2O3,
            contact_species: OptContactMetal::Ruthenium,
            gate_length_nm: 6.0,
            channel_thickness_nm: 0.65, // Monolayer thickness
            channel_width_nm: 20.0,
            eot_nm: 0.60,
            sd_doping_cm3: 1.0e20,
            body_doping_cm3: 1.0e15,
            workfunction_ev: 4.60,
            num_sheets: 2,
            v_dd_volts: 0.50,
        }
    }

    /// Clamps all continuous genes to the specified bounds.
    pub fn clamp_to_bounds(&mut self, bounds: &GeneBounds) {
        self.gate_length_nm = self
            .gate_length_nm
            .clamp(bounds.gate_length_nm_range.0, bounds.gate_length_nm_range.1);

        // 2D monolayers are physically locked to their monolayer lattice thickness
        if self.channel_material.is_2d_material() {
            self.channel_thickness_nm = 0.65;
        } else {
            self.channel_thickness_nm = self.channel_thickness_nm.clamp(
                bounds.channel_thickness_nm_range.0,
                bounds.channel_thickness_nm_range.1,
            );
        }

        self.channel_width_nm = self.channel_width_nm.clamp(
            bounds.channel_width_nm_range.0,
            bounds.channel_width_nm_range.1,
        );
        self.eot_nm = self
            .eot_nm
            .clamp(bounds.eot_nm_range.0, bounds.eot_nm_range.1);
        self.sd_doping_cm3 = self
            .sd_doping_cm3
            .clamp(bounds.sd_doping_range.0, bounds.sd_doping_range.1);
        self.body_doping_cm3 = self
            .body_doping_cm3
            .clamp(bounds.body_doping_range.0, bounds.body_doping_range.1);
        self.workfunction_ev = self.workfunction_ev.clamp(
            bounds.workfunction_ev_range.0,
            bounds.workfunction_ev_range.1,
        );
        self.num_sheets = self.num_sheets.clamp(1, 4);
        self.v_dd_volts = self.v_dd_volts.clamp(0.4, 1.2);
    }
}
