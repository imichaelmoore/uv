//! Package card component.

use gpui::{
    div, prelude::*, px, rgb, InteractiveElement, IntoElement, ParentElement, RenderOnce,
    SharedString, Styled,
};

use crate::state::Package;

/// A card component for displaying package information.
#[derive(IntoElement)]
pub struct PackageCard {
    package: Package,
    compact: bool,
    show_actions: bool,
    on_install: Option<Box<dyn Fn() + 'static>>,
    on_remove: Option<Box<dyn Fn() + 'static>>,
    on_update: Option<Box<dyn Fn() + 'static>>,
}

impl PackageCard {
    /// Create a new package card.
    pub fn new(package: Package) -> Self {
        Self {
            package,
            compact: false,
            show_actions: true,
            on_install: None,
            on_remove: None,
            on_update: None,
        }
    }

    /// Set compact mode.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Set whether to show actions.
    pub fn show_actions(mut self, show: bool) -> Self {
        self.show_actions = show;
        self
    }

    /// Set the install handler.
    pub fn on_install(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_install = Some(Box::new(handler));
        self
    }

    /// Set the remove handler.
    pub fn on_remove(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_remove = Some(Box::new(handler));
        self
    }

    /// Set the update handler.
    pub fn on_update(mut self, handler: impl Fn() + 'static) -> Self {
        self.on_update = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for PackageCard {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let is_installed = self.package.is_installed();
        let has_update = self.package.has_update();
        let padding = if self.compact { px(12.0) } else { px(16.0) };

        div()
            .id(SharedString::from(format!("pkg-{}", self.package.name)))
            .p(padding)
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
                    .gap(px(6.0))
                    // Name and version row
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
                                    .child(self.package.name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(rgb(0x6c7086))
                                    .child(
                                        self.package
                                            .installed_version
                                            .clone()
                                            .or(self.package.latest_version.clone())
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
                            })
                            .when(has_update, |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .px(px(6.0))
                                        .py(px(2.0))
                                        .bg(rgb(0xf9e2af))
                                        .text_color(rgb(0x1e1e2e))
                                        .rounded(px(4.0))
                                        .child("Update"),
                                )
                            }),
                    )
                    // Description
                    .when(!self.compact, |el| {
                        el.child(
                            div()
                                .text_sm()
                                .text_color(rgb(0xa6adc8))
                                .max_w(px(400.0))
                                .child(
                                    self.package
                                        .description
                                        .clone()
                                        .unwrap_or_else(|| "No description".to_string()),
                                ),
                        )
                    }),
            )
            .when(self.show_actions, |el| {
                el.child(
                    div()
                        .flex()
                        .gap(px(8.0))
                        .when(has_update, |el| {
                            el.child(
                                div()
                                    .id(SharedString::from(format!(
                                        "update-{}",
                                        self.package.name
                                    )))
                                    .px(px(12.0))
                                    .py(px(6.0))
                                    .bg(rgb(0xf9e2af))
                                    .text_color(rgb(0x1e1e2e))
                                    .text_sm()
                                    .rounded(px(6.0))
                                    .cursor_pointer()
                                    .hover(|style| style.bg(rgb(0xf5c2e7)))
                                    .child("Update"),
                            )
                        })
                        .child(
                            div()
                                .id(SharedString::from(format!(
                                    "action-{}",
                                    self.package.name
                                )))
                                .px(px(12.0))
                                .py(px(6.0))
                                .bg(if is_installed {
                                    rgb(0x313244)
                                } else {
                                    rgb(0x89b4fa)
                                })
                                .text_color(if is_installed {
                                    rgb(0xf38ba8)
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
                        ),
                )
            })
    }
}
