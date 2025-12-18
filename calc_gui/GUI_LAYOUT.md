# Stratify GUI Layout

This document describes the GUI layout and component naming conventions for the Stratify application.

## Overview

```
+-----------------------------------------------------------------------------------+
|                                 TOOLBAR                                           |
|  [New] [Open] [Save] [Save As] [Export PDF]                          [Settings]   |
+-----------------------------------------------------------------------------------+
|          |                              |                                         |
|  ITEMS   |         INPUT PANEL          |            RESULTS PANEL                |
|  PANEL   |                              |                                         |
|          |                              |                                         |
| +------+ | +-------------------------+  | +------------------------------------+  |
| |Project| | |  Editor content based  |  | |  Results display                   |  |
| |Info   | | |  on selection:         |  | |  - Pass/Fail status                |  |
| +------+ | |                         |  | |  - Load summary                    |  |
| |Wood   | | |  - Project Info fields |  | |  - Demand values                   |  |
| |Beams  | | |    (Engineer, Job, etc)|  | |  - Capacity checks                 |  |
| | - B-1 | | |                         |  | |  - Section properties             |  |
| | - B-2 | | |  - Beam editor          |  | |                                   |  |
| | [+]   | | |    (Label, Span, etc)  |  | |  Diagrams:                         |  |
| +------+ | |    Load table           |  | |  - Beam schematic                  |  |
| |Columns| | |    Material selection  |  | |  - Shear diagram                   |  |
| |(future)| | |    Action buttons      |  | |  - Moment diagram                 |  |
| +------+ | +-------------------------+  | |  - Deflection diagram              |  |
|          |                              | +------------------------------------+  |
+----------+------------------------------+-----------------------------------------+
|                                STATUS BAR                                         |
+-----------------------------------------------------------------------------------+
```

## Component Naming

### Toolbar
Location: Top of window
Contains: File operations and export actions
- New Project
- Open Project
- Save Project
- Save As
- Export PDF

### Items Panel
Location: Left side
Purpose: Navigation and item management
Contains:
- **Project Info Section** (collapsible)
  - Displays summary: Engineer, Job ID, Client
  - Clickable to select for editing
- **Wood Beams Section** (collapsible)
  - Shows count: "Wood Beams (N)"
  - Lists all beam items by label
  - **[+] Button**: Creates new beam immediately
  - Click beam to select for editing
- **Future Sections** (grayed out)
  - Wood Columns
  - Continuous Footings
  - Spread Footings
  - Cantilever Walls
  - Restrained Walls

### Input Panel
Location: Center
Purpose: Edit selected item properties
Shows different content based on selection:

**When Project Info selected:**
- Engineer name field
- Job ID field
- Client field
- Helper text for navigation

**When Beam selected:**
- Beam properties section
  - Label
  - Span (ft)
  - Width (in)
  - Depth (in)
- Loads section
  - Include self-weight checkbox
  - Load table (Type, Distribution, Magnitude, Position, Tributary)
  - Add/Remove load buttons
- Material section
  - Material type dropdown
  - Type-specific options (Species/Grade, Stress Class, etc.)
- NDS Adjustment Factors section
  - Load Duration (C_D)
  - Wet Service (C_M)
  - Repetitive Member (C_r)
  - Temperature (C_t)
  - Incising (C_i)
  - Flat Use (C_fu)
  - Compression edge braced checkbox (C_L)
- Action buttons
  - Delete Beam

### Results Panel
Location: Right side
Purpose: Display calculation results and diagrams
Contains:
- **Results Section**
  - Pass/Fail status with color coding
  - Governing condition
  - Load summary (design load, governing combo)
  - Demand values (moment, shear, deflection)
  - Capacity checks with unity ratios
  - Section properties
- **Diagrams Section**
  - Beam schematic with supports and reactions
  - Shear force diagram
  - Bending moment diagram
  - Deflection diagram

### Status Bar
Location: Bottom of window
Purpose: Display status messages and feedback
Shows:
- Current operation status
- File path when saved
- Lock status for shared files
- Error messages

## Interaction Flow

1. **Creating Items**
   - Click [+] in Items Panel section header
   - New item created immediately with default values
   - Item appears in list and is selected for editing

2. **Editing Items**
   - Click item in Items Panel to select
   - Input Panel shows item's properties
   - Changes update Results Panel in real-time

3. **Live Preview**
   - Results Panel updates as you type
   - No "Calculate" button needed
   - Results show immediately with current input values

4. **Deleting Items**
   - Select item in Items Panel
   - Click "Delete" in Input Panel
   - Item removed from project

## Color Coding

- **Primary (Blue)**: Selected items, active buttons
- **Secondary (Gray)**: Unselected items, inactive elements
- **Success (Green)**: Passing checks, adequate design
- **Error (Red)**: Failing checks, errors, inadequate design
- **Muted (Light Gray)**: Disabled/future features, helper text

## File Structure

The GUI is organized into panels with child modules that correspond to each selectable item type.
When an item is selected in the **Items Panel**, the **Input Panel** and **Results Panel** load
the matching child module pair (e.g., `input_wood_beam.rs` + `result_wood_beam.rs`).

```
calc_gui/src/
├── main.rs               <-- Entry point, App state, Message enum, Update loop
│
└── ui/
    ├── mod.rs            <-- Exposes all ui modules
    │
    │   # ─────────────────────────────────────────────────────────────
    │   # Top-Level Panels
    │   # ─────────────────────────────────────────────────────────────
    ├── toolbar.rs            <-- File/Export buttons, Settings
    ├── items_panel.rs        <-- Left sidebar: lists Project Info, Beams, Columns, etc.
    ├── input_panel.rs        <-- Center panel: dispatches to correct input_*.rs child
    ├── results_panel.rs      <-- Right panel: dispatches to correct result_*.rs child
    ├── status_bar.rs         <-- Bottom status messages
    │
    │   # ─────────────────────────────────────────────────────────────
    │   # Input Panel Children (one per item type)
    │   # ─────────────────────────────────────────────────────────────
    ├── input_project_info.rs <-- Engineer, Job ID, Client fields
    ├── input_wood_beam.rs    <-- Beam properties, loads, material, NDS factors
    ├── input_wood_column.rs  <-- (future) Column properties, loads, material
    │   # ... add input_*.rs for each new item type
    │
    │   # ─────────────────────────────────────────────────────────────
    │   # Results Panel Children (one per item type)
    │   # ─────────────────────────────────────────────────────────────
    ├── result_project_info.rs <-- (optional) Project summary / cover sheet preview
    ├── result_wood_beam.rs    <-- Pass/Fail, demand/capacity, diagrams
    ├── result_wood_column.rs  <-- (future) Column results, diagrams
    │   # ... add result_*.rs for each new item type
    │
    │   # ─────────────────────────────────────────────────────────────
    │   # Shared Components (reusable across input/result modules)
    │   # ─────────────────────────────────────────────────────────────
    └── shared/
        ├── mod.rs             <-- Exposes shared components
        ├── load_table.rs      <-- Load input table (used by beams, columns, footings)
        ├── material_picker.rs <-- Material type + species/grade dropdowns
        ├── nds_factors.rs     <-- NDS adjustment factor controls (C_D, C_M, etc.)
        └── diagrams.rs        <-- Canvas drawing utilities (shear, moment, deflection)
```

### How It Works

1. **Items Panel** (`items_panel.rs`)  
   Renders the navigation tree (Project Info, Wood Beams, Wood Columns, etc.).  
   When the user clicks an item, it updates the `EditorSelection` in `App` state.

2. **Input Panel** (`input_panel.rs`)  
   Acts as a dispatcher. Based on `EditorSelection`, it calls:
   - `input_project_info::view(...)` for Project Info
   - `input_wood_beam::view(...)` for a Wood Beam
   - `input_wood_column::view(...)` for a Wood Column
   - etc.

3. **Results Panel** (`results_panel.rs`)  
   Acts as a dispatcher. Based on `EditorSelection`, it calls:
   - `result_project_info::view(...)` (or shows a placeholder)
   - `result_wood_beam::view(...)` for beam results + diagrams
   - `result_wood_column::view(...)` for column results
   - etc.

4. **Shared Components** (`shared/`)  
   Contains reusable widgets:
   - `load_table.rs` — The multi-row load input table (Type, Distribution, Magnitude, etc.)
   - `material_picker.rs` — Material type dropdown + sub-options (species/grade, glulam class, etc.)
   - `nds_factors.rs` — All NDS adjustment factor controls
   - `diagrams.rs` — Canvas drawing helpers for shear/moment/deflection diagrams

   Each `input_*.rs` module imports what it needs from `shared/`.

### Adding a New Item Type

To add support for a new calculation type (e.g., Spread Footings):

1. Create `input_spread_footing.rs` — form fields for footing properties
2. Create `result_spread_footing.rs` — results display + any diagrams
3. Add `SpreadFooting` variant to `EditorSelection` enum in `main.rs`
4. Update `input_panel.rs` and `results_panel.rs` dispatchers
5. Update `items_panel.rs` to enable the section and show footing items
6. Reuse `shared/load_table.rs` if footings have applied loads