# Fyrox Blueprint Visual Scripting - UI/UX Styling Guide

## Overview

This document describes the visual styling improvements made to the Blueprint visual scripting editor to match Unreal Engine's iconic node-based interface design.

## Design Philosophy

The visual scripting interface follows these key principles inspired by Unreal Engine:

1. **Color-Coded Data Types**: Each data type has a distinct, vibrant color for quick visual identification
2. **Categorical Header Colors**: Node categories use specific header colors for visual organization
3. **Clear Pin Styling**: Larger, more prominent socket designs for better visibility
4. **Enhanced Connection Lines**: Thicker lines with matching colors to their connected pins
5. **Dark Theme**: Professional dark background for code-like editing experience

## Color Palette

### Data Type Colors

| Data Type  | Color RGB       | Purpose                             |
| ---------- | --------------- | ----------------------------------- |
| **Exec**   | (255, 255, 255) | White - Execution flow pins         |
| **Bool**   | (220, 96, 96)   | Warm Red - Boolean values           |
| **I32**    | (100, 220, 255) | Bright Cyan - Integer values        |
| **F32**    | (120, 220, 100) | Bright Green - Float/Decimal values |
| **String** | (255, 100, 220) | Hot Pink/Magenta - Text values      |
| **Unit**   | (160, 160, 160) | Light Gray - Empty/No value         |

### Node Category Header Colors

| Category         | Color RGB       | Example Nodes                        |
| ---------------- | --------------- | ------------------------------------ |
| **Events**       | (220, 64, 64)   | BeginPlay, Tick, Construction Script |
| **Flow Control** | (110, 110, 110) | Branch, Switch, Loop                 |
| **Utilities**    | (64, 180, 200)  | Print, Log, Debug                    |
| **Variables**    | (64, 180, 96)   | Get Variable, Set Variable           |
| **Math**         | (80, 220, 80)   | Add, Multiply, Divide                |
| **String**       | (255, 100, 220) | Concat, Format, Compare              |
| **Custom**       | (120, 120, 120) | User-defined nodes                   |

## Visual Elements

### Nodes

**Header Design:**

-   Category-specific colored header bar
-   Bold white display name for readability
-   Darker selected state with +60 brightness increase
-   Rounded corners for modern appearance

**Socket Pins:**

-   Margin: 5.0 units (increased from 3.0 for better visibility)
-   Color matches the data type
-   Larger hit area for easier connection
-   Clear Input/Output distinction (left/right positioning)

**Content Area:**

-   Padding: 6.0 units
-   Minimum height: 28.0 units for text inputs
-   Clean, professional appearance

### Connection Lines

**Styling:**

-   **Exec Pins**: 7.0 pixel thickness, pure white
-   **Data Pins**: 5.0 pixel thickness, data-type colored
-   Smooth bezier curves for organic feel
-   Hover state: +50 brightness for visual feedback

**Color Matching:**

-   Every connection line uses the color of its corresponding data type
-   Provides visual continuity from pin to pin
-   Makes data flow instantly recognizable

### Canvas Background

**Grid System:**

-   **Minor Grid**: 16.0 unit spacing, color (38, 38, 38) - subtle reference grid
-   **Major Grid**: 80.0 unit spacing, color (52, 52, 52) - stronger alignment grid
-   **Background**: Dark gray (28, 28, 28) - reduced eye strain

## Implementation Details

### File Locations

1. **Blueprint Editor Styling**: `/editor/src/plugins/blueprint/mod.rs`

    - Node header colors
    - Pin socket styling
    - Connection line rendering
    - Pin label margins and alignment

2. **Core Node Definitions**: `/fyrox-visual-scripting/src/nodes/mod.rs`

    - Category header colors
    - Pin colors for node palette

3. **Canvas Rendering**: `/editor/src/plugins/absm/canvas.rs`
    - Background styling
    - Grid rendering

### Key Functions

**Node Creation:**

```rust
// Enhanced node styling in rebuild_graph_view_for_view()
// Location: editor/src/plugins/blueprint/mod.rs:646-820
- Header color selection by node type
- Socket margin increased to 5.0
- Selected state calculation with safe saturation
```

**Connection Rendering:**

```rust
// Enhanced connection colors in spawn_connection_view()
// Location: editor/src/plugins/blueprint/mod.rs:1897-1956
- Dynamic color selection by data type
- Thickness variation for exec vs data pins
- Hover color calculation with saturation bounds
```

**Color Functions:**

```rust
// Node category colors in nodes/mod.rs
pub fn header_color(&self) -> (u8, u8, u8)

// Pin colors in nodes/mod.rs
pub fn pin_color_for_type(data_type: DataType) -> (u8, u8, u8)
```

## User Experience Improvements

### Visual Clarity

-   **Type Identification**: Users instantly recognize data types by color
-   **Data Flow**: Following colored wires shows information flow
-   **Node Purpose**: Header colors indicate node category at a glance

### Professional Appearance

-   **Dark Theme**: Reduces eye strain during long editing sessions
-   **Modern Colors**: Vibrant yet balanced color scheme
-   **Clear Hierarchy**: Header colors establish visual priority

### Consistency with Unreal Engine

-   **Familiar Interface**: Blueprint users transition easily
-   **Intuitive Colors**: Standard data type color mapping
-   **Grid-Based Layout**: Encourages organized node graphs

## Customization Guidelines

To modify styling, update these key color definitions:

### To Change a Data Type Color:

**File**: `/fyrox-visual-scripting/src/nodes/mod.rs`

```rust
DataType::YourType => (R, G, B),  // Change (R, G, B) values
```

### To Change a Category Color:

**File**: `/fyrox-visual-scripting/src/nodes/mod.rs`

```rust
NodeCategory::YourCategory => (R, G, B),  // Change (R, G, B) values
```

### To Adjust Pin Spacing:

**File**: `/editor/src/plugins/blueprint/mod.rs`
Search for: `.with_margin(Thickness::uniform(5.0))`
Change `5.0` to your desired margin value.

### To Adjust Line Thickness:

**File**: `/editor/src/plugins/blueprint/mod.rs`
Search for: `.with_thickness(if is_exec { 7.0 } else { 5.0 })`
Change the values to your desired thickness.

## Testing

To verify the styling improvements:

1. Build the editor: `cargo run --bin fyroxed --profile=editor-standalone`
2. Open or create a Blueprint asset
3. Verify:
    - Node headers have correct category colors
    - Pins/sockets display correct data type colors
    - Connection lines match their pin colors
    - Hover effects brighten appropriately
    - Grid background is visible but subtle

## Future Enhancements

Potential improvements for future versions:

1. **Custom Color Schemes**: Allow users to customize colors
2. **Pin Labels**: Display pin name hints on hover
3. **Node Previews**: Show evaluated values on nodes
4. **Dark/Light Theme Toggle**: Support both dark and light themes
5. **Category Filtering**: Show/hide nodes by category in palette
6. **Advanced Pin Types**: Add more data types with custom colors
7. **Animation Feedback**: Animate execution flow through nodes
8. **Accessibility**: High contrast mode option

## References

-   Unreal Engine Blueprint Documentation: https://docs.unrealengine.com/
-   Fyrox Engine: https://github.com/fyrox-rs/Fyrox
-   Visual Design Best Practices in Node-Based Interfaces

---

**Last Updated**: December 16, 2025
**Version**: 1.0
