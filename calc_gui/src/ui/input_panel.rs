//! Input Panel (Center)
//!
//! Dispatches to the appropriate input editor based on EditorSelection:
//! - ProjectInfo -> input_project_info
//! - Beam -> input_wood_beam
//! - None -> welcome message

use iced::widget::{container, scrollable, text, Column};
use iced::{Element, Length};

use crate::{App, EditorSelection, Message};
use super::{input_project_info, input_wood_beam};

/// Render the input panel based on current selection
pub fn view_input_panel(app: &App) -> Element<'_, Message> {
    let panel: Column<'_, Message> = match app.selection {
        EditorSelection::ProjectInfo => {
            input_project_info::view(&app.project.meta)
        }
        EditorSelection::None => {
            Column::new().push(
                text("Select an item from the left panel").size(14).color([0.5, 0.5, 0.5])
            )
        }
        EditorSelection::Beam(_) => {
            input_wood_beam::view(app)
        }
    };

    container(scrollable(panel.width(Length::FillPortion(3)).padding(8)))
        .style(container::bordered_box)
        .padding(5)
        .into()
}
