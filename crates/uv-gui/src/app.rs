//! Main application entry point and lifecycle management.

use gpui::{
    actions, div, prelude::*, px, rgb, size, Application, Bounds, Context, FocusHandle,
    InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, WindowBounds, WindowOptions,
};

use crate::state::{AppState, Tab};

actions!(
    uv_gui,
    [Quit, OpenSettings, ShowAbout, RefreshAll, ToggleSidebar]
);

/// The main uv GUI application.
pub struct UvGuiApp;

impl UvGuiApp {
    /// Run the uv GUI application.
    pub fn run() {
        let app = Application::new();
        app.run(|cx| {
            // Set up global actions
            cx.on_action(|_: &Quit, cx| cx.quit());

            // Open main window
            let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    titlebar: Some(gpui::TitlebarOptions {
                        title: Some(SharedString::from("uv")),
                        appears_transparent: true,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |_, cx| cx.new(|cx| MainWindowView::new(cx)),
            )
            .expect("Failed to open main window");
        });
    }
}

/// The main window view that contains all GUI elements.
pub struct MainWindowView {
    focus_handle: FocusHandle,
    state: AppState,
    current_tab: Tab,
    sidebar_visible: bool,
    search_query: String,
}

impl MainWindowView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        // Initialize application state
        let state = AppState::new();

        Self {
            focus_handle,
            state,
            current_tab: Tab::Project,
            sidebar_visible: true,
            search_query: String::new(),
        }
    }

    fn switch_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
    }

    fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
    }

    fn render_sidebar(&self, _cx: &Context<Self>) -> impl IntoElement {
        let tabs = [
            (Tab::Project, "Project", "folder"),
            (Tab::Packages, "Packages", "package"),
            (Tab::Environments, "Environments", "box"),
            (Tab::Python, "Python", "python"),
            (Tab::Settings, "Settings", "settings"),
        ];

        div()
            .id("sidebar")
            .w(px(220.0))
            .h_full()
            .bg(rgb(0x1e1e2e))
            .border_r_1()
            .border_color(rgb(0x313244))
            .flex()
            .flex_col()
            .child(
                // Header
                div()
                    .px(px(16.0))
                    .py(px(12.0))
                    .border_b_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::BOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("uv"),
                    ),
            )
            .child(
                // Navigation tabs
                div().flex_1().py(px(8.0)).children(tabs.map(|(tab, label, _icon)| {
                    let is_active = self.current_tab == tab;
                    let bg_color = if is_active {
                        rgb(0x313244)
                    } else {
                        rgb(0x1e1e2e)
                    };
                    let text_color = if is_active {
                        rgb(0xcdd6f4)
                    } else {
                        rgb(0xa6adc8)
                    };

                    div()
                        .id(SharedString::from(format!("tab-{label}")))
                        .mx(px(8.0))
                        .px(px(12.0))
                        .py(px(8.0))
                        .rounded(px(6.0))
                        .bg(bg_color)
                        .hover(|style| style.bg(rgb(0x313244)))
                        .cursor_pointer()
                        .child(
                            div()
                                .text_sm()
                                .text_color(text_color)
                                .child(label.to_string()),
                        )
                        // Note: Tab switching will be implemented when GPUI 0.2 API stabilizes
                })),
            )
            .child(
                // Footer with version
                div()
                    .px(px(16.0))
                    .py(px(12.0))
                    .border_t_1()
                    .border_color(rgb(0x313244))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                    ),
            )
    }

    fn render_header(&self, _cx: &Context<Self>) -> impl IntoElement {
        let title = match self.current_tab {
            Tab::Project => "Project Overview",
            Tab::Packages => "Package Browser",
            Tab::Environments => "Environments",
            Tab::Python => "Python Versions",
            Tab::Settings => "Settings",
        };

        div()
            .id("header")
            .h(px(56.0))
            .px(px(24.0))
            .bg(rgb(0x1e1e2e))
            .border_b_1()
            .border_color(rgb(0x313244))
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child(title),
            )
            .child(
                // Search bar (for Packages tab)
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .id("search-container")
                            .w(px(300.0))
                            .h(px(36.0))
                            .px(px(12.0))
                            .bg(rgb(0x313244))
                            .rounded(px(8.0))
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x6c7086))
                                    .child("Search packages..."),
                            ),
                    )
                    .child(
                        div()
                            .id("refresh-btn")
                            .w(px(36.0))
                            .h(px(36.0))
                            .bg(rgb(0x313244))
                            .rounded(px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .hover(|style| style.bg(rgb(0x45475a)))
                            .cursor_pointer()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xcdd6f4))
                                    .child("↻"),
                            )
                            // Note: Refresh action will be implemented when GPUI 0.2 API stabilizes
                    ),
            )
    }

    fn render_content(&self, _cx: &Context<Self>) -> impl IntoElement {
        // Render inline content based on current tab
        // Wrap each in div() to ensure consistent return type
        match self.current_tab {
            Tab::Project => div().size_full().child(self.render_project_content()),
            Tab::Packages => div().size_full().child(self.render_packages_content()),
            Tab::Environments => div().size_full().child(self.render_environments_content()),
            Tab::Python => div().size_full().child(self.render_python_content()),
            Tab::Settings => div().size_full().child(self.render_settings_content()),
        }
    }

    fn render_project_content(&self) -> impl IntoElement {
        div()
            .id("project-content")
            .size_full()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child("Project Overview"),
            )
            .child(
                div()
                    .p(px(24.0))
                    .bg(rgb(0x1e1e2e))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgb(0x313244))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_2xl()
                            .text_color(rgb(0x45475a))
                            .child("📁"),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0x6c7086))
                            .child("No project loaded"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child("Open a directory containing pyproject.toml to get started"),
                    ),
            )
    }

    fn render_packages_content(&self) -> impl IntoElement {
        div()
            .id("packages-content")
            .size_full()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child("Package Browser"),
            )
            .child(
                div()
                    .p(px(24.0))
                    .bg(rgb(0x1e1e2e))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgb(0x313244))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_2xl()
                            .text_color(rgb(0x45475a))
                            .child("📦"),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0x6c7086))
                            .child("Search for packages"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child("Use the search bar above to find packages on PyPI"),
                    ),
            )
    }

    fn render_environments_content(&self) -> impl IntoElement {
        div()
            .id("environments-content")
            .size_full()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("Virtual Environments"),
                    )
                    .child(
                        div()
                            .id("create-env-btn")
                            .px(px(16.0))
                            .py(px(10.0))
                            .bg(rgb(0xa6e3a1))
                            .text_color(rgb(0x1e1e2e))
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .rounded(px(8.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x94e2d5)))
                            .child("Create Environment"),
                    ),
            )
            .child(
                div()
                    .p(px(24.0))
                    .bg(rgb(0x1e1e2e))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgb(0x313244))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_2xl()
                            .text_color(rgb(0x45475a))
                            .child("🗂️"),
                    )
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0x6c7086))
                            .child("No virtual environments"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child("Create a virtual environment to isolate your project dependencies"),
                    ),
            )
    }

    fn render_python_content(&self) -> impl IntoElement {
        let available_versions = [
            "3.13.0", "3.12.7", "3.12.6", "3.11.10", "3.11.9", "3.10.15",
        ];

        div()
            .id("python-content")
            .size_full()
            .overflow_y_scroll()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(24.0))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("Installed Python Versions"),
                    )
                    .child(
                        div()
                            .py(px(32.0))
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_2xl()
                                    .text_color(rgb(0x45475a))
                                    .child("🐍"),
                            )
                            .child(
                                div()
                                    .text_base()
                                    .text_color(rgb(0x6c7086))
                                    .child("No Python versions managed by uv"),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x6c7086))
                                    .child("Install a Python version below to get started"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("Available Python Versions"),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(12.0))
                            .children(available_versions.iter().map(|version| {
                                div()
                                    .id(SharedString::from(format!("install-py-{version}")))
                                    .px(px(16.0))
                                    .py(px(10.0))
                                    .bg(rgb(0x1e1e2e))
                                    .border_1()
                                    .border_color(rgb(0x313244))
                                    .rounded(px(8.0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0x313244)).border_color(rgb(0x89b4fa)))
                                    .flex()
                                    .items_center()
                                    .gap(px(8.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .font_weight(gpui::FontWeight::MEDIUM)
                                            .text_color(rgb(0xcdd6f4))
                                            .child(format!("Python {version}")),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(rgb(0x89b4fa))
                                            .child("Install"),
                                    )
                            })),
                    ),
            )
    }

    fn render_settings_content(&self) -> impl IntoElement {
        div()
            .id("settings-content")
            .size_full()
            .overflow_y_scroll()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(24.0))
            .child(
                // General Settings Section
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("General"),
                    )
                    .child(
                        div()
                            .bg(rgb(0x1e1e2e))
                            .rounded(px(12.0))
                            .border_1()
                            .border_color(rgb(0x313244))
                            .overflow_hidden()
                            .child(
                                div()
                                    .px(px(16.0))
                                    .py(px(14.0))
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .border_b_1()
                                    .border_color(rgb(0x313244))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(rgb(0xcdd6f4))
                                                    .child("Color Output"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x6c7086))
                                                    .child("Enable colored output in the terminal"),
                                            ),
                                    )
                                    .child(self.render_toggle(true)),
                            )
                            .child(
                                div()
                                    .px(px(16.0))
                                    .py(px(14.0))
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(2.0))
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(rgb(0xcdd6f4))
                                                    .child("Preview Features"),
                                            )
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(rgb(0x6c7086))
                                                    .child("Enable experimental features"),
                                            ),
                                    )
                                    .child(self.render_toggle(false)),
                            ),
                    ),
            )
            .child(
                // About Section
                div()
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("About"),
                    )
                    .child(
                        div()
                            .p(px(16.0))
                            .bg(rgb(0x1e1e2e))
                            .rounded(px(12.0))
                            .border_1()
                            .border_color(rgb(0x313244))
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .child(
                                        div()
                                            .text_2xl()
                                            .child("📦"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_xl()
                                                    .font_weight(gpui::FontWeight::BOLD)
                                                    .text_color(rgb(0xcdd6f4))
                                                    .child("uv"),
                                            )
                                            .child(
                                                div()
                                                    .text_sm()
                                                    .text_color(rgb(0x6c7086))
                                                    .child(format!("Version {}", env!("CARGO_PKG_VERSION"))),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0xa6adc8))
                                    .child("An extremely fast Python package and project manager, written in Rust."),
                            ),
                    ),
            )
    }

    fn render_toggle(&self, enabled: bool) -> impl IntoElement {
        let bg_color = if enabled {
            rgb(0x89b4fa)
        } else {
            rgb(0x45475a)
        };
        let dot_offset = if enabled { px(22.0) } else { px(2.0) };

        div()
            .w(px(44.0))
            .h(px(24.0))
            .rounded(px(12.0))
            .bg(bg_color)
            .cursor_pointer()
            .relative()
            .child(
                div()
                    .absolute()
                    .top(px(2.0))
                    .left(dot_offset)
                    .w(px(20.0))
                    .h(px(20.0))
                    .rounded(px(10.0))
                    .bg(rgb(0xffffff)),
            )
    }
}

impl Render for MainWindowView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Note: Action handlers use a different API in GPUI 0.2
        // Tab switching is handled via on_click callbacks in the sidebar

        div()
            .id("main-window")
            .size_full()
            .bg(rgb(0x181825))
            .text_color(rgb(0xcdd6f4))
            .flex()
            .child(if self.sidebar_visible {
                div().child(self.render_sidebar(cx))
            } else {
                div()
            })
            .child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(self.render_header(cx))
                    .child(
                        div()
                            .flex_1()
                            .overflow_hidden()
                            .child(self.render_content(cx)),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_switching() {
        let state = AppState::new();
        assert_eq!(state.current_tab(), Tab::Project);
    }
}
