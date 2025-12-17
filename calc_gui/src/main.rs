//! # Stratify GUI Application
//!
//! Full-featured graphical interface for structural engineering calculations.
//! Built with Iced framework for cross-platform support (Windows, macOS, Linux, WASM).

use std::fs;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable,
    text, text_input, vertical_space, Column,
};
use iced::{Alignment, Element, Font, Length, Padding, Task, Theme};

use calc_core::calculations::beam::{calculate, BeamInput, BeamResult};
use calc_core::materials::{WoodGrade, WoodMaterial, WoodSpecies};
use calc_core::pdf::render_beam_pdf;

// Embed BerkeleyMono font at compile time
const BERKELEY_MONO: &[u8] =
    include_bytes!("../../assets/fonts/BerkleyMono/BerkeleyMono-Regular.otf");
const BERKELEY_MONO_BOLD: &[u8] =
    include_bytes!("../../assets/fonts/BerkleyMono/BerkeleyMono-Bold.otf");

fn main() -> iced::Result {
    iced::application("Stratify - Structural Engineering", App::update, App::view)
        .theme(|_| Theme::Light)
        .window_size((900.0, 700.0))
        .font(BERKELEY_MONO)
        .font(BERKELEY_MONO_BOLD)
        .default_font(Font::with_name("Berkeley Mono"))
        .run_with(App::new)
}

// ============================================================================
// Application State
// ============================================================================

#[derive(Debug, Clone)]
struct App {
    // Project info
    engineer_name: String,
    job_id: String,

    // Beam input fields
    beam_label: String,
    span_ft: String,
    load_plf: String,
    width_in: String,
    depth_in: String,

    // Material selection
    selected_species: Option<WoodSpecies>,
    selected_grade: Option<WoodGrade>,

    // Calculation results
    result: Option<BeamResult>,
    error_message: Option<String>,

    // Status message
    status: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            engineer_name: "Engineer".to_string(),
            job_id: "25-001".to_string(),
            beam_label: "B-1".to_string(),
            span_ft: "12.0".to_string(),
            load_plf: "150.0".to_string(),
            width_in: "1.5".to_string(),
            depth_in: "9.25".to_string(),
            selected_species: Some(WoodSpecies::DouglasFirLarch),
            selected_grade: Some(WoodGrade::No2),
            result: None,
            error_message: None,
            status: "Ready".to_string(),
        }
    }
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }
}

// ============================================================================
// Messages
// ============================================================================

#[derive(Debug, Clone)]
enum Message {
    // Input field changes
    EngineerNameChanged(String),
    JobIdChanged(String),
    BeamLabelChanged(String),
    SpanChanged(String),
    LoadChanged(String),
    WidthChanged(String),
    DepthChanged(String),

    // Material selection
    SpeciesSelected(WoodSpecies),
    GradeSelected(WoodGrade),

    // Actions
    Calculate,
    ExportPdf,
    ClearResults,
}

// ============================================================================
// Update Logic
// ============================================================================

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::EngineerNameChanged(value) => {
                self.engineer_name = value;
            }
            Message::JobIdChanged(value) => {
                self.job_id = value;
            }
            Message::BeamLabelChanged(value) => {
                self.beam_label = value;
            }
            Message::SpanChanged(value) => {
                self.span_ft = value;
            }
            Message::LoadChanged(value) => {
                self.load_plf = value;
            }
            Message::WidthChanged(value) => {
                self.width_in = value;
            }
            Message::DepthChanged(value) => {
                self.depth_in = value;
            }
            Message::SpeciesSelected(species) => {
                self.selected_species = Some(species);
            }
            Message::GradeSelected(grade) => {
                self.selected_grade = Some(grade);
            }
            Message::Calculate => {
                self.run_calculation();
            }
            Message::ExportPdf => {
                self.export_pdf();
            }
            Message::ClearResults => {
                self.result = None;
                self.error_message = None;
                self.status = "Results cleared".to_string();
            }
        }
        Task::none()
    }

    fn run_calculation(&mut self) {
        self.error_message = None;

        // Parse inputs
        let span_ft = match self.span_ft.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                self.error_message = Some("Invalid span value".to_string());
                return;
            }
        };

        let load_plf = match self.load_plf.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                self.error_message = Some("Invalid load value".to_string());
                return;
            }
        };

        let width_in = match self.width_in.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                self.error_message = Some("Invalid width value".to_string());
                return;
            }
        };

        let depth_in = match self.depth_in.parse::<f64>() {
            Ok(v) => v,
            Err(_) => {
                self.error_message = Some("Invalid depth value".to_string());
                return;
            }
        };

        let species = match self.selected_species {
            Some(s) => s,
            None => {
                self.error_message = Some("Please select a wood species".to_string());
                return;
            }
        };

        let grade = match self.selected_grade {
            Some(g) => g,
            None => {
                self.error_message = Some("Please select a wood grade".to_string());
                return;
            }
        };

        // Build input
        let input = BeamInput {
            label: self.beam_label.clone(),
            span_ft,
            uniform_load_plf: load_plf,
            material: WoodMaterial::new(species, grade),
            width_in,
            depth_in,
        };

        // Run calculation
        match calculate(&input) {
            Ok(result) => {
                let status = if result.passes() {
                    "Calculation complete - PASS"
                } else {
                    "Calculation complete - FAIL"
                };
                self.status = status.to_string();
                self.result = Some(result);
            }
            Err(e) => {
                self.error_message = Some(format!("Calculation error: {}", e));
                self.result = None;
            }
        }
    }

    fn export_pdf(&mut self) {
        let result = match &self.result {
            Some(r) => r,
            None => {
                self.status = "Run calculation first before exporting PDF".to_string();
                return;
            }
        };

        // Build input for PDF
        let input = BeamInput {
            label: self.beam_label.clone(),
            span_ft: self.span_ft.parse().unwrap_or(0.0),
            uniform_load_plf: self.load_plf.parse().unwrap_or(0.0),
            material: WoodMaterial::new(
                self.selected_species.unwrap_or(WoodSpecies::DouglasFirLarch),
                self.selected_grade.unwrap_or(WoodGrade::No2),
            ),
            width_in: self.width_in.parse().unwrap_or(0.0),
            depth_in: self.depth_in.parse().unwrap_or(0.0),
        };

        // Generate PDF
        match render_beam_pdf(&input, result, &self.engineer_name, &self.job_id) {
            Ok(pdf_bytes) => {
                // Use file dialog to save
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Save PDF Report")
                    .set_file_name(&format!("{}_beam_report.pdf", self.beam_label))
                    .add_filter("PDF", &["pdf"])
                    .save_file()
                {
                    match fs::write(&path, &pdf_bytes) {
                        Ok(_) => {
                            self.status = format!("PDF saved to: {}", path.display());
                        }
                        Err(e) => {
                            self.status = format!("Failed to save PDF: {}", e);
                        }
                    }
                } else {
                    self.status = "PDF export cancelled".to_string();
                }
            }
            Err(e) => {
                self.status = format!("PDF generation failed: {}", e);
            }
        }
    }
}

// ============================================================================
// View
// ============================================================================

impl App {
    fn view(&self) -> Element<'_, Message> {
        let content = row![
            // Left panel - Input form
            self.view_input_panel(),
            // Right panel - Results
            self.view_results_panel(),
        ]
        .spacing(20);

        let main_content = column![
            // Header
            self.view_header(),
            horizontal_rule(2),
            vertical_space().height(10),
            // Main content
            content,
            // Status bar
            vertical_space().height(10),
            horizontal_rule(1),
            self.view_status_bar(),
        ]
        .padding(20);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn view_header(&self) -> Element<'_, Message> {
        row![
            text("Stratify").size(28),
            horizontal_space(),
            text("Structural Engineering Suite").size(14),
        ]
        .align_y(Alignment::Center)
        .into()
    }

    fn view_input_panel(&self) -> Element<'_, Message> {
        let project_section = column![
            text("Project Information").size(16),
            vertical_space().height(10),
            labeled_input("Engineer:", &self.engineer_name, Message::EngineerNameChanged),
            labeled_input("Job ID:", &self.job_id, Message::JobIdChanged),
        ]
        .spacing(8);

        let beam_section = column![
            text("Beam Parameters").size(16),
            vertical_space().height(10),
            labeled_input("Label:", &self.beam_label, Message::BeamLabelChanged),
            labeled_input("Span (ft):", &self.span_ft, Message::SpanChanged),
            labeled_input("Uniform Load (plf):", &self.load_plf, Message::LoadChanged),
            labeled_input("Width (in):", &self.width_in, Message::WidthChanged),
            labeled_input("Depth (in):", &self.depth_in, Message::DepthChanged),
        ]
        .spacing(8);

        let material_section = column![
            text("Material").size(16),
            vertical_space().height(10),
            text("Species:").size(12),
            pick_list(
                &WoodSpecies::ALL[..],
                self.selected_species,
                Message::SpeciesSelected
            )
            .width(Length::Fill)
            .placeholder("Select species..."),
            vertical_space().height(5),
            text("Grade:").size(12),
            pick_list(
                &WoodGrade::ALL[..],
                self.selected_grade,
                Message::GradeSelected
            )
            .width(Length::Fill)
            .placeholder("Select grade..."),
        ]
        .spacing(4);

        let buttons = row![
            button("Calculate")
                .on_press(Message::Calculate)
                .padding(Padding::from([8, 20])),
            horizontal_space().width(10),
            button("Export PDF")
                .on_press(Message::ExportPdf)
                .padding(Padding::from([8, 20])),
            horizontal_space().width(10),
            button("Clear")
                .on_press(Message::ClearResults)
                .padding(Padding::from([8, 20])),
        ];

        let panel = column![
            project_section,
            vertical_space().height(20),
            beam_section,
            vertical_space().height(20),
            material_section,
            vertical_space().height(20),
            buttons,
        ]
        .width(Length::FillPortion(2))
        .padding(10);

        container(scrollable(panel))
            .style(container::bordered_box)
            .padding(10)
            .into()
    }

    fn view_results_panel(&self) -> Element<'_, Message> {
        let content = if let Some(ref error) = self.error_message {
            column![
                text("Error").size(16),
                vertical_space().height(10),
                text(error).size(14).color([0.8, 0.2, 0.2]),
            ]
        } else if let Some(ref result) = self.result {
            self.view_calculation_results(result)
        } else {
            column![
                text("Results").size(16),
                vertical_space().height(10),
                text("Enter beam parameters and click Calculate").size(14),
            ]
        };

        let panel = container(scrollable(content.padding(10)))
            .width(Length::FillPortion(3))
            .style(container::bordered_box)
            .padding(10);

        panel.into()
    }

    fn view_calculation_results(&self, result: &BeamResult) -> Column<'_, Message> {
        let pass_fail = if result.passes() {
            text("DESIGN ADEQUATE").size(18).color([0.2, 0.6, 0.2])
        } else {
            text("DESIGN INADEQUATE").size(18).color([0.8, 0.2, 0.2])
        };

        let governing = text(format!("Governing: {}", result.governing_condition())).size(12);

        let bending_status = if result.bending_unity <= 1.0 { "OK" } else { "FAIL" };
        let shear_status = if result.shear_unity <= 1.0 { "OK" } else { "FAIL" };
        let defl_status = if result.deflection_unity <= 1.0 { "OK" } else { "FAIL" };

        column![
            text("Calculation Results").size(16),
            vertical_space().height(10),
            pass_fail,
            governing,
            vertical_space().height(15),
            // Demand section
            text("Demand").size(14),
            text(format!("Max Moment: {:.0} ft-lb", result.max_moment_ftlb)).size(12),
            text(format!("Max Shear: {:.0} lb", result.max_shear_lb)).size(12),
            text(format!("Max Deflection: {:.3} in", result.max_deflection_in)).size(12),
            vertical_space().height(15),
            // Capacity checks
            text("Capacity Checks").size(14),
            text(format!(
                "Bending: {:.0}/{:.0} psi = {:.2} [{}]",
                result.actual_fb_psi, result.allowable_fb_psi, result.bending_unity, bending_status
            )).size(12),
            text(format!(
                "Shear: {:.0}/{:.0} psi = {:.2} [{}]",
                result.actual_fv_psi, result.allowable_fv_psi, result.shear_unity, shear_status
            )).size(12),
            text(format!(
                "Deflection: L/{:.0} vs L/{:.0} = {:.2} [{}]",
                result.deflection_ratio, result.deflection_limit_ratio, result.deflection_unity, defl_status
            )).size(12),
            vertical_space().height(15),
            // Section properties
            text("Section Properties").size(14),
            text(format!("Section Modulus (S): {:.2} in³", result.section_modulus_in3)).size(12),
            text(format!("Moment of Inertia (I): {:.2} in⁴", result.moment_of_inertia_in4)).size(12),
        ]
    }

    fn view_status_bar(&self) -> Element<'_, Message> {
        row![text(&self.status).size(12),]
            .padding(Padding::from([5, 0]))
            .into()
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

fn labeled_input<'a>(
    label: &'a str,
    value: &'a str,
    on_change: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    row![
        text(label).size(12).width(Length::Fixed(130.0)),
        text_input("", value)
            .on_input(on_change)
            .width(Length::Fill)
            .padding(5),
    ]
    .align_y(Alignment::Center)
    .into()
}

