#![deny(unsafe_code)]

//! SPICE `.MODEL` parameter card deck generation and syntax validation for BSIM4.

use super::curve_data::ExtractionError;
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bsim4_deck_generation_and_netlist_parser_validation() {
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

        // Validate complete AST parsing through phonon_netlist
        let parsed = phonon_netlist::parse_netlist(&deck);
        assert!(parsed.is_ok(), "Netlist parsing error: {:?}", parsed.err());
        let netlist = parsed.unwrap();
        assert!(netlist.models.contains_key("NMOS_EXTRACTED"));
        let model = &netlist.models["NMOS_EXTRACTED"];
        assert_eq!(model.model_type, "NMOS");
        assert_eq!(model.params.get("LEVEL"), Some(&54.0));
        assert_eq!(model.params.get("TNOM"), Some(&300.0));
    }
}
