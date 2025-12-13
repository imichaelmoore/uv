//! Settings view.

use gpui::{
    div, px, rgb, Context, FocusHandle, InteractiveElement, IntoElement, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Window,
};

/// View for application settings.
pub struct SettingsView {
    focus_handle: FocusHandle,
    cache_dir: Option<String>,
    python_preference: String,
    color_output: bool,
    offline_mode: bool,
    native_tls: bool,
    preview_features: bool,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            cache_dir: None,
            python_preference: "managed".to_string(),
            color_output: true,
            offline_mode: false,
            native_tls: false,
            preview_features: false,
        }
    }

    fn render_section(&self, title: &str, children: impl IntoElement) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap(px(16.0))
            .child(
                div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(rgb(0xcdd6f4))
                    .child(title.to_string()),
            )
            .child(
                div()
                    .bg(rgb(0x1e1e2e))
                    .rounded(px(12.0))
                    .border_1()
                    .border_color(rgb(0x313244))
                    .overflow_hidden()
                    .child(children),
            )
    }

    fn render_toggle_setting(
        &self,
        id: &str,
        label: &str,
        description: &str,
        enabled: bool,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(id.to_string()))
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
                            .child(label.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(description.to_string()),
                    ),
            )
            .child(self.render_toggle(enabled))
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

    fn render_select_setting(
        &self,
        id: &str,
        label: &str,
        description: &str,
        value: &str,
        _options: &[&str],
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(id.to_string()))
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
                            .child(label.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(description.to_string()),
                    ),
            )
            .child(
                div()
                    .px(px(12.0))
                    .py(px(6.0))
                    .bg(rgb(0x313244))
                    .rounded(px(6.0))
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .cursor_pointer()
                    .hover(|style| style.bg(rgb(0x45475a)))
                    .child(
                        div()
                            .text_sm()
                            .text_color(rgb(0xcdd6f4))
                            .child(value.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child("▼"),
                    ),
            )
    }

    fn render_text_setting(
        &self,
        id: &str,
        label: &str,
        description: &str,
        value: &str,
        placeholder: &str,
    ) -> impl IntoElement {
        div()
            .id(SharedString::from(id.to_string()))
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
                            .child(label.to_string()),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x6c7086))
                            .child(description.to_string()),
                    ),
            )
            .child(
                div()
                    .w(px(250.0))
                    .px(px(12.0))
                    .py(px(8.0))
                    .bg(rgb(0x313244))
                    .rounded(px(6.0))
                    .child(
                        div()
                            .text_sm()
                            .text_color(if value.is_empty() {
                                rgb(0x6c7086)
                            } else {
                                rgb(0xcdd6f4)
                            })
                            .child(if value.is_empty() {
                                placeholder.to_string()
                            } else {
                                value.to_string()
                            }),
                    ),
            )
    }

    fn render_general_settings(&self) -> impl IntoElement {
        self.render_section(
            "General",
            div()
                .child(self.render_select_setting(
                    "python-preference",
                    "Python Preference",
                    "Prefer managed or system Python installations",
                    &self.python_preference,
                    &["managed", "system", "only-managed", "only-system"],
                ))
                .child(self.render_toggle_setting(
                    "color-output",
                    "Color Output",
                    "Enable colored output in the terminal",
                    self.color_output,
                ))
                .child(self.render_toggle_setting(
                    "preview-features",
                    "Preview Features",
                    "Enable experimental features",
                    self.preview_features,
                )),
        )
    }

    fn render_network_settings(&self) -> impl IntoElement {
        self.render_section(
            "Network",
            div()
                .child(self.render_toggle_setting(
                    "offline-mode",
                    "Offline Mode",
                    "Disable network access for package operations",
                    self.offline_mode,
                ))
                .child(self.render_toggle_setting(
                    "native-tls",
                    "Native TLS",
                    "Use the system's native TLS implementation",
                    self.native_tls,
                )),
        )
    }

    fn render_paths_settings(&self) -> impl IntoElement {
        self.render_section(
            "Paths",
            div().child(self.render_text_setting(
                "cache-dir",
                "Cache Directory",
                "Directory for storing cached packages",
                self.cache_dir.as_deref().unwrap_or(""),
                "Default cache location",
            )),
        )
    }

    fn render_about_section(&self) -> impl IntoElement {
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
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(12.0))
                            .child(
                                div()
                                    .id("link-docs")
                                    .text_sm()
                                    .text_color(rgb(0x89b4fa))
                                    .cursor_pointer()
                                    .hover(|style| style.text_color(rgb(0xb4befe)))
                                    .child("Documentation"),
                            )
                            .child(
                                div()
                                    .id("link-github")
                                    .text_sm()
                                    .text_color(rgb(0x89b4fa))
                                    .cursor_pointer()
                                    .hover(|style| style.text_color(rgb(0xb4befe)))
                                    .child("GitHub"),
                            )
                            .child(
                                div()
                                    .id("link-changelog")
                                    .text_sm()
                                    .text_color(rgb(0x89b4fa))
                                    .cursor_pointer()
                                    .hover(|style| style.text_color(rgb(0xb4befe)))
                                    .child("Changelog"),
                            ),
                    ),
            )
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("settings-view")
            .size_full()
            .overflow_y_scroll()
            .bg(rgb(0x181825))
            .p(px(24.0))
            .flex()
            .flex_col()
            .gap(px(24.0))
            .child(self.render_general_settings())
            .child(self.render_network_settings())
            .child(self.render_paths_settings())
            .child(self.render_about_section())
    }
}
