//! Results view for Project Information
//!
//! Shows project summary and keyboard shortcuts when no beam is selected.

use iced::widget::{column, text, Column, Space};

use calc_core::project::Project;

use crate::Message;

/// Render the project summary view
pub fn view(project: &Project) -> Column<'_, Message> {
    let item_count = project.item_count();

    column![
        text("Project Summary").size(14),
        Space::new().height(8),
        text(format!("Engineer: {}", project.meta.engineer)).size(11),
        text(format!("Job ID: {}", project.meta.job_id)).size(11),
        text(format!("Client: {}", project.meta.client)).size(11),
        text(format!("Items: {}", item_count)).size(11),
        Space::new().height(15),
        text("Select an item from the list or create a new beam.").size(11),
        Space::new().height(8),
        text("Keyboard shortcuts:").size(11),
        text("  Ctrl+N: New project").size(10),
        text("  Ctrl+O: Open project").size(10),
        text("  Ctrl+S: Save project").size(10),
        text("  Ctrl+Shift+S: Save as").size(10),
    ]
}
