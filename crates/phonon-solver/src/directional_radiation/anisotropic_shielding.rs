#![deny(unsafe_code)]

//! 3D Spacecraft Anisotropic Shielding & Ray-Tracing Attenuation Engine.
//!
//! Models anisotropic vehicle geometries (hull, propellant tanks, spot shielding),
//! multi-material layers (Aluminum, Polyethylene, Tungsten), and directional line-of-sight
//! attenuation over 4*pi steradians to compute localized mission Total Ionizing Dose (TID).

use std::f64::consts::PI;

/// Shielding material type and physical density properties.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShieldingMaterial {
    /// Aerospace Aluminum 6061-T6 (Z=13, rho=2.70 g/cm^3).
    Aluminum,
    /// High-Density Polyethylene / HDPE (Z_eff ~ 5.3, rho=0.95 g/cm^3, hydrogen-rich).
    Polyethylene,
    /// High-Z Tungsten spot shielding (Z=74, rho=19.30 g/cm^3).
    Tungsten,
    /// Graded-Z composite: Ta/Sn/Cu/Al multi-layer preventing bremsstrahlung buildup.
    GradedZComposite,
}

impl ShieldingMaterial {
    /// Mass density in g/cm^3.
    pub fn density_g_cm3(&self) -> f64 {
        match self {
            Self::Aluminum => 2.70,
            Self::Polyethylene => 0.95,
            Self::Tungsten => 19.30,
            Self::GradedZComposite => 8.45,
        }
    }

    /// Effective mass stopping attenuation coefficient mu / rho in cm^2/g for heavy ions.
    pub fn mass_attenuation_coeff_cm2_g(&self) -> f64 {
        match self {
            Self::Aluminum => 0.085,
            Self::Polyethylene => 0.125, // Enhanced stopping per unit mass due to hydrogen
            Self::Tungsten => 0.220,     // High-Z dense electron stopping
            Self::GradedZComposite => 0.165,
        }
    }

    /// Display name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Aluminum => "Aluminum (Al 6061-T6)",
            Self::Polyethylene => "HDPE (Polyethylene)",
            Self::Tungsten => "Tungsten (W Spot Shield)",
            Self::GradedZComposite => "Graded-Z (Ta/Sn/Cu/Al)",
        }
    }
}

/// Spacecraft anisotropic shielding component in 3D coordinates.
#[derive(Debug, Clone)]
pub struct ShieldingComponent {
    pub name: String,
    pub material: ShieldingMaterial,
    pub thickness_mm: f64,
    /// Directional coverage angle window: [theta_min, theta_max] in radians.
    pub theta_window: (f64, f64),
    /// Azimuthal coverage angle window: [phi_min, phi_max] in radians.
    pub phi_window: (f64, f64),
}

/// 3D Spacecraft Vehicle Shielding Model.
#[derive(Debug, Clone)]
pub struct SpacecraftShieldingModel {
    /// Baseline spacecraft pressurized hull thickness in mm Aluminum equivalent.
    pub baseline_hull_thickness_mm: f64,
    /// Primary shielding material.
    pub primary_material: ShieldingMaterial,
    /// Localized components (e.g. fuel tanks, avionics bay bulkheads, spot shields).
    pub components: Vec<ShieldingComponent>,
    /// Unshielded annual baseline cosmic ray dose rate in krad(Si)/year (e.g. 8.5 krad/yr in GEO).
    pub unshielded_annual_dose_krad: f64,
    /// Mission duration in years.
    pub mission_duration_years: f64,
}

impl Default for SpacecraftShieldingModel {
    fn default() -> Self {
        let tank = ShieldingComponent {
            name: "Propellant Tank Shadow".to_string(),
            material: ShieldingMaterial::Polyethylene,
            thickness_mm: 45.0,
            theta_window: (PI * 0.4, PI * 0.8),
            phi_window: (PI * 0.7, PI * 1.3),
        };

        let spot = ShieldingComponent {
            name: "Tungsten Die Spot Lid".to_string(),
            material: ShieldingMaterial::Tungsten,
            thickness_mm: 1.5,
            theta_window: (0.0, PI * 0.3),
            phi_window: (0.0, 2.0 * PI),
        };

        Self {
            baseline_hull_thickness_mm: 3.5,
            primary_material: ShieldingMaterial::Aluminum,
            components: vec![tank, spot],
            unshielded_annual_dose_krad: 8.5,
            mission_duration_years: 5.0,
        }
    }
}

impl SpacecraftShieldingModel {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate effective line-of-sight areal mass thickness t_eff in g/cm^2 along ray direction (theta, phi).
    pub fn effective_areal_mass_g_cm2(&self, theta_rad: f64, phi_rad: f64) -> f64 {
        let baseline_cm = self.baseline_hull_thickness_mm * 0.1;
        let obliquity = (1.0 / theta_rad.cos().abs().max(0.15)).min(5.0);
        let mut total_areal_mass = baseline_cm * self.primary_material.density_g_cm3() * obliquity;

        for comp in &self.components {
            let in_theta = theta_rad >= comp.theta_window.0 && theta_rad <= comp.theta_window.1;
            let in_phi = phi_rad >= comp.phi_window.0 && phi_rad <= comp.phi_window.1;

            if in_theta && in_phi {
                let thickness_cm = comp.thickness_mm * 0.1;
                total_areal_mass += thickness_cm * comp.material.density_g_cm3() * obliquity;
            }
        }

        total_areal_mass.max(0.1)
    }

    /// Calculate directional flux attenuation factor A(theta, phi) in [0.0, 1.0].
    pub fn directional_attenuation_factor(&self, theta_rad: f64, phi_rad: f64) -> f64 {
        let areal_mass = self.effective_areal_mass_g_cm2(theta_rad, phi_rad);
        let mu_rho = self.primary_material.mass_attenuation_coeff_cm2_g();

        // Exponential heavy ion attenuation with secondary build-up ceiling
        let atten = (-mu_rho * areal_mass).exp();
        atten.clamp(0.001, 1.0)
    }

    /// Calculate localized Total Ionizing Dose (TID) in krad(Si) for the mission.
    pub fn localized_mission_tid_krad(&self, incident_theta_rad: f64, incident_phi_rad: f64) -> f64 {
        let total_unshielded = self.unshielded_annual_dose_krad * self.mission_duration_years;
        let atten = self.directional_attenuation_factor(incident_theta_rad, incident_phi_rad);

        total_unshielded * atten
    }

    /// Compute polar 2D attenuation map across 360-degree azimuthal sweep at current theta.
    ///
    /// Returns tuples of (phi in degrees, effective areal mass in g/cm^2, attenuation percent).
    pub fn compute_polar_azimuth_sweep(&self, theta_rad: f64, samples: usize) -> Vec<(f64, f64, f64)> {
        let n = samples.max(16);
        let mut sweep = Vec::with_capacity(n);

        for i in 0..n {
            let phi_rad = (i as f64) * 2.0 * PI / (n as f64);
            let phi_deg = phi_rad * 180.0 / PI;
            let areal_mass = self.effective_areal_mass_g_cm2(theta_rad, phi_rad);
            let atten = self.directional_attenuation_factor(theta_rad, phi_rad);
            let atten_percent = (1.0 - atten) * 100.0;

            sweep.push((phi_deg, areal_mass, atten_percent));
        }

        sweep
    }

    /// Compute TID vs Shielding Thickness trade-off curve (Al vs Graded-Z).
    ///
    /// Returns pairs of (thickness in mm, TID in krad(Si)).
    pub fn compute_tid_thickness_tradeoff(&self, max_thickness_mm: f64, steps: usize) -> (Vec<[f64; 2]>, Vec<[f64; 2]>) {
        let n = steps.max(10);
        let mut al_curve = Vec::with_capacity(n);
        let mut gz_curve = Vec::with_capacity(n);
        let total_unshielded = self.unshielded_annual_dose_krad * self.mission_duration_years;

        for i in 0..n {
            let t_mm = (i as f64) * max_thickness_mm / ((n - 1) as f64);
            let t_cm = t_mm * 0.1;

            // Aluminum
            let areal_al = t_cm * ShieldingMaterial::Aluminum.density_g_cm3();
            let atten_al = (-ShieldingMaterial::Aluminum.mass_attenuation_coeff_cm2_g() * areal_al).exp();
            let tid_al = total_unshielded * atten_al;
            al_curve.push([t_mm, tid_al]);

            // Graded-Z
            let areal_gz = t_cm * ShieldingMaterial::GradedZComposite.density_g_cm3();
            let atten_gz = (-ShieldingMaterial::GradedZComposite.mass_attenuation_coeff_cm2_g() * areal_gz).exp();
            let tid_gz = total_unshielded * atten_gz;
            gz_curve.push([t_mm, tid_gz]);
        }

        (al_curve, gz_curve)
    }
}
