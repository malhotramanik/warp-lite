use warpui::elements::{CrossAxisAlignment, Flex};
use warpui::{AppContext, Element};

use crate::ai::blocklist::block::ImportedCommentGroup;

pub(crate) fn render_imported_comments(
    group: &ImportedCommentGroup,
    app: &AppContext,
) -> Box<dyn Element> {
    // warp-lite: code review comment cards are removed.
    let _ = (group, app);
    Flex::column()
        .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
        .finish()
}
