//! Project overview view.

use gpui::{
    Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::*, px, rgb,
};

use crate::state::{Package, ProjectState};

/// View displaying project overview and dependencies.
pub struct ProjectView {
    focus_handle: FocusHandle,
    project: Option<ProjectState>,
}

impl ProjectView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            project: None,
        }
    }

    pub fn set_project(&mut self, project: Option<ProjectState>) {
        self.project = project;
    }

    fn render_empty_state(&self) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(16.0))
            .child(div().text_2xl().text_color(rgb(0x45475a)).child("📦"))
            .child(
                div()
                    .text_xl()
                    .text_color(rgb(0x6c7086))
                    .child("No project loaded"),
            )
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x6c7086))
                    .child("Open a folder containing a pyproject.toml or requirements.txt"),
            )
            .child(
                div()
                    .id("open-project-btn")
                    .mt(px(16.0))
                    .px(px(24.0))
                    .py(px(12.0))
                    .bg(rgb(0x89b4fa))
                    .text_color(rgb(0x1e1e2e))
                    .rounded(px(8.0))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0xb4befe)))
                    .child("Open Project"),
            )
    }

    fn render_project_info(&self, project: &ProjectState) -> impl IntoElement {
        div()
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(24.0))
            // Project header
            .child(
                div()
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
                                    .text_2xl()
                                    .font_weight(gpui::FontWeight::BOLD)
                                    .text_color(rgb(0xcdd6f4))
                                    .child(project.name.clone()),
                            )
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(rgb(0x6c7086))
                                    .child(project.root.display().to_string()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.0))
                            .child(self.render_action_button("Sync", "sync"))
                            .child(self.render_action_button("Lock", "lock"))
                            .child(self.render_action_button("Run", "play")),
                    ),
            )
            // Stats cards
            .child(
                div()
                    .flex()
                    .gap(px(16.0))
                    .child(self.render_stat_card(
                        "Dependencies",
                        &project.dependencies.len().to_string(),
                        rgb(0x89b4fa),
                    ))
                    .child(self.render_stat_card(
                        "Dev Dependencies",
                        &project.dev_dependencies.len().to_string(),
                        rgb(0xf9e2af),
                    ))
                    .child(self.render_stat_card(
                        "Python",
                        project.python_version.as_deref().unwrap_or("Not set"),
                        rgb(0xa6e3a1),
                    ))
                    .child(self.render_stat_card(
                        "Environments",
                        &project.environments.len().to_string(),
                        rgb(0xf5c2e7),
                    )),
            )
            // Dependencies section
            .child(self.render_dependencies_section("Dependencies", &project.dependencies))
            .child(
                self.render_dependencies_section(
                    "Development Dependencies",
                    &project.dev_dependencies,
                ),
            )
    }

    fn render_action_button(&self, label: &str, _icon: &str) -> impl IntoElement {
        let label_text = label.to_string();
        div()
            .id(SharedString::from(format!("btn-{label}")))
            .px(px(16.0))
            .py(px(8.0))
            .bg(rgb(0x313244))
            .text_color(rgb(0xcdd6f4))
            .text_sm()
            .rounded(px(6.0))
            .cursor_pointer()
            .hover(|style| style.bg(rgb(0x45475a)))
            .child(label_text)
    }

    fn render_stat_card(&self, label: &str, value: &str, color: gpui::Rgba) -> impl IntoElement {
        div()
            .flex_1()
            .p(px(16.0))
            .bg(rgb(0x1e1e2e))
            .rounded(px(12.0))
            .border_1()
            .border_color(rgb(0x313244))
            .flex()
            .flex_col()
            .gap(px(8.0))
            .child(
                div()
                    .text_sm()
                    .text_color(rgb(0x6c7086))
                    .child(label.to_string()),
            )
            .child(
                div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(color)
                    .child(value.to_string()),
            )
    }

    fn render_dependencies_section(&self, title: &str, packages: &[Package]) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(12.0))
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child(title.to_string()),
            )
            .child(if packages.is_empty() {
                div()
                    .bg(rgb(0x1e1e2e))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgb(0x313244))
                    .overflow_hidden()
                    .child(
                        div()
                            .p(px(16.0))
                            .text_sm()
                            .text_color(rgb(0x6c7086))
                            .child("No dependencies"),
                    )
            } else {
                div()
                    .bg(rgb(0x1e1e2e))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgb(0x313244))
                    .overflow_hidden()
                    .children(
                        packages
                            .iter()
                            .enumerate()
                            .map(|(i, pkg)| self.render_package_row(pkg, i)),
                    )
            })
    }

    fn render_package_row(&self, package: &Package, index: usize) -> impl IntoElement {
        let bg_color = if index % 2 == 0 {
            rgb(0x1e1e2e)
        } else {
            rgb(0x181825)
        };

        div()
            .id(SharedString::from(format!("pkg-{}", package.name)))
            .px(px(16.0))
            .py(px(12.0))
            .bg(bg_color)
            .flex()
            .items_center()
            .justify_between()
            .hover(|style| style.bg(rgb(0x313244)))
            .cursor_pointer()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::MEDIUM)
                            .text_color(rgb(0xcdd6f4))
                            .child(package.name.clone()),
                    )
                    .child(
                        div().text_xs().text_color(rgb(0x6c7086)).child(
                            package
                                .installed_version
                                .clone()
                                .unwrap_or_else(|| "not installed".to_string()),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .when(package.update_available, |el| {
                        el.child(
                            div()
                                .text_xs()
                                .px(px(8.0))
                                .py(px(2.0))
                                .bg(rgb(0xa6e3a1))
                                .text_color(rgb(0x1e1e2e))
                                .rounded(px(4.0))
                                .child("Update available"),
                        )
                    }),
            )
    }
}

impl Render for ProjectView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("project-view")
            .size_full()
            .overflow_y_scroll()
            .bg(rgb(0x181825))
            .child(match &self.project {
                Some(project) => div().child(self.render_project_info(project)),
                None => div().child(self.render_empty_state()),
            })
    }
}
