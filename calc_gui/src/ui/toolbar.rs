//! Toolbar component
//!
//! Contains file operations (New, Open, Save, Save As, Export PDF) and settings.

use iced::widget::{button, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

use crate::Message;

/// Render the application header with title (borrowed)
#[allow(dead_code)]
pub fn view_header(window_title: &str) -> Element<'_, Message> {
    row![
        text("Stratify").size(28),
        Space::new().width(Length::Fill),
        text(window_title).size(14),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Render the application header with title (owned)
pub fn view_header_owned(window_title: String) -> Element<'static, Message> {
    row![
        text("Stratify").size(28),
        Space::new().width(Length::Fill),
        text(window_title).size(14),
    ]
    .align_y(Alignment::Center)
    .into()
}

/// Render the toolbar with file operations and settings
pub fn view_toolbar(dark_mode: bool) -> Element<'static, Message> {
    let file_buttons = row![
        button(text("New").size(11))
            .on_press(Message::NewProject)
            .padding(Padding::from([4, 8]))
            .style(button::secondary),
        button(text("Open").size(11))
            .on_press(Message::OpenProject)
            .padding(Padding::from([4, 8]))
            .style(button::secondary),
        button(text("Save").size(11))
            .on_press(Message::SaveProject)
            .padding(Padding::from([4, 8]))
            .style(button::secondary),
        button(text("Save As").size(11))
            .on_press(Message::SaveProjectAs)
            .padding(Padding::from([4, 8]))
            .style(button::secondary),
        button(text("Export PDF").size(11))
            .on_press(Message::ExportPdf)
            .padding(Padding::from([4, 8]))
            .style(button::primary),
    ]
    .spacing(4);

    // Settings section
    let theme_icon = if dark_mode { "Light Mode" } else { "Dark Mode" };
    let settings_buttons = row![
        button(text(theme_icon).size(11))
            .on_press(Message::ToggleDarkMode)
            .padding(Padding::from([4, 8]))
            .style(button::secondary),
    ]
    .spacing(4);

    row![
        file_buttons,
        Space::new().width(Length::Fill),
        text("Settings").size(11),
        Space::new().width(8),
        settings_buttons,
    ]
    .padding(Padding::from([4, 0]))
    .align_y(Alignment::Center)
    .into()
}
