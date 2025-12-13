//! Package browser view.

use gpui::{
    div, prelude::*, px, rgb, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window,
};

use crate::state::Package;

/// View for browsing and searching packages.
pub struct PackagesView {
    focus_handle: FocusHandle,
    search_query: String,
    search_results: Vec<Package>,
    installed_packages: Vec<Package>,
    is_searching: bool,
    selected_package: Option<String>,
}

impl PackagesView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            search_query: String::new(),
            search_results: Vec::new(),
            installed_packages: Vec::new(),
            is_searching: false,
            selected_package: None,
        }
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
    }

    pub fn set_search_results(&mut self, results: Vec<Package>) {
        self.search_results = results;
        self.is_searching = false;
    }

    pub fn set_installed_packages(&mut self, packages: Vec<Package>) {
        self.installed_packages = packages;
    }

    fn render_search_section(&self) -> impl IntoElement {
        div()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child("Search Packages"),
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
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(if self.search_query.is_empty() {
                                        rgb(0x6c7086)
                                    } else {
                                        rgb(0xcdd6f4)
                                    })
                                    .child(if self.search_query.is_empty() {
                                        "Search PyPI for packages...".to_string()
                                    } else {
                                        self.search_query.clone()
                                    }),
                            ),
                    )
                    .child(
                        div()
                            .id("search-btn")
                            .px(px(24.0))
                            .h(px(44.0))
                            .bg(rgb(0x89b4fa))
                            .text_color(rgb(0x1e1e2e))
                            .rounded(px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0xb4befe)))
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(gpui::FontWeight::MEDIUM)
                                    .child("Search"),
                            ),
                    ),
            )
    }

    fn render_results_section(&self) -> impl IntoElement {
        div()
            .px(px(24.0))
            .pb(px(24.0))
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
                            .child(if self.search_query.is_empty() {
                                "Popular Packages".to_string()
                            } else {
                                format!("Results for \"{}\"", self.search_query)
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child(format!("{} packages", self.search_results.len())),
                    ),
            )
            .child(if self.search_results.is_empty() && self.search_query.is_empty() {
                // Show popular packages placeholder
                let popular_packages = self.get_popular_packages();
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .children(
                        popular_packages
                            .iter()
                            .map(|pkg| self.render_package_card(pkg)),
                    )
            } else if self.search_results.is_empty() {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(self.render_no_results())
            } else {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .children(
                        self.search_results
                            .iter()
                            .map(|pkg| self.render_package_card(pkg)),
                    )
            })
    }

    fn render_package_card(&self, package: &Package) -> impl IntoElement {
        let is_installed = package.is_installed();

        div()
            .id(SharedString::from(format!("pkg-card-{}", package.name)))
            .p(px(16.0))
            .bg(rgb(0x1e1e2e))
            .rounded(px(12.0))
            .border_1()
            .border_color(rgb(0x313244))
            .hover(|style| style.border_color(rgb(0x45475a)))
            .cursor_pointer()
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
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6c7086))
                                    .child(
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
                    ),
            )
            .child(
                div()
                    .id(SharedString::from(format!("install-{}", package.name)))
                    .px(px(16.0))
                    .py(px(8.0))
                    .bg(if is_installed {
                        rgb(0x313244)
                    } else {
                        rgb(0x89b4fa)
                    })
                    .text_color(if is_installed {
                        rgb(0xcdd6f4)
                    } else {
                        rgb(0x1e1e2e)
                    })
                    .text_sm()
                    .rounded(px(6.0))
                    .cursor_pointer()
                    .hover(|style| {
                        style.bg(if is_installed {
                            rgb(0x45475a)
                        } else {
                            rgb(0xb4befe)
                        })
                    })
                    .child(if is_installed { "Remove" } else { "Install" }),
            )
    }

    fn render_no_results(&self) -> impl IntoElement {
        div()
            .py(px(48.0))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(12.0))
            .child(
                div()
                    .text_2xl()
                    .text_color(rgb(0x45475a))
                    .child("🔍"),
            )
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
                    .child("Try adjusting your search query"),
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
                description: Some(
                    "Fundamental package for array computing in Python".to_string(),
                ),
                ..Default::default()
            },
            Package {
                name: "pandas".to_string(),
                latest_version: Some("2.1.3".to_string()),
                description: Some(
                    "Powerful data structures for data analysis".to_string(),
                ),
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
                description: Some(
                    "A high-level Python web framework".to_string(),
                ),
                ..Default::default()
            },
            Package {
                name: "pytest".to_string(),
                latest_version: Some("7.4.3".to_string()),
                description: Some(
                    "Simple powerful testing with Python".to_string(),
                ),
                ..Default::default()
            },
        ]
    }
}

impl Render for PackagesView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("packages-view")
            .size_full()
            .overflow_y_scroll()
            .bg(rgb(0x181825))
            .flex()
            .flex_col()
            .child(self.render_search_section())
            .child(self.render_results_section())
    }
}
