//! Python version management view.

use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px, rgb,
};

use crate::state::PythonInstallation;

/// View for managing Python installations.
pub struct PythonView {
    focus_handle: FocusHandle,
    installed_versions: Vec<PythonInstallation>,
    available_versions: Vec<String>,
    is_installing: bool,
    install_progress: Option<f32>,
}

impl PythonView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            installed_versions: Vec::new(),
            available_versions: vec![
                "3.13.0".to_string(),
                "3.12.7".to_string(),
                "3.12.6".to_string(),
                "3.11.10".to_string(),
                "3.11.9".to_string(),
                "3.10.15".to_string(),
                "3.10.14".to_string(),
                "3.9.20".to_string(),
                "3.9.19".to_string(),
                "3.8.20".to_string(),
            ],
            is_installing: false,
            install_progress: None,
        }
    }

    pub fn set_installed_versions(&mut self, versions: Vec<PythonInstallation>) {
        self.installed_versions = versions;
    }

    fn render_installed_section(&self) -> impl IntoElement {
        div()
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
                            .child("Installed Python Versions"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child(format!("{} installed", self.installed_versions.len())),
                    ),
            )
            .child(if self.installed_versions.is_empty() {
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.0))
                    .child(self.render_no_installed())
            } else {
                div().flex().flex_col().gap(px(8.0)).children(
                    self.installed_versions
                        .iter()
                        .map(|py| self.render_installed_card(py)),
                )
            })
    }

    fn render_installed_card(&self, py: &PythonInstallation) -> impl IntoElement {
        let border_color = if py.is_default {
            rgb(0xa6e3a1)
        } else {
            rgb(0x313244)
        };

        div()
            .id(SharedString::from(format!("py-{}", py.version)))
            .p(px(16.0))
            .bg(rgb(0x1e1e2e))
            .rounded(px(12.0))
            .border_2()
            .border_color(border_color)
            .hover(|style| style.border_color(rgb(0x45475a)))
            .flex()
            .justify_between()
            .items_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
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
                                    .child(format!("Python {}", py.version)),
                            )
                            .when(py.is_default, |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .px(px(8.0))
                                        .py(px(2.0))
                                        .bg(rgb(0xa6e3a1))
                                        .text_color(rgb(0x1e1e2e))
                                        .rounded(px(4.0))
                                        .child("Default"),
                                )
                            })
                            .when(py.is_managed, |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .px(px(8.0))
                                        .py(px(2.0))
                                        .bg(rgb(0x89b4fa))
                                        .text_color(rgb(0x1e1e2e))
                                        .rounded(px(4.0))
                                        .child("Managed"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x6c7086))
                                    .child(py.implementation.clone()),
                            )
                            .when(py.architecture.is_some(), |el| {
                                el.child(
                                    div()
                                        .text_sm()
                                        .text_color(rgb(0x6c7086))
                                        .child(py.architecture.clone().unwrap_or_default()),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(py.path.display().to_string()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .when(!py.is_default, |el| {
                        el.child(
                            div()
                                .id(SharedString::from(format!("set-default-{}", py.version)))
                                .px(px(12.0))
                                .py(px(6.0))
                                .bg(rgb(0x89b4fa))
                                .text_color(rgb(0x1e1e2e))
                                .text_sm()
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(0xb4befe)))
                                .child("Set Default"),
                        )
                    })
                    .when(py.is_managed, |el| {
                        el.child(
                            div()
                                .id(SharedString::from(format!("uninstall-{}", py.version)))
                                .px(px(12.0))
                                .py(px(6.0))
                                .bg(rgb(0x313244))
                                .text_color(rgb(0xf38ba8))
                                .text_sm()
                                .rounded(px(6.0))
                                .cursor_pointer()
                                .hover(|style| style.bg(rgb(0x45475a)))
                                .child("Uninstall"),
                        )
                    }),
            )
    }

    fn render_no_installed(&self) -> impl IntoElement {
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
    }

    fn render_available_section(&self) -> impl IntoElement {
        let installed_versions: std::collections::HashSet<_> = self
            .installed_versions
            .iter()
            .map(|py| &py.version)
            .collect();

        div()
            .px(px(24.0))
            .pb(px(24.0))
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
            .child(div().flex().flex_wrap().gap(px(12.0)).children(
                self.available_versions.iter().map(|version| {
                    let is_installed = installed_versions.contains(version);
                    self.render_version_chip(version, is_installed)
                }),
            ))
    }

    fn render_version_chip(&self, version: &str, is_installed: bool) -> impl IntoElement {
        div()
            .id(SharedString::from(format!("install-py-{version}")))
            .px(px(16.0))
            .py(px(10.0))
            .bg(if is_installed {
                rgb(0x313244)
            } else {
                rgb(0x1e1e2e)
            })
            .border_1()
            .border_color(rgb(0x313244))
            .rounded(px(8.0))
            .cursor(if is_installed {
                gpui::CursorStyle::default()
            } else {
                gpui::CursorStyle::PointingHand
            })
            .when(!is_installed, |el| {
                el.hover(|style| style.bg(rgb(0x313244)).border_color(rgb(0x89b4fa)))
            })
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                div()
                    .text_sm()
                    .font_weight(gpui::FontWeight::MEDIUM)
                    .text_color(if is_installed {
                        rgb(0x6c7086)
                    } else {
                        rgb(0xcdd6f4)
                    })
                    .child(format!("Python {version}")),
            )
            .when(is_installed, |el| {
                el.child(div().text_xs().text_color(rgb(0xa6e3a1)).child("✓"))
            })
            .when(!is_installed, |el| {
                el.child(div().text_xs().text_color(rgb(0x89b4fa)).child("Install"))
            })
    }
}

impl Render for PythonView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("python-view")
            .size_full()
            .overflow_y_scroll()
            .bg(rgb(0x181825))
            .flex()
            .flex_col()
            .child(self.render_installed_section())
            .child(self.render_available_section())
    }
}
