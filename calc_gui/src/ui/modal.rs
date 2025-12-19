//! Modal dialog component
//!
//! Provides a reusable modal overlay system for confirmation dialogs,
//! alerts, and other popup interactions.

use iced::widget::{button, column, container, row, text, Space};
use iced::{Alignment, Element, Length, Padding};

use crate::Message;

/// Types of modal dialogs
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModalType {
    /// Prompt to save unsaved changes before an action
    UnsavedChanges {
        /// The action that triggered this modal (for display)
        action: PendingAction,
    },
}

/// Actions that can be pending while a modal is shown
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingAction {
    /// User wants to create a new project
    NewProject,
    /// User wants to open an existing project
    OpenProject,
}

impl std::fmt::Display for PendingAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PendingAction::NewProject => write!(f, "create a new project"),
            PendingAction::OpenProject => write!(f, "open another project"),
        }
    }
}

/// Render a modal backdrop (semi-transparent overlay that catches clicks)
pub fn view_backdrop() -> Element<'static, Message> {
    button(Space::new())
        .on_press(Message::ModalCancel)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_, _| {
            iced::widget::button::Style::default()
                .with_background(iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5))
        })
        .into()
}

/// Render a modal dialog based on its type
pub fn view_modal(modal_type: &ModalType) -> Element<'_, Message> {
    match modal_type {
        ModalType::UnsavedChanges { action } => view_unsaved_changes_modal(*action),
    }
}

/// Render the "Save current progress?" modal
fn view_unsaved_changes_modal(action: PendingAction) -> Element<'static, Message> {
    let title = text("Save Changes?").size(18);

    let description = text(format!(
        "You have unsaved changes. Would you like to save before you {}?",
        action
    ))
    .size(12);

    let buttons = row![
        button(text("Don't Save").size(11))
            .on_press(Message::ModalDontSave)
            .padding(Padding::from([6, 16]))
            .style(button::secondary),
        Space::new().width(8),
        button(text("Cancel").size(11))
            .on_press(Message::ModalCancel)
            .padding(Padding::from([6, 16]))
            .style(button::secondary),
        Space::new().width(8),
        button(text("Save").size(11))
            .on_press(Message::ModalSave)
            .padding(Padding::from([6, 16]))
            .style(button::primary),
    ]
    .align_y(Alignment::Center);

    let content = column![
        title,
        Space::new().height(12),
        description,
        Space::new().height(20),
        buttons,
    ]
    .width(Length::Fixed(400.0));

    let modal_box = container(content)
        .padding(20)
        .style(container::bordered_box);

    // Center the modal in the screen
    container(modal_box)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(iced::alignment::Horizontal::Center)
        .align_y(iced::alignment::Vertical::Center)
        .into()
}
