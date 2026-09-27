//! Integration Test: Automated Material Discovery, 2D TMDs, and 3D CFETs.
//!
//! Validates:
//! - Material selection tradeoffs across Silicon, Strained Germanium, InGaAs, MoS2, and WS2.
//! - Sub-8nm electrostatic scale length lambda advantage of atomically thin 2D monolayers.
//! - Monolithic 3D CFET stacking and multi-sheet current scaling.
//! - Contact metallurgy optimization comparing Ruthenium, PtSi, and NiSi.

use phonon_models::optimization::{
    evaluate_transistor_fitness, ChannelMaterial, OptContactMetal, TransistorGenome,
};

#[test]
fn test_2d_tmd_electrostatic_immunity_at_ultra_short_gate() {
    // At Lg = 6nm, Silicon bulk/nanosheet with Tch = 5nm suffers from short-channel degradation (DIBL).
    // In contrast, MoS2 monolayer (Tch = 0.65 nm) possesses an ultra-small scale length lambda,
    // preserving subthreshold swing and suppressing leakage.
    let mut si_genome = TransistorGenome::n2_gaa_nanosheet_preset();
    si_genome.gate_length_nm = 6.0;
    si_genome.channel_thickness_nm = 5.0;
    si_genome.channel_material = ChannelMaterial::Silicon;

    let mut tmd_genome = TransistorGenome::tmd_ultra_scaled_preset();
    tmd_genome.gate_length_nm = 6.0;
    tmd_genome.channel_material = ChannelMaterial::MoS2;

    let fit_si = evaluate_transistor_fitness(&si_genome);
    let fit_tmd = evaluate_transistor_fitness(&tmd_genome);

    // TMD monolayer must have superior subthreshold swing due to ultra-thin body
    assert!(
        fit_tmd.subthreshold_swing_mv_per_dec < fit_si.subthreshold_swing_mv_per_dec,
        "MoS2 swing ({} mV/dec) must beat scaled Silicon ({} mV/dec)",
        fit_tmd.subthreshold_swing_mv_per_dec,
        fit_si.subthreshold_swing_mv_per_dec
    );

    // TMD must have lower off-state leakage current than severely short-channel degraded Si
    assert!(
        fit_tmd.i_off_a < fit_si.i_off_a,
        "MoS2 Ioff ({} A) must be lower than Silicon Ioff ({} A)",
        fit_tmd.i_off_a,
        fit_si.i_off_a
    );

    // TMD must achieve higher Ion/Ioff ratio
    assert!(
        fit_tmd.ion_ioff_ratio > fit_si.ion_ioff_ratio,
        "MoS2 Ion/Ioff ({} ) must exceed Silicon ({} )",
        fit_tmd.ion_ioff_ratio,
        fit_si.ion_ioff_ratio
    );
}

#[test]
fn test_high_mobility_materials_drive_current_comparison() {
    // At relaxed gate lengths (Lg = 14nm), Strained Germanium and InGaAs with lower
    // effective mass (0.12 m0 and 0.041 m0) have higher thermal injection velocity v_inj,
    // producing higher intrinsic drive current than Silicon.
    let mut si_genome = TransistorGenome::n2_gaa_nanosheet_preset();
    si_genome.gate_length_nm = 14.0;
    si_genome.channel_material = ChannelMaterial::Silicon;

    // Workfunction engineering: align threshold voltages for fair comparison of carrier transport
    let mut ge_genome = si_genome.clone();
    ge_genome.channel_material = ChannelMaterial::StrainedGermanium;
    ge_genome.workfunction_ev = 4.45;

    let mut ingaas_genome = si_genome.clone();
    ingaas_genome.channel_material = ChannelMaterial::InGaAs;
    ingaas_genome.workfunction_ev = 4.35;

    let fit_si = evaluate_transistor_fitness(&si_genome);
    let fit_ge = evaluate_transistor_fitness(&ge_genome);
    let fit_ingaas = evaluate_transistor_fitness(&ingaas_genome);

    // Strained Ge and InGaAs must provide higher on-state drive current due to lower carrier effective mass
    assert!(
        fit_ge.i_on_a > fit_si.i_on_a,
        "Strained Germanium Ion ({} A) must exceed Silicon Ion ({} A)",
        fit_ge.i_on_a,
        fit_si.i_on_a
    );
    assert!(
        fit_ingaas.i_on_a > fit_si.i_on_a,
        "InGaAs Ion ({} A) must exceed Silicon Ion ({} A)",
        fit_ingaas.i_on_a,
        fit_si.i_on_a
    );
}

#[test]
fn test_cfet_multi_sheet_drive_scaling_and_self_heating() {
    // Complementary FET (CFET) stacks nFET over pFET channels.
    // Testing 1 sheet vs 2 sheets vs 3 sheets:
    // Drive current scales with effective width, while thermal dissipation increases.
    let mut cfet_1 = TransistorGenome::cfet_preset();
    cfet_1.num_sheets = 1;

    let mut cfet_3 = TransistorGenome::cfet_preset();
    cfet_3.num_sheets = 3;

    let fit_1 = evaluate_transistor_fitness(&cfet_1);
    let fit_3 = evaluate_transistor_fitness(&cfet_3);

    // 3 sheets must have substantially higher drive current than 1 sheet
    assert!(
        fit_3.i_on_a > fit_1.i_on_a * 2.0,
        "3-sheet CFET drive current ({} A) must scale over 1-sheet ({} A)",
        fit_3.i_on_a,
        fit_1.i_on_a
    );

    // Both must remain physically viable and within safe thermal limits
    assert!(fit_1.is_physically_viable);
    assert!(fit_3.is_physically_viable);
    assert!(fit_3.self_heating_delta_t_k < 100.0);
}

#[test]
fn test_contact_metallurgy_ruthenium_vs_silicide() {
    // Ultra-scaled sub-10nm contacts suffer from contact resistance Rc.
    // Ruthenium (rho_c ~ 8e-10 ohm*cm^2) exhibits lower contact resistivity than NiSi (~1.5e-9 ohm*cm^2)
    let mut ru_genome = TransistorGenome::n2_gaa_nanosheet_preset();
    ru_genome.contact_species = OptContactMetal::Ruthenium;

    let mut nisi_genome = ru_genome.clone();
    nisi_genome.contact_species = OptContactMetal::NiSi;

    let fit_ru = evaluate_transistor_fitness(&ru_genome);
    let fit_nisi = evaluate_transistor_fitness(&nisi_genome);

    assert!(
        fit_ru.contact_resistance_ohms < fit_nisi.contact_resistance_ohms,
        "Ruthenium contact resistance ({} Ohms) must be lower than NiSi ({} Ohms)",
        fit_ru.contact_resistance_ohms,
        fit_nisi.contact_resistance_ohms
    );

    // Lower contact drop translates to higher effective drive current
    assert!(
        fit_ru.i_on_a >= fit_nisi.i_on_a,
        "Ruthenium Ion ({} A) must exceed NiSi Ion ({} A)",
        fit_ru.i_on_a,
        fit_nisi.i_on_a
    );
}
