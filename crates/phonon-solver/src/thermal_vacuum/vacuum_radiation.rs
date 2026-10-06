#![deny(unsafe_code)]

//! Extreme Aerospace Thermal-Vacuum Stefan-Boltzmann Radiation Engine.
//!
//! Models zero-convection vacuum boundary conditions (h_conv = 0), Stefan-Boltzmann radiative
//! cooling q_rad = eps * sigma * (T^4 - T_sink^4) to deep-space background (2.7K), multi-material
//! surface emissivity (Gold, White Paint, Black Anodize, MLI), and orbital view factors.

/// Stefan-Boltzmann constant sigma in W / (m^2 * K^4).
pub const STEFAN_BOLTZMANN: f64 = 5.670374419e-8;

/// Deep-space cosmic microwave background (CMB) temperature in Kelvin.
pub const DEEP_SPACE_SINK_KELVIN: f64 = 2.725;

/// Aerospace thermal control surface finish / coating kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceCoatingKind {
    /// Vapor-deposited Gold / Polished Aluminum (low infrared emissivity ~0.04).
    PolishedGold,
    /// White thermal control paint (e.g. AZ-93 / Z-93, eps ~ 0.90, solar absorptance alpha ~ 0.18).
    WhiteThermalPaint,
    /// Black anodized aluminum / Carbon nanotube blackbody (eps ~ 0.96).
    BlackAnodize,
    /// Multi-Layer Insulation (MLI) blanket outer layer (eps_eff ~ 0.02).
    MultiLayerInsulation,
    /// Bare silicon die surface (eps ~ 0.65).
    BareSilicon,
}

impl SurfaceCoatingKind {
    /// Infrared hemispherical emissivity epsilon in [0.0, 1.0].
    pub fn emissivity(&self) -> f64 {
        match self {
            Self::PolishedGold => 0.04,
            Self::WhiteThermalPaint => 0.90,
            Self::BlackAnodize => 0.96,
            Self::MultiLayerInsulation => 0.02,
            Self::BareSilicon => 0.65,
        }
    }

    /// Solar spectrum absorptance alpha_s in [0.0, 1.0].
    pub fn solar_absorptance(&self) -> f64 {
        match self {
            Self::PolishedGold => 0.25,
            Self::WhiteThermalPaint => 0.18,
            Self::BlackAnodize => 0.95,
            Self::MultiLayerInsulation => 0.12,
            Self::BareSilicon => 0.70,
        }
    }

    /// Ratio of solar absorptance to infrared emissivity alpha_s / eps.
    /// Determines equilibrium temperature in direct sunlight.
    pub fn alpha_over_eps_ratio(&self) -> f64 {
        self.solar_absorptance() / self.emissivity()
    }
}

/// Thermal-vacuum radiator boundary condition model.
#[derive(Debug, Clone)]
pub struct VacuumRadiationModel {
    /// Radiating surface coating.
    pub coating: SurfaceCoatingKind,
    /// Active surface area in square meters (m^2).
    pub area_m2: f64,
    /// Deep-space / planetary radiation sink temperature in Kelvin.
    pub sink_temp_k: f64,
    /// View factor to cold sky F_sky in [0.0, 1.0].
    pub view_factor_sky: f64,
    /// Incident direct solar irradiance in W/m^2 (0 in eclipse, ~1361 W/m^2 in sunlight at 1 AU).
    pub solar_flux_w_m2: f64,
    /// Planetary Earth albedo infrared flux in W/m^2.
    pub albedo_flux_w_m2: f64,
}

impl Default for VacuumRadiationModel {
    fn default() -> Self {
        Self {
            coating: SurfaceCoatingKind::WhiteThermalPaint,
            area_m2: 0.05, // 500 cm^2 radiator panel
            sink_temp_k: DEEP_SPACE_SINK_KELVIN,
            view_factor_sky: 0.95,
            solar_flux_w_m2: 1361.0,
            albedo_flux_w_m2: 0.0,
        }
    }
}

impl VacuumRadiationModel {
    /// Create a new thermal-vacuum model with specified parameters.
    pub fn new(coating: SurfaceCoatingKind, area_m2: f64, sink_temp_k: f64) -> Self {
        Self {
            coating,
            area_m2: area_m2.max(1e-6),
            sink_temp_k: sink_temp_k.max(0.1),
            ..Default::default()
        }
    }

    /// Calculate net radiative heat rejection flux q_net in W / m^2 at surface temperature T_k.
    ///
    /// q_net = eps * sigma * F_sky * (T^4 - T_sink^4) - (alpha_s * q_solar + alpha_s * q_albedo)
    pub fn net_heat_rejection_flux_w_m2(&self, surface_temp_k: f64) -> f64 {
        let t = surface_temp_k.max(0.1);
        let eps = self.coating.emissivity();
        let alpha = self.coating.solar_absorptance();

        let emitted = eps * STEFAN_BOLTZMANN * self.view_factor_sky * (t.powi(4) - self.sink_temp_k.powi(4));
        let absorbed_solar = alpha * (self.solar_flux_w_m2 + self.albedo_flux_w_m2);

        emitted - absorbed_solar
    }

    /// Total rejected thermal power in Watts.
    pub fn total_rejected_power_w(&self, surface_temp_k: f64) -> f64 {
        self.net_heat_rejection_flux_w_m2(surface_temp_k) * self.area_m2
    }

    /// Calculate steady-state radiative equilibrium temperature in Kelvin under zero internal dissipation.
    ///
    /// T_eq = [ (alpha_s * (q_solar + q_albedo) / (eps * sigma * F_sky)) + T_sink^4 ]^(1/4)
    pub fn equilibrium_temperature_k(&self) -> f64 {
        let eps = self.coating.emissivity();
        let alpha = self.coating.solar_absorptance();
        let denom = eps * STEFAN_BOLTZMANN * self.view_factor_sky;

        let incident = alpha * (self.solar_flux_w_m2 + self.albedo_flux_w_m2);
        let t4 = (incident / denom) + self.sink_temp_k.powi(4);

        t4.max(0.0).powf(0.25)
    }

    /// Calculate equilibrium temperature in degrees Celsius.
    pub fn equilibrium_temperature_c(&self) -> f64 {
        self.equilibrium_temperature_k() - 273.15
    }

    /// Simulate non-linear transient cooldown trajectory in vacuum from T_start_k to equilibrium.
    /// Returns timestamps in seconds and temperatures in Celsius.
    pub fn simulate_vacuum_cooldown(
        &self,
        start_temp_k: f64,
        heat_capacity_j_per_k: f64,
        internal_heat_w: f64,
        duration_s: f64,
        points: usize,
    ) -> Vec<(f64, f64)> {
        let pts = points.clamp(50, 1000);
        let dt = duration_s / (pts - 1) as f64;
        let c_th = heat_capacity_j_per_k.max(1.0);

        let mut samples = Vec::with_capacity(pts);
        let mut temp_k = start_temp_k;

        for i in 0..pts {
            let t_s = i as f64 * dt;
            samples.push((t_s, temp_k - 273.15));

            // Explicit Runge-Kutta 2nd order step
            let q_net1 = self.total_rejected_power_w(temp_k) - internal_heat_w;
            let d_temp1 = -q_net1 / c_th;

            let temp_mid = (temp_k + 0.5 * dt * d_temp1).max(0.1);
            let q_net2 = self.total_rejected_power_w(temp_mid) - internal_heat_w;
            let d_temp2 = -q_net2 / c_th;

            temp_k = (temp_k + dt * d_temp2).max(DEEP_SPACE_SINK_KELVIN);
        }

        samples
    }
}
