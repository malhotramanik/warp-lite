pub mod manager;
pub mod templatable_manager;

#[cfg(not(target_family = "wasm"))]
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use strum::IntoEnumIterator;
use strum_macros::EnumIter;
#[cfg(not(target_family = "wasm"))]
pub use templatable_manager::McpIntegration;
pub use templatable_manager::TemplatableMCPServerManager;
use warp_core::ui::Icon;
use warp_core::ui::appearance::Appearance;

use crate::cloud_object::model::generic_string_model::StringModel;
use crate::cloud_object::model::json_model::JsonModel;
use crate::cloud_object::{
     GenericStringObjectFormat, GenericStringObjectUniqueKey, JsonObjectType,
    Revision,
};
use crate::drive::CloudObjectTypeAndId;
use crate::drive::items::WarpDriveItem;
use crate::drive::items::mcp_server::WarpDriveMCPServer;
use crate::server::ids::SyncId;
use crate::server::sync_queue::QueueItem;

// warp-lite: MCP config files are never watched; the dummy implementations are used everywhere.
mod dummy_file_based_manager;
pub use dummy_file_based_manager::FileBasedMCPManager;
mod dummy_file_mcp_watcher;
pub use dummy_file_mcp_watcher::FileMCPWatcher;

pub mod gallery;
pub use gallery::MCPGalleryManager;
#[cfg(not(target_family = "wasm"))]
pub mod builtin;
pub mod templatable;
#[cfg(not(target_family = "wasm"))]
pub use cloud_object_models::{
    JSONMCPServer, JSONTransportType, StaticEnvVar, StaticHeader,
};
pub use cloud_object_models::{
    CloudMCPServer, CloudMCPServerModel, MCPServer, MCPServerState,
};
pub use templatable::{TemplatableMCPServer, TemplateVariable};
pub mod logs;
pub mod templatable_installation;
pub use templatable_installation::TemplatableMCPServerInstallation;
pub mod parsing;
#[cfg(not(target_family = "wasm"))]
pub use parsing::ParsedTemplatableMCPServerResult;
#[cfg(not(target_family = "wasm"))]
pub mod reconnecting_peer;


impl StringModel for MCPServer {
    type CloudObjectType = CloudMCPServer;

    fn model_type_name(&self) -> &'static str {
        "MCP server"
    }

    fn should_enforce_revisions() -> bool {
        true
    }

    fn model_format() -> GenericStringObjectFormat {
        GenericStringObjectFormat::Json(JsonObjectType::MCPServer)
    }

    fn should_show_activity_toasts() -> bool {
        true
    }

    fn warn_if_unsaved_at_quit() -> bool {
        true
    }

    fn display_name(&self) -> String {
        self.name.clone()
    }

    fn update_object_queue_item(
        &self,
        revision_ts: Option<Revision>,
        object: &Self::CloudObjectType,
    ) -> QueueItem {
        QueueItem::UpdateMCPServer {
            model: object.model().clone().into(),
            id: object.id,
            revision: revision_ts.or(object.metadata.revision),
        }
    }

    fn uniqueness_key(&self) -> Option<GenericStringObjectUniqueKey> {
        None
    }

    fn renders_in_warp_drive(&self) -> bool {
        false
    }

    fn to_warp_drive_item(
        &self,
        id: SyncId,
        _appearance: &Appearance,
        mcp_server: &CloudMCPServer,
    ) -> Option<Box<dyn WarpDriveItem>> {
        Some(Box::new(WarpDriveMCPServer::new(
            CloudObjectTypeAndId::GenericStringObject {
                object_type: GenericStringObjectFormat::Json(JsonObjectType::MCPServer),
                id,
            },
            mcp_server.clone(),
        )))
    }
}

impl JsonModel for MCPServer {
    fn json_object_type() -> JsonObjectType {
        JsonObjectType::MCPServer
    }
}

/// Trait for types that have a name and value field.
/// Used for shared operations on `StaticEnvVar` and `StaticHeader`.
#[cfg(not(target_family = "wasm"))]
trait NameValuePair {
    fn name(&self) -> &str;
    fn value(&self) -> &str;
}

#[cfg(not(target_family = "wasm"))]
impl NameValuePair for StaticEnvVar {
    fn name(&self) -> &str {
        &self.name
    }
    fn value(&self) -> &str {
        &self.value
    }
}
#[cfg(not(target_family = "wasm"))]
impl NameValuePair for StaticHeader {
    fn name(&self) -> &str {
        &self.name
    }
    fn value(&self) -> &str {
        &self.value
    }
}


/// Converts a slice of name/value pair items to a HashMap.
#[cfg(not(target_family = "wasm"))]
#[allow(dead_code)]
fn items_to_hashmap<T: NameValuePair>(items: &[T]) -> HashMap<String, String> {
    items
        .iter()
        .map(|item| (item.name().to_owned(), item.value().to_owned()))
        .collect()
}








/// The home directory that global (non-Warp) provider configs are resolved against. Tests
/// redirect it through `HOME`; `dirs::home_dir()` ignores that variable on Windows.
pub(crate) fn home_dir() -> Option<PathBuf> {
    #[cfg(test)]
    if let Some(home) = std::env::var_os("HOME")
        && !home.is_empty()
    {
        return Some(PathBuf::from(home));
    }
    dirs::home_dir()
}

pub(crate) fn home_config_file_path(provider: MCPProvider) -> Option<PathBuf> {
    match provider {
        MCPProvider::Warp => warp_core::paths::warp_home_mcp_config_file_path(),
        _ => home_dir().map(|home_dir| home_dir.join(provider.home_config_path())),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
pub enum MCPProvider {
    Warp,
    Claude,
    Codex,
    Agents,
}

impl MCPProvider {
    pub fn display_name(&self) -> &str {
        match self {
            MCPProvider::Warp => "Warp",
            MCPProvider::Claude => "Claude",
            MCPProvider::Codex => "Codex",
            MCPProvider::Agents => "Other Agents",
        }
    }

    pub fn icon(&self) -> Icon {
        match self {
            // Warp's own agent MCP config — use the Warp agent brand mark.
            MCPProvider::Warp => Icon::Agent,
            MCPProvider::Claude => Icon::ClaudeLogo,
            MCPProvider::Codex => Icon::OpenAILogo,
            // "Other Agents" is the cross-tool .agents/.mcp.json convention for
            // third-party agent tooling (not Warp-branded). Use a neutral AI
            // icon so this row never carries the Warp agent mark, and the two
            // rows remain visually distinct once Icon::Agent gets its own asset.
            MCPProvider::Agents => Icon::AiAssistant,
        }
    }

    /// Returns the path of the provider's config file relative to the home directory.
    pub fn home_config_path(&self) -> &'static Path {
        match self {
            MCPProvider::Warp => Path::new(".warp/.mcp.json"),
            MCPProvider::Claude => Path::new(".claude.json"),
            MCPProvider::Codex => Path::new(".codex/config.toml"),
            MCPProvider::Agents => Path::new(".agents/.mcp.json"),
        }
    }

    /// Returns the path of the provider's config file relative to a project root.
    pub fn project_config_path(&self) -> &'static Path {
        match self {
            MCPProvider::Warp => Path::new(".warp/.mcp.json"),
            MCPProvider::Claude => Path::new(".mcp.json"),
            MCPProvider::Codex => Path::new(".codex/config.toml"),
            MCPProvider::Agents => Path::new(".agents/.mcp.json"),
        }
    }
}

/// Returns the [`MCPProvider`] that owns `file_path` as a config file, if any.
///
/// Matches against both home-level configs (e.g. `~/.claude.json`) and
/// project-level configs (e.g. `.mcp.json` anywhere in the path).
pub fn mcp_provider_from_file_path(file_path: &Path) -> Option<MCPProvider> {
    // Try exact home-config match first (unambiguous).
    for provider in MCPProvider::iter() {
        if home_config_file_path(provider)
            .as_ref()
            .is_some_and(|home_config_path| file_path == home_config_path)
        {
            return Some(provider);
        }
    }
    // Fall back to project-config suffix match, preferring the longest
    // (most-specific) suffix.
    // This avoids `.mcp.json` shadowing `.warp/.mcp.json`, for example.
    let mut best: Option<(MCPProvider, usize)> = None;
    for provider in MCPProvider::iter() {
        let cfg = provider.project_config_path();
        if file_path.ends_with(cfg) {
            let len = cfg.as_os_str().len();
            if best.is_none_or(|(_, best_len)| len > best_len) {
                best = Some((provider, len));
            }
        }
    }
    best.map(|(p, _)| p)
}

#[cfg(test)]
#[path = "mod_tests.rs"]
mod tests;
