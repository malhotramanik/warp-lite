use std::path::Path;

use anyhow::anyhow;
use ui_components::lightbox::{LightboxImage, LightboxImageSource};
use warp_errors::report_error;
use warp_multi_agent_api as api;
use warpui::SingletonEntity;

#[cfg(feature = "local_fs")]
use crate::ai::artifact_download::sanitized_basename;
use crate::notebooks::NotebookId;
use crate::server::server_api::ServerApiProvider;
use crate::server::server_api::ai::ArtifactDownloadResponse;
use crate::workspace::WorkspaceAction;

pub mod buttons;
pub use buttons::{ArtifactButtonsRow, ArtifactButtonsRowEvent};

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(tag = "artifact_type", content = "data")]
pub enum Artifact {
    #[serde(rename = "PLAN")]
    Plan {
        document_uid: String,
        /// None until the plan is synced to Warp Drive.
        notebook_uid: Option<NotebookId>,
        title: Option<String>,
    },
    #[serde(rename = "PULL_REQUEST")]
    PullRequest {
        url: String,
        branch: String,
        #[serde(skip_serializing)] // We derive this field from the url on deserialize
        repo: Option<String>,
        #[serde(skip_serializing)] // We derive this field from the url on deserialize
        number: Option<u32>,
    },
    #[serde(rename = "EXTERNAL_REFERENCE")]
    ExternalReference {
        reference_type: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        metadata: Option<serde_json::Value>,
    },
    #[serde(rename = "SCREENSHOT")]
    Screenshot {
        artifact_uid: String,
        mime_type: String,
        description: Option<String>,
    },
    #[serde(rename = "FILE")]
    File {
        artifact_uid: String,
        filepath: String,
        filename: String,
        mime_type: String,
        description: Option<String>,
        size_bytes: Option<i32>,
    },
}

/// The tag names of [`Artifact`], used for unknown-variant errors.
const ARTIFACT_TYPES: &[&str] = &[
    "PLAN",
    "PULL_REQUEST",
    "EXTERNAL_REFERENCE",
    "SCREENSHOT",
    "FILE",
];

/// The adjacently tagged envelope shape of a serialized [`Artifact`]. The
/// `data` payload is parsed once the tag is known.
#[derive(serde::Deserialize)]
struct ArtifactEnvelope {
    artifact_type: String,
    data: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct PlanData {
    document_uid: String,
    notebook_uid: Option<NotebookId>,
    title: Option<String>,
}

#[derive(serde::Deserialize)]
struct PullRequestData {
    url: String,
    branch: String,
}

#[derive(serde::Deserialize)]
struct ExternalReferenceData {
    reference_type: String,
    url: String,
    title: Option<String>,
    metadata: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct ScreenshotData {
    artifact_uid: String,
    mime_type: String,
    description: Option<String>,
}

#[derive(serde::Deserialize)]
struct FileData {
    artifact_uid: String,
    filepath: String,
    filename: String,
    mime_type: String,
    description: Option<String>,
    size_bytes: Option<i32>,
}

/// Parses a buffered JSON value into an artifact payload type.
fn parse_artifact_data<T: serde::de::DeserializeOwned, E: serde::de::Error>(
    data: serde_json::Value,
) -> Result<T, E> {
    serde_json::from_value(data).map_err(E::custom)
}

impl<'de> serde::Deserialize<'de> for Artifact {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let envelope = ArtifactEnvelope::deserialize(deserializer)?;
        Ok(match envelope.artifact_type.as_str() {
            "PLAN" => {
                let PlanData {
                    document_uid,
                    notebook_uid,
                    title,
                } = parse_artifact_data::<_, D::Error>(envelope.data)?;
                Artifact::Plan {
                    document_uid,
                    notebook_uid,
                    title,
                }
            }
            "PULL_REQUEST" => {
                let PullRequestData { url, branch } =
                    parse_artifact_data::<_, D::Error>(envelope.data)?;
                let (repo, number) = parse_github_pr_url(&url).unzip();
                Artifact::PullRequest {
                    url,
                    branch,
                    repo,
                    number,
                }
            }
            "EXTERNAL_REFERENCE" => {
                let ExternalReferenceData {
                    reference_type,
                    url,
                    title,
                    metadata,
                } = parse_artifact_data::<_, D::Error>(envelope.data)?;
                Artifact::ExternalReference {
                    reference_type,
                    url,
                    title,
                    metadata,
                }
            }
            "SCREENSHOT" => {
                let ScreenshotData {
                    artifact_uid,
                    mime_type,
                    description,
                } = parse_artifact_data::<_, D::Error>(envelope.data)?;
                Artifact::Screenshot {
                    artifact_uid,
                    mime_type,
                    description,
                }
            }
            "FILE" => {
                let FileData {
                    artifact_uid,
                    filepath,
                    filename,
                    mime_type,
                    description,
                    size_bytes,
                } = parse_artifact_data::<_, D::Error>(envelope.data)?;
                Artifact::File {
                    artifact_uid,
                    filepath,
                    filename,
                    mime_type,
                    description,
                    size_bytes,
                }
            }
            unknown => {
                return Err(serde::de::Error::unknown_variant(unknown, ARTIFACT_TYPES));
            }
        })
    }
}

impl From<api::message::artifact_event::PullRequestArtifact> for Artifact {
    fn from(pr: api::message::artifact_event::PullRequestArtifact) -> Self {
        let (repo, number) = parse_github_pr_url(&pr.url).unzip();
        Artifact::PullRequest {
            url: pr.url,
            branch: pr.branch,
            repo,
            number,
        }
    }
}

impl From<api::message::artifact_event::ScreenshotArtifact> for Artifact {
    fn from(screenshot: api::message::artifact_event::ScreenshotArtifact) -> Self {
        Artifact::Screenshot {
            artifact_uid: screenshot.artifact_uid,
            mime_type: screenshot.mime_type,
            description: if screenshot.description.is_empty() {
                None
            } else {
                Some(screenshot.description)
            },
        }
    }
}

impl From<api::message::artifact_event::FileArtifact> for Artifact {
    fn from(file: api::message::artifact_event::FileArtifact) -> Self {
        Artifact::File {
            artifact_uid: file.artifact_uid,
            filepath: file.filepath.clone(),
            filename: Path::new(&file.filepath)
                .file_name()
                .and_then(|file_name| file_name.to_str())
                .filter(|file_name| !file_name.trim().is_empty())
                .unwrap_or("File")
                .to_string(),
            mime_type: file.mime_type,
            description: if file.description.is_empty() {
                None
            } else {
                Some(file.description)
            },
            size_bytes: i32::try_from(file.size_bytes).ok(),
        }
    }
}

impl From<api::message::artifact_event::PlanArtifact> for Artifact {
    fn from(plan: api::message::artifact_event::PlanArtifact) -> Self {
        Artifact::Plan {
            document_uid: plan.document_id,
            notebook_uid: if plan.notebook_uid.is_empty() {
                None
            } else {
                Some(NotebookId::from(plan.notebook_uid))
            },
            title: if plan.title.is_empty() {
                None
            } else {
                Some(plan.title)
            },
        }
    }
}

impl TryFrom<warp_graphql::ai::AIConversationArtifact> for Artifact {
    type Error = ();

    fn try_from(value: warp_graphql::ai::AIConversationArtifact) -> Result<Self, Self::Error> {
        match value {
            warp_graphql::ai::AIConversationArtifact::PlanArtifact(plan) => Ok(Artifact::Plan {
                document_uid: plan.document_uid.into_inner(),
                notebook_uid: plan
                    .notebook_uid
                    .map(|id| NotebookId::from(id.into_inner())),
                title: plan.title,
            }),
            warp_graphql::ai::AIConversationArtifact::PullRequestArtifact(pr) => {
                let (repo, number) = parse_github_pr_url(&pr.url).unzip();
                Ok(Artifact::PullRequest {
                    url: pr.url,
                    branch: pr.branch,
                    repo,
                    number,
                })
            }
            warp_graphql::ai::AIConversationArtifact::ScreenshotArtifact(screenshot) => {
                Ok(Artifact::Screenshot {
                    artifact_uid: screenshot.artifact_uid.into_inner(),
                    mime_type: screenshot.mime_type,
                    description: screenshot.description,
                })
            }
            warp_graphql::ai::AIConversationArtifact::FileArtifact(file) => Ok(Artifact::File {
                artifact_uid: file.artifact_uid.into_inner(),
                filepath: file.filepath.clone(),
                filename: sanitized_basename(&file.filepath).unwrap_or(file.filepath),
                mime_type: file.mime_type,
                description: file.description,
                size_bytes: file.size_bytes,
            }),
            warp_graphql::ai::AIConversationArtifact::Unknown => Err(()),
        }
    }
}

/// Parse GitHub PR URL to extract repo and number.
/// Expected format: https://github.com/{owner}/{repo}/pull/{number}
pub fn parse_github_pr_url(url: &str) -> Option<(String, u32)> {
    if !url.contains("github.com") {
        return None;
    }
    let segments: Vec<&str> = url.split('/').collect();
    segments.windows(3).find_map(|w| {
        if w[1] != "pull" {
            return None;
        }
        Some((w[0].to_string(), w[2].parse().ok()?))
    })
}

/// Deserialize artifacts, skipping any that fail to parse.
/// This ensures task loading doesn't fail entirely if an artifact has an unknown format.
pub fn deserialize_artifacts<'de, D>(deserializer: D) -> Result<Vec<Artifact>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let values: Vec<serde_json::Value> = serde::Deserialize::deserialize(deserializer)?;
    Ok(values
        .into_iter()
        .filter_map(|value| match serde_json::from_value::<Artifact>(value) {
            Ok(artifact) => Some(artifact),
            Err(e) => {
                report_error!(anyhow!("Failed to deserialize artifact, skipping: {}", e));
                None
            }
        })
        .collect())
}

pub fn file_button_label(filename: &str, filepath: &str) -> String {
    if let Some(filename) = non_empty_trimmed(filename) {
        return filename.to_string();
    }
    if let Some(filepath_basename) = sanitized_basename(filepath)
        .as_deref()
        .and_then(non_empty_trimmed)
    {
        return filepath_basename.to_string();
    }
    "File".to_string()
}

pub fn open_screenshot_lightbox<V: warpui::View>(
    artifact_uids: &[String],
    ctx: &mut warpui::ViewContext<V>,
) {
    // Open lightbox immediately with Loading placeholders.
    let loading_images: Vec<LightboxImage> = artifact_uids
        .iter()
        .map(|_| LightboxImage {
            source: LightboxImageSource::Loading,
            description: None,
        })
        .collect();
    ctx.dispatch_typed_action(&WorkspaceAction::OpenLightbox {
        images: loading_images,
        initial_index: 0,
    });

    // Fetch each signed URL independently and update the lightbox as each resolves.
    // TODO(QUALITY-318): We should cache the signed URL for each artifact UUID so
    // we avoid fetching screenshots already in the asset cache.
    let ai_client = ServerApiProvider::handle(ctx).as_ref(ctx).get_ai_client();

    for (i, uid) in artifact_uids.iter().enumerate() {
        let ai_client = ai_client.clone();
        let uid = uid.clone();
        let uid_for_callback = uid.clone();

        ctx.spawn(
            async move { ai_client.get_artifact_download(&uid).await },
            move |_me, result, ctx| {
                if let Some(image) =
                    screenshot_lightbox_image_from_download_result(result, &uid_for_callback, i)
                {
                    ctx.dispatch_typed_action(&WorkspaceAction::UpdateLightboxImage {
                        index: i,
                        image,
                    });
                }
            },
        );
    }
}

fn screenshot_lightbox_image_from_download_result(
    result: anyhow::Result<ArtifactDownloadResponse>,
    uid_for_callback: &str,
    index: usize,
) -> Option<LightboxImage> {
    match result {
        Ok(ArtifactDownloadResponse::Screenshot { data, .. }) => Some(LightboxImage {
            source: LightboxImageSource::Resolved {
                asset_source: asset_cache::url_source(data.download_url),
            },
            description: data
                .description
                .filter(|description| !description.is_empty()),
        }),
        Ok(ArtifactDownloadResponse::File { .. }) => {
            log::warn!("Artifact {uid_for_callback} was not a screenshot");
            None
        }
        Err(e) => {
            log::warn!("Failed to load screenshot artifact {index}: {e}");
            Some(LightboxImage {
                source: LightboxImageSource::Loading,
                description: Some("Failed to load".to_string()),
            })
        }
    }
}









fn non_empty_trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed)
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
