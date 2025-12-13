//! Launch the graphical user interface.

use anyhow::Result;
use uv_cli::GuiArgs;

use crate::commands::ExitStatus;
use crate::printer::Printer;

/// Launch the uv graphical user interface.
pub(crate) fn gui(_args: GuiArgs, printer: Printer) -> Result<ExitStatus> {
    // Check if GUI feature is enabled
    #[cfg(feature = "gui")]
    {
        writeln!(
            printer.stderr(),
            "Launching uv GUI..."
        )?;

        // Launch the GUI application
        uv_gui::UvGuiApp::run();

        Ok(ExitStatus::Success)
    }

    #[cfg(not(feature = "gui"))]
    {
        use std::fmt::Write;

        writeln!(
            printer.stderr(),
            "The GUI feature is not enabled in this build of uv."
        )?;
        writeln!(
            printer.stderr(),
            "To use the GUI, rebuild uv with the `gui` feature:"
        )?;
        writeln!(
            printer.stderr(),
            "  cargo build --features gui"
        )?;

        Ok(ExitStatus::Failure)
    }
}
