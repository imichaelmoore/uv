//! Reusable dependency list component.
//!
//! This component renders a list of package dependencies with
//! version information and source badges.

use gpui::{
    IntoElement, ParentElement, RenderOnce, SharedString, Styled, div, prelude::*, px, rgb,
};

use crate::state::Package;

/// A reusable component for displaying a list of dependencies.
#[derive(IntoElement)]
pub struct DependencyList {
    /// The title for this section.
    title: String,
    /// The packages to display.
    packages: Vec<Package>,
    /// Whether to show source badges (for dev deps).
    show_source_badge: bool,
    /// Whether to use compact mode.
    compact: bool,
}

impl DependencyList {
    /// Create a new dependency list.
    pub fn new(title: impl Into<String>, packages: Vec<Package>) -> Self {
        Self {
            title: title.into(),
            packages,
            show_source_badge: false,
            compact: false,
        }
    }

    /// Show source badges for dev dependencies.
    pub fn with_source_badges(mut self) -> Self {
        self.show_source_badge = true;
        self
    }

    /// Use compact mode with smaller spacing.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }

    fn render_package_row(&self, package: &Package, index: usize) -> impl IntoElement {
        let bg_color = if index % 2 == 0 {
            rgb(0x1e1e2e)
        } else {
            rgb(0x181825)
        };

        let padding = if self.compact { px(10.0) } else { px(12.0) };

        div()
            .id(SharedString::from(format!("dep-{}", package.name)))
            .px(px(16.0))
            .py(padding)
            .bg(bg_color)
            .flex()
            .items_center()
            .justify_between()
            .hover(|style| style.bg(rgb(0x313244)))
            .cursor_pointer()
            .child(
                // Left side: name, required version, source badge
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    // Package name
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(0xcdd6f4))
                            .min_w(px(120.0))
                            .child(package.name.clone()),
                    )
                    // Required version specifier
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(Self::format_required_version(package)),
                    )
                    // Source badge (for dev deps)
                    .when(self.show_source_badge && package.is_dev, |el| {
                        if let Some(source_label) = &package.source_label {
                            el.child(
                                div()
                                    .text_xs()
                                    .px(px(6.0))
                                    .py(px(2.0))
                                    .bg(rgb(0x45475a))
                                    .text_color(rgb(0xcdd6f4))
                                    .rounded(px(4.0))
                                    .child(source_label.clone()),
                            )
                        } else {
                            el
                        }
                    }),
            )
            .child(
                // Right side: installed version
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xa6adc8))
                            .child(Self::format_installed_version(package)),
                    )
                    // Update available badge
                    .when(package.update_available, |el| {
                        el.child(
                            div()
                                .text_xs()
                                .px(px(6.0))
                                .py(px(2.0))
                                .bg(rgb(0xa6e3a1))
                                .text_color(rgb(0x1e1e2e))
                                .rounded(px(4.0))
                                .child("Update"),
                        )
                    }),
            )
    }

    fn format_required_version(package: &Package) -> String {
        package
            .required_version
            .as_ref()
            .map(|v| {
                // Extract just the version specifier part (after package name)
                let name_len = package.name.len();
                if v.len() > name_len {
                    let rest = &v[name_len..];
                    // Clean up common patterns
                    rest.trim_start_matches(|c: char| c.is_whitespace() || c == '[')
                        .split(']')
                        .last()
                        .unwrap_or(rest)
                        .trim()
                        .to_string()
                } else {
                    "*".to_string()
                }
            })
            .unwrap_or_else(|| "*".to_string())
    }

    fn format_installed_version(package: &Package) -> String {
        package
            .installed_version
            .as_ref()
            .map(|v| format!("v{v}"))
            .unwrap_or_else(|| "not locked".to_string())
    }

    fn render_empty_state(&self) -> impl IntoElement {
        div()
            .p(px(16.0))
            .text_sm()
            .text_color(rgb(0x6c7086))
            .child("No dependencies")
    }
}

impl RenderOnce for DependencyList {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            // Section title
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child(self.title.clone()),
            )
            // Content container
            .child(
                div()
                    .bg(rgb(0x1e1e2e))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgb(0x313244))
                    .overflow_hidden()
                    .child(if self.packages.is_empty() {
                        div().child(self.render_empty_state())
                    } else {
                        div().children(
                            self.packages
                                .iter()
                                .enumerate()
                                .map(|(i, pkg)| self.render_package_row(pkg, i)),
                        )
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_required_version_simple() {
        let pkg = Package {
            name: "requests".to_string(),
            required_version: Some("requests>=2.28.0".to_string()),
            ..Default::default()
        };
        assert_eq!(DependencyList::format_required_version(&pkg), ">=2.28.0");
    }

    #[test]
    fn test_format_required_version_with_extras() {
        let pkg = Package {
            name: "requests".to_string(),
            required_version: Some("requests[security]>=2.28.0".to_string()),
            ..Default::default()
        };
        assert_eq!(DependencyList::format_required_version(&pkg), ">=2.28.0");
    }

    #[test]
    fn test_format_installed_version() {
        let mut pkg = Package::new("requests");
        assert_eq!(DependencyList::format_installed_version(&pkg), "not locked");

        pkg.installed_version = Some("2.31.0".to_string());
        assert_eq!(DependencyList::format_installed_version(&pkg), "v2.31.0");
    }
}
