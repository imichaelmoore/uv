//! Button component.

use gpui::{
    div, px, rgb, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled,
};

/// Button visual style variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonVariant {
    /// Primary action button.
    #[default]
    Primary,
    /// Secondary action button.
    Secondary,
    /// Destructive action button.
    Danger,
    /// Ghost button with minimal styling.
    Ghost,
}

/// Custom button styling options.
#[derive(Clone, Debug, Default)]
pub struct ButtonStyle {
    /// The button variant.
    pub variant: ButtonVariant,
    /// Whether the button is disabled.
    pub disabled: bool,
    /// Whether the button shows a loading state.
    pub loading: bool,
    /// Whether the button is full width.
    pub full_width: bool,
    /// Custom size (small, medium, large).
    pub size: ButtonSize,
}

/// Button size options.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    /// Small button.
    Small,
    /// Medium (default) button.
    #[default]
    Medium,
    /// Large button.
    Large,
}

/// A styled button component.
#[derive(IntoElement)]
pub struct Button {
    id: SharedString,
    label: SharedString,
    style: ButtonStyle,
}

impl Button {
    /// Create a new button with the given label.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            style: ButtonStyle::default(),
        }
    }

    /// Set the button variant.
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.style.variant = variant;
        self
    }

    /// Set the button size.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.style.size = size;
        self
    }

    /// Set whether the button is disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.style.disabled = disabled;
        self
    }

    /// Set whether the button shows a loading state.
    pub fn loading(mut self, loading: bool) -> Self {
        self.style.loading = loading;
        self
    }

    /// Set whether the button is full width.
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.style.full_width = full_width;
        self
    }

    fn get_colors(&self) -> (gpui::Rgba, gpui::Rgba, gpui::Rgba) {
        match self.style.variant {
            ButtonVariant::Primary => (rgb(0x89b4fa), rgb(0xb4befe), rgb(0x1e1e2e)),
            ButtonVariant::Secondary => (rgb(0x313244), rgb(0x45475a), rgb(0xcdd6f4)),
            ButtonVariant::Danger => (rgb(0xf38ba8), rgb(0xeba0ac), rgb(0x1e1e2e)),
            ButtonVariant::Ghost => (rgb(0x00000000), rgb(0x313244), rgb(0xcdd6f4)),
        }
    }

    fn get_padding(&self) -> (gpui::Pixels, gpui::Pixels) {
        match self.style.size {
            ButtonSize::Small => (px(8.0), px(4.0)),
            ButtonSize::Medium => (px(16.0), px(8.0)),
            ButtonSize::Large => (px(24.0), px(12.0)),
        }
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut gpui::Window, _cx: &mut gpui::App) -> impl IntoElement {
        let (bg, hover_bg, text_color) = self.get_colors();
        let (px_padding, py_padding) = self.get_padding();

        let mut el = div()
            .id(self.id)
            .px(px_padding)
            .py(py_padding)
            .bg(bg)
            .text_color(text_color)
            .rounded(px(6.0))
            .cursor_pointer()
            .text_sm()
            .font_weight(gpui::FontWeight::MEDIUM);

        if self.style.full_width {
            el = el.w_full().flex().items_center().justify_center();
        }

        if self.style.disabled {
            el = el.opacity(0.5).cursor_default();
        } else {
            el = el.hover(move |style| style.bg(hover_bg));
        }

        let label = if self.style.loading {
            "Loading...".to_string()
        } else {
            self.label.to_string()
        };

        el.child(label)
    }
}
