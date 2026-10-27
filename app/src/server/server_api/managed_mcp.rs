// We don't resolve managed MCPs from agent run CLI flows on WASM, so this code is unused there.
#![cfg_attr(target_family = "wasm", expect(dead_code))]

#[cfg(test)]
use mockall::automock;



