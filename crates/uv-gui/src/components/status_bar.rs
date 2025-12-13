//! Status bar component.

use gpui::{
    div, prelude::*, px, rgb, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
};

/// Status bar position.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StatusBarPosition {
    /// Status bar at the top.
    Top,
    /// Status bar at the bottom.
    #[default]
    Bottom,
}

/// A status bar component for showing application state.
#[derive(IntoElement)]
pub struct StatusBar {
    left_items: Vec<StatusBarItem>,
    center_items: Vec<StatusBarItem>,
    right_items: Vec<StatusBarItem>,
    position: StatusBarPosition,
}

/// An item in the status bar.
#[derive(Clone)]
pub struct StatusBarItem {
    id: SharedString,
    content: SharedString,
    icon: Option<SharedString>,
    color: Option<gpui::Rgba>,
}

impl StatusBarItem {
    /// Create a new status bar item.
    pub fn new(id: impl Into<SharedString>, content: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            icon: None,
            color: None,
        }
    }

    /// Set an icon for the item.
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set a color for the item.
    pub fn color(mut self, color: gpui::Rgba) -> Self {
        self.color = Some(color);
        self
    }
}

impl StatusBar {
    /// Create a new status bar.
    pub fn new() -> Self {
        Self {
            left_items: Vec::new(),
            center_items: Vec::new(),
            right_items: Vec::new(),
            position: StatusBarPosition::Bottom,
        }
    }

    /// Set the position.
    pub fn position(mut self, position: StatusBarPosition) -> Self {
        self.position = position;
        self
    }

    /// Add an item to the left side.
    pub fn left(mut self, item: StatusBarItem) -> Self {
        self.left_items.push(item);
        self
    }

    /// Add an item to the center.
    pub fn center(mut self, item: StatusBarItem) -> Self {
        self.center_items.push(item);
        self
    }

    /// Add an item to the right side.
    pub fn right(mut self, item: StatusBarItem) -> Self {
        self.right_items.push(item);
        self
    }

    fn render_item(&self, item: &StatusBarItem) -> impl IntoElement {
        let text_color = item.color.unwrap_or(rgb(0xa6adc8));

        div()
            .id(item.id.clone())
            .flex()
            .items_center()
            .gap(px(4.0))
            .when(item.icon.is_some(), |el| {
                el.child(
                    div()
                        .text_xs()
                        .text_color(text_color)
                        .child(item.icon.clone().unwrap_or_default().to_string()),
                )
            })
            .child(
                div()
                    .text_xs()
                    .text_color(text_color)
                    .child(item.content.to_string()),
            )
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for StatusBar {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let border = match self.position {
            StatusBarPosition::Top => div().border_b_1(),
            StatusBarPosition::Bottom => div().border_t_1(),
        };

        border
            .id("status-bar")
            .w_full()
            .h(px(28.0))
            .px(px(12.0))
            .bg(rgb(0x1e1e2e))
            .border_color(rgb(0x313244))
            .flex()
            .items_center()
            .justify_between()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .children(self.left_items.iter().map(|item| self.render_item(item))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .children(self.center_items.iter().map(|item| self.render_item(item))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.0))
                    .children(self.right_items.iter().map(|item| self.render_item(item))),
            )
    }
}
