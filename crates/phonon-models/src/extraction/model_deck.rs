#![deny(unsafe_code)]

//! SPICE `.MODEL` parameter card deck generation and syntax validation for BSIM4, EKV, and Gummel-Poon BJT.

use super::bjt::BjtTargetParams;
use super::curve_data::ExtractionError;
use super::ekv::EkvTargetParams;
use super::optimizer::{Bsim4TargetParams, FittingResult};

/// Generates a valid SPICE BSIM4 `.MODEL` card deck from extracted target parameters.
pub fn generate_bsim4_model_deck(
    model_name: &str,
    is_nmos: bool,
    params: &Bsim4TargetParams,
    result: &FittingResult,
) -> String {
    let polarity = if is_nmos { "NMOS" } else { "PMOS" };
    let sanitized_name = if model_name.trim().is_empty() {
        "BSIM4_EXTRACTED"
    } else {
        model_name.trim()
    };

    format!(
        "* Phonon Universal BSIM4 Extracted Model Deck\n\
         * Target: {name} ({polarity})\n\
         * Fitting Metric: RMSE = {rmse:.6e} A, R^2 = {r2:.6}\n\
         * Converged: {converged}, Generations: {iters}\n\
         .MODEL {name} {polarity}\n\
         + LEVEL=54 VERSION=4.8.2 TNOM=300.0\n\
         + VTH0={vth0:.6} U0={u0:.6} VSAT={vsat:.1}\n\
         + DVT0={dvt0:.6} ETA0={eta0:.6} RDSW={rdsw:.2}\n\
         .END\n",
        name = sanitized_name,
        polarity = polarity,
        rmse = result.rmse,
        r2 = result.r_squared,
        converged = result.converged,
        iters = result.iterations,
        vth0 = params.vth0,
        u0 = params.u0,
        vsat = params.vsat,
        dvt0 = params.dvt0,
        eta0 = params.eta0,
        rdsw = params.rdsw,
    )
}

/// Generates a valid SPICE EKV `.MODEL` card deck from extracted target parameters.
pub fn generate_ekv_model_deck(
    model_name: &str,
    is_nmos: bool,
    params: &EkvTargetParams,
) -> String {
    let polarity = if is_nmos { "NMOS" } else { "PMOS" };
    let sanitized_name = if model_name.trim().is_empty() {
        "EKV_EXTRACTED"
    } else {
        model_name.trim()
    };

    format!(
        "* Phonon Universal EKV Extracted Model Deck\n\
         * Target: {name} ({polarity})\n\
         .MODEL {name} {polarity}\n\
         + LEVEL=55\n\
         + VTO={vto:.6} KP={kp:.6e} GAMMA={gamma:.6} THETA={theta:.6}\n\
         .END\n",
        name = sanitized_name,
        polarity = polarity,
        vto = params.vto,
        kp = params.kp,
        gamma = params.gamma,
        theta = params.theta,
    )
}

/// Generates a valid SPICE Gummel-Poon BJT `.MODEL` card deck from extracted target parameters.
pub fn generate_bjt_model_deck(
    model_name: &str,
    is_npn: bool,
    params: &BjtTargetParams,
) -> String {
    let polarity = if is_npn { "NPN" } else { "PNP" };
    let sanitized_name = if model_name.trim().is_empty() {
        "BJT_EXTRACTED"
    } else {
        model_name.trim()
    };

    format!(
        "* Phonon Universal Gummel-Poon BJT Extracted Model Deck\n\
         * Target: {name} ({polarity})\n\
         .MODEL {name} {polarity}\n\
         + IS={is_val:.6e} BF={bf:.4} VAF={vaf:.4}\n\
         .END\n",
        name = sanitized_name,
        polarity = polarity,
        is_val = params.is,
        bf = params.bf,
        vaf = params.vaf,
    )
}

/// Validates that a SPICE BSIM4 `.MODEL` card deck complies with SPICE LEVEL=54 syntax requirements.
pub fn validate_bsim4_model_deck(deck: &str) -> Result<(), ExtractionError> {
    if !deck.contains(".MODEL") {
        return Err(ExtractionError::InvalidFormat(
            "Missing .MODEL directive in SPICE deck".to_string(),
        ));
    }

    if !deck.contains("LEVEL=54") && !deck.contains("level=54") {
        return Err(ExtractionError::InvalidFormat(
            "BSIM4 deck must specify LEVEL=54".to_string(),
        ));
    }

    if !deck.contains("VTH0=") && !deck.contains("vth0=") {
        return Err(ExtractionError::InvalidFormat(
            "BSIM4 deck must specify threshold parameter VTH0".to_string(),
        ));
    }

    if !deck.contains("U0=") && !deck.contains("u0=") {
        return Err(ExtractionError::InvalidFormat(
            "BSIM4 deck must specify mobility parameter U0".to_string(),
        ));
    }

    Ok(())
}

/// Validates that a SPICE EKV `.MODEL` card deck complies with EKV syntax requirements.
pub fn validate_ekv_model_deck(deck: &str) -> Result<(), ExtractionError> {
    if !deck.contains(".MODEL") {
        return Err(ExtractionError::InvalidFormat(
            "Missing .MODEL directive in SPICE deck".to_string(),
        ));
    }

    if !deck.contains("LEVEL=55") && !deck.contains("level=55") {
        return Err(ExtractionError::InvalidFormat(
            "EKV deck must specify LEVEL=55".to_string(),
        ));
    }

    if !deck.contains("VTO=") && !deck.contains("vto=") {
        return Err(ExtractionError::InvalidFormat(
            "EKV deck must specify nominal threshold parameter VTO".to_string(),
        ));
    }

    if !deck.contains("KP=") && !deck.contains("kp=") {
        return Err(ExtractionError::InvalidFormat(
            "EKV deck must specify transconductance parameter KP".to_string(),
        ));
    }

    Ok(())
}

/// Validates that a SPICE BJT `.MODEL` card deck complies with Gummel-Poon syntax requirements.
pub fn validate_bjt_model_deck(deck: &str) -> Result<(), ExtractionError> {
    if !deck.contains(".MODEL") {
        return Err(ExtractionError::InvalidFormat(
            "Missing .MODEL directive in SPICE deck".to_string(),
        ));
    }

    if !deck.contains("IS=") && !deck.contains("is=") {
        return Err(ExtractionError::InvalidFormat(
            "BJT deck must specify saturation current parameter IS".to_string(),
        ));
    }

    if !deck.contains("BF=") && !deck.contains("bf=") {
        return Err(ExtractionError::InvalidFormat(
            "BJT deck must specify forward beta parameter BF".to_string(),
        ));
    }

    if !deck.contains("VAF=") && !deck.contains("vaf=") {
        return Err(ExtractionError::InvalidFormat(
            "BJT deck must specify Early voltage parameter VAF".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsim4_deck_generation_and_validation() {
        let params = Bsim4TargetParams {
            vth0: 0.45,
            u0: 0.05,
            vsat: 1.0e5,
            dvt0: 1.0,
            eta0: 0.08,
            rdsw: 100.0,
            subthreshold_swing_mv_dec: 80.0,
        };

        let result = FittingResult {
            params,
            rmse: 1.25e-6,
            r_squared: 0.9995,
            iterations: 25,
            converged: true,
        };

        let deck = generate_bsim4_model_deck("NMOS_EXTRACTED", true, &params, &result);
        assert!(validate_bsim4_model_deck(&deck).is_ok());
        assert!(deck.contains(".MODEL NMOS_EXTRACTED NMOS"));
        assert!(deck.contains("LEVEL=54"));
        assert!(deck.contains("VTH0=0.450000"));
        assert!(deck.contains("U0=0.050000"));
    }

    #[test]
    fn test_ekv_deck_generation_and_validation() {
        let params = EkvTargetParams {
            vto: 0.52,
            kp: 1.45e-4,
            gamma: 0.48,
            theta: 0.045,
        };

        let deck = generate_ekv_model_deck("EKV_NMOS_EXTRACTED", true, &params);
        assert!(validate_ekv_model_deck(&deck).is_ok());
        assert!(deck.contains(".MODEL EKV_NMOS_EXTRACTED NMOS"));
        assert!(deck.contains("LEVEL=55"));
        assert!(deck.contains("VTO=0.520000"));
        assert!(deck.contains("KP=1.450000e-4"));
        assert!(deck.contains("GAMMA=0.480000"));
        assert!(deck.contains("THETA=0.045000"));
    }

    #[test]
    fn test_bjt_deck_generation_and_validation() {
        let params = BjtTargetParams {
            is: 2.0e-15,
            bf: 150.0,
            vaf: 85.0,
        };

        let deck = generate_bjt_model_deck("BJT_2N2222_EXTRACTED", true, &params);
        assert!(validate_bjt_model_deck(&deck).is_ok());
        assert!(deck.contains(".MODEL BJT_2N2222_EXTRACTED NPN"));
        assert!(deck.contains("IS=2.000000e-15"));
        assert!(deck.contains("BF=150.0000"));
        assert!(deck.contains("VAF=85.0000"));
    }
}
