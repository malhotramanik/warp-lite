//! Skill resolution for agent runs.
//!
//! This module exists primarily for `warp agent run --skill ...` (and related flows) where we need to
//! resolve a CLI-provided `--skill` specifier (`SkillSpec`) into a concrete `SKILL.md` file and its
//! parsed instruction body.
//!
//! While `SkillManager` maintains a cached view of known skills and can list skills "in scope",
//! agent runs need a single-shot resolver that:
//! - Works when invoked from directories *above* any detected repos (ambient/root-scope case).
//! - Supports qualified specs (`repo:skill` and `org/repo:skill`) and returns good errors on ambiguity.
//! - Applies consistent skill-directory precedence (e.g. `.claude/` vs `.codex/`, etc.).
//! - Falls back to scanning disk when the manager cache has not warmed yet.























#[cfg(test)]
#[path = "resolve_skill_spec_tests.rs"]
mod tests;
