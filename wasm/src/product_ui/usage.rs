use ochub_ui::gpui::{self, AnyElement, ParentElement, Styled, div};

/// Documentation-scene usage-summary grid.
pub fn summary_grid(cards: impl IntoIterator<Item = AnyElement>) -> gpui::Div {
    div().grid().grid_cols(2).gap_3().children(cards)
}
