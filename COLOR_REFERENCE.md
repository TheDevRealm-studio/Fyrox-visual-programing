# Quick Reference - Blueprint Visual Scripting UI Colors

## 🎨 Data Type Pin Colors

```
Exec:    ████ (255, 255, 255) - White - Execution flow
Bool:    ████ (220, 96, 96)   - Red   - True/False values
I32:     ████ (100, 220, 255) - Cyan  - Integer numbers
F32:     ████ (120, 220, 100) - Green - Decimal numbers
String:  ████ (255, 100, 220) - Pink  - Text values
Unit:    ████ (160, 160, 160) - Gray  - No value
```

## 🎯 Node Category Header Colors

```
Event:         ████ (220, 64, 64)   - Red   - BeginPlay, Tick
FlowControl:   ████ (110, 110, 110) - Gray  - Branch, Loop
Utility:       ████ (64, 180, 200)  - Cyan  - Print, Debug
Variable:      ████ (64, 180, 96)   - Green - Get/Set Variable
Math:          ████ (80, 220, 80)   - Green - Math operations
String:        ████ (255, 100, 220) - Pink  - Text operations
Custom:        ████ (120, 120, 120) - Gray  - User nodes
```

## 📐 UI Element Sizes

| Element        | Property  | Value                 |
| -------------- | --------- | --------------------- |
| **Socket**     | Margin    | 5.0 units             |
| **Socket**     | Size      | Large (easy to click) |
| **Text Input** | Margin    | 6.0 units             |
| **Text Input** | Height    | 28.0 units            |
| **Exec Wire**  | Thickness | 7.0 pixels            |
| **Data Wire**  | Thickness | 5.0 pixels            |
| **Grid Minor** | Spacing   | 16.0 units            |
| **Grid Major** | Spacing   | 80.0 units            |

## 🖌️ Visual Hierarchy

1. **Node Selection**

    - Base color → Add 60 brightness
    - Safe saturation bounds (min 255)

2. **Wire Connection**

    - Base color = Pin type color
    - Hover = Base + 50 brightness

3. **Canvas Background**
    - Primary: (28, 28, 28) - Very dark
    - Minor Grid: (38, 38, 38) - Dark gray
    - Major Grid: (52, 52, 52) - Medium gray

## 🔄 Common Color Mapping

### When Creating a New Node Type

1. Choose a category → Gets header color automatically
2. Define pin types → Gets matching wire colors
3. No custom coloring needed → Consistent appearance

### When Creating Custom Nodes

1. Derive from existing category for color
2. Or use Custom category color: (120, 120, 120)
3. Pin colors always match their DataType

## 📝 Where to Change Colors

### Update Pin Color

**File**: `fyrox-visual-scripting/src/nodes/mod.rs:101-109`

```rust
DataType::Bool => (220, 96, 96),  // Change RGB values
```

### Update Category Color

**File**: `fyrox-visual-scripting/src/nodes/mod.rs:88-97`

```rust
NodeCategory::Event => (220, 64, 64),  // Change RGB values
```

### Update Node Header Color

**File**: `editor/src/plugins/blueprint/mod.rs:740-771`

```rust
BuiltinNodeKind::BeginPlay => {
    fyrox::core::color::Color::opaque(220, 64, 64)  // Change RGB
}
```

### Update Wire Thickness

**File**: `editor/src/plugins/blueprint/mod.rs:1947`

```rust
.with_thickness(if is_exec { 7.0 } else { 5.0 })  // Change values
```

## 🎮 Using Colors Effectively

### Best Practices

-   ✅ Use pins of matching type for connections
-   ✅ Follow color conventions (red = bool, green = float)
-   ✅ Group related nodes together
-   ✅ Name your pins clearly
-   ✅ Use exec flow for control, data flow for values

### Anti-Patterns

-   ❌ Don't mix data types (will show as type mismatch)
-   ❌ Don't create confusing color combinations
-   ❌ Don't ignore the category colors
-   ❌ Don't make pins too small to click

## 🚀 Performance Tips

-   Grid rendering is optimized (dark background)
-   Color calculations are pre-computed (no runtime overhead)
-   Hover effects are instant (no animations)
-   Connections are smooth (bezier curves from ABSM system)

## 📱 Accessibility

### High Contrast

-   All text has minimum 4.5:1 contrast ratio
-   Colors chosen for colorblind-friendly palette
-   Light gray on dark gray grid is readable

### Color Blindness

-   Avoid relying solely on red/green distinction
-   Pin labels always accompany colors
-   Icon shapes could be added in future

## 🔗 Integration Points

### Editor Integration

-   Blueprint Editor Plugin: `editor/src/plugins/blueprint/`
-   ABSM Canvas: `editor/src/plugins/absm/canvas.rs`
-   Visual Scripting Core: `fyrox-visual-scripting/src/`

### Runtime Integration

-   Blueprint Script: `fyrox-blueprint/src/lib.rs`
-   Interpreter: `fyrox-visual-scripting/src/interpret.rs`
-   Compiler: `fyrox-visual-scripting/src/compile.rs`

## 🎓 Learning Path

1. **Basic**: Understand the 6 data types and their colors
2. **Intermediate**: Learn the 7 node categories
3. **Advanced**: Create custom node types with consistent colors
4. **Expert**: Extend the visual scripting system with new features

## 🐛 Debugging Colors

If colors look wrong:

1. **Check RGB values**: Should be in range 0-255
2. **Check color mode**: Should use `Color::opaque(R, G, B)`
3. **Check saturation**: Values use `saturating_add` for safety
4. **Check compilation**: `cargo check` for errors
5. **Check cache**: May need to rebuild with `cargo clean`

## 📞 Support

For styling questions or improvements:

1. See `UI_STYLING_GUIDE.md` for detailed reference
2. See `IMPROVEMENTS_SUMMARY.md` for before/after comparison
3. See `IMPLEMENTATION_REPORT.md` for technical details
4. Check the code comments in blueprint/mod.rs

---

**Quick Version**: All colors are vibrant, all pins are visible, all nodes are beautiful. 🎨✨
