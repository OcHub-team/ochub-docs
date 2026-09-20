use ochub_ui::gpui::{self, IntoElement, ParentElement, SharedString, Styled};

use ochub_ui::layout;

/// Documentation-scene page frame.
pub fn page(
    title: impl Into<SharedString>,
    actions: impl IntoElement,
    body: impl IntoElement,
) -> gpui::Div {
    layout::page()
        .relative()
        .child(layout::page_header(title, None).child(actions))
        .child(body)
}
