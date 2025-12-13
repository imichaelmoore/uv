//! Virtual environment management view.

use gpui::{
    div, prelude::*, px, rgb, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement,
    Render, SharedString, StatefulInteractiveElement, Styled, Window,
};

use crate::state::Environment;

/// View for managing virtual environments.
pub struct EnvironmentsView {
    focus_handle: FocusHandle,
    environments: Vec<Environment>,
    selected_environment: Option<String>,
    is_creating: bool,
    new_env_name: String,
    new_env_python: String,
}

impl EnvironmentsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            environments: Vec::new(),
            selected_environment: None,
            is_creating: false,
            new_env_name: String::new(),
            new_env_python: String::new(),
        }
    }

    pub fn set_environments(&mut self, environments: Vec<Environment>) {
        self.environments = environments;
    }

    fn render_header(&self) -> impl IntoElement {
        div()
            .p(px(24.0))
            .flex()
            .justify_between()
            .items_center()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(4.0))
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(rgb(0xcdd6f4))
                            .child("Virtual Environments"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child(format!("{} environments", self.environments.len())),
                    ),
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
            )
    }

    fn render_environments_list(&self) -> impl IntoElement {
        if self.environments.is_empty() {
            div()
                .px(px(24.0))
                .pb(px(24.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .child(self.render_empty_state())
        } else {
            div()
                .px(px(24.0))
                .pb(px(24.0))
                .flex()
                .flex_col()
                .gap(px(12.0))
                .children(
                    self.environments
                        .iter()
                        .map(|env| self.render_environment_card(env)),
                )
        }
    }

    fn render_environment_card(&self, env: &Environment) -> impl IntoElement {
        let is_selected = self
            .selected_environment
            .as_ref()
            .is_some_and(|s| s == &env.name);
        let border_color = if env.is_active {
            rgb(0xa6e3a1)
        } else if is_selected {
            rgb(0x89b4fa)
        } else {
            rgb(0x313244)
        };

        div()
            .id(SharedString::from(format!("env-{}", env.name)))
            .p(px(16.0))
            .bg(rgb(0x1e1e2e))
            .rounded(px(12.0))
            .border_2()
            .border_color(border_color)
            .hover(|style| style.border_color(rgb(0x45475a)))
            .cursor_pointer()
            .flex()
            .justify_between()
            .items_center()
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
                                    .child(env.name.clone()),
                            )
                            .when(env.is_active, |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .px(px(8.0))
                                        .py(px(2.0))
                                        .bg(rgb(0xa6e3a1))
                                        .text_color(rgb(0x1e1e2e))
                                        .rounded(px(4.0))
                                        .child("Active"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x6c7086))
                                            .child("Python:"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xa6adc8))
                                            .child(env.python_version.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x6c7086))
                                            .child("Packages:"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xa6adc8))
                                            .child(env.package_count.to_string()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(4.0))
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0x6c7086))
                                            .child("Size:"),
                                    )
                                    .child(
                                        div()
                                            .text_sm()
                                            .text_color(rgb(0xa6adc8))
                                            .child(env.size_display()),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(env.path.display().to_string()),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.0))
                    .child(
                        div()
                            .id(SharedString::from(format!("activate-{}", env.name)))
                            .px(px(12.0))
                            .py(px(6.0))
                            .bg(if env.is_active {
                                rgb(0x45475a)
                            } else {
                                rgb(0x89b4fa)
                            })
                            .text_color(if env.is_active {
                                rgb(0x6c7086)
                            } else {
                                rgb(0x1e1e2e)
                            })
                            .text_sm()
                            .rounded(px(6.0))
                            .cursor(if env.is_active {
                                gpui::CursorStyle::default()
                            } else {
                                gpui::CursorStyle::PointingHand
                            })
                            .when(!env.is_active, |el| {
                                el.hover(|style| style.bg(rgb(0xb4befe)))
                            })
                            .child(if env.is_active {
                                "Active"
                            } else {
                                "Activate"
                            }),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("delete-{}", env.name)))
                            .px(px(12.0))
                            .py(px(6.0))
                            .bg(rgb(0x313244))
                            .text_color(rgb(0xf38ba8))
                            .text_sm()
                            .rounded(px(6.0))
                            .cursor_pointer()
                            .hover(|style| style.bg(rgb(0x45475a)))
                            .child("Delete"),
                    ),
            )
    }

    fn render_empty_state(&self) -> impl IntoElement {
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
                    .child("📦"),
            )
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(0x6c7086))
                    .child("No virtual environments"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x6c7086))
                    .child("Create a virtual environment to isolate your project dependencies"),
            )
    }
}

impl Render for EnvironmentsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("environments-view")
            .size_full()
            .overflow_y_scroll()
            .bg(rgb(0x181825))
            .flex()
            .flex_col()
            .child(self.render_header())
            .child(self.render_environments_list())
    }
}
