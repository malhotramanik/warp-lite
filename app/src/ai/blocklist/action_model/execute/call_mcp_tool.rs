use futures::FutureExt;
use futures::future::BoxFuture;
#[cfg(not(target_family = "wasm"))]
use itertools::Itertools;
#[cfg(not(target_family = "wasm"))]
use warpui::SingletonEntity;
use warpui::{Entity, EntityId, ModelContext, ModelHandle};

use super::{ActionExecution, AnyActionExecution, ExecuteActionInput, PreprocessActionInput};
use crate::terminal::model::session::active_session::ActiveSession;
#[cfg(not(target_family = "wasm"))]
use crate::ai::{
        agent::AIAgentAction,
        blocklist::{BlocklistAIPermissions, action_model::AIAgentActionType},
    };

pub struct CallMCPToolExecutor {
    _active_session: ModelHandle<ActiveSession>,
    #[allow(dead_code)]
    terminal_view_id: EntityId,
}

impl CallMCPToolExecutor {
    pub fn new(_active_session: ModelHandle<ActiveSession>, terminal_view_id: EntityId) -> Self {
        Self {
            _active_session,
            terminal_view_id,
        }
    }

    #[cfg_attr(target_family = "wasm", allow(unused_variables), allow(dead_code))]
    pub(super) fn should_autoexecute(
        &self,
        input: ExecuteActionInput,
        ctx: &mut ModelContext<Self>,
    ) -> bool {
        #[cfg(target_family = "wasm")]
        {
            false
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let ExecuteActionInput {
                action:
                    AIAgentAction {
                        action:
                            AIAgentActionType::CallMCPTool {
                                server_id, name, ..
                            },
                        ..
                    },
                conversation_id,
            } = input
            else {
                return false;
            };

            BlocklistAIPermissions::as_ref(ctx).can_call_mcp_tool(
                server_id.as_ref(),
                name.as_str(),
                &conversation_id,
                Some(self.terminal_view_id),
                ctx,
            )
        }
    }

    pub(super) fn execute(
        &mut self,
        _input: ExecuteActionInput,
        _ctx: &mut ModelContext<Self>,
    ) -> impl Into<AnyActionExecution> + use<> {
        // MCP servers are not supported in warp-lite.
        ActionExecution::<()>::InvalidAction
    }

    pub(super) fn preprocess_action(
        &mut self,
        _action: PreprocessActionInput,
        _ctx: &mut ModelContext<Self>,
    ) -> BoxFuture<'static, ()> {
        futures::future::ready(()).boxed()
    }
}

impl Entity for CallMCPToolExecutor {
    type Event = ();
}

/// Coerces whole-number floats in `args` to integers for fields declared as
/// [`"type": "integer"`](https://json-schema.org/understanding-json-schema/reference/type)
/// in the tool's JSON Schema `input_schema`.
///
/// MCP tool args round-trip through `google.protobuf.Struct` on the wire, whose
/// `NumberValue` stores everything as `f64`. Without this fix, serde_json emits
/// whole-number floats as `"5.0"`, which strict MCP servers reject for integer fields.
///
/// Walks the schema recursively so integer fields nested inside objects, arrays,
/// or `oneOf`/`anyOf`/`allOf` branches are all coerced. `$ref` is not resolved
/// (the root schema would be required) and is skipped.
pub(crate) fn coerce_integer_args(
    args: &mut serde_json::Map<String, serde_json::Value>,
    input_schema: &serde_json::Map<String, serde_json::Value>,
) {
    // Delegate to the recursive walker by wrapping `args` in a borrowed `Value`.
    // This keeps the root-level traversal consistent with nested levels, so
    // top-level `oneOf`/`anyOf`/`allOf` and `additionalProperties` are honored
    // the same way they are deeper in the schema.
    let mut wrapped = serde_json::Value::Object(std::mem::take(args));
    let schema_value = serde_json::Value::Object(input_schema.clone());
    coerce_value_against_schema(&mut wrapped, &schema_value);
    if let serde_json::Value::Object(restored) = wrapped {
        *args = restored;
    }
}

/// Returns true if the schema's `type` declares `"integer"`, including the
/// nullable form `"type": ["integer", "null"]`.
fn schema_declares_integer(schema: &serde_json::Value) -> bool {
    match schema.get("type") {
        Some(serde_json::Value::String(s)) => s == "integer",
        Some(serde_json::Value::Array(types)) => {
            types.iter().any(|t| t.as_str() == Some("integer"))
        }
        _ => false,
    }
}

/// In-place coerces a whole-number `f64` `Number` to `i64`.
fn coerce_number_to_int(n: &mut serde_json::Number) {
    let Some(f) = n.as_f64() else { return };
    if n.is_i64() || n.is_u64() {
        return;
    }
    if f.fract() != 0.0 {
        return;
    }
    if let Ok(i) = i64::try_from(f as i128) {
        *n = serde_json::Number::from(i);
    }
}

/// Recursively walks `value` against `schema`, coercing whole-number f64s to
/// i64 wherever the schema declares `"type": "integer"`. Safe to call against
/// multiple `oneOf`/`anyOf`/`allOf` branches: coercion is a no-op on values
/// the schema does not match.
fn coerce_value_against_schema(value: &mut serde_json::Value, schema: &serde_json::Value) {
    if schema_declares_integer(schema)
        && let serde_json::Value::Number(n) = value
    {
        coerce_number_to_int(n);
    }

    // Visit every combinator key independently — a schema may declare more than
    // one of {oneOf, anyOf, allOf} at the same level, and we need to walk every
    // branch in every present combinator. Coercion is monotonic, so visiting
    // branches whose constraints don't match `value` is a safe no-op.
    for combinator in ["oneOf", "anyOf", "allOf"] {
        if let Some(branches) = schema.get(combinator).and_then(|b| b.as_array()) {
            for branch in branches {
                coerce_value_against_schema(value, branch);
            }
        }
    }

    match value {
        serde_json::Value::Object(map) => {
            let properties = schema.get("properties").and_then(|p| p.as_object());
            let additional = schema.get("additionalProperties");
            // Per-key handling for the outer schema only. Per-branch handling
            // for keys covered by `oneOf`/`anyOf`/`allOf` is reached through
            // the top-level combinator recursion above: each branch is invoked
            // with the full `value`, so the branch's own object handling runs
            // and walks its `properties[k]`. Coercion is monotonic so the two
            // passes stack safely.
            for (k, v) in map.iter_mut() {
                if let Some(prop_schema) = properties.and_then(|p| p.get(k)) {
                    coerce_value_against_schema(v, prop_schema);
                } else if let Some(extra_schema) = additional
                    && extra_schema.is_object()
                {
                    coerce_value_against_schema(v, extra_schema);
                }
            }
        }
        serde_json::Value::Array(items) => {
            if let Some(item_schema) = schema.get("items") {
                match item_schema {
                    // `items` as an object schema: applies to every element.
                    serde_json::Value::Object(_) => {
                        for elem in items.iter_mut() {
                            coerce_value_against_schema(elem, item_schema);
                        }
                    }
                    // `items` as an array (tuple validation): positional schemas.
                    serde_json::Value::Array(schemas) => {
                        for (elem, elem_schema) in items.iter_mut().zip(schemas.iter()) {
                            coerce_value_against_schema(elem, elem_schema);
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
#[path = "call_mcp_tool_tests.rs"]
mod tests;

