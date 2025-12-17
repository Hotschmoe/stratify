//! # Stratify GUI Application
//!
//! Full-featured graphical interface for structural engineering calculations.
//! Built with Iced framework for cross-platform support (Windows, macOS, Linux, WASM).

use std::path::PathBuf;

use iced::keyboard::{self, Key, Modifiers};
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::widget::{
    button, column, container, horizontal_rule, horizontal_space, pick_list, row, scrollable,
    text, text_input, vertical_space, Column,
};
use iced::{
    event, Alignment, Color, Element, Event, Font, Length, Padding, Point, Rectangle, Renderer,
    Subscription, Task, Theme,
};
use uuid::Uuid;

use calc_core::calculations::beam::{calculate, BeamInput, BeamResult};
use calc_core::calculations::CalculationItem;
use calc_core::file_io::{load_project, save_project, FileLock};
use calc_core::materials::{WoodGrade, WoodMaterial, WoodSpecies};
use calc_core::pdf::render_project_pdf;
use calc_core::project::Project;

// Embed BerkeleyMono font at compile time
const BERKELEY_MONO: &[u8] =
    include_bytes!("../../assets/fonts/BerkleyMono/BerkeleyMono-Regular.otf");
const BERKELEY_MONO_BOLD: &[u8] =
    include_bytes!("../../assets/fonts/BerkleyMono/BerkeleyMono-Bold.otf");

fn main() -> iced::Result {
    iced::application("Stratify - Structural Engineering", App::update, App::view)
        .subscription(App::subscription)
        .theme(|_| Theme::Light)
        .window_size((1000.0, 750.0))
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
    file_lock: Option<FileLock>, // Hold the lock while file is open!
    is_modified: bool,
    lock_holder: Option<String>, // Who holds the lock if we opened read-only

    // Currently selected/editing beam
    selected_beam_id: Option<Uuid>,

    // Beam input fields (for editing)
    beam_label: String,
    span_ft: String,
    load_plf: String,
    width_in: String,
    depth_in: String,
    selected_species: Option<WoodSpecies>,
    selected_grade: Option<WoodGrade>,

    // Calculation results (store input too for diagram plotting)
    calc_input: Option<BeamInput>,
    result: Option<BeamResult>,
    error_message: Option<String>,
    diagram_cache: canvas::Cache,

    // Status message
    status: String,
}

impl Default for App {
    fn default() -> Self {
        App {
            project: Project::new("Engineer", "25-001", "Client"),
            current_file: None,
            file_lock: None,
            is_modified: false,
            lock_holder: None,
            selected_beam_id: None,
            beam_label: "B-1".to_string(),
            span_ft: "12.0".to_string(),
            load_plf: "150.0".to_string(),
            width_in: "1.5".to_string(),
            depth_in: "9.25".to_string(),
            selected_species: Some(WoodSpecies::DouglasFirLarch),
            selected_grade: Some(WoodGrade::No2),
            calc_input: None,
            result: None,
            error_message: None,
            diagram_cache: canvas::Cache::default(),
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
        let file_name = self
            .current_file
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled");

        let modified = if self.is_modified { " *" } else { "" };
        let read_only = if self.lock_holder.is_some() {
            " [Read-Only]"
        } else {
            ""
        };

        format!("{}{}{} - Stratify", file_name, modified, read_only)
    }

    /// Check if we can edit (not read-only)
    fn can_edit(&self) -> bool {
        self.lock_holder.is_none()
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

    // Beam selection
    SelectBeam(Uuid),
    DeselectBeam,

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
    AddOrUpdateBeam,
    DeleteSelectedBeam,
    Calculate,
    ExportPdf,
    ClearResults,

    // Keyboard events
    KeyPressed(Key, Modifiers),
}

// ============================================================================
// Subscriptions (for keyboard shortcuts)
// ============================================================================

impl App {
    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                Some(Message::KeyPressed(key, modifiers))
            }
            _ => None,
        })
    }
}

// ============================================================================
// Update Logic
// ============================================================================

impl App {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Keyboard shortcuts
            Message::KeyPressed(key, modifiers) => {
                if modifiers.control() {
                    match key.as_ref() {
                        Key::Character("s") => {
                            if modifiers.shift() {
                                self.save_project_as();
                            } else {
                                self.save_project();
                            }
                        }
                        Key::Character("o") => {
                            self.open_project();
                        }
                        Key::Character("n") => {
                            self.new_project();
                        }
                        _ => {}
                    }
                }
            }

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
                if self.can_edit() {
                    self.project.meta.engineer = value;
                    self.mark_modified();
                }
            }
            Message::JobIdChanged(value) => {
                if self.can_edit() {
                    self.project.meta.job_id = value;
                    self.mark_modified();
                }
            }
            Message::ClientChanged(value) => {
                if self.can_edit() {
                    self.project.meta.client = value;
                    self.mark_modified();
                }
            }

            // Beam selection
            Message::SelectBeam(id) => {
                self.select_beam(id);
            }
            Message::DeselectBeam => {
                self.deselect_beam();
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
            Message::AddOrUpdateBeam => {
                self.add_or_update_beam();
            }
            Message::DeleteSelectedBeam => {
                self.delete_selected_beam();
            }
            Message::Calculate => {
                self.run_calculation();
            }
            Message::ExportPdf => {
                self.export_pdf();
            }
            Message::ClearResults => {
                self.calc_input = None;
                self.result = None;
                self.error_message = None;
                self.diagram_cache.clear();
                self.status = "Results cleared".to_string();
            }
        }
        Task::none()
    }

    fn mark_modified(&mut self) {
        if self.can_edit() {
            self.is_modified = true;
        }
    }

    fn new_project(&mut self) {
        // Release any existing lock
        self.file_lock = None;

        // TODO: Check for unsaved changes and prompt
        self.project = Project::new("Engineer", "25-001", "Client");
        self.current_file = None;
        self.is_modified = false;
        self.lock_holder = None;
        self.selected_beam_id = None;
        self.result = None;
        self.error_message = None;
        self.deselect_beam();
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
            // Release any existing lock first
            self.file_lock = None;

            // Try to acquire lock
            let username = whoami::username();
            match FileLock::acquire(&path, &username) {
                Ok(lock) => {
                    // We got the lock, load the project
                    match load_project(&path) {
                        Ok(project) => {
                            self.project = project;
                            self.current_file = Some(path.clone());
                            self.file_lock = Some(lock); // Keep the lock!
                            self.is_modified = false;
                            self.lock_holder = None;
                            self.selected_beam_id = None;
                            self.result = None;
                            self.error_message = None;
                            self.deselect_beam();
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
                            self.file_lock = None; // No lock in read-only mode
                            self.is_modified = false;
                            self.lock_holder = Some(locked_by.clone());
                            self.selected_beam_id = None;
                            self.result = None;
                            self.error_message = None;
                            self.deselect_beam();
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
        if !self.can_edit() {
            self.status = "Cannot save: file is read-only".to_string();
            return;
        }

        if let Some(ref path) = self.current_file.clone() {
            self.do_save(path.clone());
        } else {
            self.save_project_as();
        }
    }

    fn save_project_as(&mut self) {
        if !self.can_edit() {
            self.status = "Cannot save: file is read-only".to_string();
            return;
        }

        let default_name = self
            .current_file
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
            // Release old lock if saving to new location
            if self.current_file.as_ref() != Some(&path) {
                self.file_lock = None;
            }
            self.do_save(path);
        }
    }

    fn do_save(&mut self, path: PathBuf) {
        // Update modified timestamp
        self.project.meta.modified = chrono::Utc::now();

        // If we don't have a lock for this path, acquire one
        let need_new_lock = match &self.file_lock {
            Some(lock) => lock.info.user_id != whoami::username(), // Lock exists but not ours
            None => true,
        };

        if need_new_lock {
            let username = whoami::username();
            match FileLock::acquire(&path, &username) {
                Ok(lock) => {
                    self.file_lock = Some(lock);
                }
                Err(e) => {
                    self.status = format!("Cannot save (lock failed): {}", e);
                    return;
                }
            }
        }

        // Now save
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
    }

    fn select_beam(&mut self, id: Uuid) {
        if let Some(item) = self.project.items.get(&id) {
            if let CalculationItem::Beam(beam) = item {
                self.selected_beam_id = Some(id);
                self.beam_label = beam.label.clone();
                self.span_ft = beam.span_ft.to_string();
                self.load_plf = beam.uniform_load_plf.to_string();
                self.width_in = beam.width_in.to_string();
                self.depth_in = beam.depth_in.to_string();
                self.selected_species = Some(beam.material.species);
                self.selected_grade = Some(beam.material.grade);
                self.result = None;
                self.error_message = None;
                self.status = format!("Selected: {}", beam.label);
            }
        }
    }

    fn deselect_beam(&mut self) {
        self.selected_beam_id = None;
        self.beam_label = "B-1".to_string();
        self.span_ft = "12.0".to_string();
        self.load_plf = "150.0".to_string();
        self.width_in = "1.5".to_string();
        self.depth_in = "9.25".to_string();
        self.selected_species = Some(WoodSpecies::DouglasFirLarch);
        self.selected_grade = Some(WoodGrade::No2);
    }

    fn add_or_update_beam(&mut self) {
        if !self.can_edit() {
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

        if let Some(id) = self.selected_beam_id {
            // Update existing beam
            self.project.items.insert(id, CalculationItem::Beam(beam));
            self.mark_modified();
            self.error_message = None;
            self.status = format!("Updated beam '{}'", self.beam_label);
        } else {
            // Add new beam
            let id = self.project.add_item(CalculationItem::Beam(beam));
            self.selected_beam_id = Some(id);
            self.mark_modified();
            self.error_message = None;
            self.status = format!("Added beam '{}'", self.beam_label);
        }
    }

    fn delete_selected_beam(&mut self) {
        if !self.can_edit() {
            self.status = "Cannot modify: file is read-only".to_string();
            return;
        }

        if let Some(id) = self.selected_beam_id {
            if let Some(item) = self.project.items.remove(&id) {
                self.mark_modified();
                self.status = format!("Deleted: {}", item.label());
                self.deselect_beam();
            }
        } else {
            self.status = "No beam selected to delete".to_string();
        }
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
                self.calc_input = Some(input);
                self.result = Some(result);
                self.diagram_cache.clear(); // Redraw diagrams
            }
            Err(e) => {
                self.error_message = Some(format!("Calculation error: {}", e));
                self.calc_input = None;
                self.result = None;
            }
        }
    }

    fn export_pdf(&mut self) {
        if self.project.items.is_empty() {
            self.status = "No beams in project to export".to_string();
            return;
        }

        // Generate PDF for all beams in project
        match render_project_pdf(&self.project) {
            Ok(pdf_bytes) => {
                // Default filename based on job ID or project name
                let default_name = format!("{}_calculations.pdf", self.project.meta.job_id);

                // Use file dialog to save
                if let Some(path) = rfd::FileDialog::new()
                    .set_title("Export PDF Report")
                    .set_file_name(&default_name)
                    .add_filter("PDF", &["pdf"])
                    .save_file()
                {
                    match std::fs::write(&path, &pdf_bytes) {
                        Ok(_) => {
                            let beam_count = self
                                .project
                                .items
                                .values()
                                .filter(|i| matches!(i, CalculationItem::Beam(_)))
                                .count();
                            self.status = format!(
                                "PDF exported: {} ({} beams)",
                                path.display(),
                                beam_count
                            );
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
            // Left panel - Project items list
            self.view_items_panel(),
            // Middle panel - Input form
            self.view_input_panel(),
            // Right panel - Results
            self.view_results_panel(),
        ]
        .spacing(10);

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
        .padding(15);

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
            button("New (Ctrl+N)")
                .on_press(Message::NewProject)
                .padding(Padding::from([6, 10])),
            button("Open (Ctrl+O)")
                .on_press(Message::OpenProject)
                .padding(Padding::from([6, 10])),
            button("Save (Ctrl+S)")
                .on_press(Message::SaveProject)
                .padding(Padding::from([6, 10])),
            button("Save As")
                .on_press(Message::SaveProjectAs)
                .padding(Padding::from([6, 10])),
            button("Export PDF")
                .on_press(Message::ExportPdf)
                .padding(Padding::from([6, 10])),
        ]
        .spacing(6);

        row![file_buttons, horizontal_space(),]
            .padding(Padding::from([8, 0]))
            .into()
    }

    fn view_items_panel(&self) -> Element<'_, Message> {
        let mut items_list = column![text("Project Items").size(16), vertical_space().height(10),]
            .spacing(4);

        if self.project.items.is_empty() {
            items_list = items_list.push(text("No items yet").size(11));
        } else {
            for (id, item) in &self.project.items {
                let is_selected = self.selected_beam_id == Some(*id);
                let label = format!("{}: {}", item.calc_type(), item.label());

                let btn = if is_selected {
                    button(text(label).size(11))
                        .on_press(Message::DeselectBeam)
                        .padding(Padding::from([4, 8]))
                        .style(button::primary)
                        .width(Length::Fill)
                } else {
                    button(text(label).size(11))
                        .on_press(Message::SelectBeam(*id))
                        .padding(Padding::from([4, 8]))
                        .style(button::secondary)
                        .width(Length::Fill)
                };

                items_list = items_list.push(btn);
            }
        }

        // New item button
        items_list = items_list.push(vertical_space().height(10));
        items_list = items_list.push(
            button(text("+ New Beam").size(11))
                .on_press(Message::DeselectBeam)
                .padding(Padding::from([4, 8]))
                .width(Length::Fill),
        );

        let panel = container(scrollable(items_list.padding(8)))
            .width(Length::Fixed(160.0))
            .height(Length::Fill)
            .style(container::bordered_box)
            .padding(5);

        panel.into()
    }

    fn view_input_panel(&self) -> Element<'_, Message> {
        let project_section = column![
            text("Project Information").size(14),
            vertical_space().height(8),
            labeled_input(
                "Engineer:",
                &self.project.meta.engineer,
                Message::EngineerNameChanged
            ),
            labeled_input("Job ID:", &self.project.meta.job_id, Message::JobIdChanged),
            labeled_input("Client:", &self.project.meta.client, Message::ClientChanged),
        ]
        .spacing(6);

        let editing_label = if self.selected_beam_id.is_some() {
            "Edit Beam"
        } else {
            "New Beam"
        };

        let beam_section = column![
            text(editing_label).size(14),
            vertical_space().height(8),
            labeled_input("Label:", &self.beam_label, Message::BeamLabelChanged),
            labeled_input("Span (ft):", &self.span_ft, Message::SpanChanged),
            labeled_input("Load (plf):", &self.load_plf, Message::LoadChanged),
            labeled_input("Width (in):", &self.width_in, Message::WidthChanged),
            labeled_input("Depth (in):", &self.depth_in, Message::DepthChanged),
        ]
        .spacing(6);

        let material_section = column![
            text("Material").size(14),
            vertical_space().height(8),
            text("Species:").size(11),
            pick_list(
                &WoodSpecies::ALL[..],
                self.selected_species,
                Message::SpeciesSelected
            )
            .width(Length::Fill)
            .placeholder("Select..."),
            vertical_space().height(4),
            text("Grade:").size(11),
            pick_list(
                &WoodGrade::ALL[..],
                self.selected_grade,
                Message::GradeSelected
            )
            .width(Length::Fill)
            .placeholder("Select..."),
        ]
        .spacing(2);

        let save_btn_text = if self.selected_beam_id.is_some() {
            "Update Beam"
        } else {
            "Add Beam"
        };

        let mut action_buttons = row![
            button(save_btn_text)
                .on_press(Message::AddOrUpdateBeam)
                .padding(Padding::from([6, 12])),
            button("Calculate")
                .on_press(Message::Calculate)
                .padding(Padding::from([6, 12])),
        ]
        .spacing(6);

        if self.selected_beam_id.is_some() {
            action_buttons = action_buttons.push(
                button("Delete")
                    .on_press(Message::DeleteSelectedBeam)
                    .padding(Padding::from([6, 12])),
            );
        }

        action_buttons = action_buttons.push(
            button("Clear Results")
                .on_press(Message::ClearResults)
                .padding(Padding::from([6, 12])),
        );

        let panel = column![
            project_section,
            vertical_space().height(15),
            beam_section,
            vertical_space().height(15),
            material_section,
            vertical_space().height(15),
            action_buttons,
        ]
        .width(Length::FillPortion(2))
        .padding(8);

        container(scrollable(panel))
            .style(container::bordered_box)
            .padding(5)
            .into()
    }

    fn view_results_panel(&self) -> Element<'_, Message> {
        let content: Column<'_, Message> = if let Some(ref error) = self.error_message {
            column![
                text("Error").size(14),
                vertical_space().height(8),
                text(error).size(12).color([0.8, 0.2, 0.2]),
            ]
        } else if let (Some(ref input), Some(ref result)) = (&self.calc_input, &self.result) {
            let results_text = self.view_calculation_results(result);
            let diagram_data = BeamDiagramData::from_calc(input, result);
            let diagram = BeamDiagram::new(diagram_data);

            let canvas_widget: Element<'_, Message> = Canvas::new(diagram)
                .width(Length::Fill)
                .height(Length::Fixed(340.0))
                .into();

            results_text
                .push(vertical_space().height(15))
                .push(text("Diagrams").size(14))
                .push(vertical_space().height(8))
                .push(canvas_widget)
        } else {
            self.view_project_summary()
        };

        let panel = container(scrollable(content.padding(8)))
            .width(Length::FillPortion(3))
            .style(container::bordered_box)
            .padding(5);

        panel.into()
    }

    fn view_project_summary(&self) -> Column<'_, Message> {
        let item_count = self.project.item_count();

        column![
            text("Project Summary").size(14),
            vertical_space().height(8),
            text(format!("Engineer: {}", self.project.meta.engineer)).size(11),
            text(format!("Job ID: {}", self.project.meta.job_id)).size(11),
            text(format!("Client: {}", self.project.meta.client)).size(11),
            text(format!("Items: {}", item_count)).size(11),
            vertical_space().height(15),
            text("Select an item from the list or create a new beam.").size(11),
            vertical_space().height(8),
            text("Keyboard shortcuts:").size(11),
            text("  Ctrl+N: New project").size(10),
            text("  Ctrl+O: Open project").size(10),
            text("  Ctrl+S: Save project").size(10),
            text("  Ctrl+Shift+S: Save as").size(10),
        ]
    }

    fn view_calculation_results(&self, result: &BeamResult) -> Column<'_, Message> {
        let pass_fail = if result.passes() {
            text("DESIGN ADEQUATE").size(16).color([0.2, 0.6, 0.2])
        } else {
            text("DESIGN INADEQUATE").size(16).color([0.8, 0.2, 0.2])
        };

        let governing = text(format!("Governing: {}", result.governing_condition())).size(11);

        let bending_status = if result.bending_unity <= 1.0 {
            "OK"
        } else {
            "FAIL"
        };
        let shear_status = if result.shear_unity <= 1.0 {
            "OK"
        } else {
            "FAIL"
        };
        let defl_status = if result.deflection_unity <= 1.0 {
            "OK"
        } else {
            "FAIL"
        };

        column![
            text("Calculation Results").size(14),
            vertical_space().height(8),
            pass_fail,
            governing,
            vertical_space().height(12),
            text("Demand").size(12),
            text(format!("Max Moment: {:.0} ft-lb", result.max_moment_ftlb)).size(11),
            text(format!("Max Shear: {:.0} lb", result.max_shear_lb)).size(11),
            text(format!(
                "Max Deflection: {:.3} in",
                result.max_deflection_in
            ))
            .size(11),
            vertical_space().height(12),
            text("Capacity Checks").size(12),
            text(format!(
                "Bending: {:.0}/{:.0} psi = {:.2} [{}]",
                result.actual_fb_psi, result.allowable_fb_psi, result.bending_unity, bending_status
            ))
            .size(11),
            text(format!(
                "Shear: {:.0}/{:.0} psi = {:.2} [{}]",
                result.actual_fv_psi, result.allowable_fv_psi, result.shear_unity, shear_status
            ))
            .size(11),
            text(format!(
                "Deflection: L/{:.0} vs L/{:.0} = {:.2} [{}]",
                result.deflection_ratio,
                result.deflection_limit_ratio,
                result.deflection_unity,
                defl_status
            ))
            .size(11),
            vertical_space().height(12),
            text("Section Properties").size(12),
            text(format!(
                "Section Modulus (S): {:.2} in³",
                result.section_modulus_in3
            ))
            .size(11),
            text(format!(
                "Moment of Inertia (I): {:.2} in⁴",
                result.moment_of_inertia_in4
            ))
            .size(11),
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
            text(format!("{}{}", file_info, modified_indicator)).size(10),
            text(lock_info).size(10).color([0.6, 0.3, 0.0]),
            horizontal_space(),
            text(&self.status).size(10),
        ]
        .padding(Padding::from([4, 0]))
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
        text(label).size(11).width(Length::Fixed(80.0)),
        text_input("", value)
            .on_input(on_change)
            .width(Length::Fill)
            .padding(4)
            .size(11),
    ]
    .align_y(Alignment::Center)
    .into()
}

// ============================================================================
// Beam Diagram Rendering
// ============================================================================

/// Data needed to draw beam diagrams
struct BeamDiagramData {
    span_ft: f64,
    load_plf: f64,
    max_shear_lb: f64,
    max_moment_ftlb: f64,
    max_deflection_in: f64,
}

impl BeamDiagramData {
    fn from_calc(input: &BeamInput, result: &BeamResult) -> Self {
        Self {
            span_ft: input.span_ft,
            load_plf: input.uniform_load_plf,
            max_shear_lb: result.max_shear_lb,
            max_moment_ftlb: result.max_moment_ftlb,
            max_deflection_in: result.max_deflection_in,
        }
    }

    /// Shear at position x (0 to L): V(x) = w(L/2 - x)
    #[allow(dead_code)]
    fn shear_at(&self, x_ratio: f64) -> f64 {
        let x = x_ratio * self.span_ft;
        self.load_plf * (self.span_ft / 2.0 - x)
    }

    /// Moment at position x (0 to L): M(x) = w*x*(L-x)/2
    fn moment_at(&self, x_ratio: f64) -> f64 {
        let x = x_ratio * self.span_ft;
        self.load_plf * x * (self.span_ft - x) / 2.0
    }

    /// Deflection ratio at position x (normalized 0-1, max at center)
    /// δ(x) = wx(L³ - 2Lx² + x³)/(24EI) - simplified to shape
    fn deflection_ratio_at(&self, x_ratio: f64) -> f64 {
        // Normalized deflection shape for uniform load: x(1-x)(1+x-x²)
        // Simplified: use polynomial that peaks at center
        let x = x_ratio;
        // Shape: 16x(1-x)(1-x+x) normalized to peak=1 at x=0.5
        // Actual shape: x - 2x³ + x⁴ normalized
        let shape = x * (1.0 - x) * (1.0 + x - x * x);
        // Normalize so max = 1 (occurs at x=0.5)
        shape / 0.3125 // 0.5 * 0.5 * 1.25 = 0.3125
    }
}

/// Canvas program for drawing beam diagrams
struct BeamDiagram {
    data: BeamDiagramData,
}

impl canvas::Program<Message> for BeamDiagram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        let width = bounds.width;
        let height = bounds.height;

        // Layout: divide into 4 sections vertically
        let section_height = height / 4.0;
        let margin = 20.0;
        let plot_width = width - 2.0 * margin;

        // Colors
        let beam_color = Color::from_rgb(0.3, 0.3, 0.3);
        let shear_color = Color::from_rgb(0.2, 0.5, 0.8);
        let moment_color = Color::from_rgb(0.8, 0.4, 0.2);
        let defl_color = Color::from_rgb(0.2, 0.7, 0.3);
        let axis_color = Color::from_rgb(0.5, 0.5, 0.5);

        // Section 1: Beam schematic with uniform load
        self.draw_beam_schematic(&mut frame, margin, 0.0, plot_width, section_height, beam_color);

        // Section 2: Shear diagram
        self.draw_shear_diagram(
            &mut frame,
            margin,
            section_height,
            plot_width,
            section_height,
            shear_color,
            axis_color,
        );

        // Section 3: Moment diagram
        self.draw_moment_diagram(
            &mut frame,
            margin,
            section_height * 2.0,
            plot_width,
            section_height,
            moment_color,
            axis_color,
        );

        // Section 4: Deflection diagram
        self.draw_deflection_diagram(
            &mut frame,
            margin,
            section_height * 3.0,
            plot_width,
            section_height,
            defl_color,
            axis_color,
        );

        vec![frame.into_geometry()]
    }
}

impl BeamDiagram {
    fn new(data: BeamDiagramData) -> Self {
        Self { data }
    }

    fn draw_beam_schematic(
        &self,
        frame: &mut Frame,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
    ) {
        let beam_y = y + height * 0.6;
        let beam_thickness = 4.0;

        // Draw beam line
        let beam = Path::line(
            Point::new(x, beam_y),
            Point::new(x + width, beam_y),
        );
        frame.stroke(&beam, Stroke::default().with_color(color).with_width(beam_thickness));

        // Draw pin support (left) - triangle
        let support_size = 12.0;
        let left_support = Path::new(|builder| {
            builder.move_to(Point::new(x, beam_y + beam_thickness / 2.0));
            builder.line_to(Point::new(x - support_size / 2.0, beam_y + support_size));
            builder.line_to(Point::new(x + support_size / 2.0, beam_y + support_size));
            builder.close();
        });
        frame.stroke(&left_support, Stroke::default().with_color(color).with_width(2.0));

        // Draw roller support (right) - triangle with circle
        let right_support = Path::new(|builder| {
            builder.move_to(Point::new(x + width, beam_y + beam_thickness / 2.0));
            builder.line_to(Point::new(x + width - support_size / 2.0, beam_y + support_size));
            builder.line_to(Point::new(x + width + support_size / 2.0, beam_y + support_size));
            builder.close();
        });
        frame.stroke(&right_support, Stroke::default().with_color(color).with_width(2.0));

        // Draw uniform load arrows
        let num_arrows = 8;
        let arrow_spacing = width / (num_arrows as f32);
        let arrow_length = height * 0.25;

        for i in 0..=num_arrows {
            let ax = x + i as f32 * arrow_spacing;
            let arrow = Path::line(
                Point::new(ax, beam_y - arrow_length),
                Point::new(ax, beam_y - 5.0),
            );
            frame.stroke(&arrow, Stroke::default().with_color(color).with_width(1.5));

            // Arrow head
            let head = Path::new(|builder| {
                builder.move_to(Point::new(ax, beam_y - 5.0));
                builder.line_to(Point::new(ax - 3.0, beam_y - 10.0));
                builder.move_to(Point::new(ax, beam_y - 5.0));
                builder.line_to(Point::new(ax + 3.0, beam_y - 10.0));
            });
            frame.stroke(&head, Stroke::default().with_color(color).with_width(1.5));
        }

        // Load label
        let load_text = Text {
            content: format!("w = {:.0} plf", self.data.load_plf),
            position: Point::new(x + width / 2.0, y + 8.0),
            color,
            size: iced::Pixels(10.0),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..Text::default()
        };
        frame.fill_text(load_text);

        // Span label
        let span_text = Text {
            content: format!("L = {:.1} ft", self.data.span_ft),
            position: Point::new(x + width / 2.0, beam_y + support_size + 8.0),
            color,
            size: iced::Pixels(10.0),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..Text::default()
        };
        frame.fill_text(span_text);
    }

    fn draw_shear_diagram(
        &self,
        frame: &mut Frame,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        axis_color: Color,
    ) {
        let center_y = y + height / 2.0;
        let plot_height = height * 0.35;

        // Axis line
        let axis = Path::line(
            Point::new(x, center_y),
            Point::new(x + width, center_y),
        );
        frame.stroke(&axis, Stroke::default().with_color(axis_color).with_width(1.0));

        // Shear diagram: linear from +V to -V
        let shear_path = Path::new(|builder| {
            builder.move_to(Point::new(x, center_y - plot_height));
            builder.line_to(Point::new(x + width, center_y + plot_height));
            builder.line_to(Point::new(x + width, center_y));
            builder.line_to(Point::new(x, center_y));
            builder.close();
        });
        frame.fill(&shear_path, Color { a: 0.3, ..color });

        let shear_line = Path::line(
            Point::new(x, center_y - plot_height),
            Point::new(x + width, center_y + plot_height),
        );
        frame.stroke(&shear_line, Stroke::default().with_color(color).with_width(2.0));

        // Labels
        let title = Text {
            content: "Shear (V)".to_string(),
            position: Point::new(x + 5.0, y + 5.0),
            color,
            size: iced::Pixels(10.0),
            ..Text::default()
        };
        frame.fill_text(title);

        let max_label = Text {
            content: format!("+{:.0} lb", self.data.max_shear_lb),
            position: Point::new(x + 5.0, center_y - plot_height - 2.0),
            color,
            size: iced::Pixels(9.0),
            ..Text::default()
        };
        frame.fill_text(max_label);

        let min_label = Text {
            content: format!("-{:.0} lb", self.data.max_shear_lb),
            position: Point::new(x + width - 50.0, center_y + plot_height + 10.0),
            color,
            size: iced::Pixels(9.0),
            ..Text::default()
        };
        frame.fill_text(min_label);
    }

    fn draw_moment_diagram(
        &self,
        frame: &mut Frame,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        axis_color: Color,
    ) {
        let axis_y = y + height * 0.15;
        let plot_height = height * 0.7;

        // Axis line
        let axis = Path::line(
            Point::new(x, axis_y),
            Point::new(x + width, axis_y),
        );
        frame.stroke(&axis, Stroke::default().with_color(axis_color).with_width(1.0));

        // Moment diagram: parabola (positive moment = sag below axis in convention)
        let num_points = 50;
        let moment_path = Path::new(|builder| {
            builder.move_to(Point::new(x, axis_y));
            for i in 0..=num_points {
                let t = i as f64 / num_points as f64;
                let m_ratio = self.data.moment_at(t) / self.data.max_moment_ftlb;
                let px = x + (t as f32) * width;
                let py = axis_y + (m_ratio as f32) * plot_height;
                if i == 0 {
                    builder.move_to(Point::new(px, py));
                } else {
                    builder.line_to(Point::new(px, py));
                }
            }
            // Close back to axis
            builder.line_to(Point::new(x + width, axis_y));
            builder.line_to(Point::new(x, axis_y));
            builder.close();
        });
        frame.fill(&moment_path, Color { a: 0.3, ..color });

        // Parabola outline
        let outline = Path::new(|builder| {
            for i in 0..=num_points {
                let t = i as f64 / num_points as f64;
                let m_ratio = self.data.moment_at(t) / self.data.max_moment_ftlb;
                let px = x + (t as f32) * width;
                let py = axis_y + (m_ratio as f32) * plot_height;
                if i == 0 {
                    builder.move_to(Point::new(px, py));
                } else {
                    builder.line_to(Point::new(px, py));
                }
            }
        });
        frame.stroke(&outline, Stroke::default().with_color(color).with_width(2.0));

        // Labels
        let title = Text {
            content: "Moment (M)".to_string(),
            position: Point::new(x + 5.0, y + 5.0),
            color,
            size: iced::Pixels(10.0),
            ..Text::default()
        };
        frame.fill_text(title);

        let max_label = Text {
            content: format!("{:.0} ft-lb", self.data.max_moment_ftlb),
            position: Point::new(x + width / 2.0, axis_y + plot_height + 10.0),
            color,
            size: iced::Pixels(9.0),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..Text::default()
        };
        frame.fill_text(max_label);
    }

    fn draw_deflection_diagram(
        &self,
        frame: &mut Frame,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Color,
        axis_color: Color,
    ) {
        let axis_y = y + height * 0.15;
        let plot_height = height * 0.6;

        // Axis line (represents undeflected beam)
        let axis = Path::line(
            Point::new(x, axis_y),
            Point::new(x + width, axis_y),
        );
        frame.stroke(&axis, Stroke::default().with_color(axis_color).with_width(1.0));

        // Deflection curve
        let num_points = 50;
        let defl_path = Path::new(|builder| {
            for i in 0..=num_points {
                let t = i as f64 / num_points as f64;
                let d_ratio = self.data.deflection_ratio_at(t);
                let px = x + (t as f32) * width;
                let py = axis_y + (d_ratio as f32) * plot_height;
                if i == 0 {
                    builder.move_to(Point::new(px, py));
                } else {
                    builder.line_to(Point::new(px, py));
                }
            }
        });
        frame.stroke(&defl_path, Stroke::default().with_color(color).with_width(2.0));

        // Fill under curve
        let fill_path = Path::new(|builder| {
            builder.move_to(Point::new(x, axis_y));
            for i in 0..=num_points {
                let t = i as f64 / num_points as f64;
                let d_ratio = self.data.deflection_ratio_at(t);
                let px = x + (t as f32) * width;
                let py = axis_y + (d_ratio as f32) * plot_height;
                builder.line_to(Point::new(px, py));
            }
            builder.line_to(Point::new(x + width, axis_y));
            builder.close();
        });
        frame.fill(&fill_path, Color { a: 0.2, ..color });

        // Labels
        let title = Text {
            content: "Deflection (δ)".to_string(),
            position: Point::new(x + 5.0, y + 5.0),
            color,
            size: iced::Pixels(10.0),
            ..Text::default()
        };
        frame.fill_text(title);

        let max_label = Text {
            content: format!("{:.3} in", self.data.max_deflection_in),
            position: Point::new(x + width / 2.0, axis_y + plot_height + 10.0),
            color,
            size: iced::Pixels(9.0),
            horizontal_alignment: iced::alignment::Horizontal::Center,
            ..Text::default()
        };
        frame.fill_text(max_label);
    }
}
