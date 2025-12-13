//! Entry point for the uv graphical user interface.
//!
//! This binary provides a native GUI for uv, allowing users to manage
//! Python packages, virtual environments, and Python installations
//! through an intuitive graphical interface.
//!
//! Built with GPUI, Zed's GPU-accelerated UI framework.

use std::process::ExitCode;

fn main() -> ExitCode {
    uv_gui::UvGuiApp::run();
    ExitCode::SUCCESS
}
