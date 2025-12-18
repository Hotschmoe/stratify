//! # Stratify GUI Application
//!
//! Full-featured graphical interface for structural engineering calculations.
//! Built with Iced framework for cross-platform support (Windows, macOS, Linux, WASM).

use std::collections::HashSet;
use std::path::PathBuf;

use iced::keyboard::{self, Key, Modifiers};
use iced::widget::canvas::{self, Canvas, Frame, Geometry, Path, Stroke, Text};
use iced::widget::{
    button, checkbox, column, container, pick_list, row, rule,
    scrollable, text, text_input, Column, Row, Space,
    operation,
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
use calc_core::nds_factors::{
    AdjustmentFactors, FlatUse, Incising, LoadDuration, RepetitiveMember, Temperature, WetService,
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
    // Set up panic hook for WASM to show errors in browser console
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    iced::application(App::new, App::update, App::view)
        .title(App::window_title)
        .subscription(App::subscription)
        .theme(App::theme)
        .window_size((1200.0, 750.0))
        .font(BERKELEY_MONO)
        .font(BERKELEY_MONO_BOLD)
        .default_font(Font::with_name("Berkeley Mono"))
        .run()
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

    // NDS Adjustment Factors
    selected_load_duration: LoadDuration,
    selected_wet_service: WetService,
    selected_temperature: Temperature,
    selected_incising: Incising,
    selected_repetitive_member: RepetitiveMember,
    selected_flat_use: FlatUse,
    compression_edge_braced: bool,

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
            // NDS Adjustment Factors - typical interior floor beam defaults
            selected_load_duration: LoadDuration::Normal,
            selected_wet_service: WetService::Dry,
            selected_temperature: Temperature::Normal,
            selected_incising: Incising::None,
            selected_repetitive_member: RepetitiveMember::Single,
            selected_flat_use: FlatUse::Normal,
            compression_edge_braced: true,
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

    /// Get the application theme
    fn theme(&self) -> Theme {
        Theme::Light
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
    CreateBeam, // Creates a new beam immediately and selects it

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

    // NDS Adjustment Factors
    LoadDurationSelected(LoadDuration),
    WetServiceSelected(WetService),
    TemperatureSelected(Temperature),
    IncisingSelected(Incising),
    RepetitiveMemberSelected(RepetitiveMember),
    FlatUseSelected(FlatUse),
    CompressionBracedToggled(bool),

    // Actions
    DeleteSelectedBeam,
    ExportPdf,

    // Keyboard events
    KeyPressed(Key, Modifiers),

    // Focus navigation (Tab / Shift+Tab)
    FocusNext,
    FocusPrevious,
}

// ============================================================================
// Subscriptions (for keyboard shortcuts)
// ============================================================================

impl App {
    fn subscription(&self) -> Subscription<Message> {
        event::listen_with(|event, _status, _id| match event {
            Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) => {
                // Handle Tab / Shift+Tab for focus navigation
                if key == Key::Named(keyboard::key::Named::Tab) {
                    if modifiers.shift() {
                        return Some(Message::FocusPrevious);
                    } else {
                        return Some(Message::FocusNext);
                    }
                }
                // Handle other keyboard shortcuts
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
            // Focus navigation (Tab / Shift+Tab)
            Message::FocusNext => {
                return operation::focus_next();
            }
            Message::FocusPrevious => {
                return operation::focus_previous();
            }

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
            Message::CreateBeam => {
                self.create_beam();
            }

            // Beam fields - all trigger live preview update
            Message::BeamLabelChanged(value) => {
                self.beam_label = value.clone();
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::SpanChanged(value) => {
                self.span_ft = value;
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::WidthChanged(value) => {
                self.width_in = value;
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::DepthChanged(value) => {
                self.depth_in = value;
                self.auto_save_beam();
                self.try_calculate();
            }

            // Load table operations - all trigger live preview update
            Message::AddLoad => {
                self.load_table.push(LoadTableRow::new());
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::RemoveLoad(id) => {
                self.load_table.retain(|row| row.id != id);
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::LoadTypeChanged(id, load_type) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.load_type = load_type;
                }
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::LoadDistributionChanged(id, dist) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.distribution = dist;
                }
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::LoadMagnitudeChanged(id, value) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.magnitude = value;
                }
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::LoadPositionChanged(id, value) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.position = value;
                }
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::LoadTributaryChanged(id, value) => {
                if let Some(row) = self.load_table.iter_mut().find(|r| r.id == id) {
                    row.tributary_width = value;
                }
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::IncludeSelfWeightToggled(value) => {
                self.include_self_weight = value;
                self.auto_save_beam();
                self.try_calculate();
            }

            // Material selection - all trigger live preview update
            Message::MaterialTypeSelected(material_type) => {
                self.selected_material_type = material_type;
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::SpeciesSelected(species) => {
                self.selected_species = Some(species);
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::GradeSelected(grade) => {
                self.selected_grade = Some(grade);
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::GlulamClassSelected(class) => {
                self.selected_glulam_class = Some(class);
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::GlulamLayupSelected(layup) => {
                self.selected_glulam_layup = Some(layup);
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::LvlGradeSelected(grade) => {
                self.selected_lvl_grade = Some(grade);
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::PslGradeSelected(grade) => {
                self.selected_psl_grade = Some(grade);
                self.auto_save_beam();
                self.try_calculate();
            }

            // NDS Adjustment Factors - all trigger live preview update
            Message::LoadDurationSelected(duration) => {
                self.selected_load_duration = duration;
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::WetServiceSelected(wet) => {
                self.selected_wet_service = wet;
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::TemperatureSelected(temp) => {
                self.selected_temperature = temp;
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::IncisingSelected(incising) => {
                self.selected_incising = incising;
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::RepetitiveMemberSelected(rep) => {
                self.selected_repetitive_member = rep;
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::FlatUseSelected(flat) => {
                self.selected_flat_use = flat;
                self.auto_save_beam();
                self.try_calculate();
            }
            Message::CompressionBracedToggled(braced) => {
                self.compression_edge_braced = braced;
                self.auto_save_beam();
                self.try_calculate();
            }

            // Actions
            Message::DeleteSelectedBeam => {
                self.delete_selected_beam();
            }
            Message::ExportPdf => {
                self.export_pdf();
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

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(target_arch = "wasm32")]
    fn open_project(&mut self) {
        self.status = "File open not available in browser. Use drag-and-drop.".to_string();
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

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(target_arch = "wasm32")]
    fn save_project_as(&mut self) {
        self.status = "File save not available in browser.".to_string();
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

                // Extract adjustment factors
                self.selected_load_duration = beam.adjustment_factors.load_duration;
                self.selected_wet_service = beam.adjustment_factors.wet_service;
                self.selected_temperature = beam.adjustment_factors.temperature;
                self.selected_incising = beam.adjustment_factors.incising;
                self.selected_repetitive_member = beam.adjustment_factors.repetitive_member;
                self.selected_flat_use = beam.adjustment_factors.flat_use;
                self.compression_edge_braced = beam.adjustment_factors.compression_edge_braced;

                self.error_message = None;
                self.status = format!("Selected: {}", beam.label);

                // Calculate immediately to show live preview
                self.try_calculate();
            }
        }
    }

    /// Helper to get selected beam ID if any
    fn selected_beam_id(&self) -> Option<Uuid> {
        match self.selection {
            EditorSelection::Beam(Some(id)) => Some(id),
            _ => None,
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
                // Select project info after deletion
                self.selection = EditorSelection::ProjectInfo;
                self.result = None;
                self.calc_input = None;
            }
        } else {
            self.status = "No beam selected to delete".to_string();
        }
    }

    /// Create a new beam immediately with default values and add to project
    fn create_beam(&mut self) {
        if !self.can_edit() {
            self.status = "Cannot modify: file is read-only".to_string();
            return;
        }

        // Generate unique label
        let beam_count = self
            .project
            .items
            .values()
            .filter(|i| matches!(i, CalculationItem::Beam(_)))
            .count();
        let new_label = format!("B-{}", beam_count + 1);

        // Create default load case
        let load_case = EnhancedLoadCase::new("Service Loads")
            .with_load(DiscreteLoad::uniform(LoadType::Dead, 15.0))
            .with_load(DiscreteLoad::uniform(LoadType::Live, 40.0));

        // Create beam with defaults
        let beam = BeamInput {
            label: new_label.clone(),
            span_ft: 12.0,
            load_case,
            material: Material::SawnLumber(WoodMaterial::new(
                WoodSpecies::DouglasFirLarch,
                WoodGrade::No2,
            )),
            width_in: 1.5,
            depth_in: 9.25,
            adjustment_factors: calc_core::nds_factors::AdjustmentFactors::default(),
        };

        // Add to project and get the ID
        let id = self.project.add_item(CalculationItem::Beam(beam));
        self.mark_modified();

        // Select the new beam for editing
        self.select_beam(id);
        self.status = format!("Created beam '{}'", new_label);

        // Try to calculate immediately
        self.try_calculate();
    }

    /// Auto-save beam editor state to the currently selected beam (silent, no validation errors)
    fn auto_save_beam(&mut self) {
        if !self.can_edit() {
            return;
        }

        // Only auto-save if we have an existing beam selected
        let beam_id = match self.selection {
            EditorSelection::Beam(Some(id)) => id,
            _ => return,
        };

        // Try to build a valid beam from current state - silently fail if invalid
        let span_ft = match self.span_ft.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => return, // Invalid, don't save
        };
        let width_in = match self.width_in.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => return,
        };
        let depth_in = match self.depth_in.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => return,
        };

        // Build material
        let material = match self.selected_material_type {
            MaterialType::SawnLumber => {
                match (self.selected_species, self.selected_grade) {
                    (Some(species), Some(grade)) => {
                        Material::SawnLumber(WoodMaterial::new(species, grade))
                    }
                    _ => return,
                }
            }
            MaterialType::Glulam => {
                match (self.selected_glulam_class, self.selected_glulam_layup) {
                    (Some(stress_class), Some(layup)) => {
                        Material::Glulam(GlulamMaterial::new(stress_class, layup))
                    }
                    _ => return,
                }
            }
            MaterialType::Lvl => {
                match self.selected_lvl_grade {
                    Some(grade) => Material::Lvl(LvlMaterial::new(grade)),
                    None => return,
                }
            }
            MaterialType::Psl => {
                match self.selected_psl_grade {
                    Some(grade) => Material::Psl(PslMaterial::new(grade)),
                    None => return,
                }
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

        // Need at least one load
        if load_case.loads.is_empty() {
            return;
        }

        // Build adjustment factors from UI state
        let adjustment_factors = AdjustmentFactors {
            load_duration: self.selected_load_duration,
            wet_service: self.selected_wet_service,
            temperature: self.selected_temperature,
            incising: self.selected_incising,
            repetitive_member: self.selected_repetitive_member,
            flat_use: self.selected_flat_use,
            compression_edge_braced: self.compression_edge_braced,
            unbraced_length_in: None, // TODO: Add UI for unbraced length if not braced
        };

        let beam = BeamInput {
            label: self.beam_label.clone(),
            span_ft,
            load_case,
            material,
            width_in,
            depth_in,
            adjustment_factors,
        };

        // Update the beam in the project
        self.project.items.insert(beam_id, CalculationItem::Beam(beam));
        self.mark_modified();
    }

    /// Try to run calculation silently - no error messages for invalid input
    fn try_calculate(&mut self) {
        // Only calculate if we're editing a beam
        if !matches!(self.selection, EditorSelection::Beam(_)) {
            return;
        }

        // Try to parse inputs - silently return on invalid
        let span_ft = match self.span_ft.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => {
                self.result = None;
                self.calc_input = None;
                return;
            }
        };
        let width_in = match self.width_in.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => {
                self.result = None;
                self.calc_input = None;
                return;
            }
        };
        let depth_in = match self.depth_in.parse::<f64>() {
            Ok(v) if v > 0.0 => v,
            _ => {
                self.result = None;
                self.calc_input = None;
                return;
            }
        };

        // Build material
        let material = match self.selected_material_type {
            MaterialType::SawnLumber => {
                match (self.selected_species, self.selected_grade) {
                    (Some(species), Some(grade)) => {
                        Material::SawnLumber(WoodMaterial::new(species, grade))
                    }
                    _ => {
                        self.result = None;
                        self.calc_input = None;
                        return;
                    }
                }
            }
            MaterialType::Glulam => {
                match (self.selected_glulam_class, self.selected_glulam_layup) {
                    (Some(stress_class), Some(layup)) => {
                        Material::Glulam(GlulamMaterial::new(stress_class, layup))
                    }
                    _ => {
                        self.result = None;
                        self.calc_input = None;
                        return;
                    }
                }
            }
            MaterialType::Lvl => {
                match self.selected_lvl_grade {
                    Some(grade) => Material::Lvl(LvlMaterial::new(grade)),
                    None => {
                        self.result = None;
                        self.calc_input = None;
                        return;
                    }
                }
            }
            MaterialType::Psl => {
                match self.selected_psl_grade {
                    Some(grade) => Material::Psl(PslMaterial::new(grade)),
                    None => {
                        self.result = None;
                        self.calc_input = None;
                        return;
                    }
                }
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
            self.result = None;
            self.calc_input = None;
            return;
        }

        // Build adjustment factors from UI state
        let adjustment_factors = AdjustmentFactors {
            load_duration: self.selected_load_duration,
            wet_service: self.selected_wet_service,
            temperature: self.selected_temperature,
            incising: self.selected_incising,
            repetitive_member: self.selected_repetitive_member,
            flat_use: self.selected_flat_use,
            compression_edge_braced: self.compression_edge_braced,
            unbraced_length_in: None, // TODO: Add UI for unbraced length if not braced
        };

        // Build input
        let input = BeamInput {
            label: self.beam_label.clone(),
            span_ft,
            load_case,
            material,
            width_in,
            depth_in,
            adjustment_factors,
        };

        // Run calculation
        let design_method = self.project.settings.design_method;
        match calculate(&input, design_method) {
            Ok(result) => {
                self.calc_input = Some(input);
                self.result = Some(result);
                self.error_message = None;
                self.diagram_cache.clear();
            }
            Err(_) => {
                // Silently clear results on calculation error
                self.calc_input = None;
                self.result = None;
            }
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
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

    #[cfg(target_arch = "wasm32")]
    fn export_pdf(&mut self) {
        self.status = "PDF export not available in browser.".to_string();
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
            rule::horizontal(2),
            // Toolbar
            self.view_toolbar(),
            rule::horizontal(1),
            Space::new().height(10),
            // Main content
            content,
            // Status bar
            Space::new().height(10),
            rule::horizontal(1),
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
            Space::new().width(Length::Fill),
            text(title).size(14),
        ]
        .align_y(Alignment::Center)
        .into()
    }

    fn view_toolbar(&self) -> Element<'_, Message> {
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

        row![file_buttons, Space::new().width(Length::Fill),]
            .padding(Padding::from([4, 0]))
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

        panel_content = panel_content.push(rule::horizontal(1));

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

        panel_content = panel_content.push(rule::horizontal(1));

        // ===== Wood Columns Section (Future) =====
        let columns_header = self.view_section_header_disabled("Wood Columns", 0);
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
    fn view_section_header_disabled(&self, title: &str, count: usize) -> Element<'_, Message> {
        row![
            text("▶").size(10).color([0.6, 0.6, 0.6]),
            Space::new().width(4),
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
                    Space::new().height(8),
                    labeled_input(
                        "Engineer:",
                        &self.project.meta.engineer,
                        Message::EngineerNameChanged
                    ),
                    labeled_input("Job ID:", &self.project.meta.job_id, Message::JobIdChanged),
                    labeled_input("Client:", &self.project.meta.client, Message::ClientChanged),
                    Space::new().height(20),
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

        container(scrollable(panel.width(Length::FillPortion(3)).padding(8)))
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
            Space::new().height(8),
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
                Space::new().height(4),
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
                Space::new().height(4),
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
            Space::new().height(8),
            text("Type:").size(11),
            pick_list(
                &MaterialType::ALL[..],
                Some(self.selected_material_type),
                Message::MaterialTypeSelected
            )
            .width(Length::Fill),
            Space::new().height(8),
            material_options,
        ]
        .spacing(2);

        // NDS Adjustment Factors section
        let adjustment_factors_section = self.view_adjustment_factors();

        // Only show Delete button for existing beams
        let action_buttons = if self.selected_beam_id().is_some() {
            row![
                button("Delete Beam")
                    .on_press(Message::DeleteSelectedBeam)
                    .padding(Padding::from([6, 12])),
            ]
            .spacing(6)
        } else {
            row![].spacing(6)
        };

        column![
            beam_section,
            Space::new().height(10),
            loads_section,
            Space::new().height(10),
            material_section,
            Space::new().height(10),
            adjustment_factors_section,
            Space::new().height(15),
            action_buttons,
        ]
    }

    fn view_adjustment_factors(&self) -> Element<'_, Message> {
        // Core factors that are commonly adjusted
        let core_factors = column![
            row![
                text("Load Duration:").size(10).width(Length::Fixed(100.0)),
                pick_list(
                    &LoadDuration::ALL[..],
                    Some(self.selected_load_duration),
                    Message::LoadDurationSelected
                )
                .width(Length::Fill)
                .text_size(10),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
            row![
                text("Wet Service:").size(10).width(Length::Fixed(100.0)),
                pick_list(
                    &WetService::ALL[..],
                    Some(self.selected_wet_service),
                    Message::WetServiceSelected
                )
                .width(Length::Fill)
                .text_size(10),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
            row![
                text("Repetitive:").size(10).width(Length::Fixed(100.0)),
                pick_list(
                    &RepetitiveMember::ALL[..],
                    Some(self.selected_repetitive_member),
                    Message::RepetitiveMemberSelected
                )
                .width(Length::Fill)
                .text_size(10),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        ]
        .spacing(4);

        // Less common factors (temperature, incising, flat use)
        let other_factors = column![
            row![
                text("Temperature:").size(10).width(Length::Fixed(100.0)),
                pick_list(
                    &Temperature::ALL[..],
                    Some(self.selected_temperature),
                    Message::TemperatureSelected
                )
                .width(Length::Fill)
                .text_size(10),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
            row![
                text("Incising:").size(10).width(Length::Fixed(100.0)),
                pick_list(
                    &Incising::ALL[..],
                    Some(self.selected_incising),
                    Message::IncisingSelected
                )
                .width(Length::Fill)
                .text_size(10),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
            row![
                text("Flat Use:").size(10).width(Length::Fixed(100.0)),
                pick_list(
                    &FlatUse::ALL[..],
                    Some(self.selected_flat_use),
                    Message::FlatUseSelected
                )
                .width(Length::Fill)
                .text_size(10),
            ]
            .spacing(4)
            .align_y(Alignment::Center),
        ]
        .spacing(4);

        // Beam stability (bracing)
        let bracing = checkbox(self.compression_edge_braced)
            .label("Compression edge braced (C_L = 1.0)")
            .on_toggle(Message::CompressionBracedToggled)
            .text_size(10);

        column![
            text("NDS Adjustment Factors").size(14),
            Space::new().height(6),
            core_factors,
            Space::new().height(6),
            other_factors,
            Space::new().height(6),
            bracing,
        ]
        .spacing(2)
        .into()
    }

    fn view_load_table(&self) -> Element<'_, Message> {
        let self_weight_checkbox = checkbox(self.include_self_weight)
            .label("Include self-weight")
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
            Space::new().height(6),
            self_weight_checkbox,
            Space::new().height(6),
            header,
            rule::horizontal(1),
            load_rows,
            Space::new().height(6),
            add_load_btn,
        ]
        .spacing(2)
        .into()
    }

    fn view_results_panel(&self) -> Element<'_, Message> {
        let content: Column<'_, Message> = if let Some(ref error) = self.error_message {
            column![
                text("Error").size(14),
                Space::new().height(8),
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
                .push(Space::new().height(15))
                .push(text("Diagrams").size(14))
                .push(Space::new().height(8))
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
            Space::new().height(8),
            text(format!("Engineer: {}", self.project.meta.engineer)).size(11),
            text(format!("Job ID: {}", self.project.meta.job_id)).size(11),
            text(format!("Client: {}", self.project.meta.client)).size(11),
            text(format!("Items: {}", item_count)).size(11),
            Space::new().height(15),
            text("Select an item from the list or create a new beam.").size(11),
            Space::new().height(8),
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
            Space::new().height(8),
            pass_fail,
            governing,
            Space::new().height(12),
            text("Load Summary").size(12),
            text(format!("Design Load: {:.1} plf{}", result.design_load_plf, self_weight_text)).size(11),
            text(format!("Governing Combo: {}", result.governing_combination)).size(11),
            Space::new().height(12),
            text("Demand").size(12),
            text(format!("Max Moment: {:.0} ft-lb", result.max_moment_ftlb)).size(11),
            text(format!("Max Shear: {:.0} lb", result.max_shear_lb)).size(11),
            text(format!(
                "Max Deflection: {:.3} in",
                result.max_deflection_in
            ))
            .size(11),
            Space::new().height(12),
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
            Space::new().height(12),
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
            Space::new().width(Length::Fill),
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
    #[allow(dead_code)]
    load_plf: f64,
    max_shear_lb: f64,
    max_moment_ftlb: f64,
    max_deflection_in: f64,
    // Pre-computed diagram points from analysis
    shear_diagram: Vec<(f64, f64)>,
    moment_diagram: Vec<(f64, f64)>,
    deflection_diagram: Vec<(f64, f64)>,
}

impl BeamDiagramData {
    fn from_calc(input: &BeamInput, result: &BeamResult) -> Self {
        Self {
            span_ft: input.span_ft,
            load_plf: result.design_load_plf,
            max_shear_lb: result.max_shear_lb,
            max_moment_ftlb: result.max_moment_ftlb,
            max_deflection_in: result.max_deflection_in,
            shear_diagram: result.shear_diagram.clone(),
            moment_diagram: result.moment_diagram.clone(),
            deflection_diagram: result.deflection_diagram.clone(),
        }
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
            align_x: iced::alignment::Horizontal::Center.into(),
            ..Text::default()
        };
        frame.fill_text(load_text);

        // Span label
        let span_text = Text {
            content: format!("L = {:.1} ft", self.data.span_ft),
            position: Point::new(x + width / 2.0, beam_y + support_size + 8.0),
            color,
            size: iced::Pixels(10.0),
            align_x: iced::alignment::Horizontal::Center.into(),
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

        // Draw shear diagram using pre-computed points
        if !self.data.shear_diagram.is_empty() && self.data.max_shear_lb.abs() > 1e-6 {
            // Find min/max shear for scaling
            let max_v = self.data.shear_diagram.iter()
                .map(|(_, v)| v.abs())
                .fold(0.0f64, |a, b| a.max(b));

            if max_v > 1e-6 {
                // Draw filled area
                let shear_path = Path::new(|builder| {
                    let first = &self.data.shear_diagram[0];
                    let px = x + (first.0 as f32 / self.data.span_ft as f32) * width;
                    let v_norm = first.1 / max_v;
                    let py = center_y - (v_norm as f32) * plot_height;
                    builder.move_to(Point::new(px, center_y));
                    builder.line_to(Point::new(px, py));

                    for (pos, v) in &self.data.shear_diagram {
                        let px = x + (*pos as f32 / self.data.span_ft as f32) * width;
                        let v_norm = v / max_v;
                        let py = center_y - (v_norm as f32) * plot_height;
                        builder.line_to(Point::new(px, py));
                    }

                    let last = self.data.shear_diagram.last().unwrap();
                    let px = x + (last.0 as f32 / self.data.span_ft as f32) * width;
                    builder.line_to(Point::new(px, center_y));
                    builder.close();
                });
                frame.fill(&shear_path, Color { a: 0.3, ..color });

                // Draw line
                let shear_line = Path::new(|builder| {
                    let first = &self.data.shear_diagram[0];
                    let px = x + (first.0 as f32 / self.data.span_ft as f32) * width;
                    let v_norm = first.1 / max_v;
                    let py = center_y - (v_norm as f32) * plot_height;
                    builder.move_to(Point::new(px, py));

                    for (pos, v) in &self.data.shear_diagram {
                        let px = x + (*pos as f32 / self.data.span_ft as f32) * width;
                        let v_norm = v / max_v;
                        let py = center_y - (v_norm as f32) * plot_height;
                        builder.line_to(Point::new(px, py));
                    }
                });
                frame.stroke(&shear_line, Stroke::default().with_color(color).with_width(2.0));
            }
        }

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

        // Draw moment diagram using pre-computed points
        if !self.data.moment_diagram.is_empty() && self.data.max_moment_ftlb.abs() > 1e-6 {
            let max_m = self.data.max_moment_ftlb;

            // Draw filled area
            let moment_path = Path::new(|builder| {
                builder.move_to(Point::new(x, axis_y));
                for (pos, m) in &self.data.moment_diagram {
                    let px = x + (*pos as f32 / self.data.span_ft as f32) * width;
                    let m_ratio = m / max_m;
                    let py = axis_y + (m_ratio as f32) * plot_height;
                    builder.line_to(Point::new(px, py));
                }
                builder.line_to(Point::new(x + width, axis_y));
                builder.close();
            });
            frame.fill(&moment_path, Color { a: 0.3, ..color });

            // Draw outline
            let outline = Path::new(|builder| {
                let first = &self.data.moment_diagram[0];
                let px = x + (first.0 as f32 / self.data.span_ft as f32) * width;
                let m_ratio = first.1 / max_m;
                let py = axis_y + (m_ratio as f32) * plot_height;
                builder.move_to(Point::new(px, py));

                for (pos, m) in &self.data.moment_diagram {
                    let px = x + (*pos as f32 / self.data.span_ft as f32) * width;
                    let m_ratio = m / max_m;
                    let py = axis_y + (m_ratio as f32) * plot_height;
                    builder.line_to(Point::new(px, py));
                }
            });
            frame.stroke(&outline, Stroke::default().with_color(color).with_width(2.0));
        }

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
            align_x: iced::alignment::Horizontal::Center.into(),
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

        // Draw deflection using pre-computed points
        if !self.data.deflection_diagram.is_empty() && self.data.max_deflection_in.abs() > 1e-9 {
            let max_d = self.data.max_deflection_in;

            // Draw curve
            let defl_path = Path::new(|builder| {
                let first = &self.data.deflection_diagram[0];
                let px = x + (first.0 as f32 / self.data.span_ft as f32) * width;
                let d_ratio = first.1 / max_d;
                let py = axis_y + (d_ratio as f32) * plot_height;
                builder.move_to(Point::new(px, py));

                for (pos, d) in &self.data.deflection_diagram {
                    let px = x + (*pos as f32 / self.data.span_ft as f32) * width;
                    let d_ratio = d / max_d;
                    let py = axis_y + (d_ratio as f32) * plot_height;
                    builder.line_to(Point::new(px, py));
                }
            });
            frame.stroke(&defl_path, Stroke::default().with_color(color).with_width(2.0));

            // Fill under curve
            let fill_path = Path::new(|builder| {
                builder.move_to(Point::new(x, axis_y));
                for (pos, d) in &self.data.deflection_diagram {
                    let px = x + (*pos as f32 / self.data.span_ft as f32) * width;
                    let d_ratio = d / max_d;
                    let py = axis_y + (d_ratio as f32) * plot_height;
                    builder.line_to(Point::new(px, py));
                }
                builder.line_to(Point::new(x + width, axis_y));
                builder.close();
            });
            frame.fill(&fill_path, Color { a: 0.2, ..color });
        }

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
            align_x: iced::alignment::Horizontal::Center.into(),
            ..Text::default()
        };
        frame.fill_text(max_label);
    }
}
