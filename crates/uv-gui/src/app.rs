//! Main application entry point and lifecycle management.

use std::path::PathBuf;
use std::process::Command;

use gpui::{
    Application, Bounds, Context, FocusHandle, InteractiveElement, IntoElement, KeyBinding,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, Window, WindowBounds,
    WindowOptions, actions, div, prelude::*, px, rgb, size,
};

use crate::state::{Environment, ProjectState, PythonInstallation, Tab};

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
            // Bind Cmd+Q to Quit
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);

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
pub(crate) struct MainWindowView {
    focus_handle: FocusHandle,
    current_tab: Tab,
    sidebar_visible: bool,
    // Settings
    color_output: bool,
    preview_features: bool,
    // Python versions
    installed_pythons: Vec<PythonInstallation>,
    available_pythons: Vec<String>,
    installing_python: Option<String>,
    // Environments
    environments: Vec<Environment>,
    creating_environment: bool,
    // Project
    project: Option<ProjectState>,
}

impl MainWindowView {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        let mut view = Self {
            focus_handle,
            current_tab: Tab::Project,
            sidebar_visible: true,
            color_output: true,
            preview_features: false,
            installed_pythons: Vec::new(),
            available_pythons: vec![
                "3.13".to_string(),
                "3.12".to_string(),
                "3.11".to_string(),
                "3.10".to_string(),
                "3.9".to_string(),
            ],
            installing_python: None,
            environments: Vec::new(),
            creating_environment: false,
            project: None,
        };

        // Load initial data
        view.refresh_all();

        view
    }

    fn refresh_all(&mut self) {
        self.refresh_pythons();
        self.refresh_environments();
        self.refresh_project();
    }

    fn refresh_pythons(&mut self) {
        // Run `uv python list` to get installed Python versions
        if let Ok(output) = Command::new("uv").args(["python", "list"]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                self.installed_pythons = parse_python_list(&stdout);
            }
        }
    }

    fn refresh_environments(&mut self) {
        self.environments.clear();

        // Check for .venv in current directory
        if let Ok(cwd) = std::env::current_dir() {
            let venv_path = cwd.join(".venv");
            if venv_path.exists() {
                let python_version = get_venv_python_version(&venv_path);
                self.environments.push(Environment {
                    name: ".venv".to_string(),
                    path: venv_path,
                    python_version,
                    is_active: std::env::var("VIRTUAL_ENV").is_ok(),
                    package_count: 0,
                    created_at: None,
                    size_bytes: None,
                });
            }
        }
    }

    fn refresh_project(&mut self) {
        if let Ok(cwd) = std::env::current_dir() {
            let pyproject_path = cwd.join("pyproject.toml");
            if pyproject_path.exists() {
                let mut project = ProjectState::from_path(cwd.clone());
                project.pyproject_path = Some(pyproject_path.clone());

                // Try to read project name from pyproject.toml
                if let Ok(content) = fs_err::read_to_string(&pyproject_path) {
                    if let Some(name) = extract_project_name(&content) {
                        project.name = name;
                    }
                    if let Some(version) = extract_project_version(&content) {
                        project.version = Some(version);
                    }
                }

                // Check for lockfile
                project.has_lockfile = cwd.join("uv.lock").exists();

                self.project = Some(project);
            } else {
                self.project = None;
            }
        }
    }

    fn install_python(&mut self, version: String) {
        self.installing_python = Some(version.clone());

        // Run installation synchronously for now (TODO: make async)
        let result = std::process::Command::new("uv")
            .args(["python", "install", &version])
            .output();

        self.installing_python = None;
        if result.is_ok() {
            self.refresh_pythons();
        }
    }

    fn create_environment(&mut self) {
        self.creating_environment = true;

        // Run creation synchronously for now (TODO: make async)
        let result = std::process::Command::new("uv").args(["venv"]).output();

        self.creating_environment = false;
        if result.is_ok() {
            self.refresh_environments();
        }
    }

    fn switch_tab(&mut self, tab: Tab) {
        self.current_tab = tab;
    }

    #[allow(dead_code)]
    fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
    }

    fn toggle_color_output(&mut self) {
        self.color_output = !self.color_output;
    }

    fn toggle_preview_features(&mut self) {
        self.preview_features = !self.preview_features;
    }

    fn render_sidebar(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                // Navigation tabs
                div().flex_1().pt(px(44.0)).pb(px(8.0)).children(tabs.map(
                    |(tab, label, _icon)| {
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
                            .on_click(cx.listener(move |this, _event, _window, _cx| {
                                this.switch_tab(tab);
                            }))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(text_color)
                                    .child(label.to_string()),
                            )
                    },
                )),
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

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                div().flex().items_center().gap(px(12.0)).child(
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
                        .on_click(cx.listener(|this, _event, _window, _cx| {
                            this.refresh_all();
                        }))
                        .child(div().text_sm().text_color(rgb(0xcdd6f4)).child("↻")),
                ),
            )
    }

    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        match self.current_tab {
            Tab::Project => div().size_full().child(self.render_project_content()),
            Tab::Packages => div().size_full().child(self.render_packages_content()),
            Tab::Environments => div()
                .size_full()
                .child(self.render_environments_content(cx)),
            Tab::Python => div().size_full().child(self.render_python_content(cx)),
            Tab::Settings => div().size_full().child(self.render_settings_content(cx)),
        }
    }

    fn render_project_content(&self) -> impl IntoElement {
        let content = if let Some(project) = &self.project {
            div()
                .p(px(24.0))
                .bg(rgb(0x1e1e2e))
                .rounded(px(12.0))
                .border_1()
                .border_color(rgb(0x313244))
                .flex()
                .flex_col()
                .gap(px(16.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(12.0))
                        .child(div().text_2xl().child("📦"))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_xl()
                                        .font_weight(gpui::FontWeight::BOLD)
                                        .text_color(rgb(0xcdd6f4))
                                        .child(project.name.clone()),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0x6c7086))
                                        .child(project.version.clone().unwrap_or_default()),
                                ),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(24.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(div().text_sm().text_color(rgb(0x6c7086)).child("Lockfile:"))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(if project.has_lockfile {
                                            rgb(0xa6e3a1)
                                        } else {
                                            rgb(0xf38ba8)
                                        })
                                        .child(if project.has_lockfile { "✓" } else { "✗" }),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0x6c7086))
                                        .child("Dependencies:"),
                                )
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0xcdd6f4))
                                        .child(format!("{}", project.dependency_count())),
                                ),
                        ),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x6c7086))
                        .child(project.root.display().to_string()),
                )
        } else {
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
                .child(div().text_2xl().text_color(rgb(0x45475a)).child("📁"))
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
                )
        };

        div()
            .id("project-content")
            .size_full()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(content)
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
                    .child(div().text_2xl().text_color(rgb(0x45475a)).child("📦"))
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0x6c7086))
                            .child("Package search coming soon"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child("Use `uv add <package>` in the terminal for now"),
                    ),
            )
    }

    fn render_environments_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let env_list =
            if self.environments.is_empty() {
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
                    .child(div().text_2xl().text_color(rgb(0x45475a)).child("🗂️"))
                    .child(
                        div()
                            .text_base()
                            .text_color(rgb(0x6c7086))
                            .child("No virtual environments"),
                    )
                    .child(
                        div().text_sm().text_color(rgb(0x6c7086)).child(
                            "Create a virtual environment to isolate your project dependencies",
                        ),
                    )
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .children(self.environments.iter().map(|env| {
                        div()
                            .p(px(16.0))
                            .bg(rgb(0x1e1e2e))
                            .rounded(px(12.0))
                            .border_1()
                            .border_color(if env.is_active {
                                rgb(0xa6e3a1)
                            } else {
                                rgb(0x313244)
                            })
                            .flex()
                            .justify_between()
                            .items_center()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .child(div().text_xl().child("🐍"))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(
                                                div()
                                                    .text_base()
                                                    .font_weight(gpui::FontWeight::MEDIUM)
                                                    .text_color(rgb(0xcdd6f4))
                                                    .child(env.name.clone()),
                                            )
                                            .child(
                                                div().text_sm().text_color(rgb(0x6c7086)).child(
                                                    format!("Python {}", env.python_version),
                                                ),
                                            ),
                                    ),
                            )
                            .child(if env.is_active {
                                div()
                                    .px(px(8.0))
                                    .py(px(4.0))
                                    .bg(rgb(0xa6e3a1))
                                    .text_color(rgb(0x1e1e2e))
                                    .text_xs()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .rounded(px(4.0))
                                    .child("Active")
                            } else {
                                div()
                            })
                    }))
            };

        let button_text = if self.creating_environment {
            "Creating..."
        } else {
            "Create Environment"
        };

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
                            .bg(if self.creating_environment {
                                rgb(0x45475a)
                            } else {
                                rgb(0xa6e3a1)
                            })
                            .text_color(rgb(0x1e1e2e))
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .rounded(px(8.0))
                            .cursor_pointer()
                            .when(!self.creating_environment, |el| {
                                el.hover(|style| style.bg(rgb(0x94e2d5)))
                            })
                            .on_click(cx.listener(|this, _event, _window, _cx| {
                                if !this.creating_environment {
                                    this.create_environment();
                                }
                            }))
                            .child(button_text),
                    ),
            )
            .child(env_list)
    }

    fn render_python_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let installed_section = if self.installed_pythons.is_empty() {
            div()
                .py(px(32.0))
                .flex()
                .flex_col()
                .items_center()
                .gap(px(12.0))
                .child(div().text_2xl().text_color(rgb(0x45475a)).child("🐍"))
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
                )
        } else {
            div()
                .flex()
                .flex_col()
                .gap(px(8.0))
                .children(self.installed_pythons.iter().map(|py| {
                    div()
                        .p(px(16.0))
                        .bg(rgb(0x1e1e2e))
                        .rounded(px(8.0))
                        .border_1()
                        .border_color(rgb(0x313244))
                        .flex()
                        .justify_between()
                        .items_center()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(12.0))
                                .child(div().text_lg().child("🐍"))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(
                                            div()
                                                .text_base()
                                                .font_weight(gpui::FontWeight::MEDIUM)
                                                .text_color(rgb(0xcdd6f4))
                                                .child(format!("Python {}", py.version.clone())),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(rgb(0x6c7086))
                                                .child(py.path.display().to_string()),
                                        ),
                                ),
                        )
                        .child(if py.is_managed {
                            div()
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(rgb(0x89b4fa))
                                .text_color(rgb(0x1e1e2e))
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .rounded(px(4.0))
                                .child("Managed")
                        } else {
                            div()
                                .px(px(8.0))
                                .py(px(4.0))
                                .bg(rgb(0x45475a))
                                .text_color(rgb(0xcdd6f4))
                                .text_xs()
                                .font_weight(gpui::FontWeight::MEDIUM)
                                .rounded(px(4.0))
                                .child("System")
                        })
                }))
        };

        let installing = self.installing_python.clone();

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
                    .child(installed_section),
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
                            .child("Install Python"),
                    )
                    .child(div().flex().flex_wrap().gap(px(12.0)).children(
                        self.available_pythons.iter().map(|version| {
                            let version_clone = version.clone();
                            let is_installing = installing.as_ref().map_or(false, |v| v == version);
                            let button_text = if is_installing {
                                "Installing...".to_string()
                            } else {
                                format!("Python {version}")
                            };

                            div()
                                .id(SharedString::from(format!("install-py-{version}")))
                                .px(px(16.0))
                                .py(px(10.0))
                                .bg(if is_installing {
                                    rgb(0x45475a)
                                } else {
                                    rgb(0x1e1e2e)
                                })
                                .border_1()
                                .border_color(rgb(0x313244))
                                .rounded(px(8.0))
                                .cursor_pointer()
                                .when(!is_installing, |el| {
                                    el.hover(|style| {
                                        style.bg(rgb(0x313244)).border_color(rgb(0x89b4fa))
                                    })
                                })
                                .on_click(cx.listener(move |this, _event, _window, _cx| {
                                    if this.installing_python.is_none() {
                                        this.install_python(version_clone.clone());
                                    }
                                }))
                                .flex()
                                .items_center()
                                .gap(px(8.0))
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(gpui::FontWeight::MEDIUM)
                                        .text_color(rgb(0xcdd6f4))
                                        .child(button_text),
                                )
                                .when(!is_installing, |el| {
                                    el.child(
                                        div().text_xs().text_color(rgb(0x89b4fa)).child("Install"),
                                    )
                                })
                        }),
                    )),
            )
    }

    fn render_settings_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
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
                                    .id("color-output-toggle")
                                    .px(px(16.0))
                                    .py(px(14.0))
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .border_b_1()
                                    .border_color(rgb(0x313244))
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                        this.toggle_color_output();
                                    }))
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
                                    .child(self.render_toggle(self.color_output)),
                            )
                            .child(
                                div()
                                    .id("preview-features-toggle")
                                    .px(px(16.0))
                                    .py(px(14.0))
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .cursor_pointer()
                                    .on_click(cx.listener(|this, _event, _window, _cx| {
                                        this.toggle_preview_features();
                                    }))
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
                                    .child(self.render_toggle(self.preview_features)),
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
                                    .child(div().text_2xl().child("📦"))
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
                                                div().text_sm().text_color(rgb(0x6c7086)).child(
                                                    format!("Version {}", env!("CARGO_PKG_VERSION")),
                                                ),
                                            ),
                                    ),
                            )
                            .child(
                                div().text_sm().text_color(rgb(0xa6adc8)).child(
                                    "An extremely fast Python package and project manager, written in Rust.",
                                ),
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
        div()
            .id("main-window")
            .size_full()
            .bg(rgb(0x181825))
            .text_color(rgb(0xcdd6f4))
            .flex()
            .track_focus(&self.focus_handle)
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

/// Parse the output of `uv python list` to get installed Python versions.
fn parse_python_list(output: &str) -> Vec<PythonInstallation> {
    let mut pythons = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Parse lines like:
        // cpython-3.12.7-macos-aarch64-none    /Users/.../python3.12
        // cpython-3.11.9-macos-aarch64-none    /opt/homebrew/bin/python3.11 -> ...
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let version_part = parts[0];
            let path_part = parts[1];

            // Extract version from cpython-3.12.7-... format
            if let Some(version) = version_part.strip_prefix("cpython-") {
                let version = version.split('-').next().unwrap_or(version);
                let path = PathBuf::from(path_part);
                let is_managed =
                    path_part.contains(".local/share/uv") || path_part.contains("uv/python");

                pythons.push(PythonInstallation {
                    version: version.to_string(),
                    path,
                    is_default: false,
                    is_managed,
                    implementation: "CPython".to_string(),
                    architecture: None,
                });
            }
        }
    }

    pythons
}

/// Get the Python version from a virtual environment.
fn get_venv_python_version(venv_path: &PathBuf) -> String {
    let python_path = venv_path.join("bin").join("python");
    if let Ok(output) = Command::new(&python_path).args(["--version"]).output() {
        if output.status.success() {
            let version = String::from_utf8_lossy(&output.stdout);
            return version
                .trim()
                .strip_prefix("Python ")
                .unwrap_or(&version)
                .trim()
                .to_string();
        }
    }
    "Unknown".to_string()
}

/// Extract project name from pyproject.toml content.
fn extract_project_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("name") {
            if let Some(value) = line.split('=').nth(1) {
                return Some(
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            }
        }
    }
    None
}

/// Extract project version from pyproject.toml content.
fn extract_project_version(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("version") {
            if let Some(value) = line.split('=').nth(1) {
                return Some(
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string(),
                );
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_tab() {
        assert_eq!(Tab::default(), Tab::Project);
    }

    #[test]
    fn test_parse_python_list() {
        let output = "cpython-3.12.7-macos-aarch64-none    /Users/test/.local/share/uv/python/cpython-3.12.7/bin/python3.12
cpython-3.11.9-macos-aarch64-none    /opt/homebrew/bin/python3.11";
        let pythons = parse_python_list(output);
        assert_eq!(pythons.len(), 2);
        assert_eq!(pythons[0].version, "3.12.7");
        assert!(pythons[0].is_managed);
        assert_eq!(pythons[1].version, "3.11.9");
        assert!(!pythons[1].is_managed);
    }

    #[test]
    fn test_extract_project_name() {
        let content = r#"
[project]
name = "my-project"
version = "0.1.0"
"#;
        assert_eq!(
            extract_project_name(content),
            Some("my-project".to_string())
        );
    }
}
