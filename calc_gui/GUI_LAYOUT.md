# Stratify GUI Layout

This document describes the GUI layout and component naming conventions for the Stratify application.

## Overview

```
+-----------------------------------------------------------------------------------+
|                                 TOOLBAR                                            |
|  [New] [Open] [Save] [Save As]                              [Export PDF]          |
+-----------------------------------------------------------------------------------+
|          |                              |                                         |
|  ITEMS   |         INPUT PANEL          |            RESULTS PANEL                |
|  PANEL   |                              |                                         |
|          |                              |                                         |
| +------+ | +-------------------------+  | +------------------------------------+  |
| |Project| | |  Editor content based  |  | |  Results display                   |  |
| |Info   | | |  on selection:         |  | |  - Pass/Fail status               |  |
| +------+ | |                         |  | |  - Load summary                   |  |
| |Wood   | | |  - Project Info fields |  | |  - Demand values                  |  |
| |Beams  | | |    (Engineer, Job, etc)|  | |  - Capacity checks                |  |
| | - B-1 | | |                         |  | |  - Section properties             |  |
| | - B-2 | | |  - Beam editor          |  | |                                    |  |
| | [+]   | | |    (Label, Span, etc)  |  | |  Diagrams:                         |  |
| +------+ | |    Load table           |  | |  - Beam schematic                  |  |
| |Columns| | |    Material selection  |  | |  - Shear diagram                   |  |
| |(future)| | |    Action buttons      |  | |  - Moment diagram                  |  |
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
