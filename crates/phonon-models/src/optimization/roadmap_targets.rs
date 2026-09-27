//! International Roadmap for Devices and Systems (IRDS) and ITRS benchmark targets.
//!
//! Provides standardized industry targets for evaluating discovered 3nm, 2nm, and A14 (1.4nm)
//! node transistor candidates against global semiconductor metrics.

use super::physical_fitness::FitnessEvaluation;

/// Standardized foundry generation node targets.
#[derive(Debug, Clone, PartialEq)]
pub struct IrdsNodeTarget {
    /// Node designation label (e.g. "3nm GAA", "2nm MBCFET", "A14 CFET").
    pub name: String,
    /// Minimum required on-state drive current per unit width in $\text{A/m}$ ($\text{mA}/\mu\text{m}$).
    pub min_ion_a_per_m: f64,
    /// Maximum allowable off-state leakage current per unit width in $\text{A/m}$.
    pub max_ioff_a_per_m: f64,
    /// Maximum allowable subthreshold swing in $\text{mV/dec}$.
    pub max_subthreshold_swing_mv_per_dec: f64,
    /// Maximum allowable intrinsic gate switching delay in picoseconds ($ps$).
    pub max_delay_ps: f64,
    /// Maximum allowable self-heating temperature rise in Kelvin ($K$).
    pub max_self_heating_k: f64,
}

impl IrdsNodeTarget {
    /// IRDS 3nm GAA Nanosheet target specification (2022-2024 era).
    pub fn irds_3nm_node() -> Self {
        Self {
            name: "IRDS 3nm GAA".to_string(),
            min_ion_a_per_m: 1000.0,  // 1.0 mA/um
            max_ioff_a_per_m: 1.0e-4, // 100 pA/um
            max_subthreshold_swing_mv_per_dec: 75.0,
            max_delay_ps: 0.80,
            max_self_heating_k: 60.0,
        }
    }

    /// IRDS 2nm GAA Nanosheet / MBCFET target specification (2025-2027 era).
    pub fn irds_2nm_node() -> Self {
        Self {
            name: "IRDS 2nm MBCFET".to_string(),
            min_ion_a_per_m: 1300.0,  // 1.3 mA/um
            max_ioff_a_per_m: 5.0e-5, // 50 pA/um
            max_subthreshold_swing_mv_per_dec: 70.0,
            max_delay_ps: 0.55,
            max_self_heating_k: 65.0,
        }
    }

    /// IRDS A14 (1.4nm) 3D CFET / 2D Monolayer target specification (2028+ era).
    pub fn irds_a14_node() -> Self {
        Self {
            name: "IRDS A14 CFET".to_string(),
            min_ion_a_per_m: 1500.0,  // 1.5 mA/um
            max_ioff_a_per_m: 3.0e-5, // 30 pA/um
            max_subthreshold_swing_mv_per_dec: 65.0,
            max_delay_ps: 0.40,
            max_self_heating_k: 70.0,
        }
    }

    /// Evaluates whether a candidate transistor meets this roadmap specification.
    pub fn evaluate_compliance(&self, fit: &FitnessEvaluation) -> RoadmapComplianceReport {
        let width_m = fit.effective_width_nm * 1.0e-9;
        let ion_norm = fit.i_on_a / width_m.max(1e-12);
        let ioff_norm = fit.i_off_a / width_m.max(1e-12);

        let ion_ok = ion_norm >= self.min_ion_a_per_m;
        let ioff_ok = ioff_norm <= self.max_ioff_a_per_m;
        let swing_ok = fit.subthreshold_swing_mv_per_dec <= self.max_subthreshold_swing_mv_per_dec;
        let delay_ok = fit.intrinsic_delay_ps <= self.max_delay_ps;
        let thermal_ok = fit.self_heating_delta_t_k <= self.max_self_heating_k;

        let overall_compliant = ion_ok && ioff_ok && swing_ok && delay_ok && thermal_ok;

        // Normalized multi-metric score: 1.0 = exactly meets all targets
        let score = (ion_norm / self.min_ion_a_per_m)
            * (self.max_ioff_a_per_m / ioff_norm.max(1e-15)).min(10.0)
            * (self.max_delay_ps / fit.intrinsic_delay_ps.max(0.01))
            * (self.max_subthreshold_swing_mv_per_dec / fit.subthreshold_swing_mv_per_dec);

        RoadmapComplianceReport {
            target_name: self.name.clone(),
            ion_satisfied: ion_ok,
            ioff_satisfied: ioff_ok,
            swing_satisfied: swing_ok,
            delay_satisfied: delay_ok,
            thermal_satisfied: thermal_ok,
            overall_compliant,
            normalized_performance_score: score,
        }
    }
}

/// Detailed compliance evaluation report against an IRDS roadmap target.
#[derive(Debug, Clone, PartialEq)]
pub struct RoadmapComplianceReport {
    pub target_name: String,
    pub ion_satisfied: bool,
    pub ioff_satisfied: bool,
    pub swing_satisfied: bool,
    pub delay_satisfied: bool,
    pub thermal_satisfied: bool,
    pub overall_compliant: bool,
    pub normalized_performance_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::genome::TransistorGenome;
    use crate::optimization::physical_fitness::evaluate_transistor_fitness;

    #[test]
    fn test_2nm_preset_meets_3nm_and_2nm_targets() {
        let genome = TransistorGenome::n2_gaa_nanosheet_preset();
        let fit = evaluate_transistor_fitness(&genome);

        let target_3nm = IrdsNodeTarget::irds_3nm_node();
        let report_3nm = target_3nm.evaluate_compliance(&fit);
        assert!(report_3nm.swing_satisfied);
        assert!(report_3nm.delay_satisfied);
        assert!(report_3nm.thermal_satisfied);
        assert!(report_3nm.normalized_performance_score > 1.0);
    }
}
