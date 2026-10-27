
/// Basic file search result structure that can be used across different UI components.
/// This is the common data format returned by the FileSearchModel.
#[derive(Debug, Clone)]
pub struct FileSearchResult {
    pub path: String,
    pub project_directory: String,
    pub is_directory: bool,
}


