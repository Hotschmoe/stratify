//! Resizable Panel Divider
//!
//! A draggable vertical divider that allows users to resize adjacent panels.

use iced::widget::{button, container, rule};
use iced::{Element, Length};

use crate::{DividerType, Message};

/// Create a draggable vertical divider between panels
///
/// The divider includes 15px padding on each side of a 1px vertical line,
/// matching the layout: panel[15px]|[15px]panel
pub fn view_divider(divider_type: DividerType, is_dragging: bool) -> Element<'static, Message> {
    // The actual visible line (1px wide)
    let line = rule::vertical(1);

    // Container with padding to create the 15px gaps on each side
    // Total width: 15px (left gap) + 1px (line) + 15px (right gap) = 31px
    let divider_content = container(line)
        .padding(iced::Padding {
            top: 0.0,
            right: 14.0,
            bottom: 0.0,
            left: 14.0,
        })
        .height(Length::Fill);

    // Use a button for the interactive area - simpler than mouse_area
    // The button sends a message with x=0, which will be adjusted by the subscription
    // that tracks actual cursor position during drag
    let interactive = button(divider_content)
        .on_press(Message::DividerDragStart(divider_type, 0.0))
        .padding(0)
        .style(move |theme: &iced::Theme, status| {
            let palette = theme.extended_palette();
            let is_hovered = matches!(status, button::Status::Hovered | button::Status::Pressed);

            button::Style {
                background: if is_dragging {
                    Some(palette.primary.weak.color.into())
                } else if is_hovered {
                    Some(iced::Color::from_rgba(0.5, 0.5, 0.5, 0.1).into())
                } else {
                    None
                },
                border: iced::Border::default(),
                text_color: palette.background.base.text,
                shadow: iced::Shadow::default(),
                snap: false,
            }
        });

    interactive.into()
}
