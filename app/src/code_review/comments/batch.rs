use std::collections::HashMap;

use warp_editor::render::model::LineCount;
use warpui::{Entity, ModelContext};

use super::{
    AttachedReviewComment, AttachedReviewCommentTarget, CommentId, PendingImportedReviewComment,
};
use crate::code::buffer_location::LocalOrRemotePath;
use crate::code_review::diff_state::DiffMode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReviewCommentBatchEvent {
    Changed { should_reposition_comments: bool },
}

#[derive(Clone, Debug, Default)]
pub struct ReviewCommentBatch {
    /// Comments that are attached to local editors and visible to the user.
    pub comments: Vec<AttachedReviewComment>,
    /// Imported comments waiting for editors and diffs to load before they can be displayed to the user.
    /// Comments are grouped by base branch.
    pending_imported_comments: HashMap<DiffMode, Vec<PendingImportedReviewComment>>,
}

impl Entity for ReviewCommentBatch {
    type Event = ReviewCommentBatchEvent;
}

impl ReviewCommentBatch {
    pub fn from_comments(comments: Vec<AttachedReviewComment>) -> Self {
        Self {
            comments,
            pending_imported_comments: HashMap::new(),
        }
    }

    pub(crate) fn get_review_comment_by_id(&self, id: CommentId) -> Option<&AttachedReviewComment> {
        self.comments.iter().find(|comment| comment.id == id)
    }

    pub(super) fn get_mut_review_comment_by_id(
        &mut self,
        id: CommentId,
    ) -> Option<&mut AttachedReviewComment> {
        self.comments.iter_mut().find(|comment| comment.id == id)
    }



    /// `file` should be the host-aware absolute path for the editor file.
    pub fn file_comments<'a>(
        &'a self,
        file: &'a LocalOrRemotePath,
    ) -> impl Iterator<Item = &'a AttachedReviewComment> + 'a {
        self.comments.iter().filter(move |comment| {
            comment
                .target
                .absolute_file_path()
                .is_some_and(|comment_file| comment_file == file)
        })
    }

    /// `file` should be the host-aware absolute path for the editor file.
    pub fn comment_line_numbers_for_file<'a>(
        &'a self,
        file: &'a LocalOrRemotePath,
    ) -> impl Iterator<Item = LineCount> + 'a {
        self.file_comments(file).filter_map(move |comment| {
            if let AttachedReviewCommentTarget::Line {
                absolute_file_path: comment_file_path,
                line,
                ..
            } = &comment.target
            {
                if comment_file_path == file {
                    line.line_number()
                } else {
                    None
                }
            } else {
                None
            }
        })
    }




    /// Comments with existing IDs are updated.
    /// New comments are inserted into the batch.
    pub fn upsert_comments(
        &mut self,
        comments: Vec<AttachedReviewComment>,
        ctx: &mut ModelContext<Self>,
    ) {
        self.upsert_comments_inner(comments);
        ctx.emit(ReviewCommentBatchEvent::Changed {
            should_reposition_comments: false,
        });
    }

    fn upsert_comments_inner(&mut self, comments: Vec<AttachedReviewComment>) {
        let (existing_comments, new_comments): (
            Vec<AttachedReviewComment>,
            Vec<AttachedReviewComment>,
        ) = comments
            .into_iter()
            .partition(|c| self.get_review_comment_by_id(c.id).is_some());

        self.comments.extend(new_comments);
        for c in existing_comments {
            if let Some(existing_entry) = self.get_mut_review_comment_by_id(c.id) {
                *existing_entry = c;
            }
        }
    }





}

#[cfg(test)]
#[path = "batch_tests.rs"]
mod tests;
