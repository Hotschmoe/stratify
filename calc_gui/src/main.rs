//! # Stratify GUI Application
//!
//! Full-featured graphical interface for structural engineering calculations.
//! Built with Iced framework for cross-platform support (Windows, macOS, Linux, WASM).

use std::path::PathBuf;

use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable,
    text, text_input, vertical_space, Column,
};
use iced::{Alignment, Element, Font, Length, Padding, Task, Theme};
use uuid::Uuid;

use calc_core::calculations::beam::{calculate, BeamInput, BeamResult};
use calc_core::calculations::CalculationItem;
use calc_core::file_io::{load_project, save_project, FileLock};
use calc_core::materials::{WoodGrade, WoodMaterial, WoodSpecies};
use calc_core::pdf::render_beam_pdf;
use calc_core::project::Project;

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

struct App {
    // Project data
    project: Project,

    // File management
    current_file: Option<PathBuf>,
    is_modified: bool,
    lock_holder: Option<String>, // Who holds the lock if we opened read-only

    // Currently selected/editing beam
    current_beam_id: Option<Uuid>,

    // Beam input fields (for editing)
    beam_label: String,
    span_ft: String,
    load_plf: String,
    width_in: String,
    depth_in: String,
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
            project: Project::new("Engineer", "25-001", "Client"),
            current_file: None,
            is_modified: false,
            lock_holder: None,
            current_beam_id: None,
            beam_label: "B-1".to_string(),
            span_ft: "12.0".to_string(),
            load_plf: "150.0".to_string(),
            width_in: "1.5".to_string(),
            depth_in: "9.25".to_string(),
            selected_species: Some(WoodSpecies::DouglasFirLarch),
            selected_grade: Some(WoodGrade::No2),
            result: None,
            error_message: None,
            status: "Ready - New Project".to_string(),
        }
    }
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    /// Get window title with file name and modified indicator
    fn window_title(&self) -> String {
        let file_name = self.current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let modified = if self.is_modified { " *" } else { "" };
        let read_only = if self.lock_holder.is_some() { " [Read-Only]" } else { "" };

        format!("{}{}{} - Stratify", file_name, modified, read_only)
    }
}

// ============================================================================
// Messages
// ============================================================================

#[derive(Debug, Clone)]
enum Message {
    // File operations
    NewProject,
    OpenProject,
    SaveProject,
    SaveProjectAs,

    // Project info changes
    EngineerNameChanged(String),
    JobIdChanged(String),
    ClientChanged(String),

    // Beam input field changes
    BeamLabelChanged(String),
    SpanChanged(String),
    LoadChanged(String),
    WidthChanged(String),
    DepthChanged(String),

    // Material selection
    SpeciesSelected(WoodSpecies),
    GradeSelected(WoodGrade),

    // Actions
    AddBeamToProject,
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
            // File operations
            Message::NewProject => {
                self.new_project();
            }
            Message::OpenProject => {
                self.open_project();
            }
            Message::SaveProject => {
                self.save_project();
            }
            Message::SaveProjectAs => {
                self.save_project_as();
            }

            // Project info
            Message::EngineerNameChanged(value) => {
                self.project.meta.engineer = value;
                self.mark_modified();
            }
            Message::JobIdChanged(value) => {
                self.project.meta.job_id = value;
                self.mark_modified();
            }
            Message::ClientChanged(value) => {
                self.project.meta.client = value;
                self.mark_modified();
            }

            // Beam fields
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

            // Actions
            Message::AddBeamToProject => {
                self.add_beam_to_project();
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

    fn mark_modified(&mut self) {
        if self.lock_holder.is_none() {
            self.is_modified = true;
        }
    }

    fn new_project(&mut self) {
        // TODO: Check for unsaved changes and prompt
        self.project = Project::new("Engineer", "25-001", "Client");
        self.current_file = None;
        self.is_modified = false;
        self.lock_holder = None;
        self.current_beam_id = None;
        self.result = None;
        self.error_message = None;
        self.status = "New project created".to_string();
    }

    fn open_project(&mut self) {
        // Show file dialog
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Project")
            .add_filter("Stratify Project", &["stf"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            // Try to acquire lock
            let username = whoami::username();
            match FileLock::acquire(&path, &username) {
                Ok(lock) => {
                    // We got the lock, load the project
                    match load_project(&path) {
                        Ok(project) => {
                            // Release lock immediately since we're not keeping it in state
                            // (In a real app, we'd store the lock)
                            drop(lock);

                            self.project = project;
                            self.current_file = Some(path.clone());
                            self.is_modified = false;
                            self.lock_holder = None;
                            self.result = None;
                            self.error_message = None;
                            self.status = format!("Opened: {}", path.display());
                        }
                        Err(e) => {
                            self.status = format!("Failed to open: {}", e);
                        }
                    }
                }
                Err(calc_core::errors::CalcError::FileLocked { locked_by, .. }) => {
                    // File is locked, open read-only
                    match load_project(&path) {
                        Ok(project) => {
                            self.project = project;
                            self.current_file = Some(path.clone());
                            self.is_modified = false;
                            self.lock_holder = Some(locked_by.clone());
                            self.result = None;
                            self.error_message = None;
                            self.status = format!("Opened read-only (locked by {})", locked_by);
                        }
                        Err(e) => {
                            self.status = format!("Failed to open: {}", e);
                        }
                    }
                }
                Err(e) => {
                    self.status = format!("Failed to open: {}", e);
                }
            }
        }
    }

    fn save_project(&mut self) {
        if self.lock_holder.is_some() {
            self.status = "Cannot save: file is read-only".to_string();
            return;
        }

        if let Some(ref path) = self.current_file {
            self.do_save(path.clone());
        } else {
            self.save_project_as();
        }
    }

    fn save_project_as(&mut self) {
        if self.lock_holder.is_some() {
            self.status = "Cannot save: file is read-only".to_string();
            return;
        }

        let default_name = self.current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("project.stf");

        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save Project As")
            .set_file_name(default_name)
            .add_filter("Stratify Project", &["stf"])
            .save_file()
        {
            self.do_save(path);
        }
    }

    fn do_save(&mut self, path: PathBuf) {
        // Update modified timestamp
        self.project.meta.modified = chrono::Utc::now();

        // Acquire lock, save, release
        let username = whoami::username();
        match FileLock::acquire(&path, &username) {
            Ok(lock) => {
                match save_project(&self.project, &path) {
                    Ok(()) => {
                        self.current_file = Some(path.clone());
                        self.is_modified = false;
                        self.status = format!("Saved: {}", path.display());
                    }
                    Err(e) => {
                        self.status = format!("Save failed: {}", e);
                    }
                }
                drop(lock);
            }
            Err(e) => {
                self.status = format!("Cannot save (lock failed): {}", e);
            }
        }
    }

    fn add_beam_to_project(&mut self) {
        if self.lock_holder.is_some() {
            self.status = "Cannot modify: file is read-only".to_string();
            return;
        }

        // Parse and validate
        let span_ft = match self.span_ft.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => {
                self.error_message = Some("Invalid span value".to_string());
                return;
            }
        };
        let load_plf = match self.load_plf.parse::<f64>() {
            Ok(v) => v,
            _ => {
                self.error_message = Some("Invalid load value".to_string());
                return;
            }
        };
        let width_in = match self.width_in.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => {
                self.error_message = Some("Invalid width value".to_string());
                return;
            }
        };
        let depth_in = match self.depth_in.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => {
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

        let beam = BeamInput {
            label: self.beam_label.clone(),
            span_ft,
            uniform_load_plf: load_plf,
            material: WoodMaterial::new(species, grade),
            width_in,
            depth_in,
        };

        let id = self.project.add_item(CalculationItem::Beam(beam));
        self.current_beam_id = Some(id);
        self.mark_modified();
        self.error_message = None;
        self.status = format!("Added beam '{}' to project", self.beam_label);
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
        match render_beam_pdf(&input, result, &self.project.meta.engineer, &self.project.meta.job_id) {
            Ok(pdf_bytes) => {
                // Use file dialog to save
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Save PDF Report")
                    .set_file_name(&format!("{}_beam_report.pdf", self.beam_label))
                    .add_filter("PDF", &["pdf"])
                    .save_file()
                {
                    match std::fs::write(&path, &pdf_bytes) {
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
            // Header with file operations
            self.view_header(),
            horizontal_rule(2),
            // Toolbar
            self.view_toolbar(),
            horizontal_rule(1),
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
        let title = self.window_title();
        row![
            text("Stratify").size(28),
            horizontal_space(),
            text(title).size(14),
        ]
        .align_y(Alignment::Center)
        .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
        let file_buttons = row![
            button("New")
                .on_press(Message::NewProject)
                .padding(Padding::from([6, 12])),
            button("Open")
                .on_press(Message::OpenProject)
                .padding(Padding::from([6, 12])),
            button("Save")
                .on_press(Message::SaveProject)
                .padding(Padding::from([6, 12])),
            button("Save As")
                .on_press(Message::SaveProjectAs)
                .padding(Padding::from([6, 12])),
        ]
        .spacing(8);

        row![
            file_buttons,
            horizontal_space(),
            // Could add more toolbar sections here
        ]
        .padding(Padding::from([8, 0]))
        .into()
    }

    fn view_input_panel(&self) -> Element<'_, Message> {
        let project_section = column![
            text("Project Information").size(16),
            vertical_space().height(10),
            labeled_input("Engineer:", &self.project.meta.engineer, Message::EngineerNameChanged),
            labeled_input("Job ID:", &self.project.meta.job_id, Message::JobIdChanged),
            labeled_input("Client:", &self.project.meta.client, Message::ClientChanged),
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

        let calc_buttons = row![
            button("Calculate")
                .on_press(Message::Calculate)
                .padding(Padding::from([8, 16])),
            button("Add to Project")
                .on_press(Message::AddBeamToProject)
                .padding(Padding::from([8, 16])),
        ]
        .spacing(8);

        let export_buttons = row![
            button("Export PDF")
                .on_press(Message::ExportPdf)
                .padding(Padding::from([8, 16])),
            button("Clear")
                .on_press(Message::ClearResults)
                .padding(Padding::from([8, 16])),
        ]
        .spacing(8);

        let panel = column![
            project_section,
            vertical_space().height(20),
            beam_section,
            vertical_space().height(20),
            material_section,
            vertical_space().height(20),
            calc_buttons,
            vertical_space().height(10),
            export_buttons,
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
            // Show project summary
            self.view_project_summary()
        };

        let panel = container(scrollable(content.padding(10)))
            .width(Length::FillPortion(3))
            .style(container::bordered_box)
            .padding(10);

        panel.into()
    }

    fn view_project_summary(&self) -> Column<'_, Message> {
        let item_count = self.project.item_count();

        let mut items_list = column![
            text("Project Items").size(14),
        ].spacing(4);

        if item_count == 0 {
            items_list = items_list.push(text("No items yet. Add a beam to get started.").size(12));
        } else {
            for (id, item) in &self.project.items {
                let label = format!("{}: {} ({})", item.calc_type(), item.label(), &id.to_string()[..8]);
                items_list = items_list.push(text(label).size(12));
            }
        }

        column![
            text("Project Summary").size(16),
            vertical_space().height(10),
            text(format!("Engineer: {}", self.project.meta.engineer)).size(12),
            text(format!("Job ID: {}", self.project.meta.job_id)).size(12),
            text(format!("Client: {}", self.project.meta.client)).size(12),
            text(format!("Items: {}", item_count)).size(12),
            vertical_space().height(15),
            items_list,
            vertical_space().height(15),
            text("Enter beam parameters and click Calculate").size(12),
        ]
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
        let file_info = match &self.current_file {
            Some(path) => path.display().to_string(),
            None => "Untitled".to_string(),
        };

        let lock_info = match &self.lock_holder {
            Some(holder) => format!(" [Locked by: {}]", holder),
            None => String::new(),
        };

        let modified_indicator = if self.is_modified { " *" } else { "" };

        row![
            text(format!("{}{}", file_info, modified_indicator)).size(11),
            text(lock_info).size(11).color([0.6, 0.3, 0.0]),
            horizontal_space(),
            text(&self.status).size(11),
        ]
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
