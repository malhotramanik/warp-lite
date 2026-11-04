# Warp Lite

A lightweight macOS terminal built from the open-source [Warp](https://github.com/warpdotdev/warp) client, with the AI and cloud features removed.

> **This is an unofficial fork.** It is not made, endorsed or supported by Warp (Denver Technologies, Inc). Please do not file Warp Lite issues on the upstream repository.

## What's different

**Removed:** all AI features, Warp Drive, login and cloud sync, telemetry, autoupdate, onboarding, and the built-in code editor.

**Kept:** the core terminal, all tab customisation (vertical, grouped and pinned tabs, tab configs, directory-based tab colors), themes including background images, keybindings, vim mode, and SSH / Warpify.

| | Stock Warp | Warp Lite |
|---|---|---|
| App bundle size | ~457 MB | ~154 MB |
| Idle memory footprint | — | ~180–270 MB (mostly GPU buffers; app heap ~78 MB) |

## Install (macOS, Apple Silicon)

1. Download `WarpLite-macos-arm64.zip` from the [Releases](../../releases) page (or from the artifacts of a [build-macos workflow run](../../actions/workflows/build-macos.yml)).
2. Unzip it.
3. The app is ad-hoc signed, not notarized, so remove the quarantine flag:
   ```bash
   xattr -dr com.apple.quarantine WarpLite.app
   ```
4. Move `WarpLite.app` to `/Applications`.

## Build from source

Requires the full Xcode install (not just the Command Line Tools) for the Metal shader compiler, plus Rust, `protobuf` and [`cargo-bundle`](script/install_cargo_bundle).

```bash
script/install_cargo_bundle
cd app
cargo bundle --release --bin warp-oss --features gui,release_bundle
# output: target/release/bundle/osx/WarpLite.app
```

CI does the same in [`.github/workflows/build-macos.yml`](.github/workflows/build-macos.yml). Pushing a `v*` tag builds the app and publishes a GitHub Release with the zip attached.

## Licensing

The UI framework crates (`warpui_core` and `warpui`) are licensed under the [MIT license](LICENSE-MIT).

The rest of the code in this repository is licensed under the [AGPL v3](LICENSE-AGPL).

Warp Lite is derived from [warpdotdev/warp](https://github.com/warpdotdev/warp) and is distributed under the same licenses.
