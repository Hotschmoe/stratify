//! Results view for Wood Beam calculations
//!
//! Shows:
//! - Pass/Fail status
//! - Governing condition
//! - Demand values (moment, shear, deflection)
//! - Capacity checks with unity ratios
//! - Section properties
//! - Support reactions (max and min)
//! - Diagrams (beam schematic, shear, moment, deflection)

use iced::widget::{column, text, Canvas, Column, Space};
use iced::{Element, Length};

use calc_core::calculations::continuous_beam::{ContinuousBeamInput, ContinuousBeamResult};

use crate::Message;
use super::shared::diagrams::{BeamDiagram, BeamDiagramData};

/// Render the beam calculation results
pub fn view<'a>(input: &'a ContinuousBeamInput, result: &'a ContinuousBeamResult) -> Column<'a, Message> {
    let results_text = view_calculation_results(input, result);
    let diagram_data = BeamDiagramData::from_calc(input, result);
    let diagram = BeamDiagram::new(diagram_data);

    let canvas_widget: Element<'_, Message> = Canvas::new(diagram)
        .width(Length::Fill)
        .height(Length::Fixed(340.0))
        .into();

    results_text
        .push(Space::new().height(15))
        .push(text("Diagrams").size(14))
        .push(Space::new().height(8))
        .push(canvas_widget)
}

/// Render calculation results text
fn view_calculation_results<'a>(input: &'a ContinuousBeamInput, result: &'a ContinuousBeamResult) -> Column<'a, Message> {
    let pass_fail = if result.passes() {
        text("DESIGN ADEQUATE").size(16).color([0.2, 0.6, 0.2])
    } else {
        text("DESIGN INADEQUATE").size(16).color([0.8, 0.2, 0.2])
    };

    let governing = text(format!("Governing: {}", result.governing_condition)).size(11);

    // Get first span result for detailed stress checks (single-span case)
    let span_result = result.span_results.first();

    let (bending_status, bending_unity, actual_fb, allowable_fb) = span_result
        .map(|sr| {
            let status = if sr.bending_unity <= 1.0 { "OK" } else { "FAIL" };
            (status, sr.bending_unity, sr.actual_fb_psi, sr.allowable_fb_psi)
        })
        .unwrap_or(("N/A", 0.0, 0.0, 0.0));

    let (shear_status, shear_unity, actual_fv, allowable_fv) = span_result
        .map(|sr| {
            let status = if sr.shear_unity <= 1.0 { "OK" } else { "FAIL" };
            (status, sr.shear_unity, sr.actual_fv_psi, sr.allowable_fv_psi)
        })
        .unwrap_or(("N/A", 0.0, 0.0, 0.0));

    let (defl_status, defl_unity) = span_result
        .map(|sr| {
            let status = if sr.deflection_unity <= 1.0 { "OK" } else { "FAIL" };
            (status, sr.deflection_unity)
        })
        .unwrap_or(("N/A", 0.0));

    // Build reactions display string (R_1, R_2, R_3, etc.)
    let reactions_str = result.reactions
        .iter()
        .enumerate()
        .map(|(i, r)| format!("R_{} = {:.0} lb", i + 1, r))
        .collect::<Vec<_>>()
        .join(", ");

    // Get section properties from input
    let (section_modulus, moment_inertia) = input.spans.first()
        .map(|span| (span.section_modulus_in3(), span.moment_of_inertia_in4()))
        .unwrap_or((0.0, 0.0));

    column![
        text("Calculation Results").size(14),
        Space::new().height(8),
        pass_fail,
        governing,
        Space::new().height(12),
        text("Load Summary").size(12),
        text(format!("Governing Combo: {}", result.governing_combination)).size(11),
        Space::new().height(12),
        text("Demand").size(12),
        text(format!("Max Moment: {:.0} ft-lb", result.max_positive_moment_ftlb)).size(11),
        text(format!("Max Shear: {:.0} lb", result.max_shear_lb)).size(11),
        text(format!("Max Deflection: {:.3} in", result.max_deflection_in)).size(11),
        Space::new().height(12),
        text("Capacity Checks").size(12),
        text(format!(
            "Bending: {:.0}/{:.0} psi = {:.2} [{}]",
            actual_fb, allowable_fb, bending_unity, bending_status
        )).size(11),
        text(format!(
            "Shear: {:.0}/{:.0} psi = {:.2} [{}]",
            actual_fv, allowable_fv, shear_unity, shear_status
        )).size(11),
        text(format!(
            "Deflection: {:.2} [{}]",
            defl_unity, defl_status
        )).size(11),
        Space::new().height(12),
        text("Section Properties").size(12),
        text(format!("Section Modulus (S): {:.2} in³", section_modulus)).size(11),
        text(format!("Moment of Inertia (I): {:.2} in⁴", moment_inertia)).size(11),
        Space::new().height(12),
        text("Support Reactions").size(12),
        text(format!("Max: {}", reactions_str)).size(11),
        text(format!("  ({})", result.governing_combination)).size(10),
        view_min_reactions(result),
    ]
}

/// Render minimum reactions section with uplift warning
fn view_min_reactions<'a>(result: &'a ContinuousBeamResult) -> Element<'a, Message> {
    // Build min reactions display string
    let min_reactions_str = result.min_reactions
        .iter()
        .enumerate()
        .map(|(i, r)| format!("R_{} = {:.0} lb", i + 1, r))
        .collect::<Vec<_>>()
        .join(", ");

    // Check for uplift at any reaction
    let has_uplift = result.min_reactions.iter().any(|&r| r < 0.0);

    let min_reactions_text = text(format!("Min: {}", min_reactions_str)).size(11);
    let combo_text = text(format!("  ({})", result.min_reaction_combination)).size(10);

    if has_uplift {
        let uplift_warning = text("UPLIFT - Hold-downs required!")
            .size(11)
            .color([0.9, 0.5, 0.0]);

        // Find the worst uplift value and its location
        let (worst_idx, worst_value) = result.min_reactions
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, v)| (i + 1, *v))
            .unwrap_or((1, 0.0));

        let uplift_detail = text(format!(
            "Max uplift at R_{}: {:.0} lb",
            worst_idx,
            worst_value.abs()
        ))
        .size(10)
        .color([0.9, 0.5, 0.0]);

        column![
            min_reactions_text,
            combo_text,
            Space::new().height(4),
            uplift_warning,
            uplift_detail,
        ]
        .spacing(2)
        .into()
    } else {
        column![min_reactions_text, combo_text]
            .spacing(2)
            .into()
    }
}
