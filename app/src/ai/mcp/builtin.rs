//! Built-in Warp-hosted MCP servers.
//!
//! Built-in servers are attached automatically for logged-in users: their
//! definitions are constructed in code and authenticated with the user's
//! existing session credentials (warp-server accepts both session ID tokens
//! and API keys as `Bearer` credentials), so they require no MCP
//! configuration and no manually minted API key.
//!
//! Lifecycle is owned by [`TemplatableMCPServerManager::sync_builtin_servers`]
//! (attach on login, re-attach on token rotation, detach on logout), gated by
//! [`warp_core::features::FeatureFlag::FactoryMcp`].
//!
//! CLI agent runs (`oz agent run`, including the cloud workers behind
//! `oz agent run-cloud`) bypass that manager path; the agent driver attaches
//! the same installation per-run instead (see
//! `AgentDriver::resolve_mcp_specs_to_json`), with the token pinned for the
//! duration of the run.
//!
//! The preview URLs server is only useful inside a cloud run, so only the agent
//! driver attaches it, gated by [`warp_core::features::FeatureFlag::PreviewUrlsMcp`].
//!
//! [`TemplatableMCPServerManager::sync_builtin_servers`]: super::TemplatableMCPServerManager::sync_builtin_servers















#[cfg(test)]
#[path = "builtin_tests.rs"]
mod tests;
