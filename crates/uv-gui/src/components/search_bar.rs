//! Search bar component.

use gpui::{
    InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, Styled, div,
    prelude::*, px, rgb,
};

/// A search input component.
#[derive(IntoElement)]
pub struct SearchBar {
    id: SharedString,
    placeholder: SharedString,
    value: String,
    on_change: Option<Box<dyn Fn(&str) + 'static>>,
    on_submit: Option<Box<dyn Fn(&str) + 'static>>,
}

impl SearchBar {
    /// Create a new search bar.
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            placeholder: SharedString::from("Search..."),
            value: String::new(),
            on_change: None,
            on_submit: None,
        }
    }

    /// Set the placeholder text.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the current value.
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Set the change handler.
    pub fn on_change(mut self, handler: impl Fn(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }

    /// Set the submit handler.
    pub fn on_submit(mut self, handler: impl Fn(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SearchBar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let display_text = if self.value.is_empty() {
            self.placeholder.to_string()
        } else {
            self.value.clone()
        };

        let text_color = if self.value.is_empty() {
            rgb(0x6c7086)
        } else {
            rgb(0xcdd6f4)
        };

        div()
            .id(self.id)
            .w_full()
            .h(px(40.0))
            .px(px(12.0))
            .bg(rgb(0x313244))
            .rounded(px(8.0))
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                // Search icon
                div().text_sm().text_color(rgb(0x6c7086)).child("🔍"),
            )
            .child(
                div()
                    .flex_1()
                    .text_sm()
                    .text_color(text_color)
                    .child(display_text),
            )
            .when(!self.value.is_empty(), |el| {
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
                        .child("×"),
                )
            })
    }
}
