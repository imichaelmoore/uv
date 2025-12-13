//! Main window view container.

use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, Render, Styled, Window, div, rgb,
};

/// The main window container that manages the overall layout.
pub struct MainWindow {
    #[allow(dead_code)]
    focus_handle: FocusHandle,
}

impl MainWindow {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("main-window-container")
            .size_full()
            .bg(rgb(0x181825))
            .text_color(rgb(0xcdd6f4))
    }
}
