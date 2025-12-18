//! Status Bar (Bottom)
//!
//! Displays:
//! - Current file path
//! - Modified indicator (*)
//! - Lock holder (if read-only)
//! - Status messages

use std::path::PathBuf;

use iced::widget::{row, text, Space};
use iced::{Element, Length, Padding};

use crate::Message;

/// Render the status bar
pub fn view_status_bar<'a>(
    current_file: &'a Option<PathBuf>,
    is_modified: bool,
    lock_holder: &'a Option<String>,
    status: &'a str,
) -> Element<'a, Message> {
    let file_info = match current_file {
        Some(path) => path.display().to_string(),
        None => "Untitled".to_string(),
    };

    let lock_info = match lock_holder {
        Some(holder) => format!(" [Locked by: {}]", holder),
        None => String::new(),
    };

    let modified_indicator = if is_modified { " *" } else { "" };

    row![
        text(format!("{}{}", file_info, modified_indicator)).size(10),
        text(lock_info).size(10).color([0.6, 0.3, 0.0]),
        Space::new().width(Length::Fill),
        text(status).size(10),
    ]
    .padding(Padding::from([4, 0]))
    .into()
}
