use std::path::Path;
#[cfg(feature = "local_fs")]
use std::path::PathBuf;

#[cfg(feature = "local_fs")]
use crate::server::server_api::ai::ArtifactDownloadResponse;

pub(crate) fn sanitized_basename(path_or_filename: &str) -> Option<String> {
    let file_name = Path::new(path_or_filename).file_name()?.to_str()?;
    if file_name.is_empty() {
        return None;
    }
    Some(file_name.to_string())
}






#[cfg(test)]
#[path = "artifact_download_tests.rs"]
mod tests;
