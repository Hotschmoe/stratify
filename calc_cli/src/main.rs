//! # Stratify CLI Application
//!
//! Terminal-based interface for structural engineering calculations.
//! Built with Ratatui for a rich TUI experience.
//!
//! ## Status
//!
//! This is a placeholder. The CLI will be implemented after
//! calc_core and calc_gui are functional.

use std::io::{self, BufRead, Write};

use calc_core::calculations::beam::{calculate, BeamInput};
use calc_core::loads::{DesignMethod, DiscreteLoad, EnhancedLoadCase, LoadType};
use calc_core::materials::{Material, WoodGrade, WoodMaterial, WoodSpecies};

fn prompt_f64(prompt: &str, default: f64) -> f64 {
    print!("{}", prompt);
    if io::stdout().flush().is_err() {
        return default;
    }

    let mut input = String::new();
    if io::stdin().lock().read_line(&mut input).is_err() {
        return default;
    }

    input.trim().parse().unwrap_or(default)
}

fn main() {
    println!("Stratify CLI - Structural Engineering Calculator");
    println!("================================================");
    println!();
    println!("TUI not yet implemented. Running simple CLI demo...");
    println!();

    let span_ft = prompt_f64("Enter beam span (ft) [12.0]: ", 12.0);
    let load_plf = prompt_f64("Enter uniform load (plf) [100.0]: ", 100.0);

    println!();
    println!("Calculating 2x10 DF-L No.2 beam...");
    println!();

    // Create load case (assume 30% dead, 70% live for demo)
    let dead_plf = load_plf * 0.3;
    let live_plf = load_plf * 0.7;
    let load_case = EnhancedLoadCase::new("Demo Loads")
        .with_load(DiscreteLoad::uniform(LoadType::Dead, dead_plf))
        .with_load(DiscreteLoad::uniform(LoadType::Live, live_plf));

    let beam = BeamInput {
        label: "CLI-Demo".to_string(),
        span_ft,
        load_case,
        material: Material::SawnLumber(WoodMaterial::new(
            WoodSpecies::DouglasFirLarch,
            WoodGrade::No2,
        )),
        width_in: 1.5,
        depth_in: 9.25,
        adjustment_factors: calc_core::nds_factors::AdjustmentFactors::default(),
    };

    match calculate(&beam, DesignMethod::Asd) {
        Ok(result) => {
            println!("═══════════════════════════════════════");
            println!("  BEAM CALCULATION RESULTS");
            println!("═══════════════════════════════════════");
            println!();
            println!("Input:");
            println!("  Span:     {:.1} ft", beam.span_ft);
            println!("  Load:     {:.0} plf (D={:.0}, L={:.0})", load_plf, dead_plf, live_plf);
            println!("  Section:  2x10 (1.5\" x 9.25\")");
            println!("  Material: DF-L No.2");
            println!();
            println!("Demand:");
            println!("  M_max = {:.0} ft-lb", result.max_moment_ftlb);
            println!("  V_max = {:.0} lb", result.max_shear_lb);
            println!("  δ_max = {:.3}\"", result.max_deflection_in);
            println!();
            println!("Capacity Checks:");
            println!("  Bending:    {:.2} ({:.0}/{:.0} psi) {}",
                result.bending_unity,
                result.actual_fb_psi,
                result.allowable_fb_psi,
                status_icon(result.bending_unity <= 1.0)
            );
            println!("  Shear:      {:.2} ({:.0}/{:.0} psi) {}",
                result.shear_unity,
                result.actual_fv_psi,
                result.allowable_fv_psi,
                status_icon(result.shear_unity <= 1.0)
            );
            println!("  Deflection: {:.2} (L/{:.0} vs L/{:.0}) {}",
                result.deflection_unity,
                result.deflection_ratio,
                result.deflection_limit_ratio,
                status_icon(result.deflection_unity <= 1.0)
            );
            println!();
            println!("═══════════════════════════════════════");
            println!("  RESULT: {} (governs: {})",
                if result.passes() { "PASS" } else { "FAIL" },
                result.governing_condition()
            );
            println!("═══════════════════════════════════════");

            println!();
            println!("JSON Output (for LLM/API use):");
            if let Ok(json) = serde_json::to_string_pretty(&result) {
                println!("{}", json);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            if let Ok(json) = serde_json::to_string_pretty(&e) {
                eprintln!();
                eprintln!("Error JSON:");
                eprintln!("{}", json);
            }
        }
    }
}

fn status_icon(pass: bool) -> &'static str {
    if pass { "[OK]" } else { "[FAIL]" }
}
