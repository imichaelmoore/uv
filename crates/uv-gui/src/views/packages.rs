//! Package browser view.

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::{Duration, Instant};

use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::*, px, rgb,
};

use crate::loaders::{PyPiPackageLoader, PyPiSearchError};
use crate::state::Package;

/// Cache entry with expiration time.
struct CacheEntry {
    package: Package,
    expires_at: Instant,
}

/// Simple in-memory cache for PyPI package lookups.
/// Entries expire after 5 minutes to ensure fresh data.
struct PackageCache {
    entries: HashMap<String, CacheEntry>,
    ttl: Duration,
}

impl PackageCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            ttl: Duration::from_secs(300), // 5 minutes
        }
    }

    fn get(&self, name: &str) -> Option<Package> {
        let key = name.to_lowercase();
        self.entries.get(&key).and_then(|entry| {
            if Instant::now() < entry.expires_at {
                Some(entry.package.clone())
            } else {
                None
            }
        })
    }

    fn insert(&mut self, name: &str, package: Package) {
        let key = name.to_lowercase();
        self.entries.insert(
            key,
            CacheEntry {
                package,
                expires_at: Instant::now() + self.ttl,
            },
        );
    }

    /// Remove expired entries to prevent unbounded growth.
    fn cleanup(&mut self) {
        let now = Instant::now();
        self.entries.retain(|_, entry| entry.expires_at > now);
    }
}

/// Operation being performed on a package.
#[derive(Clone, Debug, PartialEq)]
enum PackageOperation {
    Installing(String),
    Removing(String),
}

/// View for browsing and searching packages.
pub struct PackagesView {
    focus_handle: FocusHandle,
    search_query: String,
    search_results: Vec<Package>,
    installed_packages: Vec<Package>,
    is_searching: bool,
    search_error: Option<String>,
    /// Cache for PyPI package lookups.
    cache: PackageCache,
    /// Current package operation (install/remove).
    current_operation: Option<PackageOperation>,
    /// Success message to display.
    success_message: Option<String>,
    /// Project root directory for running uv commands.
    project_root: Option<PathBuf>,
}

impl PackagesView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            search_query: String::new(),
            search_results: Vec::new(),
            installed_packages: Vec::new(),
            is_searching: false,
            search_error: None,
            cache: PackageCache::new(),
            current_operation: None,
            success_message: None,
            project_root: std::env::current_dir().ok(),
        }
    }

    /// Set installed packages for checking install status.
    pub fn set_installed_packages(&mut self, packages: Vec<Package>) {
        self.installed_packages = packages;
    }

    /// Set the project root directory.
    pub fn set_project_root(&mut self, root: PathBuf) {
        self.project_root = Some(root);
    }

    /// Handle a key press in the search input.
    fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        let key = &event.keystroke.key;

        match key.as_str() {
            "enter" => {
                self.perform_search(cx);
            }
            "backspace" => {
                self.search_query.pop();
                self.search_error = None;
                self.success_message = None;
                cx.notify();
            }
            "escape" => {
                self.search_query.clear();
                self.search_results.clear();
                self.search_error = None;
                self.success_message = None;
                cx.notify();
            }
            _ => {
                if key.len() == 1 {
                    if let Some(c) = key.chars().next() {
                        if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                            self.search_query.push(c);
                            self.search_error = None;
                            self.success_message = None;
                            cx.notify();
                        }
                    }
                }
            }
        }
    }

    /// Perform a PyPI package lookup.
    fn perform_search(&mut self, cx: &mut Context<Self>) {
        let query = self.search_query.trim().to_string();
        if query.is_empty() {
            return;
        }

        // Check cache first
        if let Some(mut cached_package) = self.cache.get(&query) {
            // Update installed status
            if let Some(installed_version) = self.get_installed_version(&cached_package.name) {
                cached_package.installed_version = Some(installed_version);
            }
            self.search_results = vec![cached_package];
            self.search_error = None;
            self.is_searching = false;
            cx.notify();
            return;
        }

        // Set loading state
        self.is_searching = true;
        self.search_error = None;
        self.success_message = None;
        self.search_results.clear();
        cx.notify();

        // Perform blocking search
        let Some(loader) = PyPiPackageLoader::new() else {
            self.is_searching = false;
            self.search_error = Some("Failed to initialize HTTP client".to_string());
            cx.notify();
            return;
        };

        match loader.lookup(&query) {
            Ok(response) => {
                let mut package = response.info.into_package();

                // Cache the result
                self.cache.insert(&package.name, package.clone());
                self.cache.cleanup();

                // Check if package is installed
                if let Some(installed_version) = self.get_installed_version(&package.name) {
                    package.installed_version = Some(installed_version);
                }

                self.search_results = vec![package];
                self.search_error = None;
            }
            Err(PyPiSearchError::NotFound(name)) => {
                self.search_results.clear();
                self.search_error = Some(format!("Package `{name}` not found on PyPI"));
            }
            Err(PyPiSearchError::InvalidName(name)) => {
                self.search_results.clear();
                self.search_error = Some(format!("Invalid package name: `{name}`"));
            }
            Err(PyPiSearchError::Network(e)) => {
                self.search_results.clear();
                self.search_error = Some(format!("Network error: {e}. Check your connection."));
            }
        }
        self.is_searching = false;
        cx.notify();
    }

    /// Install a package using `uv add`.
    fn install_package(&mut self, package_name: String, cx: &mut Context<Self>) {
        if self.current_operation.is_some() {
            return;
        }

        self.current_operation = Some(PackageOperation::Installing(package_name.clone()));
        self.search_error = None;
        self.success_message = None;
        cx.notify();

        // Run uv add
        let mut cmd = Command::new("uv");
        cmd.args(["add", &package_name]);

        if let Some(root) = &self.project_root {
            cmd.current_dir(root);
        }

        match cmd.output() {
            Ok(output) if output.status.success() => {
                self.success_message = Some(format!("Successfully installed `{package_name}`"));
                self.search_error = None;

                // Update installed status in search results
                for pkg in &mut self.search_results {
                    if pkg.name.eq_ignore_ascii_case(&package_name) {
                        pkg.installed_version = pkg.latest_version.clone();
                    }
                }
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                self.search_error = Some(format!("Failed to install `{package_name}`: {stderr}"));
                self.success_message = None;
            }
            Err(e) => {
                self.search_error = Some(format!("Failed to run `uv add`: {e}"));
                self.success_message = None;
            }
        }

        self.current_operation = None;
        cx.notify();
    }

    /// Remove a package using `uv remove`.
    fn remove_package(&mut self, package_name: String, cx: &mut Context<Self>) {
        if self.current_operation.is_some() {
            return;
        }

        self.current_operation = Some(PackageOperation::Removing(package_name.clone()));
        self.search_error = None;
        self.success_message = None;
        cx.notify();

        // Run uv remove
        let mut cmd = Command::new("uv");
        cmd.args(["remove", &package_name]);

        if let Some(root) = &self.project_root {
            cmd.current_dir(root);
        }

        match cmd.output() {
            Ok(output) if output.status.success() => {
                self.success_message = Some(format!("Successfully removed `{package_name}`"));
                self.search_error = None;

                // Update installed status in search results
                for pkg in &mut self.search_results {
                    if pkg.name.eq_ignore_ascii_case(&package_name) {
                        pkg.installed_version = None;
                    }
                }

                // Remove from installed packages list
                self.installed_packages
                    .retain(|p| !p.name.eq_ignore_ascii_case(&package_name));
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                self.search_error = Some(format!("Failed to remove `{package_name}`: {stderr}"));
                self.success_message = None;
            }
            Err(e) => {
                self.search_error = Some(format!("Failed to run `uv remove`: {e}"));
                self.success_message = None;
            }
        }

        self.current_operation = None;
        cx.notify();
    }

    /// Check if a specific package operation is in progress.
    fn is_operating_on(&self, package_name: &str) -> bool {
        match &self.current_operation {
            Some(PackageOperation::Installing(name) | PackageOperation::Removing(name)) => {
                name.eq_ignore_ascii_case(package_name)
            }
            None => false,
        }
    }

    /// Get the installed version of a package if it's in the current project.
    fn get_installed_version(&self, package_name: &str) -> Option<String> {
        self.installed_packages
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(package_name))
            .and_then(|p| p.installed_version.clone())
    }

    /// Check if a package is installed in the current project.
    fn is_package_installed(&self, package_name: &str) -> bool {
        self.get_installed_version(package_name).is_some()
    }

    fn render_search_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("Package Lookup"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child("(exact name match)"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(12.0))
                    .child(
                        div()
                            .id("package-search-input")
                            .flex_1()
                            .h(px(44.0))
                            .px(px(16.0))
                            .bg(rgb(0x313244))
                            .rounded(px(8.0))
                            .border_1()
                            .border_color(rgb(0x313244))
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .cursor_text()
                            .track_focus(&self.focus_handle)
                            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                                this.handle_key_down(event, cx);
                            }))
                            .child(div().text_sm().text_color(rgb(0x6c7086)).child("🔍"))
                            .child(
                                div()
                                    .flex_1()
                                    .text_sm()
                                    .text_color(if self.search_query.is_empty() {
                                        rgb(0x6c7086)
                                    } else {
                                        rgb(0xcdd6f4)
                                    })
                                    .child(if self.search_query.is_empty() {
                                        "Enter package name...".to_string()
                                    } else {
                                        self.search_query.clone()
                                    }),
                            )
                            .when(!self.search_query.is_empty(), |el| {
                                el.child(
                                    div()
                                        .id("clear-search")
                                        .w(px(20.0))
                                        .h(px(20.0))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .rounded(px(10.0))
                                        .bg(rgb(0x45475a))
                                        .text_xs()
                                        .text_color(rgb(0xcdd6f4))
                                        .cursor_pointer()
                                        .hover(|style| style.bg(rgb(0x585b70)))
                                        .on_click(cx.listener(|this, _event, _window, cx| {
                                            this.search_query.clear();
                                            this.search_results.clear();
                                            this.search_error = None;
                                            this.success_message = None;
                                            cx.notify();
                                        }))
                                        .child("×"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .id("search-btn")
                            .px(px(24.0))
                            .h(px(44.0))
                            .bg(if self.is_searching {
                                rgb(0x45475a)
                            } else {
                                rgb(0x89b4fa)
                            })
                            .text_color(rgb(0x1e1e2e))
                            .rounded(px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .when(!self.is_searching, |el| {
                                el.hover(|style| style.bg(rgb(0xb4befe)))
                            })
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                if !this.is_searching {
                                    this.perform_search(cx);
                                }
                            }))
                            .child(div().text_sm().font_weight(gpui::FontWeight::MEDIUM).child(
                                if self.is_searching {
                                    "Searching..."
                                } else {
                                    "Search"
                                },
                            )),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(rgb(0x6c7086))
                    .child("Type the exact package name and press Enter or click Search"),
            )
            .when(self.success_message.is_some(), |el| {
                el.child(
                    div()
                        .px(px(12.0))
                        .py(px(8.0))
                        .bg(rgb(0x1e1e2e))
                        .border_1()
                        .border_color(rgb(0xa6e3a1))
                        .rounded(px(8.0))
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().text_sm().text_color(rgb(0xa6e3a1)).child("✓"))
                        .child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xa6e3a1))
                                .child(self.success_message.clone().unwrap_or_default()),
                        ),
                )
            })
    }

    fn render_results_header(&self) -> impl IntoElement {
        div()
            .flex()
            .justify_between()
            .items_center()
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child(if self.search_query.is_empty() {
                        "Popular Packages".to_string()
                    } else {
                        format!("Results for \"{}\"", self.search_query)
                    }),
            )
            .when(!self.search_results.is_empty(), |el| {
                el.child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x6c7086))
                        .child(format!("{} package(s)", self.search_results.len())),
                )
            })
    }

    fn render_loading(&self) -> gpui::Div {
        div()
            .py(px(48.0))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(div().text_2xl().text_color(rgb(0x89b4fa)).child("⏳"))
            .child(
                div()
                    .text_base()
                    .text_color(rgb(0x6c7086))
                    .child("Searching PyPI..."),
            )
    }

    fn render_error(&self, error: &str) -> gpui::Div {
        div()
            .py(px(32.0))
            .px(px(24.0))
            .bg(rgb(0x1e1e2e))
            .rounded(px(12.0))
            .border_1()
            .border_color(rgb(0xf38ba8))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(div().text_2xl().text_color(rgb(0xf38ba8)).child("⚠️"))
            .child(
                div()
                    .text_base()
                    .text_color(rgb(0xf38ba8))
                    .text_align(gpui::TextAlign::Center)
                    .child(error.to_string()),
            )
    }

    fn render_package_card(
        &self,
        package: &Package,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<gpui::Div> {
        let is_installed = package.is_installed() || self.is_package_installed(&package.name);
        let is_operating = self.is_operating_on(&package.name);
        let package_name = package.name.clone();

        // Determine button state
        let (button_text, button_bg, button_text_color) = if is_operating {
            match &self.current_operation {
                Some(PackageOperation::Installing(_)) => {
                    ("Installing...", rgb(0x45475a), rgb(0xcdd6f4))
                }
                Some(PackageOperation::Removing(_)) => {
                    ("Removing...", rgb(0x45475a), rgb(0xcdd6f4))
                }
                None => ("Install", rgb(0x89b4fa), rgb(0x1e1e2e)),
            }
        } else if is_installed {
            ("Remove", rgb(0x313244), rgb(0xcdd6f4))
        } else {
            ("Install", rgb(0x89b4fa), rgb(0x1e1e2e))
        };

        let keywords_display = if !package.keywords.is_empty() {
            let joined = package.keywords.join(", ");
            if joined.len() > 50 {
                format!("{}...", joined.chars().take(50).collect::<String>())
            } else {
                joined
            }
        } else {
            String::new()
        };

        div()
            .id(SharedString::from(format!("pkg-card-{}", package.name)))
            .p(px(16.0))
            .bg(rgb(0x1e1e2e))
            .rounded(px(12.0))
            .border_1()
            .border_color(rgb(0x313244))
            .hover(|style| style.border_color(rgb(0x45475a)))
            .flex()
            .justify_between()
            .items_start()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(gpui::FontWeight::SEMIBOLD)
                                    .text_color(rgb(0xcdd6f4))
                                    .child(package.name.clone()),
                            )
                            .child(
                                div().text_xs().text_color(rgb(0x6c7086)).child(
                                    package
                                        .latest_version
                                        .clone()
                                        .or(package.installed_version.clone())
                                        .unwrap_or_else(|| "unknown".to_string()),
                                ),
                            )
                            .when(is_installed, |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .px(px(6.0))
                                        .py(px(2.0))
                                        .bg(rgb(0xa6e3a1))
                                        .text_color(rgb(0x1e1e2e))
                                        .rounded(px(4.0))
                                        .child("Installed"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xa6adc8))
                            .max_w(px(500.0))
                            .child(
                                package
                                    .description
                                    .clone()
                                    .unwrap_or_else(|| "No description available".to_string()),
                            ),
                    )
                    .when(
                        package.license.is_some() || !package.keywords.is_empty(),
                        |el| {
                            el.child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(12.0))
                                    .mt(px(4.0))
                                    .when(package.license.is_some(), |el| {
                                        el.child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(4.0))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x6c7086))
                                                        .child("License:"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xa6adc8))
                                                        .child(
                                                            package
                                                                .license
                                                                .clone()
                                                                .unwrap_or_default(),
                                                        ),
                                                ),
                                        )
                                    })
                                    .when(!package.keywords.is_empty(), |el| {
                                        el.child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap(px(4.0))
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0x6c7086))
                                                        .child("Keywords:"),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(rgb(0xa6adc8))
                                                        .child(keywords_display.clone()),
                                                ),
                                        )
                                    }),
                            )
                        },
                    ),
            )
            .child(
                div()
                    .id(SharedString::from(format!("action-{}", package.name)))
                    .px(px(16.0))
                    .py(px(8.0))
                    .bg(button_bg)
                    .text_color(button_text_color)
                    .text_sm()
                    .rounded(px(6.0))
                    .cursor_pointer()
                    .when(!is_operating && !is_installed, |el| {
                        el.hover(|style| style.bg(rgb(0xb4befe)))
                    })
                    .when(!is_operating && is_installed, |el| {
                        el.hover(|style| style.bg(rgb(0x45475a)))
                    })
                    .on_click(cx.listener(move |this, _event, _window, cx| {
                        if this.current_operation.is_some() {
                            return;
                        }

                        let name = package_name.clone();
                        let installed = this.is_package_installed(&name);

                        if installed {
                            this.remove_package(name, cx);
                        } else {
                            this.install_package(name, cx);
                        }
                    }))
                    .child(button_text),
            )
    }

    fn render_no_results(&self) -> gpui::Div {
        div()
            .py(px(48.0))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(div().text_2xl().text_color(rgb(0x45475a)).child("🔍"))
            .child(
                div()
                    .text_base()
                    .text_color(rgb(0x6c7086))
                    .child("No packages found"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x6c7086))
                    .child("Make sure you entered the exact package name"),
            )
    }

    fn get_popular_packages(&self) -> Vec<Package> {
        vec![
            Package {
                name: "requests".to_string(),
                latest_version: Some("2.31.0".to_string()),
                description: Some("Python HTTP for Humans".to_string()),
                ..Default::default()
            },
            Package {
                name: "numpy".to_string(),
                latest_version: Some("1.26.2".to_string()),
                description: Some("Fundamental package for array computing in Python".to_string()),
                ..Default::default()
            },
            Package {
                name: "pandas".to_string(),
                latest_version: Some("2.1.3".to_string()),
                description: Some("Powerful data structures for data analysis".to_string()),
                ..Default::default()
            },
            Package {
                name: "flask".to_string(),
                latest_version: Some("3.0.0".to_string()),
                description: Some(
                    "A simple framework for building complex web applications".to_string(),
                ),
                ..Default::default()
            },
            Package {
                name: "django".to_string(),
                latest_version: Some("4.2.7".to_string()),
                description: Some("A high-level Python web framework".to_string()),
                ..Default::default()
            },
            Package {
                name: "pytest".to_string(),
                latest_version: Some("7.4.3".to_string()),
                description: Some("Simple powerful testing with Python".to_string()),
                ..Default::default()
            },
        ]
    }
}

impl Render for PackagesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Collect packages to render
        let packages_to_render: Vec<Package> = if self.is_searching || self.search_error.is_some() {
            vec![]
        } else if self.search_results.is_empty() && self.search_query.is_empty() {
            self.get_popular_packages()
        } else {
            self.search_results.clone()
        };

        // Build cards with explicit loop to avoid closure lifetime issues
        let mut cards = Vec::new();
        for pkg in &packages_to_render {
            cards.push(self.render_package_card(pkg, cx));
        }

        // Build content section
        let content = if self.is_searching {
            div().child(self.render_loading())
        } else if let Some(error) = &self.search_error {
            div().child(self.render_error(error))
        } else if cards.is_empty() && !self.search_query.is_empty() {
            div().child(self.render_no_results())
        } else {
            div().flex().flex_col().gap(px(8.0)).children(cards)
        };

        div()
            .id("packages-view")
            .size_full()
            .overflow_y_scroll()
            .bg(rgb(0x181825))
            .flex()
            .flex_col()
            .child(self.render_search_section(cx))
            .child(
                div()
                    .px(px(24.0))
                    .pb(px(24.0))
                    .flex()
                    .flex_col()
                    .gap(px(16.0))
                    .child(self.render_results_header())
                    .child(content),
            )
    }
}
