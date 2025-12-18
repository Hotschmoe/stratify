//! Results Panel (Right Side)
//!
//! Dispatches to the appropriate results view based on EditorSelection:
//! - ProjectInfo -> result_project_info (project summary)
//! - Beam with results -> result_wood_beam (calculation results + diagrams)
//! - Error -> error display

use iced::widget::{column, container, scrollable, text, Column, Space};
use iced::{Element, Length};

use crate::{App, Message};
use super::{result_project_info, result_wood_beam};

/// Render the results panel based on current selection and calculation state
pub fn view_results_panel(app: &App) -> Element<'_, Message> {
    let content: Column<'_, Message> = if let Some(ref error) = app.error_message {
        // Show error message
        column![
            text("Error").size(14),
            Space::new().height(8),
            text(error).size(12).color([0.8, 0.2, 0.2]),
        ]
    } else if let (Some(ref input), Some(ref result)) = (&app.calc_input, &app.result) {
        // Show beam calculation results
        result_wood_beam::view(input, result)
    } else {
        // Show project summary
        result_project_info::view(&app.project)
    };

    let panel = container(scrollable(content.padding(8)))
        .width(Length::Fill)
        .style(container::bordered_box)
        .padding(5);

    panel.into()
}
