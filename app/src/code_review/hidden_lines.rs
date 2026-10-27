use std::ops::Range;

use ai::diff_validation::DiffDelta;
use rangemap::RangeSet;
use warp_editor::content::text::LineCount;
use warp_editor::render::model::LineCount as RenderLineCount;



#[cfg(test)]
#[path = "hidden_lines_tests.rs"]
mod tests;
