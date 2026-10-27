use std::fmt::{Display, Formatter};

use chrono::{DateTime, Local};
use warp_editor::render::model::LineCount;
use warp_multi_agent_api::{self as api};

use crate::ai::agent::{CurrentHead, DiffBase};
use crate::code::buffer_location::LocalOrRemotePath;
use crate::code::editor::line::EditorLineLocation;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum CommentOrigin {
    /// Comments originally created in the Warp UI.
    #[default]
    Native,
    /// Comments imported from a GitHub pull request.
    ImportedFromGitHub(ImportedCommentDetails),
}

impl CommentOrigin {
}

/// Imported comment metadata for GitHub-specific fields.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImportedCommentDetails {
    pub author: String,
    /// The GitHub comment ID from the API.
    pub github_comment_id: String,
    /// The GitHub parent comment ID if this was a reply.
    /// Should be None for threaded comments after flattening.
    pub github_parent_id: Option<String>,
    pub html_url: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct LineDiffContent {
    pub content: String,
    pub lines_added: LineCount,
    pub lines_removed: LineCount,
}

impl LineDiffContent {


}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct CommentId(uuid::Uuid);

impl CommentId {
    pub(crate) fn new() -> Self {
        CommentId(uuid::Uuid::new_v4())
    }

    pub(crate) fn from_uuid(uuid: uuid::Uuid) -> Self {
        Self(uuid)
    }
}

impl Display for CommentId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for CommentId {
    fn default() -> Self {
        Self::new()
    }
}

/// Code review comment attached to a local file editor.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttachedReviewComment {
    /// Locally-generated ID.
    pub id: CommentId,
    pub content: String,
    pub target: AttachedReviewCommentTarget,
    pub last_update_time: DateTime<Local>,
    pub base: Option<DiffBase>,
    pub head: Option<CurrentHead>,
    pub outdated: bool,
    pub origin: CommentOrigin,
}

impl From<AttachedReviewComment> for api::ReviewComment {
    fn from(val: AttachedReviewComment) -> Self {
        let comment_target = match val.target {
            AttachedReviewCommentTarget::Line {
                absolute_file_path,
                content,
                line,
            } => {
                // For now, comments are only attached to a single line.
                let line_range = line.line_number().map(|lc| {
                    let line_number = lc.as_usize() as u32;
                    api::FileContentLineRange {
                        start: line_number,
                        end: line_number + 1,
                    }
                });

                api::review_comment::CommentTarget::CommentedLine(api::DiffHunk {
                    // For the agent/GitHub API we send the path bytes only;
                    // the comment's owning batch is already host-scoped.
                    file_path: absolute_file_path.display_path(),
                    line_range,
                    diff_content: content.content,
                    lines_added: content.lines_added.as_u32(),
                    lines_removed: content.lines_removed.as_u32(),
                    current: val.head.to_owned().map(Into::into),
                    base: val.base.map(Into::into),
                })
            }
            AttachedReviewCommentTarget::File { absolute_file_path } => {
                api::review_comment::CommentTarget::CommentedFile(
                    api::review_comment::CommentedFile {
                        file_path: absolute_file_path.display_path(),
                        current: val.head.to_owned().map(Into::into),
                        base: val.base.map(Into::into),
                    },
                )
            }
            AttachedReviewCommentTarget::General => {
                api::review_comment::CommentTarget::CommentedDiffset(
                    api::review_comment::CommentedDiffset {
                        current: val.head.to_owned().map(Into::into),
                        base: val.base.map(Into::into),
                    },
                )
            }
        };

        api::ReviewComment {
            id: val.id.to_string(),
            comment: val.content,
            comment_target: Some(comment_target),
        }
    }
}

/// Target for an attached review comment. File paths are always absolute when present.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AttachedReviewCommentTarget {
    Line {
        absolute_file_path: LocalOrRemotePath,
        line: EditorLineLocation,
        content: LineDiffContent,
    },
    File {
        absolute_file_path: LocalOrRemotePath,
    },
    General,
}

impl AttachedReviewCommentTarget {
    pub(crate) fn absolute_file_path(&self) -> Option<&LocalOrRemotePath> {
        match self {
            AttachedReviewCommentTarget::Line {
                absolute_file_path, ..
            } => Some(absolute_file_path),
            AttachedReviewCommentTarget::File { absolute_file_path } => Some(absolute_file_path),
            AttachedReviewCommentTarget::General => None,
        }
    }

}

impl AttachedReviewComment {

    pub fn head(&self) -> Option<&CurrentHead> {
        self.head.as_ref()
    }

    pub fn origin(&self) -> &CommentOrigin {
        &self.origin
    }
}

#[cfg(test)]
#[path = "comment_tests.rs"]
mod tests;
