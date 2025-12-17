//! # Stratify GUI Application
//!
//! Full-featured graphical interface for structural engineering calculations.
//! Built with Iced framework for cross-platform support (Windows, macOS, Linux, WASM).
//!
//! ## Status
//!
//! This is a placeholder. The GUI will be implemented in Phase 2 after
//! calc_core functionality is validated.

use calc_core::calculations::beam::{calculate, BeamInput};
use calc_core::materials::{WoodGrade, WoodMaterial, WoodSpecies};
use calc_core::project::Project;

fn main() {
    println!("Stratify GUI - Structural Engineering Suite");
    println!("============================================");
    println!();
    println!("GUI not yet implemented. Running calc_core demo instead...");
    println!();

    // Demo: Create a project and run a beam calculation
    let mut project = Project::new("Demo Engineer", "25-001", "Demo Client");
    println!("Created project: {} for {}", project.meta.job_id, project.meta.client);
    println!("Code: {}", project.settings.code);
    println!();

    // Create a beam calculation
    let beam = BeamInput {
        label: "B-1 Floor Beam".to_string(),
        span_ft: 12.0,
        uniform_load_plf: 100.0, // Light load for demo
        material: WoodMaterial::new(WoodSpecies::DouglasFirLarch, WoodGrade::No2),
        width_in: 1.5,  // 2x nominal
        depth_in: 9.25, // 2x10
    };

    println!("Beam: {}", beam.label);
    println!("  Span: {} ft", beam.span_ft);
    println!("  Load: {} plf", beam.uniform_load_plf);
    println!("  Size: {:.2}\" x {:.2}\"", beam.width_in, beam.depth_in);
    println!("  Material: {} {}",
        beam.material.species.display_name(),
        beam.material.grade.display_name()
    );
    println!();

    // Run calculation
    match calculate(&beam) {
        Ok(result) => {
            println!("Results:");
            println!("  Max Moment: {:.0} ft-lb", result.max_moment_ftlb);
            println!("  Max Shear: {:.0} lb", result.max_shear_lb);
            println!("  Max Deflection: {:.3}\"", result.max_deflection_in);
            println!();
            println!("Checks:");
            println!("  Bending: fb = {:.0} psi / Fb' = {:.0} psi = {:.2} ({})",
                result.actual_fb_psi,
                result.allowable_fb_psi,
                result.bending_unity,
                if result.bending_unity <= 1.0 { "OK" } else { "FAIL" }
            );
            println!("  Shear: fv = {:.0} psi / Fv' = {:.0} psi = {:.2} ({})",
                result.actual_fv_psi,
                result.allowable_fv_psi,
                result.shear_unity,
                if result.shear_unity <= 1.0 { "OK" } else { "FAIL" }
            );
            println!("  Deflection: L/{:.0} vs L/{:.0} = {:.2} ({})",
                result.deflection_ratio,
                result.deflection_limit_ratio,
                result.deflection_unity,
                if result.deflection_unity <= 1.0 { "OK" } else { "FAIL" }
            );
            println!();
            println!("Overall: {} (governed by {})",
                if result.passes() { "PASS ✓" } else { "FAIL ✗" },
                result.governing_condition()
            );
        }
        Err(e) => {
            println!("Calculation error: {}", e);
        }
    }

    // Add beam to project and show JSON
    project.add_item(calc_core::calculations::CalculationItem::Beam(beam));
    println!();
    println!("Project JSON:");
    println!("{}", serde_json::to_string_pretty(&project).unwrap());
}
