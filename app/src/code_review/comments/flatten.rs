use ai::agent::action::{
    ReviewCommentThread, format_review_comment_thread, group_review_comment_threads,
};

use super::comment::{
    AttachedReviewComment, AttachedReviewCommentTarget, CommentId, CommentOrigin,
};
use super::pending_imported::{PendingImportedReviewComment, PendingImportedReviewCommentTarget};
use crate::code::buffer_location::LocalOrRemotePath;


