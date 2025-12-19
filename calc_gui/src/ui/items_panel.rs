//! Items Panel (Left Sidebar)
//!
//! Displays project navigation tree with:
//! - Project Info section (clickable to edit)
//! - Wood Beams section with [+] button to create new beams
//! - Future sections (grayed out)

use std::collections::HashSet;

use iced::widget::{button, column, container, row, rule, scrollable, text, Column, Space};
use iced::{Alignment, Element, Length, Padding};
use uuid::Uuid;

use calc_core::calculations::CalculationItem;
use calc_core::project::Project;

use crate::{EditorSelection, ItemSection, Message};

/// Render the items panel (left sidebar)
pub fn view_items_panel<'a>(
    project: &'a Project,
    collapsed_sections: &'a HashSet<ItemSection>,
    selection: &'a EditorSelection,
    selected_beam_id: Option<Uuid>,
    width: f32,
) -> Element<'a, Message> {
    let mut panel_content: Column<'_, Message> = column![].spacing(2);

    // ===== Project Info Section =====
    let project_expanded = !collapsed_sections.contains(&ItemSection::ProjectInfo);
    let project_selected = matches!(selection, EditorSelection::ProjectInfo);
    let project_header = view_section_header("Project Info", ItemSection::ProjectInfo, project_expanded, None);
    panel_content = panel_content.push(project_header);

    if project_expanded {
        let project_info_content = column![
            text(format!("Eng: {}", project.meta.engineer)).size(10),
            text(format!("Job: {}", project.meta.job_id)).size(10),
            text(format!("Client: {}", project.meta.client)).size(10),
        ]
        .spacing(2);

        // Make clickable to select project info for editing
        let project_btn_style = if project_selected {
            button::primary
        } else {
            button::secondary
        };
        let project_info_btn = button(project_info_content)
            .on_press(Message::SelectProjectInfo)
            .padding(Padding::from([4, 16]))
            .style(project_btn_style)
            .width(Length::Fill);

        panel_content = panel_content.push(project_info_btn);
    }

    panel_content = panel_content.push(rule::horizontal(1));

    // ===== Wood Beams Section =====
    let beams_expanded = !collapsed_sections.contains(&ItemSection::WoodBeams);
    let beam_count = project.items.values()
        .filter(|i| matches!(i, CalculationItem::Beam(_)))
        .count();

    // Section header with expand/collapse and add button
    let beams_indicator = if beams_expanded { "▼" } else { "▶" };
    let beams_header_btn = button(
        row![
            text(beams_indicator).size(10),
            Space::new().width(4),
            text(format!("Wood Beams ({})", beam_count)).size(11),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::ToggleSection(ItemSection::WoodBeams))
    .padding(Padding::from([4, 6]))
    .style(button::text)
    .width(Length::Fill);

    let beams_header = row![
        beams_header_btn,
        button(text("+").size(11))
            .on_press(Message::CreateBeam)
            .padding(Padding::from([2, 6]))
            .style(button::secondary),
    ]
    .spacing(2);
    panel_content = panel_content.push(beams_header);

    if beams_expanded {
        let mut beams_list: Column<'_, Message> = column![].spacing(2).padding(Padding::from([4, 8]));

        // List beams
        for (id, item) in &project.items {
            if let CalculationItem::Beam(beam) = item {
                let is_selected = selected_beam_id == Some(*id);
                let btn = if is_selected {
                    button(text(&beam.label).size(10))
                        .on_press(Message::SelectBeam(*id))
                        .padding(Padding::from([3, 6]))
                        .style(button::primary)
                        .width(Length::Fill)
                } else {
                    button(text(&beam.label).size(10))
                        .on_press(Message::SelectBeam(*id))
                        .padding(Padding::from([3, 6]))
                        .style(button::secondary)
                        .width(Length::Fill)
                };
                beams_list = beams_list.push(btn);
            }
        }

        if beam_count == 0 {
            beams_list = beams_list.push(text("(none)").size(10).color([0.5, 0.5, 0.5]));
        }

        panel_content = panel_content.push(beams_list);
    }

    panel_content = panel_content.push(rule::horizontal(1));

    // ===== Wood Columns Section (Future) =====
    let columns_header = view_section_header_disabled("Wood Columns", 0);
    panel_content = panel_content.push(columns_header);

    panel_content = panel_content.push(rule::horizontal(1));

    // ===== Future Sections (Grayed out) =====
    let future_sections = column![
        text("Cont. Footings").size(10).color([0.6, 0.6, 0.6]),
        text("Spread Footings").size(10).color([0.6, 0.6, 0.6]),
        text("Cantilever Walls").size(10).color([0.6, 0.6, 0.6]),
        text("Restrained Walls").size(10).color([0.6, 0.6, 0.6]),
    ]
    .spacing(4)
    .padding(Padding::from([6, 8]));
    panel_content = panel_content.push(future_sections);

    let panel = container(scrollable(panel_content.padding(4)))
        .width(Length::Fixed(width))
        .height(Length::Fill)
        .style(container::bordered_box)
        .padding(4);

    panel.into()
}

/// Create a collapsible section header with expand/collapse indicator
fn view_section_header<'a>(
    title: &'a str,
    section: ItemSection,
    expanded: bool,
    add_action: Option<Message>,
) -> Element<'a, Message> {
    let indicator = if expanded { "▼" } else { "▶" };

    let header_btn = button(
        row![
            text(indicator).size(10),
            Space::new().width(4),
            text(title).size(11),
        ]
        .align_y(Alignment::Center),
    )
    .on_press(Message::ToggleSection(section))
    .padding(Padding::from([4, 6]))
    .style(button::text)
    .width(Length::Fill);

    if let Some(action) = add_action {
        row![
            header_btn,
            button(text("+").size(11))
                .on_press(action)
                .padding(Padding::from([2, 6]))
                .style(button::secondary),
        ]
        .spacing(2)
        .into()
    } else {
        header_btn.into()
    }
}

/// Create a disabled section header for future features
fn view_section_header_disabled<'a>(title: &'a str, count: usize) -> Element<'a, Message> {
    row![
        text("▶").size(10).color([0.6, 0.6, 0.6]),
        Space::new().width(4),
        text(format!("{} ({})", title, count)).size(11).color([0.6, 0.6, 0.6]),
    ]
    .padding(Padding::from([4, 6]))
    .align_y(Alignment::Center)
    .into()
}
