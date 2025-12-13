# Graphical User Interface

uv includes an optional graphical user interface (GUI) that provides an intuitive way to manage
Python packages, virtual environments, and Python installations.

## Overview

The uv GUI is built with [GPUI](https://gpui.rs), Zed's GPU-accelerated UI framework, providing a
fast and responsive native application experience.

## Features

The GUI provides access to the following features:

- **Project Overview**: View and manage project dependencies, see project metadata, and sync/lock
  operations
- **Package Browser**: Search PyPI for packages, view package details, and install/remove packages
- **Environment Management**: Create, activate, and delete virtual environments
- **Python Version Manager**: Install, uninstall, and set default Python versions
- **Settings**: Configure uv preferences including cache directory, Python preferences, and network
  settings

## Installation

The GUI feature is optional and must be enabled during compilation:

```console
$ cargo build --features gui
```

## Usage

### Launching the GUI

You can launch the GUI using the `uv gui` command:

```console
$ uv gui
```

### Command Options

```console
$ uv gui --help
Launch the graphical user interface

Usage: uv gui [OPTIONS] [PATH]

Arguments:
  [PATH]  The path to a project directory to open in the GUI

Options:
      --view <VIEW>  Open the GUI in a specific view (project, packages, environments, python, settings)
```

### Opening a Specific Project

To open a specific project directory:

```console
$ uv gui /path/to/project
```

### Opening a Specific View

To open the GUI in a specific view:

```console
$ uv gui --view packages
```

## GUI Components

### Sidebar

The sidebar provides navigation between the main views:

- **Project**: Overview of the current project
- **Packages**: Package browser and search
- **Environments**: Virtual environment management
- **Python**: Python version management
- **Settings**: Application settings

### Project View

The project view displays:

- Project name and location
- Quick action buttons (Sync, Lock, Run)
- Statistics cards showing dependency counts and Python version
- Lists of dependencies and development dependencies
- Update availability indicators

### Package Browser

The package browser allows you to:

- Search PyPI for packages
- View package descriptions and versions
- Install packages with one click
- Remove installed packages
- See which packages have updates available

### Environment Management

The environments view shows:

- All discovered virtual environments
- Environment details (Python version, package count, disk size)
- Active environment indicator
- Create new environments
- Activate or delete existing environments

### Python Version Manager

The Python view allows you to:

- See all installed Python versions
- View which Python is the system default
- Install new Python versions managed by uv
- Uninstall managed Python versions
- Set a Python version as the default

### Settings

The settings view provides configuration for:

- Python preference (managed vs system)
- Color output preferences
- Preview features toggle
- Offline mode
- Native TLS settings
- Cache directory location

## Keyboard Shortcuts

| Shortcut       | Action               |
| -------------- | -------------------- |
| `Cmd/Ctrl + Q` | Quit the application |
| `Cmd/Ctrl + ,` | Open settings        |
| `Cmd/Ctrl + R` | Refresh current view |

## Theming

The GUI uses the Catppuccin Mocha color scheme by default, providing a comfortable dark theme
optimized for extended use.

## Building from Source

To build uv with GUI support:

```console
$ git clone https://github.com/astral-sh/uv
$ cd uv
$ cargo build --release --features gui
```

The `uv-gui` binary will be available in `target/release/`.

## Platform Support

The GUI is supported on:

- **macOS**: Metal rendering backend
- **Linux**: Wayland and X11 support
- **Windows**: Native Windows rendering

## Troubleshooting

### GUI doesn't start

Ensure you have the required graphics drivers installed and that your system supports
GPU-accelerated rendering.

### Display issues

If you experience rendering issues, try setting the `GPUI_LOG_LEVEL` environment variable:

```console
$ GPUI_LOG_LEVEL=debug uv gui
```

### Building fails

The GUI requires additional system dependencies. On Linux, you may need:

```console
# Ubuntu/Debian
$ sudo apt-get install libxkbcommon-dev libwayland-dev

# Fedora
$ sudo dnf install libxkbcommon-devel wayland-devel
```
