//! # Stratify GUI Application
//!
//! Full-featured graphical interface for structural engineering calculations.
//! Built with Iced framework for cross-platform support (Windows, macOS, Linux, WASM).

use std::collections::HashSet;
use std::path::PathBuf;

use iced::keyboard::{self, Key, Modifiers};
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::widget::{
    button, checkbox, column, container, horizontal_rule, horizontal_space, pick_list, row,
    scrollable, text, text_input, vertical_space, Column, Row,
};
use iced::{
    event, Alignment, Color, Element, Event, Font, Length, Padding, Point, Rectangle, Renderer,
    Subscription, Task, Theme,
};
use uuid::Uuid;

use calc_core::calculations::beam::{calculate, BeamInput, BeamResult};
use calc_core::calculations::CalculationItem;
use calc_core::file_io::{load_project, save_project, FileLock};
use calc_core::loads::{DiscreteLoad, EnhancedLoadCase, LoadDistribution, LoadType};
use calc_core::materials::{
    GlulamLayup, GlulamMaterial, GlulamStressClass, LvlGrade, LvlMaterial, Material, PslGrade,
    PslMaterial, WoodGrade, WoodMaterial, WoodSpecies,
};
use calc_core::pdf::render_project_pdf;
use calc_core::project::Project;

// ============================================================================
// UI Types
// ============================================================================

/// Material type for UI selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaterialType {
    #[default]
    SawnLumber,
    Glulam,
    Lvl,
    Psl,
}

impl MaterialType {
    pub const ALL: [MaterialType; 4] = [
        MaterialType::SawnLumber,
        MaterialType::Glulam,
        MaterialType::Lvl,
        MaterialType::Psl,
    ];
}

impl std::fmt::Display for MaterialType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaterialType::SawnLumber => write!(f, "Sawn Lumber"),
            MaterialType::Glulam => write!(f, "Glulam"),
            MaterialType::Lvl => write!(f, "LVL"),
            MaterialType::Psl => write!(f, "PSL"),
        }
    }
}

/// Distribution type for UI dropdown
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DistributionType {
    #[default]
    UniformFull,
    Point,
    UniformPartial,
}

impl DistributionType {
    pub const ALL: [DistributionType; 3] = [
        DistributionType::UniformFull,
        DistributionType::Point,
        DistributionType::UniformPartial,
    ];
}

impl std::fmt::Display for DistributionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DistributionType::UniformFull => write!(f, "Uniform"),
            DistributionType::Point => write!(f, "Point"),
            DistributionType::UniformPartial => write!(f, "Partial"),
        }
    }
}

/// Left panel section identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemSection {
    ProjectInfo,
    WoodBeams,
    WoodColumns,
    // Future sections (disabled in UI)
    ContinuousFootings,
    SpreadFootings,
    CantileverWalls,
    RestrainedWalls,
}

/// What is currently being edited in the middle panel
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorSelection {
    /// No selection - show welcome/instructions
    None,
    /// Editing project info
    ProjectInfo,
    /// Editing a beam (existing or new)
    Beam(Option<Uuid>), // None = new beam, Some(id) = existing beam
}

/// A row in the load table (editable UI state)
#[derive(Debug, Clone)]
pub struct LoadTableRow {
    pub id: Uuid,
    pub load_type: LoadType,
    pub distribution: DistributionType,
    pub magnitude: String,
    pub position: String,      // For point loads: position in ft
    pub tributary_width: String, // Optional tributary width
}

impl LoadTableRow {
    fn new() -> Self {
        Self {
            id: Uuid::new_v4(),
            load_type: LoadType::Dead,
            distribution: DistributionType::UniformFull,
            magnitude: "0.0".to_string(),
            position: "".to_string(),
            tributary_width: "".to_string(),
        }
    }

    /// Convert to DiscreteLoad
    fn to_discrete_load(&self) -> Option<DiscreteLoad> {
        let magnitude: f64 = self.magnitude.parse().ok()?;
        let tributary: Option<f64> = if self.tributary_width.is_empty() {
            None
        } else {
            Some(self.tributary_width.parse().ok()?)
        };

        let mut load = match self.distribution {
            DistributionType::UniformFull => {
                DiscreteLoad::uniform(self.load_type, magnitude)
            }
            DistributionType::Point => {
                let pos: f64 = self.position.parse().ok()?;
                DiscreteLoad::point(self.load_type, magnitude, pos)
            }
            DistributionType::UniformPartial => {
                // For partial uniform, position field is "start-end" format
                // For simplicity, treat as full span for now
                DiscreteLoad::uniform(self.load_type, magnitude)
            }
        };

        if let Some(tw) = tributary {
            load = load.with_tributary_width(tw);
        }

        Some(load)
    }

    /// Create from DiscreteLoad
    fn from_discrete_load(load: &DiscreteLoad) -> Self {
        let (distribution, position) = match &load.distribution {
            LoadDistribution::UniformFull => (DistributionType::UniformFull, String::new()),
            LoadDistribution::Point { position_ft } => {
                (DistributionType::Point, position_ft.to_string())
            }
            LoadDistribution::UniformPartial { start_ft, end_ft } => {
                (DistributionType::UniformPartial, format!("{}-{}", start_ft, end_ft))
            }
            _ => (DistributionType::UniformFull, String::new()),
        };

        Self {
            id: load.id,
            load_type: load.load_type,
            distribution,
            magnitude: load.magnitude.to_string(),
            position,
            tributary_width: load.tributary_width_ft
                .map(|t| t.to_string())
                .unwrap_or_default(),
        }
    }
}

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

    // Left panel state - collapsed sections
    collapsed_sections: HashSet<ItemSection>,

    // Current editor selection
    selection: EditorSelection,

    // Beam input fields (for editing)
    beam_label: String,
    span_ft: String,
    width_in: String,
    depth_in: String,

    // Load table for multiple discrete loads
    load_table: Vec<LoadTableRow>,
    include_self_weight: bool,

    // Material selection
    selected_material_type: MaterialType,
    // Sawn lumber
    selected_species: Option<WoodSpecies>,
    selected_grade: Option<WoodGrade>,
    // Glulam
    selected_glulam_class: Option<GlulamStressClass>,
    selected_glulam_layup: Option<GlulamLayup>,
    // LVL
    selected_lvl_grade: Option<LvlGrade>,
    // PSL
    selected_psl_grade: Option<PslGrade>,

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
        // Create default load table with typical D+L loads
        let default_loads = vec![
            LoadTableRow {
                id: Uuid::new_v4(),
                load_type: LoadType::Dead,
                distribution: DistributionType::UniformFull,
                magnitude: "15.0".to_string(),
                position: String::new(),
                tributary_width: String::new(),
            },
            LoadTableRow {
                id: Uuid::new_v4(),
                load_type: LoadType::Live,
                distribution: DistributionType::UniformFull,
                magnitude: "40.0".to_string(),
                position: String::new(),
                tributary_width: String::new(),
            },
        ];

        App {
            project: Project::new("Engineer", "25-001", "Client"),
            current_file: None,
            file_lock: None,
            is_modified: false,
            lock_holder: None,
            collapsed_sections: HashSet::new(), // All sections expanded by default
            selection: EditorSelection::ProjectInfo, // Start with project info selected
            beam_label: "B-1".to_string(),
            span_ft: "12.0".to_string(),
            width_in: "1.5".to_string(),
            depth_in: "9.25".to_string(),
            load_table: default_loads,
            include_self_weight: true,
            selected_material_type: MaterialType::SawnLumber,
            selected_species: Some(WoodSpecies::DouglasFirLarch),
            selected_grade: Some(WoodGrade::No2),
            selected_glulam_class: Some(GlulamStressClass::F24_V4),
            selected_glulam_layup: Some(GlulamLayup::Unbalanced),
            selected_lvl_grade: Some(LvlGrade::Standard),
            selected_psl_grade: Some(PslGrade::Standard),
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

    // Left panel section toggle
    ToggleSection(ItemSection),

    // Editor selection
    SelectProjectInfo,
    SelectBeam(Uuid),
    NewBeam,

    // Beam input field changes
    BeamLabelChanged(String),
    SpanChanged(String),
    WidthChanged(String),
    DepthChanged(String),

    // Load table operations
    AddLoad,
    RemoveLoad(Uuid),
    LoadTypeChanged(Uuid, LoadType),
    LoadDistributionChanged(Uuid, DistributionType),
    LoadMagnitudeChanged(Uuid, String),
    LoadPositionChanged(Uuid, String),
    LoadTributaryChanged(Uuid, String),
    IncludeSelfWeightToggled(bool),

    // Material selection
    MaterialTypeSelected(MaterialType),
    // Sawn lumber
    SpeciesSelected(WoodSpecies),
    GradeSelected(WoodGrade),
    // Glulam
    GlulamClassSelected(GlulamStressClass),
    GlulamLayupSelected(GlulamLayup),
    // LVL
    LvlGradeSelected(LvlGrade),
    // PSL
    PslGradeSelected(PslGrade),

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

            // Section toggle
            Message::ToggleSection(section) => {
                if self.collapsed_sections.contains(&section) {
                    self.collapsed_sections.remove(&section);
                } else {
                    self.collapsed_sections.insert(section);
                }
            }

            // Editor selection
            Message::SelectProjectInfo => {
                self.selection = EditorSelection::ProjectInfo;
                self.result = None;
                self.error_message = None;
            }
            Message::SelectBeam(id) => {
                self.select_beam(id);
            }
            Message::NewBeam => {
                self.new_beam();
            }

            // Beam fields
            Message::BeamLabelChanged(value) => {
                self.beam_label = value;
            }
            Message::SpanChanged(value) => {
                self.span_ft = value;
            }
            Message::WidthChanged(value) => {
                self.width_in = value;
            }
            Message::DepthChanged(value) => {
                self.depth_in = value;
            }

            // Load table operations
            Message::AddLoad => {
                self.load_table.push(LoadTableRow::new());
            }
            Message::RemoveLoad(id) => {
                self.load_table.retain(|row| row.id != id);
            }
            Message::LoadTypeChanged(id, load_type) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.load_type = load_type;
                }
            }
            Message::LoadDistributionChanged(id, dist) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.distribution = dist;
                }
            }
            Message::LoadMagnitudeChanged(id, value) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.magnitude = value;
                }
            }
            Message::LoadPositionChanged(id, value) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.position = value;
                }
            }
            Message::LoadTributaryChanged(id, value) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.tributary_width = value;
                }
            }
            Message::IncludeSelfWeightToggled(value) => {
                self.include_self_weight = value;
            }

            Message::MaterialTypeSelected(material_type) => {
                self.selected_material_type = material_type;
            }
            Message::SpeciesSelected(species) => {
                self.selected_species = Some(species);
            }
            Message::GradeSelected(grade) => {
                self.selected_grade = Some(grade);
            }
            Message::GlulamClassSelected(class) => {
                self.selected_glulam_class = Some(class);
            }
            Message::GlulamLayupSelected(layup) => {
                self.selected_glulam_layup = Some(layup);
            }
            Message::LvlGradeSelected(grade) => {
                self.selected_lvl_grade = Some(grade);
            }
            Message::PslGradeSelected(grade) => {
                self.selected_psl_grade = Some(grade);
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
        self.selection = EditorSelection::ProjectInfo;
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
                            self.selection = EditorSelection::ProjectInfo;
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
                            self.file_lock = None; // No lock in read-only mode
                            self.is_modified = false;
                            self.lock_holder = Some(locked_by.clone());
                            self.selection = EditorSelection::ProjectInfo;
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
                self.selection = EditorSelection::Beam(Some(id));
                self.beam_label = beam.label.clone();
                self.span_ft = beam.span_ft.to_string();
                self.width_in = beam.width_in.to_string();
                self.depth_in = beam.depth_in.to_string();

                // Populate load table from beam's load_case
                self.load_table = beam
                    .load_case
                    .loads
                    .iter()
                    .map(LoadTableRow::from_discrete_load)
                    .collect();
                self.include_self_weight = beam.load_case.include_self_weight;

                // Extract material-specific fields
                match &beam.material {
                    Material::SawnLumber(wood) => {
                        self.selected_material_type = MaterialType::SawnLumber;
                        self.selected_species = Some(wood.species);
                        self.selected_grade = Some(wood.grade);
                    }
                    Material::Glulam(glulam) => {
                        self.selected_material_type = MaterialType::Glulam;
                        self.selected_glulam_class = Some(glulam.stress_class);
                        self.selected_glulam_layup = Some(glulam.layup);
                    }
                    Material::Lvl(lvl) => {
                        self.selected_material_type = MaterialType::Lvl;
                        self.selected_lvl_grade = Some(lvl.grade);
                    }
                    Material::Psl(psl) => {
                        self.selected_material_type = MaterialType::Psl;
                        self.selected_psl_grade = Some(psl.grade);
                    }
                }

                self.result = None;
                self.error_message = None;
                self.status = format!("Selected: {}", beam.label);
            }
        }
    }

    fn new_beam(&mut self) {
        self.selection = EditorSelection::Beam(None);
        self.beam_label = "B-1".to_string();
        self.span_ft = "12.0".to_string();
        self.width_in = "1.5".to_string();
        self.depth_in = "9.25".to_string();

        // Reset to default load table
        self.load_table = vec![
            LoadTableRow {
                id: Uuid::new_v4(),
                load_type: LoadType::Dead,
                distribution: DistributionType::UniformFull,
                magnitude: "15.0".to_string(),
                position: String::new(),
                tributary_width: String::new(),
            },
            LoadTableRow {
                id: Uuid::new_v4(),
                load_type: LoadType::Live,
                distribution: DistributionType::UniformFull,
                magnitude: "40.0".to_string(),
                position: String::new(),
                tributary_width: String::new(),
            },
        ];
        self.include_self_weight = true;

        self.selected_material_type = MaterialType::SawnLumber;
        self.selected_species = Some(WoodSpecies::DouglasFirLarch);
        self.selected_grade = Some(WoodGrade::No2);
        self.selected_glulam_class = Some(GlulamStressClass::F24_V4);
        self.selected_glulam_layup = Some(GlulamLayup::Unbalanced);
        self.selected_lvl_grade = Some(LvlGrade::Standard);
        self.selected_psl_grade = Some(PslGrade::Standard);
        self.result = None;
        self.error_message = None;
    }

    /// Helper to get selected beam ID if any
    fn selected_beam_id(&self) -> Option<Uuid> {
        match self.selection {
            EditorSelection::Beam(Some(id)) => Some(id),
            _ => None,
        }
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

        // Build material based on selected type
        let material = match self.selected_material_type {
            MaterialType::SawnLumber => {
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
                Material::SawnLumber(WoodMaterial::new(species, grade))
            }
            MaterialType::Glulam => {
                let stress_class = match self.selected_glulam_class {
                    Some(c) => c,
                    None => {
                        self.error_message = Some("Please select a glulam stress class".to_string());
                        return;
                    }
                };
                let layup = match self.selected_glulam_layup {
                    Some(l) => l,
                    None => {
                        self.error_message = Some("Please select a glulam layup".to_string());
                        return;
                    }
                };
                Material::Glulam(GlulamMaterial::new(stress_class, layup))
            }
            MaterialType::Lvl => {
                let grade = match self.selected_lvl_grade {
                    Some(g) => g,
                    None => {
                        self.error_message = Some("Please select an LVL grade".to_string());
                        return;
                    }
                };
                Material::Lvl(LvlMaterial::new(grade))
            }
            MaterialType::Psl => {
                let grade = match self.selected_psl_grade {
                    Some(g) => g,
                    None => {
                        self.error_message = Some("Please select a PSL grade".to_string());
                        return;
                    }
                };
                Material::Psl(PslMaterial::new(grade))
            }
        };

        // Build load case from load table
        let mut load_case = EnhancedLoadCase::new("Service Loads");
        load_case.include_self_weight = self.include_self_weight;

        for row in &self.load_table {
            if let Some(load) = row.to_discrete_load() {
                load_case = load_case.with_load(load);
            }
        }

        if load_case.loads.is_empty() {
            self.error_message = Some("At least one valid load is required".to_string());
            return;
        }

        let beam = BeamInput {
            label: self.beam_label.clone(),
            span_ft,
            load_case,
            material,
            width_in,
            depth_in,
        };

        if let Some(id) = self.selected_beam_id() {
            // Update existing beam
            self.project.items.insert(id, CalculationItem::Beam(beam));
            self.mark_modified();
            self.error_message = None;
            self.status = format!("Updated beam '{}'", self.beam_label);
        } else {
            // Add new beam
            let id = self.project.add_item(CalculationItem::Beam(beam));
            self.selection = EditorSelection::Beam(Some(id));
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

        if let Some(id) = self.selected_beam_id() {
            if let Some(item) = self.project.items.remove(&id) {
                self.mark_modified();
                self.status = format!("Deleted: {}", item.label());
                self.new_beam(); // Reset to new beam state
            }
        } else {
            self.status = "No beam selected to delete".to_string();
        }
    }

    fn run_calculation(&mut self) {
        self.error_message = None;

        // Parse inputs
        let span_ft = match self.span_ft.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => {
                self.error_message = Some("Invalid span value".to_string());
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

        // Build material based on selected type
        let material = match self.selected_material_type {
            MaterialType::SawnLumber => {
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
                Material::SawnLumber(WoodMaterial::new(species, grade))
            }
            MaterialType::Glulam => {
                let stress_class = match self.selected_glulam_class {
                    Some(c) => c,
                    None => {
                        self.error_message = Some("Please select a glulam stress class".to_string());
                        return;
                    }
                };
                let layup = match self.selected_glulam_layup {
                    Some(l) => l,
                    None => {
                        self.error_message = Some("Please select a glulam layup".to_string());
                        return;
                    }
                };
                Material::Glulam(GlulamMaterial::new(stress_class, layup))
            }
            MaterialType::Lvl => {
                let grade = match self.selected_lvl_grade {
                    Some(g) => g,
                    None => {
                        self.error_message = Some("Please select an LVL grade".to_string());
                        return;
                    }
                };
                Material::Lvl(LvlMaterial::new(grade))
            }
            MaterialType::Psl => {
                let grade = match self.selected_psl_grade {
                    Some(g) => g,
                    None => {
                        self.error_message = Some("Please select a PSL grade".to_string());
                        return;
                    }
                };
                Material::Psl(PslMaterial::new(grade))
            }
        };

        // Build load case from load table
        let mut load_case = EnhancedLoadCase::new("Service Loads");
        load_case.include_self_weight = self.include_self_weight;

        for row in &self.load_table {
            if let Some(load) = row.to_discrete_load() {
                load_case = load_case.with_load(load);
            }
        }

        if load_case.loads.is_empty() {
            self.error_message = Some("At least one valid load is required".to_string());
            return;
        }

        // Build input
        let input = BeamInput {
            label: self.beam_label.clone(),
            span_ft,
            load_case,
            material,
            width_in,
            depth_in,
        };

        // Run calculation using project's design method
        let design_method = self.project.settings.design_method;
        match calculate(&input, design_method) {
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
        let mut panel_content: Column<'_, Message> = column![].spacing(2);

        // ===== Project Info Section =====
        let project_expanded = !self.collapsed_sections.contains(&ItemSection::ProjectInfo);
        let project_selected = matches!(self.selection, EditorSelection::ProjectInfo);
        let project_header = self.view_section_header("Project Info", ItemSection::ProjectInfo, project_expanded, None);
        panel_content = panel_content.push(project_header);

        if project_expanded {
            let project_info_content = column![
                text(format!("Eng: {}", self.project.meta.engineer)).size(10),
                text(format!("Job: {}", self.project.meta.job_id)).size(10),
                text(format!("Client: {}", self.project.meta.client)).size(10),
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

        panel_content = panel_content.push(horizontal_rule(1));

        // ===== Wood Beams Section =====
        let beams_expanded = !self.collapsed_sections.contains(&ItemSection::WoodBeams);
        let beam_count = self.project.items.values()
            .filter(|i| matches!(i, CalculationItem::Beam(_)))
            .count();

        // Section header with expand/collapse and add button
        let beams_indicator = if beams_expanded { "▼" } else { "▶" };
        let beams_header_btn = button(
            row![
                text(beams_indicator).size(10),
                horizontal_space().width(4),
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
                .on_press(Message::NewBeam)
                .padding(Padding::from([2, 6]))
                .style(button::secondary),
        ]
        .spacing(2);
        panel_content = panel_content.push(beams_header);

        if beams_expanded {
            let mut beams_list: Column<'_, Message> = column![].spacing(2).padding(Padding::from([4, 8]));

            // List beams
            for (id, item) in &self.project.items {
                if let CalculationItem::Beam(beam) = item {
                    let is_selected = self.selected_beam_id() == Some(*id);
                    let btn = if is_selected {
                        button(text(&beam.label).size(10))
                            .on_press(Message::SelectBeam(*id)) // Re-select (no-op)
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

        panel_content = panel_content.push(horizontal_rule(1));

        // ===== Wood Columns Section (Future) =====
        let columns_header = self.view_section_header_disabled("Wood Columns", 0);
        panel_content = panel_content.push(columns_header);

        panel_content = panel_content.push(horizontal_rule(1));

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
            .width(Length::Fixed(170.0))
            .height(Length::Fill)
            .style(container::bordered_box)
            .padding(4);

        panel.into()
    }

    /// Create a collapsible section header with expand/collapse indicator
    fn view_section_header<'a>(
        &'a self,
        title: &'a str,
        section: ItemSection,
        expanded: bool,
        add_action: Option<Message>,
    ) -> Element<'a, Message> {
        let indicator = if expanded { "▼" } else { "▶" };

        let header_btn = button(
            row![
                text(indicator).size(10),
                horizontal_space().width(4),
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
    fn view_section_header_disabled(&self, title: &str, count: usize) -> Element<'_, Message> {
        row![
            text("▶").size(10).color([0.6, 0.6, 0.6]),
            horizontal_space().width(4),
            text(format!("{} ({})", title, count)).size(11).color([0.6, 0.6, 0.6]),
        ]
        .padding(Padding::from([4, 6]))
        .align_y(Alignment::Center)
        .into()
    }

    fn view_input_panel(&self) -> Element<'_, Message> {
        // Show different content based on current selection
        let panel: Column<'_, Message> = match self.selection {
            EditorSelection::ProjectInfo => {
                // Project Info editor
                column![
                    text("Project Information").size(14),
                    vertical_space().height(8),
                    labeled_input(
                        "Engineer:",
                        &self.project.meta.engineer,
                        Message::EngineerNameChanged
                    ),
                    labeled_input("Job ID:", &self.project.meta.job_id, Message::JobIdChanged),
                    labeled_input("Client:", &self.project.meta.client, Message::ClientChanged),
                    vertical_space().height(20),
                    text("Select a beam from the left panel to edit,").size(11).color([0.5, 0.5, 0.5]),
                    text("or click '+' to create a new beam.").size(11).color([0.5, 0.5, 0.5]),
                ]
                .spacing(6)
            }
            EditorSelection::None => {
                // Nothing selected
                column![
                    text("Select an item from the left panel").size(14).color([0.5, 0.5, 0.5]),
                ]
            }
            EditorSelection::Beam(_) => {
                // Beam editor
                self.view_beam_editor()
            }
        };

        container(scrollable(panel.width(Length::FillPortion(2)).padding(8)))
            .style(container::bordered_box)
            .padding(5)
            .into()
    }

    fn view_beam_editor(&self) -> Column<'_, Message> {
        let editing_label = if self.selected_beam_id().is_some() {
            "Edit Beam"
        } else {
            "New Beam"
        };

        let beam_section = column![
            text(editing_label).size(14),
            vertical_space().height(8),
            labeled_input("Label:", &self.beam_label, Message::BeamLabelChanged),
            labeled_input("Span (ft):", &self.span_ft, Message::SpanChanged),
            labeled_input("Width (in):", &self.width_in, Message::WidthChanged),
            labeled_input("Depth (in):", &self.depth_in, Message::DepthChanged),
        ]
        .spacing(6);

        // Load table section
        let loads_section = self.view_load_table();

        // Build material-specific options based on selected type
        let material_options: Column<'_, Message> = match self.selected_material_type {
            MaterialType::SawnLumber => column![
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
            .spacing(2),
            MaterialType::Glulam => column![
                text("Stress Class:").size(11),
                pick_list(
                    &GlulamStressClass::ALL[..],
                    self.selected_glulam_class,
                    Message::GlulamClassSelected
                )
                .width(Length::Fill)
                .placeholder("Select..."),
                vertical_space().height(4),
                text("Layup:").size(11),
                pick_list(
                    &GlulamLayup::ALL[..],
                    self.selected_glulam_layup,
                    Message::GlulamLayupSelected
                )
                .width(Length::Fill)
                .placeholder("Select..."),
            ]
            .spacing(2),
            MaterialType::Lvl => column![
                text("Grade:").size(11),
                pick_list(
                    &LvlGrade::ALL[..],
                    self.selected_lvl_grade,
                    Message::LvlGradeSelected
                )
                .width(Length::Fill)
                .placeholder("Select..."),
            ]
            .spacing(2),
            MaterialType::Psl => column![
                text("Grade:").size(11),
                pick_list(
                    &PslGrade::ALL[..],
                    self.selected_psl_grade,
                    Message::PslGradeSelected
                )
                .width(Length::Fill)
                .placeholder("Select..."),
            ]
            .spacing(2),
        };

        let material_section = column![
            text("Material").size(14),
            vertical_space().height(8),
            text("Type:").size(11),
            pick_list(
                &MaterialType::ALL[..],
                Some(self.selected_material_type),
                Message::MaterialTypeSelected
            )
            .width(Length::Fill),
            vertical_space().height(8),
            material_options,
        ]
        .spacing(2);

        let save_btn_text = if self.selected_beam_id().is_some() {
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

        if self.selected_beam_id().is_some() {
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

        column![
            beam_section,
            vertical_space().height(10),
            loads_section,
            vertical_space().height(10),
            material_section,
            vertical_space().height(15),
            action_buttons,
        ]
    }

    fn view_load_table(&self) -> Element<'_, Message> {
        let self_weight_checkbox = checkbox("Include self-weight", self.include_self_weight)
            .on_toggle(Message::IncludeSelfWeightToggled)
            .text_size(11);

        // Header row
        let header = row![
            text("Type").size(10).width(Length::Fixed(45.0)),
            text("Dist").size(10).width(Length::Fixed(60.0)),
            text("Mag").size(10).width(Length::Fixed(55.0)),
            text("Pos").size(10).width(Length::Fixed(45.0)),
            text("Trib").size(10).width(Length::Fixed(45.0)),
            text("").size(10).width(Length::Fixed(30.0)), // Delete button column
        ]
        .spacing(4)
        .align_y(Alignment::Center);

        // Build rows for each load
        let mut load_rows: Column<'_, Message> = column![].spacing(4);

        for load_row in &self.load_table {
            let row_id = load_row.id;

            let type_picker = pick_list(
                &LoadType::ALL[..],
                Some(load_row.load_type),
                move |lt| Message::LoadTypeChanged(row_id, lt),
            )
            .width(Length::Fixed(45.0))
            .text_size(10);

            let dist_picker = pick_list(
                &DistributionType::ALL[..],
                Some(load_row.distribution),
                move |dt| Message::LoadDistributionChanged(row_id, dt),
            )
            .width(Length::Fixed(60.0))
            .text_size(10);

            let mag_input = text_input("0.0", &load_row.magnitude)
                .on_input(move |s| Message::LoadMagnitudeChanged(row_id, s))
                .width(Length::Fixed(55.0))
                .padding(2)
                .size(10);

            // Position input (only relevant for point loads)
            let pos_input = text_input("", &load_row.position)
                .on_input(move |s| Message::LoadPositionChanged(row_id, s))
                .width(Length::Fixed(45.0))
                .padding(2)
                .size(10);

            let trib_input = text_input("", &load_row.tributary_width)
                .on_input(move |s| Message::LoadTributaryChanged(row_id, s))
                .width(Length::Fixed(45.0))
                .padding(2)
                .size(10);

            let delete_btn = button(text("X").size(10))
                .on_press(Message::RemoveLoad(row_id))
                .padding(Padding::from([2, 6]));

            let load_row_widget: Row<'_, Message> = row![
                type_picker,
                dist_picker,
                mag_input,
                pos_input,
                trib_input,
                delete_btn,
            ]
            .spacing(4)
            .align_y(Alignment::Center);

            load_rows = load_rows.push(load_row_widget);
        }

        let add_load_btn = button(text("+ Add Load").size(10))
            .on_press(Message::AddLoad)
            .padding(Padding::from([4, 8]));

        column![
            text("Loads").size(14),
            vertical_space().height(6),
            self_weight_checkbox,
            vertical_space().height(6),
            header,
            horizontal_rule(1),
            load_rows,
            vertical_space().height(6),
            add_load_btn,
        ]
        .spacing(2)
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

        // Build load info section
        let self_weight_text = if result.self_weight_plf > 0.0 {
            format!(" (incl. self-wt: {:.1} plf)", result.self_weight_plf)
        } else {
            String::new()
        };

        column![
            text("Calculation Results").size(14),
            vertical_space().height(8),
            pass_fail,
            governing,
            vertical_space().height(12),
            text("Load Summary").size(12),
            text(format!("Design Load: {:.1} plf{}", result.design_load_plf, self_weight_text)).size(11),
            text(format!("Governing Combo: {}", result.governing_combination)).size(11),
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
            load_plf: result.design_load_plf, // Use the factored design load from result
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

        // Reaction arrows (upward) at supports
        let reaction_color = Color::from_rgb(0.7, 0.2, 0.2); // Red for reactions
        let reaction_arrow_length = height * 0.2;
        let reaction_start_y = beam_y + support_size + 5.0;

        // Left reaction arrow (upward)
        let left_reaction = Path::line(
            Point::new(x, reaction_start_y + reaction_arrow_length),
            Point::new(x, reaction_start_y),
        );
        frame.stroke(
            &left_reaction,
            Stroke::default().with_color(reaction_color).with_width(2.5),
        );
        // Arrow head pointing up
        let left_head = Path::new(|builder| {
            builder.move_to(Point::new(x, reaction_start_y));
            builder.line_to(Point::new(x - 4.0, reaction_start_y + 8.0));
            builder.move_to(Point::new(x, reaction_start_y));
            builder.line_to(Point::new(x + 4.0, reaction_start_y + 8.0));
        });
        frame.stroke(
            &left_head,
            Stroke::default().with_color(reaction_color).with_width(2.5),
        );

        // Right reaction arrow (upward)
        let right_reaction = Path::line(
            Point::new(x + width, reaction_start_y + reaction_arrow_length),
            Point::new(x + width, reaction_start_y),
        );
        frame.stroke(
            &right_reaction,
            Stroke::default().with_color(reaction_color).with_width(2.5),
        );
        // Arrow head pointing up
        let right_head = Path::new(|builder| {
            builder.move_to(Point::new(x + width, reaction_start_y));
            builder.line_to(Point::new(x + width - 4.0, reaction_start_y + 8.0));
            builder.move_to(Point::new(x + width, reaction_start_y));
            builder.line_to(Point::new(x + width + 4.0, reaction_start_y + 8.0));
        });
        frame.stroke(
            &right_head,
            Stroke::default().with_color(reaction_color).with_width(2.5),
        );

        // Reaction labels (R = wL/2 = max_shear)
        let left_reaction_text = Text {
            content: format!("R = {:.0} lb", self.data.max_shear_lb),
            position: Point::new(x + 5.0, reaction_start_y + reaction_arrow_length + 2.0),
            color: reaction_color,
            size: iced::Pixels(9.0),
            ..Text::default()
        };
        frame.fill_text(left_reaction_text);

        let right_reaction_text = Text {
            content: format!("R = {:.0} lb", self.data.max_shear_lb),
            position: Point::new(x + width - 55.0, reaction_start_y + reaction_arrow_length + 2.0),
            color: reaction_color,
            size: iced::Pixels(9.0),
            ..Text::default()
        };
        frame.fill_text(right_reaction_text);
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
