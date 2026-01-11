# 🐢 LofiTurtle Layout System Guide

LofiTurtle features a powerful, data-driven layout system that allows you to completely customize the Terminal User Interface (TUI). You can change the position of widgets, modify colors, adjust sizing, and remap keybindings using simple TOML configuration files.

## 🚀 How to Use

To launch LofiTurtle with a specific layout configuration:

```bash
# Using cargo
cargo run -- --layout-config path/to/your_layout.toml

# Using the binary (after build)
./lofiturtle --layout-config path/to/your_layout.toml
```

If no file is specified, LofiTurtle looks for `layout.toml` in the current directory. If that is missing, it falls back to the built-in default layout.

---

## 📄 Configuration Structure

A layout file consists of several sections:
1. **Metadata**: Basic info about the layout.
2. **Theme**: Color schemes and styles.
3. **Widgets**: The building blocks of the UI.
4. **Keybindings**: Keyboard shortcuts.
5. **Settings**: General behavior settings.

### 1. Metadata
```toml
version = "1.0"
name = "My Custom Layout"
description = "A description of this layout"
```

### 2. Theme
Define the color palette for the application. You can use color names (e.g., "red", "blue") or Hex codes (e.g., "#ff00ff").

```toml
[theme]
name = "my_theme"

[theme.colors]
primary = "#00ff99"      # Main accent color (selection, titles)
secondary = "#bd93f9"    # Secondary accent
background = "#111111"   # App background
foreground = "#eeeeee"   # Default text color
border = "#444444"       # Border color
highlight = "#ffffff"    # Highlighted text
error = "#ff5555"        # Error messages
success = "#50fa7b"      # Success messages
```

### 3. Widgets
Widgets are defined as an array of tables using `[[widgets]]`. The order matters for rendering, but positioning is determined by the `position` field.

**Available Fields:**
*   `name`: Unique identifier for the widget.
*   `type`: The kind of widget (see list below).
*   `position`: Where to place it (`top`, `bottom`, `left`, `right`, `center`).
*   `size`: How big it should be.
    *   `{ percentage = 50 }`: Takes up 50% of available space.
    *   `{ length = 5 }`: Fixed height/width of 5 rows/columns.
    *   `"fill"`: Takes up all remaining space.
*   `visible`: `true` or `false`.
*   `border`: `true` to draw a border around it.
*   `title`: Optional title displayed on the border.

**Available Widget Types:**
*   `sidebar`: Library navigation.
*   `playlist_view`: List of songs in the current playlist/queue.
*   `now_playing`: Player controls and song info.
*   `progress_bar`: Seek bar.
*   `status_bar`: Bottom status line.
*   `album_art`: ASCII/Block art display.
*   `search_box`: Search input field.

**Example:**
```toml
[[widgets]]
name = "sidebar"
type = "sidebar"
position = "left"
size = { percentage = 25 }
visible = true
border = true
title = "Library"

[[widgets]]
name = "main_view"
type = "playlist_view"
position = "center"
size = "fill"
visible = true
border = true
title = "Queue"
```

### 4. Keybindings
Map keyboard keys to application actions.

**Common Actions:**
*   `quit`: Exit the app.
*   `toggle_play`: Play/Pause.
*   `next_track`: Skip to next song.
*   `previous_track`: Go to previous song.
*   `volume_up` / `volume_down`: Adjust volume.
*   `search`: Enter search mode.
*   `toggle_art`: Show/Hide album art.
*   `switch_theme`: Cycle through themes.

**Example:**
```toml
[keybindings]
space = "toggle_play"
q = "quit"
"/" = "search"
n = "next_track"
"+" = "volume_up"
"-" = "volume_down"
```

### 5. Settings
General application settings.

```toml
[settings]
auto_save = true
debounce_ms = 300

[settings.responsive]
# Terminal width breakpoints for responsive adjustments
small_width = 80
medium_width = 120
large_width = 160
```

---

## 🧩 Layout Logic

The layout engine processes widgets in the following order:
1.  **Top**: Widgets with `position = "top"` are stacked vertically at the top.
2.  **Bottom**: Widgets with `position = "bottom"` are stacked vertically at the bottom.
3.  **Middle**: The remaining space is split horizontally:
    *   **Left**: Widgets with `position = "left"`.
    *   **Center**: Widgets with `position = "center"`.
    *   **Right**: Widgets with `position = "right"`.

---

## 💡 Example: Minimalist Layout

```toml
version = "1.0"
name = "Minimal"

[theme.colors]
primary = "cyan"
background = "black"
foreground = "white"
border = "dark_gray"

[[widgets]]
name = "status"
type = "now_playing"
position = "top"
size = { length = 3 }
visible = true
border = true

[[widgets]]
name = "list"
type = "playlist_view"
position = "center"
size = "fill"
visible = true
border = false
```
